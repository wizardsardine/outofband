use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use yew::prelude::*;

use crate::hooks::file_load::LoadedFile;
use crate::queue::{
    self, AnalyzeOutcome, NoteCard, NoteKind, QueueItem, QueueItemBody, SubmissionState,
};
use crate::slipstream::{self, SubmitOutcome};
use crate::unpack;

/// Shown once Slipstream has answered 429 past every retry: the browser, not
/// the transaction, is what upstream is refusing.
const RATE_LIMIT_EXHAUSTED: &str =
    "Slipstream is rate limiting this browser. Wait a few minutes and send again.";

/// The whole load-and-queue state: the paste box's text, the queued
/// items, and the callbacks that mutate them. Centralized here so
/// `App` only wires props, and every mutation goes through one place.
#[derive(Clone, PartialEq)]
pub struct QueueHandle {
    pub items: Vec<QueueItem>,
    pub raw_text: String,
    pub parse_error: Option<String>,
    pub broadcasting: bool,
    /// Undismissed PSBTs refused because they cannot be finalized: (name,
    /// reason). Non-empty opens the finalization modal.
    pub refused_psbts: Vec<(String, String)>,
    pub on_raw_text: Callback<String>,
    pub on_submit: Callback<()>,
    pub on_clear: Callback<()>,
    pub on_remove: Callback<u64>,
    pub on_set_total: Callback<(u64, Option<u64>)>,
    pub on_dismiss_refused: Callback<()>,
    /// Runs every submittable row, strictly sequentially, in queue order.
    pub on_broadcast: Callback<()>,
    /// Sends a single row without touching the rest of the queue, whether
    /// or not it has been tried before.
    pub on_send: Callback<u64>,
    /// Fed `(name, bytes)` pairs, already ordered lexicographically by
    /// name, by `use_file_load`. Unpacks and analyzes each in turn.
    pub on_files_loaded: Callback<Vec<LoadedFile>>,
}

/// Wraps the queue so it can be mutated via [`Reducible`] dispatch rather
/// than `UseStateHandle::set`. A dispatched action is applied against
/// whatever the queue's state actually is when Yew processes it, not a
/// snapshot captured by whoever dispatched it — so the broadcast loop
/// (which can be mid-await for a long time) can never clobber an add or
/// edit made by another callback while it's in flight.
#[derive(PartialEq)]
struct QueueState {
    items: Vec<QueueItem>,
    next_id: u64,
}

impl QueueState {
    /// Whether a transaction with this txid is already queued.
    fn is_queued(&self, txid: &str) -> bool {
        self.items.iter().any(|item| item.txid() == Some(txid))
    }
}

/// Appends rows, skipping any transaction already queued. The same hex can
/// reach here twice easily: pasted again, present in two archives, or a
/// file dropped a second time. Two rows for one transaction would submit it
/// twice and show two outcomes for one txid, so the first one wins.
/// Duplicates are matched on the locally derived txid, which means a
/// re-encoded PSBT of an already-queued transaction is caught too.
fn extend_with_ids(state: &mut QueueState, new_items: Vec<QueueItem>) {
    for mut item in new_items {
        if item.txid().is_some_and(|txid| state.is_queued(txid)) {
            continue;
        }
        item.id = state.next_id;
        state.next_id += 1;
        state.items.push(item);
    }
}

enum QueueAction {
    /// Appends newly analyzed rows (from a paste or a file/archive load).
    Extend(Vec<QueueItem>),
    Clear,
    Remove(u64),
    SetTotal(u64, Option<u64>),
    SetSubmission {
        id: u64,
        submission: SubmissionState,
        note: NoteUpdate,
    },
}

