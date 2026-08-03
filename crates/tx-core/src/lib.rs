//! Format detection and decoding for PSBTs and raw Bitcoin transactions.
//!
//! Everything here works on plain bytes: no filesystem, no network, no
//! filename or extension ever enters the decision. This is what turns
//! arbitrary user input (a paste, a dropped file, an archive entry) into a
//! typed `bitcoin` value, or a precise, displayable error.

use core::fmt;
use core::str::{self, FromStr};

use bitcoin::base64::Engine as _;
use bitcoin::hex::FromHex;
use bitcoin::psbt::PsbtParseError;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Psbt, Transaction, consensus};
use miniscript::psbt::PsbtExt;

/// Leading magic of a raw binary PSBT: ASCII `psbt` followed by `0xff`.
const PSBT_MAGIC: &[u8] = b"psbt\xff";

/// The four accepted encodings for a transaction or PSBT, detected by
/// content only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Raw binary PSBT, leading magic `psbt\xff`.
    PsbtBinary,
    /// Base64-encoded PSBT text.
    PsbtBase64,
    /// Hex-encoded transaction text.
    TxHex,
    /// Raw binary, consensus-encoded transaction.
    TxBinary,
}

/// A successfully decoded PSBT or transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decoded {
    Psbt(Psbt),
    Transaction(Transaction),
}

/// Everything that can go wrong turning input bytes into a [`Decoded`]
/// value. Every variant renders a message fit for a queue row's error note
/// card.
#[derive(Debug)]
pub enum TxCoreError {
    /// The input matches none of the four accepted formats.
    UnrecognisedFormat,
    /// Text claiming to be a base64-encoded PSBT is not valid base64.
    MalformedBase64(bitcoin::base64::DecodeError),
    /// Text claiming to be hex-encoded is not valid hex.
    MalformedHex(bitcoin::hex::HexToBytesError),
    /// The bytes are not a well-formed PSBT.
    PsbtDeserialize(bitcoin::psbt::Error),
    /// The bytes are not a well-formed transaction.
    TransactionDeserialize(consensus::encode::Error),
}

impl fmt::Display for TxCoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxCoreError::UnrecognisedFormat => {
                write!(f, "not a recognised PSBT or transaction")
            }
            TxCoreError::MalformedBase64(e) => write!(f, "not valid base64: {e}"),
            TxCoreError::MalformedHex(e) => write!(f, "not valid hex: {e}"),
            TxCoreError::PsbtDeserialize(e) => write!(f, "malformed PSBT: {e}"),
            TxCoreError::TransactionDeserialize(e) => write!(f, "malformed transaction: {e}"),
        }
    }
}

impl std::error::Error for TxCoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TxCoreError::UnrecognisedFormat => None,
            TxCoreError::MalformedBase64(e) => Some(e),
            TxCoreError::MalformedHex(e) => Some(e),
            TxCoreError::PsbtDeserialize(e) => Some(e),
            TxCoreError::TransactionDeserialize(e) => Some(e),
        }
    }
}

/// Detects the format of `input` by content only: never by filename or
/// extension. Checks, in order, the binary PSBT magic, base64 PSBT text,
/// hex text, then consensus deserialization for binary transactions.
pub fn detect(input: &[u8]) -> Option<Format> {
    if input.starts_with(PSBT_MAGIC) {
        return Some(Format::PsbtBinary);
    }

    if let Ok(text) = str::from_utf8(input) {
        let text = text.trim();
        if bitcoin::base64::engine::general_purpose::STANDARD
            .decode(text)
            .is_ok_and(|bytes| bytes.starts_with(PSBT_MAGIC))
            || text.starts_with("cHNidP")
        {
            return Some(Format::PsbtBase64);
        }

        if !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            if Vec::<u8>::from_hex(text).is_ok_and(|bytes| bytes.starts_with(PSBT_MAGIC)) {
                return None;
            }
            return Some(Format::TxHex);
        }
    }

    consensus::deserialize::<Transaction>(input)
        .ok()
        .map(|_| Format::TxBinary)
}

/// Detects the format of `input`, then decodes it into a [`Decoded`] value.
pub fn decode(input: &[u8]) -> Result<Decoded, TxCoreError> {
    let format = detect(input).ok_or(TxCoreError::UnrecognisedFormat)?;
    decode_as(format, input)
}

