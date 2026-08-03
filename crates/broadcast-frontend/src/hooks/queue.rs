use yew::prelude::*;

use crate::queue::{self, AnalyzeOutcome, QueueItem};
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
    pub refused_psbts: Vec<(String, usize)>,
    pub on_raw_text: Callback<String>,
    pub on_submit: Callback<()>,
    pub on_clear: Callback<()>,
    pub on_remove: Callback<u64>,
    pub on_set_total: Callback<(u64, Option<u64>)>,
    pub on_dismiss_refused: Callback<()>,
    /// Fed `(name, bytes)` pairs, already ordered lexicographically by
    /// name, by `use_file_load`. Unpacks and analyzes each in turn.
    pub on_files_loaded: Callback<Vec<(String, Vec<u8>)>>,
}

#[hook]
pub fn use_queue() -> QueueHandle {
    let items = use_state(Vec::<QueueItem>::new);
    let next_id = use_state(|| 0u64);
    let raw_text = use_state(String::new);
    let parse_error = use_state(|| None::<String>);
    let broadcasting = use_state(|| false);
    let refused_psbts = use_state(Vec::<(String, usize)>::new);

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
                    AnalyzeOutcome::Queued(item) => queued.push(item),
                    AnalyzeOutcome::UnfinalizablePsbt {
                        name,
                        incomplete_inputs,
                    } => refused.push((name, incomplete_inputs)),
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
                let mut updated = (*items).clone();
                updated.extend(queued);
                items.set(updated);
            }

            raw_text.set(String::new());
        })
    };

    let on_clear = {
        let items = items.clone();
        let raw_text = raw_text.clone();
        let parse_error = parse_error.clone();
        Callback::from(move |()| {
            items.set(Vec::new());
            raw_text.set(String::new());
            parse_error.set(None);
        })
    };

    let on_remove = {
        let items = items.clone();
        Callback::from(move |id: u64| {
            let updated: Vec<QueueItem> = (*items)
                .iter()
                .filter(|item| item.id != id)
                .cloned()
                .collect();
            items.set(updated);
        })
    };

    let on_set_total = {
        let items = items.clone();
        Callback::from(move |(id, total): (u64, Option<u64>)| {
            let updated: Vec<QueueItem> = (*items)
                .iter()
                .cloned()
                .map(|mut item| {
                    if item.id == id {
                        item.total_input_override = total;
                    }
                    item
                })
                .collect();
            items.set(updated);
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
                                AnalyzeOutcome::UnfinalizablePsbt {
                                    name,
                                    incomplete_inputs,
                                } => refused.push((name, incomplete_inputs)),
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
                let mut updated = (*items).clone();
                updated.extend(queued);
                items.set(updated);
            }
        })
    };

    QueueHandle {
        items: (*items).clone(),
        raw_text: (*raw_text).clone(),
        parse_error: (*parse_error).clone(),
        broadcasting: *broadcasting,
        refused_psbts: (*refused_psbts).clone(),
        on_raw_text,
        on_submit,
        on_dismiss_refused,
        on_files_loaded,
        on_clear,
        on_remove,
        on_set_total,
    }
}