impl Reducible for QueueState {
    type Action = QueueAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut state = QueueState {
            items: self.items.clone(),
            next_id: self.next_id,
        };
        match action {
            QueueAction::Extend(new_items) => extend_with_ids(&mut state, new_items),
            QueueAction::Clear => state.items.clear(),
            QueueAction::Remove(id) => state.items.retain(|item| item.id != id),
            QueueAction::SetTotal(id, total) => {
                if let Some(item) = state.items.iter_mut().find(|item| item.id == id) {
                    item.total_input_override = total;
                }
            }
            QueueAction::SetSubmission {
                id,
                submission,
                note,
            } => {
                if let Some(item) = state.items.iter_mut().find(|item| item.id == id) {
                    item.submission = submission;
                    match note {
                        NoteUpdate::Unchanged => {}
                        NoteUpdate::Clear => item.note = None,
                        NoteUpdate::Set(note) => item.note = Some(note),
                    }
                }
            }
        }
        Rc::new(state)
    }
}

#[derive(Default, PartialEq)]
struct RefusedState(Vec<(String, String)>);

enum RefusedAction {
    Extend(Vec<(String, String)>),
    Clear,
}

impl Reducible for RefusedState {
    type Action = RefusedAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut refused = self.0.clone();
        match action {
            RefusedAction::Extend(new_refused) => refused.extend(new_refused),
            RefusedAction::Clear => refused.clear(),
        }
        Rc::new(RefusedState(refused))
    }
}

#[hook]
pub fn use_queue() -> QueueHandle {
    let items = use_reducer(|| QueueState {
        items: Vec::new(),
        next_id: 0,
    });
    let raw_text = use_state(String::new);
    let parse_error = use_state(|| None::<String>);
    let broadcasting = use_state(|| false);
    let refused_psbts = use_reducer(RefusedState::default);

    let on_raw_text = {
        let raw_text = raw_text.clone();
        let parse_error = parse_error.clone();
        Callback::from(move |value: String| {
            raw_text.set(value);
            parse_error.set(None);
        })
    };

    let on_submit = {
        let raw_text = raw_text.clone();
        let items = items.clone();
        let parse_error = parse_error.clone();
        let refused_psbts = refused_psbts.clone();
        Callback::from(move |()| {
            let lines = tx_core::split_lines(&raw_text);
            if let Some(refusal) = queue::paste_gate(&lines).refusal() {
                parse_error.set(Some(refusal.to_string()));
                return;
            }
            let mut queued = Vec::with_capacity(lines.len());
            let mut refused = Vec::new();
            for line in &lines {
                match queue::analyze(0, queue::short_name(line), "pasted".to_string(), line) {
                    AnalyzeOutcome::Queued(item) => queued.push(*item),
                    AnalyzeOutcome::UnfinalizablePsbt { name, reason } => {
                        refused.push((name, reason));
                    }
                }
            }

            // A lone pasted entry that failed to decode renders inline instead
            // of entering the queue as an `Invalid` row. A lone unfinalizable
            // PSBT is a different case entirely, it opens the modal below,
            // never `parse_error`. Not made dead by the gate above: the
            // gate only sniffs, so this still catches an entry that sniffs
            // and then fails to decode, such as an oversized hex.
            if refused.is_empty() && queued.len() == 1 && queued[0].is_invalid() {
                let error = queued[0].note.as_ref().map(|note| note.text.clone());
                parse_error.set(error);
                return;
            }

            // Pasting something already queued would otherwise clear the box
            // and appear to do nothing, since the reducer drops the duplicate.
            if refused.is_empty()
                && !queued.is_empty()
                && queued
                    .iter()
                    .all(|item| item.txid().is_some_and(|txid| items.is_queued(txid)))
            {
                let message = if queued.len() == 1 {
                    "That transaction is already in the queue."
                } else {
                    "Those transactions are already in the queue."
                };
                parse_error.set(Some(message.to_string()));
                return;
            }

            parse_error.set(None);
            if !refused.is_empty() {
                refused_psbts.dispatch(RefusedAction::Extend(refused));
            }

            if !queued.is_empty() {
                items.dispatch(QueueAction::Extend(queued));
            }

            raw_text.set(String::new());
        })
    };

    let on_clear = {
        let items = items.clone();
        let raw_text = raw_text.clone();
        let parse_error = parse_error.clone();
        let broadcasting = broadcasting.clone();
        Callback::from(move |()| {
            if *broadcasting {
                return;
            }
            items.dispatch(QueueAction::Clear);
            raw_text.set(String::new());
            parse_error.set(None);
        })
    };

    let on_remove = {
        let items = items.clone();
        let broadcasting = broadcasting.clone();
        Callback::from(move |id: u64| {
            if !*broadcasting {
                items.dispatch(QueueAction::Remove(id));
            }
        })
    };

    let on_set_total = {
        let items = items.clone();
        Callback::from(move |(id, total): (u64, Option<u64>)| {
            items.dispatch(QueueAction::SetTotal(id, total));
        })
    };

    let on_dismiss_refused = {
        let refused_psbts = refused_psbts.clone();
        Callback::from(move |()| refused_psbts.dispatch(RefusedAction::Clear))
    };

    let on_files_loaded = {
        let items = items.clone();
        let refused_psbts = refused_psbts.clone();
        Callback::from(move |files: Vec<LoadedFile>| {
            let mut queued = Vec::new();
            let mut refused = Vec::new();
            let mut budget = unpack::LoadBudget::new();

            for file in files {
                match file {
                    LoadedFile::Loaded { name, bytes } => {
                        analyze_loaded_file(name, &bytes, &mut budget, &mut queued, &mut refused);
                    }
                    LoadedFile::Failed { name, error } => queued.push(queue::unreadable_file(
                        0,
                        name,
                        "dropped file".to_string(),
                        error,
                    )),
                }
            }

            if !refused.is_empty() {
                refused_psbts.dispatch(RefusedAction::Extend(refused));
            }

            if !queued.is_empty() {
                items.dispatch(QueueAction::Extend(queued));
            }
        })
    };

    let on_broadcast = {
        let items = items.clone();
        let broadcasting = broadcasting.clone();
        Callback::from(move |()| {
            if *broadcasting {
                return;
            }
            let ids: Vec<u64> = items
                .items
                .iter()
                .filter(|item| item.is_submittable())
                .map(|item| item.id)
                .collect();
            if ids.is_empty() {
                return;
            }
            broadcasting.set(true);
            wasm_bindgen_futures::spawn_local(run_broadcast(
                items.clone(),
                broadcasting.clone(),
                ids,
            ));
        })
    };

    let on_send = {
        let items = items.clone();
        let broadcasting = broadcasting.clone();
        Callback::from(move |id: u64| {
            if *broadcasting {
                return;
            }
            broadcasting.set(true);
            wasm_bindgen_futures::spawn_local(run_broadcast(
                items.clone(),
                broadcasting.clone(),
                vec![id],
            ));
        })
    };

    QueueHandle {
        items: items.items.clone(),
        raw_text: (*raw_text).clone(),
        parse_error: (*parse_error).clone(),
        broadcasting: *broadcasting,
        refused_psbts: refused_psbts.0.clone(),
        on_raw_text,
        on_submit,
        on_dismiss_refused,
        on_broadcast,
        on_send,
        on_files_loaded,
        on_clear,
        on_remove,
        on_set_total,
    }
}