/// Decodes `input` as the given `format`, without re-detecting it.
pub fn decode_as(format: Format, input: &[u8]) -> Result<Decoded, TxCoreError> {
    match format {
        Format::PsbtBinary => decode_psbt_binary(input),
        Format::PsbtBase64 => decode_psbt_base64(input),
        Format::TxHex => decode_tx_hex(input),
        Format::TxBinary => decode_tx_binary(input),
    }
}

fn decode_psbt_binary(bytes: &[u8]) -> Result<Decoded, TxCoreError> {
    Psbt::deserialize(bytes)
        .map(Decoded::Psbt)
        .map_err(TxCoreError::PsbtDeserialize)
}

fn decode_psbt_base64(bytes: &[u8]) -> Result<Decoded, TxCoreError> {
    let text = str::from_utf8(bytes)
        .map_err(|_| TxCoreError::UnrecognisedFormat)?
        .trim();
    Psbt::from_str(text)
        .map(Decoded::Psbt)
        .map_err(|e| match e {
            PsbtParseError::Base64Encoding(err) => TxCoreError::MalformedBase64(err),
            PsbtParseError::PsbtEncoding(err) => TxCoreError::PsbtDeserialize(err),
            _ => TxCoreError::UnrecognisedFormat,
        })
}

fn decode_tx_hex(bytes: &[u8]) -> Result<Decoded, TxCoreError> {
    let text = str::from_utf8(bytes)
        .map_err(|_| TxCoreError::UnrecognisedFormat)?
        .trim();
    let decoded = Vec::<u8>::from_hex(text).map_err(TxCoreError::MalformedHex)?;
    consensus::deserialize(&decoded)
        .map(Decoded::Transaction)
        .map_err(TxCoreError::TransactionDeserialize)
}

fn decode_tx_binary(bytes: &[u8]) -> Result<Decoded, TxCoreError> {
    consensus::deserialize(bytes)
        .map(Decoded::Transaction)
        .map_err(TxCoreError::TransactionDeserialize)
}

/// A transaction cannot be relayed whatever its contents are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuralError {
    NoInputs,
    NoOutputs,
}

impl fmt::Display for StructuralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoInputs => write!(f, "transaction has no inputs"),
            Self::NoOutputs => write!(f, "transaction has no outputs"),
        }
    }
}

impl std::error::Error for StructuralError {}

/// Rejects a transaction no node would ever accept: one with no inputs, or
/// one with no outputs. Both deserialize fine, so decoding alone never
/// catches them.
pub fn check_structure(tx: &Transaction) -> Result<(), StructuralError> {
    if tx.input.is_empty() {
        return Err(StructuralError::NoInputs);
    }
    if tx.output.is_empty() {
        return Err(StructuralError::NoOutputs);
    }
    Ok(())
}

/// Splits multi-entry text input into individual entries: one per non-blank
/// line, skipping lines whose first non-whitespace character is `#`.
pub fn split_lines(input: &str) -> Vec<&str> {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// The signature state of one PSBT input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputSignatureState {
    /// `final_script_sig` or `final_script_witness` is already present.
    pub finalized: bool,
    /// Number of partial signatures collected so far.
    pub partial_sig_count: usize,
}

/// Per-input signature analysis of a PSBT, in input order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureAnalysis {
    pub inputs: Vec<InputSignatureState>,
}

impl SignatureAnalysis {
    /// Number of inputs not yet finalized.
    pub fn incomplete_inputs(&self) -> usize {
        self.inputs.iter().filter(|input| !input.finalized).count()
    }
}

/// Reports, for each input of `psbt`, whether it is already finalized or
/// how many partial signatures it carries.
pub fn analyze_signatures(psbt: &Psbt) -> SignatureAnalysis {
    let inputs = psbt
        .inputs
        .iter()
        .map(|input| InputSignatureState {
            finalized: input.final_script_sig.is_some() || input.final_script_witness.is_some(),
            partial_sig_count: input.partial_sigs.len(),
        })
        .collect();
    SignatureAnalysis { inputs }
}

