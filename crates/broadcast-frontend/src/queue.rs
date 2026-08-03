//! Queue item model and analysis. Every field here is derived from
//! `tx-core`: this module never parses a PSBT or transaction itself, it
//! only shapes `tx-core`'s output for the queue table to render.

use bitcoin::{Psbt, Transaction};
use tx_core::{Decoded, FinalizeError, FinalizedPsbt, FloorComparison, Format, PsbtFee};

use crate::tokens::{
    ACCENT_TEAL_BRIGHT, ERROR_RED, IN_FLIGHT, NOTE_CARD_ERROR, NOTE_CARD_OK, NOTE_CARD_WARN,
    TEXT_DISABLED, TEXT_MUTED_7B, TEXT_PRIMARY, WARNING,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RowFormat {
    Psbt,
    RawTx,
    Unknown,
}

impl RowFormat {
    pub fn label(self) -> &'static str {
        match self {
            RowFormat::Psbt => "PSBT",
            RowFormat::RawTx => "RAW TX",
            RowFormat::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NoteKind {
    Ok,
    Warn,
    Error,
}

impl NoteKind {
    /// (border, left edge, text) colour triple.
    pub fn colors(self) -> (&'static str, &'static str, &'static str) {
        match self {
            NoteKind::Ok => NOTE_CARD_OK,
            NoteKind::Warn => NOTE_CARD_WARN,
            NoteKind::Error => NOTE_CARD_ERROR,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct NoteCard {
    pub kind: NoteKind,
    pub text: String,
}

/// What's known once an item is decoded, versus not at all.
#[derive(Clone, PartialEq)]
pub enum QueueItemBody {
    /// Detected but not decodable, or a PSBT that could not be finalized.
    Invalid,
    Decoded {
        vsize: u64,
        txid: String,
        tx_hex: String,
        /// `Some` when every input's value was known at decode time.
        known_fee_sats: Option<i128>,
        output_sum_sats: u64,
    },
}

/// A queued item's submission outcome. Every item starts `Unsent`; the
/// other variants are reached only by the broadcast loop.
#[derive(Clone, PartialEq)]
pub enum SubmissionState {
    Unsent,
    Sending,
    Accepted,
    Rejected(String),
    RateLimited,
    Failed(String),
}

#[derive(Clone, PartialEq)]
pub struct QueueItem {
    pub id: u64,
    pub name: String,
    pub origin: String,
    pub format: RowFormat,
    pub body: QueueItemBody,
    pub note: Option<NoteCard>,
    pub submission: SubmissionState,
    /// User-entered total input value, for a `Fee unknown` row.
    pub total_input_override: Option<u64>,
}

/// Mirrors `broadcast-api`'s `max_payload_bytes` default (PLAN.md section
/// 4): the frontend never learns the server's actual configured value, so
/// an item whose hex would guarantee a 413 is flagged the moment it is
/// decoded rather than left to fail at broadcast time.
const MAX_PAYLOAD_BYTES: usize = 1_048_576;

/// `None` when `tx_hex` fits the payload cap; otherwise a displayable
/// refusal reason.
fn payload_cap_error(tx_hex: &str) -> Option<String> {
    if tx_hex.len() <= MAX_PAYLOAD_BYTES {
        return None;
    }
    Some(format!(
        "Transaction is {} bytes as hex, over the {} MiB payload limit.",
        format_thousands(tx_hex.len() as i128),
        MAX_PAYLOAD_BYTES / (1024 * 1024)
    ))
}

/// The result of analyzing one loaded entry (a pasted line, or an item
/// expanded from a dropped file/archive). A PSBT that cannot be finalized
/// never becomes a [`QueueItem`] — it must be refused via the finalization
/// modal instead (PLAN.md section 1), so it gets its own variant rather
/// than an `Invalid` row.
pub enum AnalyzeOutcome {
    Queued(Box<QueueItem>),
    UnfinalizablePsbt { name: String, reason: String },
}

impl QueueItem {
    pub(crate) fn invalid(
        id: u64,
        name: String,
        origin: String,
        format: RowFormat,
        message: String,
    ) -> Self {
        QueueItem {
            id,
            name,
            origin,
            format,
            body: QueueItemBody::Invalid,
            note: Some(NoteCard {
                kind: NoteKind::Error,
                text: message,
            }),
            submission: SubmissionState::Unsent,
            total_input_override: None,
        }
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self.body, QueueItemBody::Invalid)
    }

    /// Every row that decoded successfully and has not already been
    /// accepted or is not already in flight — whatever its fee rate.
    pub fn is_submittable(&self) -> bool {
        !self.is_invalid()
            && !matches!(
                self.submission,
                SubmissionState::Accepted | SubmissionState::Sending
            )
    }
}

/// What an entry `tx-core` cannot sniff is labelled, both on its own queue
/// row and on the paste box's inline card when nothing in a paste sniffs.
const UNRECOGNISED_MESSAGE: &str = "Not valid base64 PSBT or hex transaction data.";

/// Whether a paste may be queued, and if not, why.
///
/// Single source of truth for the "Add to queue" button's enabled state,
/// the format read-out beside the heading, and `use_queue`'s `on_submit`,
/// so the mouse path and the Ctrl/Cmd+Enter path cannot disagree.
///
/// Detection is a format sniff, never a decode: an entry that sniffs but
/// then fails to decode still opens the gate and is queued as an `Invalid`
/// row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PasteGate {
    /// Nothing but blank and `#` comment lines.
    Empty,
    /// Entries exist, none of them sniffs as a PSBT or transaction.
    NothingRecognised,
    Queueable,
}

impl PasteGate {
    pub fn allows_queueing(self) -> bool {
        matches!(self, PasteGate::Queueable)
    }

    /// Text for the inline card when the paste is refused.
    pub fn refusal(self) -> Option<&'static str> {
        match self {
            PasteGate::Empty => Some("Nothing pasted yet."),
            PasteGate::NothingRecognised => Some(UNRECOGNISED_MESSAGE),
            PasteGate::Queueable => None,
        }
    }
}

pub fn paste_gate(lines: &[&str]) -> PasteGate {
    if lines.is_empty() {
        PasteGate::Empty
    } else if lines
        .iter()
        .any(|line| tx_core::detect(line.as_bytes()).is_some())
    {
        PasteGate::Queueable
    } else {
        PasteGate::NothingRecognised
    }
}

/// The mono read-out beside the heading, taken from the first entry that
/// sniffs rather than the first entry, so it cannot contradict
/// [`paste_gate`] on a paste whose leading entry is garbage.
pub fn detected_label(lines: &[&str]) -> &'static str {
    if lines.is_empty() {
        return "";
    }
    match lines
        .iter()
        .find_map(|line| tx_core::detect(line.as_bytes()))
    {
        Some(Format::PsbtBinary | Format::PsbtBase64) => "DETECTED · PSBT",
        Some(Format::TxHex | Format::TxBinary) => "DETECTED · RAW TRANSACTION",
        None => "UNRECOGNISED FORMAT",
    }
}

/// An `Invalid` row for a dropped file or archive that could not be
/// unpacked at all (too many entries, over the decompression budget,
/// nested, or malformed) — distinct from a per-item decode failure, since
/// `tx-core` never even sees this one.
pub fn unreadable_file(id: u64, name: String, origin: String, message: String) -> QueueItem {
    QueueItem::invalid(id, name, origin, RowFormat::Unknown, capitalize(&message))
}

/// Splits `text` (already one line from the paste box, or one item expanded
/// from a file/archive) into an [`AnalyzeOutcome`], entirely via `tx-core`.
pub fn analyze(id: u64, name: String, origin: String, text: &str) -> AnalyzeOutcome {
    let bytes = text.as_bytes();
    match tx_core::detect(bytes) {
        None => AnalyzeOutcome::Queued(Box::new(QueueItem::invalid(
            id,
            name,
            origin,
            RowFormat::Unknown,
            UNRECOGNISED_MESSAGE.to_string(),
        ))),
        Some(format) => {
            let row_format = row_format_for(format);
            match tx_core::decode_as(format, bytes) {
                Err(err) => AnalyzeOutcome::Queued(Box::new(QueueItem::invalid(
                    id,
                    name,
                    origin,
                    row_format,
                    capitalize(&err.to_string()),
                ))),
                Ok(Decoded::Transaction(tx)) => {
                    AnalyzeOutcome::Queued(Box::new(from_transaction(id, name, origin, tx)))
                }
                Ok(Decoded::Psbt(psbt)) => from_psbt(id, name, origin, psbt),
            }
        }
    }
}

fn row_format_for(format: Format) -> RowFormat {
    match format {
        Format::PsbtBinary | Format::PsbtBase64 => RowFormat::Psbt,
        Format::TxHex | Format::TxBinary => RowFormat::RawTx,
    }
}

fn from_transaction(id: u64, name: String, origin: String, tx: Transaction) -> QueueItem {
    let tx_hex = tx_core::serialize_hex(&tx);
    if let Some(message) = payload_cap_error(&tx_hex) {
        return QueueItem::invalid(id, name, origin, RowFormat::RawTx, message);
    }
    let output_sum_sats = match tx_core::output_sum(&tx) {
        Ok(output_sum) => output_sum,
        Err(err) => {
            return QueueItem::invalid(
                id,
                name,
                origin,
                RowFormat::RawTx,
                capitalize(&err.to_string()),
            );
        }
    };
    QueueItem {
        id,
        name,
        origin,
        format: RowFormat::RawTx,
        body: QueueItemBody::Decoded {
            vsize: tx_core::vsize(&tx),
            txid: tx.compute_txid().to_string(),
            tx_hex,
            known_fee_sats: None,
            output_sum_sats,
        },
        note: None,
        submission: SubmissionState::Unsent,
        total_input_override: None,
    }
}

fn from_psbt(id: u64, name: String, origin: String, psbt: Psbt) -> AnalyzeOutcome {
    let output_sum = match tx_core::psbt_output_sum(&psbt) {
        Ok(output_sum) => output_sum,
        Err(err) => {
            return AnalyzeOutcome::Queued(Box::new(QueueItem::invalid(
                id,
                name,
                origin,
                RowFormat::Psbt,
                capitalize(&err.to_string()),
            )));
        }
    };
    let fee = match tx_core::psbt_fee(&psbt) {
        Ok(fee) => fee,
        Err(err) => {
            return AnalyzeOutcome::Queued(Box::new(QueueItem::invalid(
                id,
                name,
                origin,
                RowFormat::Psbt,
                capitalize(&err.to_string()),
            )));
        }
    };
    match tx_core::finalize(psbt) {
        Err(err @ FinalizeError::Miniscript(_)) => AnalyzeOutcome::UnfinalizablePsbt {
            name,
            reason: err.to_string(),
        },
        Err(FinalizeError::InvalidPsbt(err)) => {
            AnalyzeOutcome::Queued(Box::new(QueueItem::invalid(
                id,
                name,
                origin,
                RowFormat::Psbt,
                capitalize(&err.to_string()),
            )))
        }
        Ok(finalized) => {
            let (tx, note) = match finalized {
                FinalizedPsbt::Validated(tx) => {
                    let note = NoteCard {
                        kind: NoteKind::Ok,
                        text: format!(
                            "Finalized locally: {} input{} to {} output{}, ready to extract.",
                            tx.input.len(),
                            if tx.input.len() > 1 { "s" } else { "" },
                            tx.output.len(),
                            if tx.output.len() > 1 { "s" } else { "" }
                        ),
                    };
                    (tx, Some(note))
                }
                FinalizedPsbt::Unchecked {
                    transaction,
                    missing_utxo_inputs,
                } => (
                    transaction,
                    Some(NoteCard {
                        kind: NoteKind::Warn,
                        text: format!(
                            "Missing UTXO data for {missing_utxo_inputs} input(s). Fee and final scripts could not be validated."
                        ),
                    }),
                ),
            };
            let tx_hex = tx_core::serialize_hex(&tx);
            if let Some(message) = payload_cap_error(&tx_hex) {
                return AnalyzeOutcome::Queued(Box::new(QueueItem::invalid(
                    id,
                    name,
                    origin,
                    RowFormat::Psbt,
                    message,
                )));
            }
            let known_fee_sats = match fee {
                PsbtFee::Known { fee_sats } => Some(fee_sats),
                PsbtFee::Unknown { .. } => None,
            };
            AnalyzeOutcome::Queued(Box::new(QueueItem {
                id,
                name,
                origin,
                format: RowFormat::Psbt,
                body: QueueItemBody::Decoded {
                    vsize: tx_core::vsize(&tx),
                    txid: tx.compute_txid().to_string(),
                    tx_hex,
                    known_fee_sats,
                    output_sum_sats: output_sum,
                },
                note,
                submission: SubmissionState::Unsent,
                total_input_override: None,
            }))
        }
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// First 14 and last 8 characters of `text` with whitespace stripped,
/// joined by an ellipsis — a short, still-recognisable label for a queue
/// row's name cell. Left unchanged when it's already short enough that
/// the prefix and suffix windows would overlap.
pub fn short_name(text: &str) -> String {
    const PREFIX_LEN: usize = 14;
    const SUFFIX_LEN: usize = 8;
    let compact: String = text.split_whitespace().collect();
    let len = compact.chars().count();
    if len <= PREFIX_LEN + SUFFIX_LEN {
        return compact;
    }
    let prefix: String = compact.chars().take(PREFIX_LEN).collect();
    let suffix: String = compact.chars().skip(len - SUFFIX_LEN).collect();
    format!("{prefix}…{suffix}")
}

/// The fee an item would report right now: its own known fee, or one
/// derived from a user-entered total input value.
pub fn effective_fee(item: &QueueItem) -> Option<i128> {
    match &item.body {
        QueueItemBody::Invalid => None,
        QueueItemBody::Decoded {
            known_fee_sats: Some(fee),
            ..
        } => Some(*fee),
        QueueItemBody::Decoded {
            known_fee_sats: None,
            output_sum_sats,
            ..
        } => {
            let total = item.total_input_override?;
            Some(tx_core::fee_from_user_total(total, *output_sum_sats))
        }
    }
}

pub fn effective_rate(item: &QueueItem) -> Option<f64> {
    let QueueItemBody::Decoded { vsize, .. } = &item.body else {
        return None;
    };
    tx_core::fee_rate(effective_fee(item)?, *vsize)
}

/// Whether the row should expose the inline total-input-value field:
/// every `Fee unknown` row, raw transaction or partial-UTXO PSBT alike,
/// while it hasn't already been sent.
pub fn shows_input_value_field(item: &QueueItem) -> bool {
    matches!(item.submission, SubmissionState::Unsent)
        && matches!(
            &item.body,
            QueueItemBody::Decoded {
                known_fee_sats: None,
                ..
            }
        )
}

pub struct RowStatusView {
    pub label: &'static str,
    pub text_color: &'static str,
    pub dot_color: &'static str,
    pub pulsing: bool,
}

/// The row's status dot and label. Driven by the submission outcome first,
/// then the fee-vs-floor comparison — never a gate, only a read-out.
pub fn row_status(item: &QueueItem, floor: f64) -> RowStatusView {
    if item.is_invalid() {
        return RowStatusView {
            label: "Invalid",
            text_color: ERROR_RED,
            dot_color: ERROR_RED,
            pulsing: false,
        };
    }
    match &item.submission {
        SubmissionState::Accepted => RowStatusView {
            label: "Accepted",
            text_color: ACCENT_TEAL_BRIGHT,
            dot_color: ACCENT_TEAL_BRIGHT,
            pulsing: false,
        },
        SubmissionState::Sending => RowStatusView {
            label: "Sending…",
            text_color: IN_FLIGHT,
            dot_color: IN_FLIGHT,
            pulsing: true,
        },
        SubmissionState::Rejected(_) => RowStatusView {
            label: "Rejected",
            text_color: ERROR_RED,
            dot_color: ERROR_RED,
            pulsing: false,
        },
        SubmissionState::RateLimited => RowStatusView {
            label: "Rate limited",
            text_color: WARNING,
            dot_color: WARNING,
            pulsing: false,
        },
        SubmissionState::Failed(_) => RowStatusView {
            label: "Failed",
            text_color: ERROR_RED,
            dot_color: ERROR_RED,
            pulsing: false,
        },
        SubmissionState::Unsent => match tx_core::compare_to_floor(effective_rate(item), floor) {
            FloorComparison::Unknown => RowStatusView {
                label: "Fee unknown",
                text_color: TEXT_MUTED_7B,
                dot_color: TEXT_DISABLED,
                pulsing: false,
            },
            FloorComparison::AtOrAbove => RowStatusView {
                label: "Ready",
                text_color: ACCENT_TEAL_BRIGHT,
                dot_color: ACCENT_TEAL_BRIGHT,
                pulsing: false,
            },
            FloorComparison::Below => RowStatusView {
                label: "Below floor",
                text_color: ERROR_RED,
                dot_color: ERROR_RED,
                pulsing: false,
            },
        },
    }
}

/// Colour of the fee-rate number itself — separate from the status label's
/// colour (e.g. a `Rejected` row's rate still reads white when it clears
/// the floor).
pub fn rate_color(item: &QueueItem, floor: f64) -> &'static str {
    match tx_core::compare_to_floor(effective_rate(item), floor) {
        FloorComparison::Unknown => TEXT_DISABLED,
        FloorComparison::AtOrAbove => TEXT_PRIMARY,
        FloorComparison::Below => ERROR_RED,
    }
}

/// Space-thousands formatting matching the mockup's
/// `toLocaleString('en-US').replace(/,/g, ' ')`.
pub fn format_thousands(n: i128) -> String {
    let negative = n < 0;
    let digits = n.unsigned_abs().to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            grouped.push(' ');
        }
        grouped.push(ch);
    }
    let grouped: String = grouped.chars().rev().collect();
    if negative {
        format!("-{grouped}")
    } else {
        grouped
    }
}

pub struct QueueStats {
    pub total: usize,
    pub at_or_above_floor: usize,
    pub below_floor: usize,
    pub fee_unknown: usize,
}

/// The stat strip's four counts. `total` counts every queued item,
/// including invalid ones; the other three only bucket items still
/// awaiting submission, by their effective fee rate.
pub fn stats(items: &[QueueItem], floor: f64) -> QueueStats {
    let mut at_or_above_floor = 0;
    let mut below_floor = 0;
    let mut fee_unknown = 0;
    for item in items {
        if item.is_invalid()
            || matches!(
                item.submission,
                SubmissionState::Accepted | SubmissionState::Sending
            )
        {
            continue;
        }
        match tx_core::compare_to_floor(effective_rate(item), floor) {
            FloorComparison::Unknown => fee_unknown += 1,
            FloorComparison::AtOrAbove => at_or_above_floor += 1,
            FloorComparison::Below => below_floor += 1,
        }
    }
    QueueStats {
        total: items.len(),
        at_or_above_floor,
        below_floor,
        fee_unknown,
    }
}

/// The broadcast button's label: a dynamic count while more than one item
/// is submittable, a static label otherwise. Never describes the fee
/// rates, only the run's shape.
pub fn broadcast_label(submittable: usize, in_flight: bool) -> String {
    if in_flight {
        "Broadcasting…".to_string()
    } else if submittable > 1 {
        format!("Broadcast {submittable} transactions")
    } else {
        "Broadcast".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Same fixture as tx-core's own `tests/fixtures/tx.hex`.
    const TX_HEX: &str = "0100000001a15d57094aa7a21a28cb20b59aab8fc7d1149a3bdbcddba9c622e4f5f6a99ece010000006c493046022100f93bb0e7d8db7bd46e40132d1f8242026e045f03a0efe71bbb8e3f475e970d790221009337cd7f1f929f00cc6ff01f03729b069a7c21b59b1736ddfee5db5946c5da8c0121033b9b137ee87d5a812d6f506efdd37f0affa7ffc310711c06c7f3e097c9447c52ffffffff0100e1f505000000001976a9140389035a9225b3839e2bbf32d826a1e222031fd888ac00000000";

    fn decoded_item(vsize: u64, known_fee_sats: Option<i128>) -> QueueItem {
        QueueItem {
            id: 1,
            name: "test".to_string(),
            origin: "test".to_string(),
            format: RowFormat::RawTx,
            body: QueueItemBody::Decoded {
                vsize,
                txid: "deadbeef".to_string(),
                tx_hex: "00".to_string(),
                known_fee_sats,
                output_sum_sats: 100_000,
            },
            note: None,
            submission: SubmissionState::Unsent,
            total_input_override: None,
        }
    }

    fn analyze_queued(id: u64, name: &str, origin: &str, text: &str) -> QueueItem {
        match analyze(id, name.to_string(), origin.to_string(), text) {
            AnalyzeOutcome::Queued(item) => *item,
            AnalyzeOutcome::UnfinalizablePsbt { .. } => panic!("expected a queued row"),
        }
    }

    #[test]
    fn analyze_valid_raw_tx_hex_decodes() {
        let item = analyze_queued(1, "name", "origin", TX_HEX);
        match item.body {
            QueueItemBody::Decoded {
                vsize,
                txid,
                known_fee_sats,
                output_sum_sats,
                ..
            } => {
                assert!(matches!(item.format, RowFormat::RawTx));
                assert_eq!(vsize, 193);
                assert_eq!(
                    txid,
                    "a6eab3c14ab5272a58a5ba91505ba1a4b6d7a3a9fcbd187b6cd99a7b6d548cb7"
                );
                assert_eq!(known_fee_sats, None);
                assert_eq!(output_sum_sats, 100_000_000);
            }
            QueueItemBody::Invalid => panic!("expected a decoded row"),
        }
    }

    #[test]
    fn analyze_garbage_is_invalid() {
        let item = analyze_queued(1, "name", "origin", "not a psbt or tx");
        assert!(item.is_invalid());
        assert!(matches!(item.format, RowFormat::Unknown));
    }

    /// A one-input PSBT with no witness/redeem data at all can never be
    /// finalized, so `from_psbt` must refuse it via `UnfinalizablePsbt`
    /// rather than queueing it as an `Invalid` row.
    fn unsigned_psbt() -> Psbt {
        use bitcoin::absolute::LockTime;
        use bitcoin::transaction::Version;
        use bitcoin::{OutPoint, ScriptBuf, Sequence, TxIn, Txid, Witness, hashes::Hash};

        let tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::new(Txid::all_zeros(), 0),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new(),
            }],
            output: vec![],
        };
        Psbt::from_unsigned_tx(tx).expect("well-formed unsigned tx")
    }

    #[test]
    fn from_psbt_missing_signatures_is_unfinalizable_not_invalid() {
        let outcome = from_psbt(
            1,
            "unsigned.psbt".to_string(),
            "pasted".to_string(),
            unsigned_psbt(),
        );
        match outcome {
            AnalyzeOutcome::UnfinalizablePsbt { name, reason } => {
                assert_eq!(name, "unsigned.psbt");
                assert!(reason.contains("cannot be finalized"));
            }
            AnalyzeOutcome::Queued(_) => panic!("expected the item to be refused, not queued"),
        }
    }

    #[test]
    fn finalized_psbt_missing_utxo_is_queued_with_validation_warning() {
        let mut psbt = unsigned_psbt();
        psbt.inputs[0].final_script_witness = Some(bitcoin::Witness::from_slice(&[vec![1]]));

        let mut item = analyze_queued(1, "final.psbt", "pasted", &psbt.to_string());
        assert!(matches!(item.format, RowFormat::Psbt));
        assert_eq!(
            item.note.as_ref().expect("unchecked extraction warns").text,
            "Missing UTXO data for 1 input(s). Fee and final scripts could not be validated."
        );
        assert!(shows_input_value_field(&item));

        let output_sum_sats = match item.body {
            QueueItemBody::Decoded {
                known_fee_sats,
                output_sum_sats,
                ..
            } => {
                assert_eq!(known_fee_sats, None);
                output_sum_sats
            }
            QueueItemBody::Invalid => panic!("finalized PSBT should be queued"),
        };
        item.total_input_override = Some(output_sum_sats + 1_000);
        assert_eq!(effective_fee(&item), Some(1_000));
    }

    #[test]
    fn payload_cap_error_flags_oversized_hex() {
        let ok_hex = "00".repeat(1000);
        assert!(payload_cap_error(&ok_hex).is_none());

        let too_big_hex = "00".repeat(MAX_PAYLOAD_BYTES + 1);
        assert!(payload_cap_error(&too_big_hex).is_some());
    }

    #[test]
    fn format_thousands_groups_by_three_digits() {
        assert_eq!(format_thousands(0), "0");
        assert_eq!(format_thousands(999), "999");
        assert_eq!(format_thousands(1_000), "1 000");
        assert_eq!(format_thousands(1_234_567), "1 234 567");
        assert_eq!(format_thousands(-42_000), "-42 000");
    }

    #[test]
    fn short_name_leaves_short_input_unchanged() {
        assert_eq!(short_name("1234567890"), "1234567890");
        assert_eq!(short_name(""), "");
        // Exactly at the 22-char boundary: still unchanged.
        assert_eq!(
            short_name("1234567890123456789012"),
            "1234567890123456789012"
        );
    }

    #[test]
    fn short_name_truncates_long_input() {
        let input = "12345678901234567890123"; // 23 chars, one past the boundary.
        assert_eq!(short_name(input), "12345678901234…67890123");
    }

    #[test]
    fn short_name_strips_whitespace_before_measuring() {
        assert_eq!(short_name("12 34 56 78 90"), "1234567890");
    }

    #[test]
    fn stats_buckets_by_effective_rate_against_floor() {
        let floor = 5.0;
        let items = vec![
            decoded_item(100, Some(1000)), // 10 sat/vB, at/above floor
            decoded_item(100, Some(100)),  // 1 sat/vB, below floor
            decoded_item(100, None),       // fee unknown
        ];
        let stats = stats(&items, floor);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.at_or_above_floor, 1);
        assert_eq!(stats.below_floor, 1);
        assert_eq!(stats.fee_unknown, 1);
    }

    #[test]
    fn row_status_and_rate_color_at_above_and_below_floor() {
        let floor = 5.0;
        let above = decoded_item(100, Some(1000)); // 10 sat/vB
        let at = decoded_item(100, Some(500)); // 5 sat/vB
        let below = decoded_item(100, Some(100)); // 1 sat/vB
        let unknown = decoded_item(100, None);

        assert_eq!(row_status(&above, floor).label, "Ready");
        assert_eq!(rate_color(&above, floor), TEXT_PRIMARY);

        assert_eq!(row_status(&at, floor).label, "Ready");
        assert_eq!(rate_color(&at, floor), TEXT_PRIMARY);

        assert_eq!(row_status(&below, floor).label, "Below floor");
        assert_eq!(rate_color(&below, floor), ERROR_RED);

        assert_eq!(row_status(&unknown, floor).label, "Fee unknown");
        assert_eq!(rate_color(&unknown, floor), TEXT_DISABLED);
    }

    // Same fixture as tx-core's own `tests/fixtures/psbt_signed.base64`.
    const PSBT_B64: &str = "cHNidP8BAHUCAAAAASaBcTce3/KF6Tet7qSze3gADAVmy7OtZGQXE8pCFxv2AAAAAAD+////AtPf9QUAAAAAGXapFNDFmQPFusKGh2DpD9UhpGZap2UgiKwA4fUFAAAAABepFDVF5uM7gyxHBQ8k0+65PJwDlIvHh7MuEwAAAQD9pQEBAAAAAAECiaPHHqtNIOA3G7ukzGmPopXJRjr6Ljl/hTPMti+VZ+UBAAAAFxYAFL4Y0VKpsBIDna89p95PUzSe7LmF/////4b4qkOnHf8USIk6UwpyN+9rRgi7st0tAXHmOuxqSJC0AQAAABcWABT+Pp7xp0XpdNkCxDVZQ6vLNL1TU/////8CAMLrCwAAAAAZdqkUhc/xCX/Z4Ai7NK9wnGIZeziXikiIrHL++E4sAAAAF6kUM5cluiHv1irHU6m80GfWx6ajnQWHAkcwRAIgJxK+IuAnDzlPVoMR3HyppolwuAJf3TskAinwf4pfOiQCIAGLONfc0xTnNMkna9b7QPZzMlvEuqFEyADS8vAtsnZcASED0uFWdJQbrUqZY3LLh+GFbTZSYG2YVi/jnF6efkE/IQUCSDBFAiEA0SuFLYXc2WHS9fSrZgZU327tzHlMDDPOXMMJ/7X85Y0CIGczio4OFyXBl/saiK9Z9R5E5CVbIBZ8hoQDHAXR8lkqASECI7cr7vCWXRC+B3jv7NYfysb3mk6haTkzgHNEZPhPKrMAAAAAAAAA";

    fn gate_of(raw: &str) -> PasteGate {
        paste_gate(&tx_core::split_lines(raw))
    }

    fn label_of(raw: &str) -> &'static str {
        detected_label(&tx_core::split_lines(raw))
    }

    #[test]
    fn paste_gate_is_empty_for_blank_and_comment_only_input() {
        for raw in ["", "   ", "\n\t\n  ", "# note", "# a\n\n#b\n"] {
            assert_eq!(gate_of(raw), PasteGate::Empty, "input {raw:?}");
            assert_eq!(gate_of(raw).refusal(), Some("Nothing pasted yet."));
            assert!(!gate_of(raw).allows_queueing());
        }
    }

    #[test]
    fn paste_gate_refuses_when_no_entry_is_recognised() {
        for raw in [
            "garbage",
            "garbage\nmore garbage",
            "not-hex!\nstill-not-hex!",
        ] {
            assert_eq!(gate_of(raw), PasteGate::NothingRecognised, "input {raw:?}");
            assert!(!gate_of(raw).allows_queueing());
            assert_eq!(
                gate_of(raw).refusal(),
                Some("Not valid base64 PSBT or hex transaction data.")
            );
        }
    }

    #[test]
    fn paste_gate_opens_when_any_entry_is_recognised() {
        let leading_garbage = format!("garbage\n{TX_HEX}");
        let trailing_garbage = format!("{TX_HEX}\ngarbage");
        for raw in [TX_HEX, &leading_garbage, &trailing_garbage] {
            assert_eq!(gate_of(raw), PasteGate::Queueable, "input {raw:?}");
            assert!(gate_of(raw).allows_queueing());
            assert_eq!(gate_of(raw).refusal(), None);
        }
    }

    #[test]
    fn paste_gate_refusal_matches_the_invalid_row_note() {
        let row = analyze_queued(1, "name", "origin", "garbage");
        let note = row.note.expect("garbage is noted on its row").text;
        assert_eq!(PasteGate::NothingRecognised.refusal(), Some(note.as_str()));
    }

    #[test]
    fn detected_label_uses_the_first_recognised_entry() {
        let leading_garbage = format!("garbage\n{TX_HEX}");

        assert_eq!(label_of(""), "");
        assert_eq!(label_of("garbage"), "UNRECOGNISED FORMAT");
        assert_eq!(label_of(TX_HEX), "DETECTED · RAW TRANSACTION");
        assert_eq!(label_of(PSBT_B64), "DETECTED · PSBT");
        assert_eq!(label_of(&leading_garbage), "DETECTED · RAW TRANSACTION");
    }

    #[test]
    fn detected_label_agrees_with_paste_gate() {
        let leading_garbage = format!("garbage\n{TX_HEX}");
        let inputs = [
            "",
            "  \n",
            "# c",
            "garbage",
            "garbage\nmore garbage",
            TX_HEX,
            &leading_garbage,
            PSBT_B64,
        ];
        for raw in inputs {
            assert_eq!(
                gate_of(raw).allows_queueing(),
                label_of(raw).starts_with("DETECTED"),
                "input {raw:?}"
            );
        }
    }
}