fn analyze_loaded_file(
    name: String,
    bytes: &[u8],
    budget: &mut unpack::LoadBudget,
    queued: &mut Vec<QueueItem>,
    refused: &mut Vec<(String, String)>,
) {
    match unpack::unpack(&name, bytes, budget) {
        Ok(unpacked) => {
            let origin = if unpack::is_archive(bytes) {
                format!("extracted from {name}")
            } else {
                "dropped file".to_string()
            };
            for item in unpacked {
                match queue::analyze(0, item.label, origin.clone(), &item.text) {
                    AnalyzeOutcome::Queued(queue_item) => queued.push(*queue_item),
                    AnalyzeOutcome::UnfinalizablePsbt { name, reason } => {
                        refused.push((name, reason));
                    }
                }
            }
        }
        Err(err) => queued.push(queue::unreadable_file(
            0,
            name,
            "dropped file".to_string(),
            err.to_string(),
        )),
    }
}

/// Whether the run should keep going to the next queued id, or stop where
/// it is because the current one needs the user's attention.
enum RunOutcome {
    Continue,
    Paused,
}

/// Submits every id in `ids`, strictly sequentially and awaited one at a
/// time — never `join_all`, never staggered. A 429 pauses just this one
/// item with a visible countdown and retries it in place; a failed request,
/// or a 429 that outlasts the retries, pauses the whole run so it never
/// silently skips an item.
///
/// Submission order is guaranteed by that sequential awaited loop: the next
/// id is only sent once the previous one has fully returned, so a parent
/// always reaches Slipstream before a child spending it.
///
/// Every submission-state update goes through `items.dispatch`, which Yew
/// applies against whatever the queue's live state is at that moment —
/// never a snapshot this run captured earlier — so an add or edit
/// made by another callback while this run is mid-await is never clobbered.
/// `local` is an immutable start-of-run snapshot used solely to look up
/// each id's finalized hex before submitting it.
async fn run_broadcast(
    items: UseReducerHandle<QueueState>,
    broadcasting: UseStateHandle<bool>,
    ids: Vec<u64>,
) {
    let local = items.items.clone();
    for id in ids {
        let Some(tx_hex) = submittable_tx_hex(&local, id) else {
            continue;
        };
        match submit_with_rate_limit_retry(&items, id, &tx_hex).await {
            RunOutcome::Continue => continue,
            RunOutcome::Paused => break,
        }
    }
    broadcasting.set(false);
}

