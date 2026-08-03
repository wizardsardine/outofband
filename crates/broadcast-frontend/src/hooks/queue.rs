use yew::prelude::*;

use crate::queue::{self, QueueItem};

/// The whole load-and-queue state: the paste box's text, the queued
/// items, and the callbacks that mutate them. Centralized here so
/// `App` only wires props, and every mutation goes through one place.
#[derive(Clone, PartialEq)]
pub struct QueueHandle {
    pub items: Vec<QueueItem>,
    pub raw_text: String,
    pub parse_error: Option<String>,
    pub broadcasting: bool,
    pub on_raw_text: Callback<String>,
    pub on_submit: Callback<()>,
    pub on_clear: Callback<()>,
    pub on_remove: Callback<u64>,
    pub on_set_total: Callback<(u64, Option<u64>)>,
}

#[hook]
pub fn use_queue() -> QueueHandle {
    let items = use_state(Vec::<QueueItem>::new);
    let next_id = use_state(|| 0u64);
    let raw_text = use_state(String::new);
    let parse_error = use_state(|| None::<String>);
    let broadcasting = use_state(|| false);

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
        Callback::from(move |()| {
            let lines = tx_core::split_lines(&raw_text);
            if let Some(refusal) = queue::paste_gate(&lines).refusal() {
                parse_error.set(Some(refusal.to_string()));
                return;
            }
            let mut id = *next_id;
            let mut added = Vec::with_capacity(lines.len());
            for line in &lines {
                added.push(queue::analyze(
                    id,
                    queue::short_name(line),
                    "pasted".to_string(),
                    line,
                ));
                id += 1;
            }

            // A lone pasted entry that failed to decode renders inline instead
            // of entering the queue as an `Invalid` row. Not made dead by the
            // gate above: the gate only sniffs, so this still catches an entry
            // that sniffs and then fails to decode, such as an oversized hex or
            // a malformed body behind a valid PSBT magic.
            if added.len() == 1 && added[0].is_invalid() {
                let error = added[0].note.as_ref().map(|note| note.text.clone());
                parse_error.set(error);
                return;
            }

            next_id.set(id);
            parse_error.set(None);

            let mut updated = (*items).clone();
            updated.extend(added);
            items.set(updated);

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

    QueueHandle {
        items: (*items).clone(),
        raw_text: (*raw_text).clone(),
        parse_error: (*parse_error).clone(),
        broadcasting: *broadcasting,
        on_raw_text,
        on_submit,
        on_clear,
        on_remove,
        on_set_total,
    }
}