/// A PSBT has inconsistent map or UTXO data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PsbtValidationError {
    InputCountMismatch {
        transaction: usize,
        maps: usize,
    },
    OutputCountMismatch {
        transaction: usize,
        maps: usize,
    },
    NonWitnessTxidMismatch {
        input: usize,
    },
    NonWitnessVoutOutOfBounds {
        input: usize,
        vout: u32,
        outputs: usize,
    },
    WitnessUtxoMismatch {
        input: usize,
    },
}

impl fmt::Display for PsbtValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputCountMismatch { transaction, maps } => write!(
                f,
                "PSBT has {transaction} transaction input(s) but {maps} input map(s)"
            ),
            Self::OutputCountMismatch { transaction, maps } => write!(
                f,
                "PSBT has {transaction} transaction output(s) but {maps} output map(s)"
            ),
            Self::NonWitnessTxidMismatch { input } => {
                write!(f, "PSBT input {input} non-witness UTXO has the wrong txid")
            }
            Self::NonWitnessVoutOutOfBounds {
                input,
                vout,
                outputs,
            } => write!(
                f,
                "PSBT input {input} references vout {vout}, but its non-witness UTXO has {outputs} output(s)"
            ),
            Self::WitnessUtxoMismatch { input } => write!(
                f,
                "PSBT input {input} witness and non-witness UTXOs do not match"
            ),
        }
    }
}

impl std::error::Error for PsbtValidationError {}

fn validate_psbt_utxos(psbt: &Psbt) -> Result<(), PsbtValidationError> {
    if psbt.unsigned_tx.input.len() != psbt.inputs.len() {
        return Err(PsbtValidationError::InputCountMismatch {
            transaction: psbt.unsigned_tx.input.len(),
            maps: psbt.inputs.len(),
        });
    }
    if psbt.unsigned_tx.output.len() != psbt.outputs.len() {
        return Err(PsbtValidationError::OutputCountMismatch {
            transaction: psbt.unsigned_tx.output.len(),
            maps: psbt.outputs.len(),
        });
    }

    for (index, (tx_in, input)) in psbt
        .unsigned_tx
        .input
        .iter()
        .zip(psbt.inputs.iter())
        .enumerate()
    {
        let Some(previous_tx) = &input.non_witness_utxo else {
            continue;
        };
        if previous_tx.compute_txid() != tx_in.previous_output.txid {
            return Err(PsbtValidationError::NonWitnessTxidMismatch { input: index });
        }
        let referenced_output = previous_tx
            .output
            .get(tx_in.previous_output.vout as usize)
            .ok_or(PsbtValidationError::NonWitnessVoutOutOfBounds {
                input: index,
                vout: tx_in.previous_output.vout,
                outputs: previous_tx.output.len(),
            })?;
        if input
            .witness_utxo
            .as_ref()
            .is_some_and(|witness_utxo| witness_utxo != referenced_output)
        {
            return Err(PsbtValidationError::WitnessUtxoMismatch { input: index });
        }
    }

    Ok(())
}

/// A PSBT could not be finalized or its finalized scripts failed validation.
#[derive(Debug)]
pub enum FinalizeError {
    InvalidPsbt(PsbtValidationError),
    Miniscript(Vec<miniscript::psbt::Error>),
}

impl fmt::Display for FinalizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPsbt(error) => write!(f, "PSBT cannot be finalized: {error}"),
            Self::Miniscript(errors) if errors.is_empty() => {
                write!(f, "PSBT cannot be finalized: no cause was provided")
            }
            Self::Miniscript(errors) if errors.len() == 1 => {
                write!(f, "PSBT cannot be finalized: {}", errors[0])
            }
            Self::Miniscript(errors) => write!(
                f,
                "PSBT cannot be finalized: {} input(s) failed; first error: {}",
                errors.len(),
                errors[0]
            ),
        }
    }
}

impl std::error::Error for FinalizeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPsbt(error) => Some(error),
            Self::Miniscript(errors) => errors
                .first()
                .map(|error| error as &(dyn std::error::Error + 'static)),
        }
    }
}

/// A transaction extracted from a finalized PSBT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalizedPsbt {
    /// Every input's final scripts passed miniscript's interpreter checks.
    Validated(Transaction),
    /// Missing UTXO data prevented final-script and fee validation.
    Unchecked {
        transaction: Transaction,
        missing_utxo_inputs: usize,
    },
}