/// The item's finalized hex from the start-of-run snapshot, if it was
/// submittable when the run began.
fn submittable_tx_hex(items: &[QueueItem], id: u64) -> Option<String> {
    let item = items.iter().find(|item| item.id == id)?;
    if !item.is_submittable() {
        return None;
    }
    match &item.body {
        QueueItemBody::Decoded { tx_hex, .. } => Some(tx_hex.clone()),
        QueueItemBody::Invalid => None,
    }
}

/// One item's full submission, retrying a 429 for as long as
/// [`slipstream::retry_delay`] allows. Returns once the item has a terminal
/// outcome (accepted or rejected) or the run needs to pause, either on a
/// failed request or once the rate-limit retries are spent.
async fn submit_with_rate_limit_retry(
    items: &UseReducerHandle<QueueState>,
    id: u64,
    tx_hex: &str,
) -> RunOutcome {
    let mut attempt = 0;
    loop {
        set_submission(items, id, SubmissionState::Sending, NoteUpdate::Unchanged);
        match slipstream::submit_tx(tx_hex).await {
            SubmitOutcome::Accepted => {
                // Clears any leftover rate-limit countdown note from a
                // 429 this same item recovered from.
                set_submission(items, id, SubmissionState::Accepted, NoteUpdate::Clear);
                return RunOutcome::Continue;
            }
            SubmitOutcome::Rejected(message) => {
                let note = NoteCard {
                    kind: NoteKind::Error,
                    text: message.clone(),
                };
                set_submission(
                    items,
                    id,
                    SubmissionState::Rejected(message),
                    NoteUpdate::Set(note),
                );
                return RunOutcome::Continue;
            }
            SubmitOutcome::Failed(message) => return fail_and_pause(items, id, message),
            SubmitOutcome::RateLimited(suggested) => {
                let Some(delay_secs) = slipstream::retry_delay(attempt, suggested) else {
                    return fail_and_pause(items, id, RATE_LIMIT_EXHAUSTED.to_string());
                };
                attempt += 1;
                count_down_and_wait(items, id, delay_secs).await;
                // Loop back and retry the same item: a 429 means nothing was
                // ever attempted against Slipstream for it.
            }
        }
    }
}

/// Marks the row failed with its reason and stops the run there, so it never
/// silently skips past an item the user has to look at.
fn fail_and_pause(items: &UseReducerHandle<QueueState>, id: u64, message: String) -> RunOutcome {
    let note = NoteCard {
        kind: NoteKind::Error,
        text: message.clone(),
    };
    set_submission(
        items,
        id,
        SubmissionState::Failed(message),
        NoteUpdate::Set(note),
    );
    RunOutcome::Paused
}

