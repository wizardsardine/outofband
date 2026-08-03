use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use yew::prelude::*;

use crate::hooks::broadcast::{self, SubmitOutcome};
use crate::queue::{
    self, AnalyzeOutcome, NoteCard, NoteKind, QueueItem, QueueItemBody, SubmissionState,
};
use crate::unpack;

/// The whole load-and-queue state: the paste box's text, the queued
/// items, and the callbacks that mutate them. Centralized here so
/// `App` only wires props, and every mutation goes through one place.
#[derive(Clone, PartialEq)]
pub struct QueueHandle {
    pub items: Vec<QueueItem>,
    pub raw_text: String,
    pub parse_error: Option<String>,
    pub broadcasting: bool,
    /// PSBTs refused by the current load operation because they cannot be
    /// finalized: (name, incomplete-input count). Non-empty opens the
    /// finalization modal; a fresh load operation replaces this list
    /// rather than appending to it.
    pub refused_psbts: Vec<(String, String)>,
    pub on_raw_text: Callback<String>,
    pub on_submit: Callback<()>,
    pub on_clear: Callback<()>,
    pub on_remove: Callback<u64>,
    pub on_set_total: Callback<(u64, Option<u64>)>,
    pub on_dismiss_refused: Callback<()>,
    /// Runs every submittable row, strictly sequentially, in queue order.
    pub on_broadcast: Callback<()>,
    /// Re-attempts a single row without touching the rest of the queue.
    pub on_retry: Callback<u64>,
    /// Fed `(name, bytes)` pairs, already ordered lexicographically by
    /// name, by `use_file_load`. Unpacks and analyzes each in turn.
    pub on_files_loaded: Callback<Vec<(String, Vec<u8>)>>,
}

/// Wraps the queue so it can be mutated via [`Reducible`] dispatch rather
/// than `UseStateHandle::set`. A dispatched action is applied against
/// whatever the queue's state actually is when Yew processes it, not a
/// snapshot captured by whoever dispatched it — so the broadcast loop
/// (which can be mid-await for a long time) can never clobber an add,
/// remove, or edit made by another callback while it's in flight.
#[derive(PartialEq)]
struct QueueState(Vec<QueueItem>);

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
        let mut items = self.0.clone();
        match action {
            QueueAction::Extend(new_items) => items.extend(new_items),
            QueueAction::Clear => items.clear(),
            QueueAction::Remove(id) => items.retain(|item| item.id != id),
            QueueAction::SetTotal(id, total) => {
                if let Some(item) = items.iter_mut().find(|item| item.id == id) {
                    item.total_input_override = total;
                }
            }
            QueueAction::SetSubmission {
                id,
                submission,
                note,
            } => {
                if let Some(item) = items.iter_mut().find(|item| item.id == id) {
                    item.submission = submission;
                    match note {
                        NoteUpdate::Unchanged => {}
                        NoteUpdate::Clear => item.note = None,
                        NoteUpdate::Set(note) => item.note = Some(note),
                    }
                }
            }
        }
        Rc::new(QueueState(items))
    }
}