/// Finalizes `psbt` and extracts its network-serialized transaction.
///
/// Already-finalized inputs are left untouched, inputs with complete
/// signatures are finalized in place, and if any input is still missing
/// signatures afterward this returns [`FinalizeError`] with the miniscript
/// causes rather than extracting a partial transaction.
///
/// Uses miniscript's checked extractor when every input has UTXO data. An
/// already-finalized PSBT missing UTXO data is extracted without fee-rate or
/// final-script validation, at the same trust level as an accepted raw
/// transaction.
pub fn finalize(mut psbt: Psbt) -> Result<FinalizedPsbt, FinalizeError> {
    validate_psbt_utxos(&psbt).map_err(FinalizeError::InvalidPsbt)?;

    let missing_utxo_inputs = psbt
        .inputs
        .iter()
        .filter(|input| input.witness_utxo.is_none() && input.non_witness_utxo.is_none())
        .count();
    let finalized = psbt
        .inputs
        .iter()
        .all(|input| input.final_script_sig.is_some() || input.final_script_witness.is_some());
    if missing_utxo_inputs > 0 && finalized {
        return Ok(FinalizedPsbt::Unchecked {
            transaction: psbt.extract_tx_unchecked_fee_rate(),
            missing_utxo_inputs,
        });
    }

    let secp = Secp256k1::new();
    psbt.finalize_mut(&secp)
        .map_err(FinalizeError::Miniscript)?;
    psbt.extract(&secp)
        .map(FinalizedPsbt::Validated)
        .map_err(|error| FinalizeError::Miniscript(vec![error]))
}

/// Network-serializes `tx`: the bytes ever sent to the server.
pub fn serialize(tx: &Transaction) -> Vec<u8> {
    consensus::serialize(tx)
}

/// Hex-encodes the network serialization of `tx`.
pub fn serialize_hex(tx: &Transaction) -> String {
    consensus::encode::serialize_hex(tx)
}

/// Exact virtual size of `tx`, in vbytes. Every queued item is finalized
/// before this is called, so the result is always exact — there is no
/// estimation path.
pub fn vsize(tx: &Transaction) -> u64 {
    tx.vsize() as u64
}

/// An exact amount sum exceeded the supported satoshi range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmountOverflowError;

impl fmt::Display for AmountOverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "amount sum exceeds u64 satoshi range")
    }
}

impl std::error::Error for AmountOverflowError {}

fn sum_sats<I: IntoIterator<Item = bitcoin::Amount>>(
    amounts: I,
) -> Result<u64, AmountOverflowError> {
    amounts.into_iter().try_fold(0u64, |sum, amount| {
        sum.checked_add(amount.to_sat()).ok_or(AmountOverflowError)
    })
}

/// Sum of `tx`'s output values, in satoshis.
pub fn output_sum(tx: &Transaction) -> Result<u64, AmountOverflowError> {
    sum_sats(tx.output.iter().map(|out| out.value))
}

/// Sum of a PSBT's unsigned-transaction output values, in satoshis:
/// available whether or not the PSBT's inputs carry UTXO data, so a
/// partial-UTXO PSBT is never stranded without a known output sum.
pub fn psbt_output_sum(psbt: &Psbt) -> Result<u64, PsbtAnalysisError> {
    validate_psbt_utxos(psbt)?;
    output_sum(&psbt.unsigned_tx).map_err(Into::into)
}

/// A PSBT cannot be analyzed exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PsbtAnalysisError {
    InvalidPsbt(PsbtValidationError),
    AmountOverflow,
}

impl fmt::Display for PsbtAnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPsbt(error) => error.fmt(f),
            Self::AmountOverflow => AmountOverflowError.fmt(f),
        }
    }
}

impl std::error::Error for PsbtAnalysisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPsbt(error) => Some(error),
            Self::AmountOverflow => None,
        }
    }
}

impl From<PsbtValidationError> for PsbtAnalysisError {
    fn from(error: PsbtValidationError) -> Self {
        Self::InvalidPsbt(error)
    }
}

impl From<AmountOverflowError> for PsbtAnalysisError {
    fn from(_: AmountOverflowError) -> Self {
        Self::AmountOverflow
    }
}