/// Marks the row `Rate limited` with a countdown note that ticks once per
/// second, so the pause is visible rather than a silent stall.
async fn count_down_and_wait(items: &UseReducerHandle<QueueState>, id: u64, delay_secs: u64) {
    let mut remaining = delay_secs;
    loop {
        let text = if remaining == 0 {
            "Rate limited, retrying now…".to_string()
        } else {
            format!(
                "Rate limited, retrying in {remaining} second{}…",
                if remaining == 1 { "" } else { "s" }
            )
        };
        let note = NoteCard {
            kind: NoteKind::Warn,
            text,
        };
        set_submission(
            items,
            id,
            SubmissionState::RateLimited,
            NoteUpdate::Set(note),
        );
        if remaining == 0 {
            return;
        }
        TimeoutFuture::new(1_000).await;
        remaining -= 1;
    }
}

/// What a submission-state transition does to a row's note card.
enum NoteUpdate {
    /// Leave whatever note is already there (e.g. the finalization note)
    /// untouched — used while a row is merely `Sending`.
    Unchanged,
    /// Drop any note — used on `Accepted`, so a rate-limit countdown a row
    /// recovered from doesn't linger under a successful row.
    Clear,
    Set(NoteCard),
}

/// Updates one item's submission state and note card by dispatching against
/// the live queue state, so it can never overwrite a concurrent edit made
/// elsewhere while this run is in flight.
fn set_submission(
    items: &UseReducerHandle<QueueState>,
    id: u64,
    submission: SubmissionState,
    note: NoteUpdate,
) {
    items.dispatch(QueueAction::SetSubmission {
        id,
        submission,
        note,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::RowFormat;

    fn item(name: &str) -> QueueItem {
        QueueItem::invalid(
            0,
            name.to_string(),
            "test".to_string(),
            RowFormat::Unknown,
            "invalid".to_string(),
        )
    }

    #[test]
    fn extending_assigns_unique_ids_in_dispatch_order() {
        let mut state = QueueState {
            items: Vec::new(),
            next_id: 0,
        };

        extend_with_ids(&mut state, vec![item("first"), item("second")]);
        extend_with_ids(&mut state, vec![item("third")]);

        assert_eq!(
            state
                .items
                .iter()
                .map(|item| (item.id, item.name.as_str()))
                .collect::<Vec<_>>(),
            vec![(0, "first"), (1, "second"), (2, "third")]
        );
    }

    #[test]
    fn extending_skips_a_transaction_already_queued() {
        fn decoded(txid: &str) -> QueueItem {
            QueueItem {
                id: 0,
                name: txid.to_string(),
                origin: "test".to_string(),
                format: RowFormat::RawTx,
                body: QueueItemBody::Decoded {
                    vsize: 100,
                    txid: txid.to_string(),
                    tx_hex: "00".to_string(),
                    known_fee_sats: None,
                    output_sum_sats: 0,
                },
                note: None,
                submission: SubmissionState::Unsent,
                total_input_override: None,
            }
        }

        let mut state = QueueState {
            items: Vec::new(),
            next_id: 0,
        };

        extend_with_ids(&mut state, vec![decoded("aa"), decoded("bb")]);
        extend_with_ids(&mut state, vec![decoded("aa"), decoded("cc")]);
        // Two invalid rows share no txid, so neither is a duplicate.
        extend_with_ids(&mut state, vec![item("bad"), item("bad")]);

        assert_eq!(
            state
                .items
                .iter()
                .map(|item| item.txid().unwrap_or("none"))
                .collect::<Vec<_>>(),
            vec!["aa", "bb", "cc", "none", "none"]
        );
    }

    #[test]
    fn refusals_accumulate_until_dismissed() {
        let state = Rc::new(RefusedState::default()).reduce(RefusedAction::Extend(vec![(
            "first".to_string(),
            "reason one".to_string(),
        )]));
        let state = state.reduce(RefusedAction::Extend(vec![(
            "second".to_string(),
            "reason two".to_string(),
        )]));

        assert_eq!(
            state.0,
            vec![
                ("first".to_string(), "reason one".to_string()),
                ("second".to_string(), "reason two".to_string()),
            ]
        );

        let state = state.reduce(RefusedAction::Clear);
        assert!(state.0.is_empty());
    }
}