#[hook]
pub fn use_queue() -> QueueHandle {
    let items = use_reducer(|| QueueState(Vec::new()));
    let next_id = use_state(|| 0u64);
    let raw_text = use_state(String::new);
    let parse_error = use_state(|| None::<String>);
    let broadcasting = use_state(|| false);
    let refused_psbts = use_state(Vec::<(String, String)>::new);

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
        let next_id = next_id.clone();
        let parse_error = parse_error.clone();
        let refused_psbts = refused_psbts.clone();
        Callback::from(move |()| {
            let lines = tx_core::split_lines(&raw_text);
            if let Some(refusal) = queue::paste_gate(&lines).refusal() {
                parse_error.set(Some(refusal.to_string()));
                return;
            }
            let mut id = *next_id;
            let mut queued = Vec::with_capacity(lines.len());
            let mut refused = Vec::new();
            for line in &lines {
                match queue::analyze(id, queue::short_name(line), "pasted".to_string(), line) {
                    AnalyzeOutcome::Queued(item) => queued.push(*item),
                    AnalyzeOutcome::UnfinalizablePsbt { name, reason } => {
                            refused.push((name, reason))
                        }
                }
                id += 1;
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

            next_id.set(id);
            parse_error.set(None);
            refused_psbts.set(refused);

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
        Callback::from(move |()| {
            items.dispatch(QueueAction::Clear);
            raw_text.set(String::new());
            parse_error.set(None);
        })
    };

    let on_remove = {
        let items = items.clone();
        Callback::from(move |id: u64| items.dispatch(QueueAction::Remove(id)))
    };

    let on_set_total = {
        let items = items.clone();
        Callback::from(move |(id, total): (u64, Option<u64>)| {
            items.dispatch(QueueAction::SetTotal(id, total));
        })
    };

    let on_dismiss_refused = {
        let refused_psbts = refused_psbts.clone();
        Callback::from(move |()| refused_psbts.set(Vec::new()))
    };

    let on_files_loaded = {
        let items = items.clone();
        let next_id = next_id.clone();
        let refused_psbts = refused_psbts.clone();
        Callback::from(move |files: Vec<(String, Vec<u8>)>| {
            let mut id = *next_id;
            let mut queued = Vec::new();
            let mut refused = Vec::new();

            for (name, bytes) in &files {
                match unpack::unpack(name, bytes) {
                    Ok(unpacked) => {
                        let origin = if unpack::is_archive(bytes) {
                            format!("extracted from {name}")
                        } else {
                            "dropped file".to_string()
                        };
                        for item in unpacked {
                            match queue::analyze(id, item.label, origin.clone(), &item.text) {
                                AnalyzeOutcome::Queued(queue_item) => queued.push(queue_item),
                                AnalyzeOutcome::UnfinalizablePsbt { name, reason } => {
                                        refused.push((name, reason))
                                    }
                            }
                            id += 1;
                        }
                    }
                    Err(err) => {
                        queued.push(queue::unreadable_file(
                            id,
                            name.clone(),
                            "dropped file".to_string(),
                            err.to_string(),
                        ));
                        id += 1;
                    }
                }
            }

            next_id.set(id);
            refused_psbts.set(refused);

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
                .0
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

    let on_retry = {
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
        items: items.0.clone(),
        raw_text: (*raw_text).clone(),
        parse_error: (*parse_error).clone(),
        broadcasting: *broadcasting,
        refused_psbts: (*refused_psbts).clone(),
        on_raw_text,
        on_submit,
        on_dismiss_refused,
        on_broadcast,
        on_retry,
        on_files_loaded,
        on_clear,
        on_remove,
        on_set_total,
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
/// item with a visible countdown and retries it in place; a network
/// failure pauses the whole run so it never silently skips an item.
///
/// Every submission-state update goes through `items.dispatch`, which Yew
/// applies against whatever the queue's live state is at that moment —
/// never a snapshot this run captured earlier — so an add, remove, or edit
/// made by another callback while this run is mid-await is never clobbered.
/// The one exception is `local`, an immutable start-of-run snapshot used
/// solely to look up each id's finalized hex before submitting it; a row
/// can still be removed or already resolved between the moment a run is
/// queued and the moment its turn comes up, which is accepted (see
/// `submittable_tx_hex`) and unrelated to the write-side clobbering this
/// design avoids.
async fn run_broadcast(
    items: UseReducerHandle<QueueState>,
    broadcasting: UseStateHandle<bool>,
    ids: Vec<u64>,
) {
    let local = items.0.clone();
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

/// The item's finalized hex, if it's still in the queue and still
/// submittable — a row can be removed or already resolved between the
/// moment a run is queued and the moment its turn comes up.
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

/// One item's full submission, including transparently retrying past any
/// number of 429s. Returns once the item has a terminal outcome (accepted
/// or rejected) or the run needs to pause on a network failure.
async fn submit_with_rate_limit_retry(
    items: &UseReducerHandle<QueueState>,
    id: u64,
    tx_hex: &str,
) -> RunOutcome {
    loop {
        set_submission(items, id, SubmissionState::Sending, NoteUpdate::Unchanged);
        match broadcast::submit_tx(tx_hex).await {
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
            SubmitOutcome::Failed(message) => {
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
                return RunOutcome::Paused;
            }
            SubmitOutcome::RateLimited(retry_after_secs) => {
                count_down_and_wait(items, id, retry_after_secs).await;
                // Loop back and retry the same item — a 429 means nothing
                // was ever attempted against Slipstream for it.
            }
        }
    }
}

/// Marks the row `Rate limited` with a countdown note that ticks once per
/// second, so the pause is visible rather than a silent stall.
async fn count_down_and_wait(items: &UseReducerHandle<QueueState>, id: u64, retry_after_secs: u64) {
    let mut remaining = retry_after_secs;
    loop {
        let text = if remaining == 0 {
            "Rate limited — retrying now…".to_string()
        } else {
            format!(
                "Rate limited — retrying in {remaining} second{}…",
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