/// The known-input-value state of a PSBT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSum {
    /// Every input carried UTXO data; this is the exact sum, in satoshis.
    Known(u64),
    /// At least one input is missing UTXO data; carries how many.
    Unknown { missing_utxo_inputs: usize },
}

/// Sums a PSBT's per-input UTXO values: `witness_utxo.value`, or the
/// referenced output of `non_witness_utxo`. If any input has neither,
/// returns [`InputSum::Unknown`] with the count missing rather than an
/// exact sum over a subset — a partial sum would silently understate the
/// fee.
pub fn psbt_input_sum(psbt: &Psbt) -> Result<InputSum, PsbtAnalysisError> {
    validate_psbt_utxos(psbt)?;
    let mut sum = 0u64;
    let mut missing = 0usize;

    for (tx_in, input) in psbt.unsigned_tx.input.iter().zip(psbt.inputs.iter()) {
        let value = input
            .witness_utxo
            .as_ref()
            .map(|utxo| utxo.value)
            .or_else(|| {
                input
                    .non_witness_utxo
                    .as_ref()
                    .and_then(|prev_tx| prev_tx.output.get(tx_in.previous_output.vout as usize))
                    .map(|out| out.value)
            });
        match value {
            Some(value) => {
                sum = sum
                    .checked_add(value.to_sat())
                    .ok_or(PsbtAnalysisError::AmountOverflow)?;
            }
            None => missing += 1,
        }
    }

    if missing > 0 {
        Ok(InputSum::Unknown {
            missing_utxo_inputs: missing,
        })
    } else {
        Ok(InputSum::Known(sum))
    }
}

/// The fee analysis of a PSBT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PsbtFee {
    /// Every input carried UTXO data; this is the exact fee, in satoshis.
    /// May be negative for an inconsistent PSBT (outputs exceeding
    /// inputs) — reported, not rejected.
    Known { fee_sats: i128 },
    /// At least one input is missing UTXO data; carries how many.
    Unknown { missing_utxo_inputs: usize },
}

/// Derives a PSBT's fee from its inputs' UTXO data and its outputs.
pub fn psbt_fee(psbt: &Psbt) -> Result<PsbtFee, PsbtAnalysisError> {
    match psbt_input_sum(psbt)? {
        InputSum::Known(input_sum) => {
            let output_sum = output_sum(&psbt.unsigned_tx)?;
            Ok(PsbtFee::Known {
                fee_sats: input_sum as i128 - output_sum as i128,
            })
        }
        InputSum::Unknown {
            missing_utxo_inputs,
        } => Ok(PsbtFee::Unknown {
            missing_utxo_inputs,
        }),
    }
}

/// Fee and known output sum derived from a user-entered total input value:
/// used for raw transactions (previous outputs unknown) and PSBTs missing
/// UTXO data on some inputs, both of which otherwise show no fee at all.
/// The fee may be negative if the entered total understates the output
/// sum; that is reported, never rejected or panicked on.
pub fn fee_from_user_total(total_input_sats: u64, output_sum_sats: u64) -> i128 {
    total_input_sats as i128 - output_sum_sats as i128
}

/// Fee rate in sat/vB, computed as `fee / vsize`. `None` if `vsize` is
/// zero: there is no meaningful rate to report, rather than an infinite
/// one.
pub fn fee_rate(fee_sats: i128, vsize: u64) -> Option<f64> {
    if vsize == 0 {
        return None;
    }
    Some(fee_sats as f64 / vsize as f64)
}

/// A fee rate's relationship to the live floor: drives the queue row's
/// Ready / Below floor / Fee unknown label only. Never a gate — every
/// queued item remains submittable regardless of this result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorComparison {
    /// The rate meets or exceeds the floor.
    AtOrAbove,
    /// The rate is below the floor.
    Below,
    /// No rate could be derived.
    Unknown,
}

/// Compares `rate` (sat/vB) against `floor` (sat/vB).
pub fn compare_to_floor(rate: Option<f64>, floor: f64) -> FloorComparison {
    match rate {
        Some(rate) if rate >= floor => FloorComparison::AtOrAbove,
        Some(_) => FloorComparison::Below,
        None => FloorComparison::Unknown,
    }
}
