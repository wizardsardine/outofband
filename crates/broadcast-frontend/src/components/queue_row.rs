use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::queue::{self, NoteCard, QueueItem, QueueItemBody, RowStatusView, SubmissionState};
use crate::tokens::{
    ACCENT_TEAL_BRIGHT, BLOCK_EXPLORER_TX_URL, BODY_COPY, BORDER_STRONG, CARD_NESTED, FIELD_TEXT,
    HAIRLINE, QUEUE_ROW_COLUMNS, SURFACE_ACCEPTED, TEXT_DISABLED, TEXT_MUTED_6A, TEXT_PRIMARY,
    TEXT_SECONDARY,
};

#[derive(Properties, PartialEq)]
pub struct QueueRowProps {
    pub item: QueueItem,
    pub floor: f64,
    pub mobile: bool,
    pub broadcasting: bool,
    pub on_remove: Callback<u64>,
    pub on_set_total: Callback<(u64, Option<u64>)>,
    pub on_retry: Callback<u64>,
}

#[function_component(QueueRow)]
pub fn queue_row(props: &QueueRowProps) -> Html {
    let item = &props.item;
    let status = queue::row_status(item, props.floor);
    let accepted = matches!(item.submission, SubmissionState::Accepted);

    let row_style = format!(
        "padding:18px 26px;border-bottom:1px solid {HAIRLINE};background:{}",
        if accepted {
            SURFACE_ACCEPTED
        } else {
            "transparent"
        }
    );

    let onclick_remove = {
        let on_remove = props.on_remove.clone();
        let id = item.id;
        Callback::from(move |_| on_remove.emit(id))
    };

    let retryable = matches!(
        item.submission,
        SubmissionState::Rejected(_) | SubmissionState::Failed(_)
    );

    html! {
        <div style={row_style}>
            <div style={row_grid_style(props.mobile)}>
                <span style={dot_style(&status)}></span>
                <div style={name_cell_style(props.mobile)}>
                    <div style={format!("font-family:'IBM Plex Mono',monospace;font-size:13px;color:{TEXT_PRIMARY};overflow:hidden;text-overflow:ellipsis;white-space:nowrap")}>{item.name.clone()}</div>
                    <div style={format!("font-size:11.5px;color:{TEXT_MUTED_6A};margin-top:3px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap")}>{item.origin.clone()}</div>
                </div>
                <span style={format_chip_style()}>{item.format.label()}</span>
                <span style={format!("font-family:'IBM Plex Mono',monospace;font-size:13px;color:{TEXT_SECONDARY};text-align:right")}>{vsize_text(item)}</span>
                <div style="text-align:right">
                    <div style={format!("font-family:'IBM Plex Mono',monospace;font-size:17px;font-weight:600;color:{}", queue::rate_color(item, props.floor))}>{rate_text(item)}</div>
                    <div style={format!("font-family:'IBM Plex Mono',monospace;font-size:10.5px;color:{TEXT_MUTED_6A};margin-top:2px")}>{fee_sub_line(item)}</div>
                </div>
                <div style="display:flex;align-items:center;justify-content:flex-end;gap:8px">
                    <span style={format!("font-size:12px;font-weight:500;color:{}", status.text_color)}>{status.label}</span>
                    if retryable {
                        { retry_button(item.id, props.broadcasting, &props.on_retry) }
                    }
                </div>
                <button onclick={onclick_remove} style={remove_button_style()} class="remove-btn">{"×"}</button>
            </div>

            if queue::shows_input_value_field(item) {
                { value_field(item, &props.on_set_total) }
            }

            if let Some(note) = &item.note {
                { note_card(note) }
            }

            { txid_line(item) }
        </div>
    }
}

fn row_grid_style(mobile: bool) -> String {
    if mobile {
        "display:flex;flex-wrap:wrap;align-items:center;gap:10px 12px".to_string()
    } else {
        format!(
            "display:grid;grid-template-columns:{QUEUE_ROW_COLUMNS};gap:16px;align-items:center"
        )
    }
}

fn name_cell_style(mobile: bool) -> &'static str {
    if mobile {
        "flex:1 1 60%;min-width:0"
    } else {
        "min-width:0"
    }
}

fn format_chip_style() -> String {
    format!(
        "font-family:'IBM Plex Mono',monospace;font-size:11px;letter-spacing:.5px;color:{TEXT_SECONDARY};border:1px solid {BORDER_STRONG};border-radius:2px;padding:4px 8px;justify-self:start"
    )
}

fn retry_button(id: u64, broadcasting: bool, on_retry: &Callback<u64>) -> Html {
    let onclick = {
        let on_retry = on_retry.clone();
        Callback::from(move |_| on_retry.emit(id))
    };
    html! {
        <button
            {onclick}
            disabled={broadcasting}
            style={retry_button_style(broadcasting)}
        >{"Retry"}</button>
    }
}

fn retry_button_style(disabled: bool) -> String {
    let color = if disabled {
        TEXT_DISABLED
    } else {
        ACCENT_TEAL_BRIGHT
    };
    let cursor = if disabled { "not-allowed" } else { "pointer" };
    format!(
        "border:0;background:none;font-family:inherit;font-size:11px;font-weight:500;letter-spacing:.4px;text-transform:uppercase;color:{color};cursor:{cursor};padding:0"
    )
}

fn remove_button_style() -> String {
    format!(
        "border:0;background:none;color:{TEXT_DISABLED};font-size:17px;line-height:1;cursor:pointer;padding:4px"
    )
}

fn dot_style(status: &RowStatusView) -> String {
    let animation = if status.pulsing {
        ";animation:wspulse 1s ease-in-out infinite"
    } else {
        ""
    };
    format!(
        "width:9px;height:9px;border-radius:999px;background:{}{animation}",
        status.dot_color
    )
}

fn vsize_text(item: &QueueItem) -> String {
    match &item.body {
        QueueItemBody::Decoded { vsize, .. } => {
            format!("{} vB", queue::format_thousands(*vsize as i128))
        }
        QueueItemBody::Invalid => "n/a".to_string(),
    }
}

fn rate_text(item: &QueueItem) -> String {
    match queue::effective_rate(item) {
        Some(rate) => format!("{rate:.2}"),
        None => "n/a".to_string(),
    }
}

fn fee_sub_line(item: &QueueItem) -> String {
    if item.is_invalid() {
        return String::new();
    }
    match queue::effective_fee(item) {
        Some(fee) => format!("{} sats", queue::format_thousands(fee)),
        None => "sat/vB".to_string(),
    }
}

fn value_field(item: &QueueItem, on_set_total: &Callback<(u64, Option<u64>)>) -> Html {
    let oninput = {
        let on_set_total = on_set_total.clone();
        let id = item.id;
        Callback::from(move |e: InputEvent| {
            let target: HtmlInputElement = e.target_unchecked_into();
            let digits: String = target
                .value()
                .chars()
                .filter(char::is_ascii_digit)
                .collect();
            on_set_total.emit((id, digits.parse::<u64>().ok()));
        })
    };
    html! {
        <div style={value_field_wrap_style()}>
            <span style={format!("font-size:13px;color:{BODY_COPY};max-width:52ch;line-height:1.5")}>
                {"This transaction's input amounts are not known, so the fee cannot be derived locally. Enter the total value being spent to check it before submitting."}
            </span>
            <input {oninput} placeholder="total input sats" style={value_field_input_style()} />
        </div>
    }
}

fn value_field_wrap_style() -> String {
    format!(
        "display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin:14px 0 0 42px;padding:14px 18px;border:1px dashed {BORDER_STRONG};border-radius:2px;background:{CARD_NESTED}"
    )
}

fn value_field_input_style() -> String {
    format!(
        "flex:1;min-width:150px;background:#000;border:1px solid {BORDER_STRONG};border-radius:2px;color:{FIELD_TEXT};font-family:'IBM Plex Mono',monospace;font-size:13px;padding:10px 12px"
    )
}

fn note_card(note: &NoteCard) -> Html {
    let (border, edge, text_color) = note.kind.colors();
    let style = format!(
        "margin:14px 0 0 42px;padding:12px 16px;border:1px solid {border};border-left:2px solid {edge};border-radius:2px;font-size:13px;line-height:1.5;color:{text_color}"
    );
    html! { <div {style}>{note.text.clone()}</div> }
}

fn txid_line(item: &QueueItem) -> Html {
    let QueueItemBody::Decoded { txid, .. } = &item.body else {
        return html! {};
    };
    let accepted = matches!(item.submission, SubmissionState::Accepted);
    let color = if accepted {
        ACCENT_TEAL_BRIGHT
    } else {
        TEXT_MUTED_6A
    };
    // An unbroadcast (or not-yet-successful) transaction is not there to
    // look up, so only an accepted row links out to the explorer.
    let txid_display = if accepted {
        html! {
            <a
                href={format!("{BLOCK_EXPLORER_TX_URL}{txid}")}
                target="_blank"
                rel="noopener noreferrer"
                style={format!("color:{color};text-decoration:none")}
            >{txid.clone()}</a>
        }
    } else {
        html! { <span>{txid.clone()}</span> }
    };
    html! {
        <div style={format!("display:flex;align-items:center;gap:12px;margin:14px 0 0 42px;font-family:'IBM Plex Mono',monospace;font-size:12px;color:{color};word-break:break-all")}>
            <span style={format!("color:{TEXT_MUTED_6A};letter-spacing:.6px")}>{"TXID"}</span>
            <span style="user-select:all">{txid_display}</span>
            { copy_button(txid.clone()) }
        </div>
    }
}

fn copy_button(text: String) -> Html {
    let onclick = Callback::from(move |_| {
        let text = text.clone();
        if let Some(window) = web_sys::window() {
            let promise = window.navigator().clipboard().write_text(&text);
            wasm_bindgen_futures::spawn_local(async move {
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            });
        }
    });
    html! {
        <button
            {onclick}
            title="Copy txid"
            aria-label="Copy txid"
            style={format!("display:flex;flex:none;border:0;background:none;color:{TEXT_MUTED_6A};cursor:pointer;padding:2px")}
        >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="9" y="9" width="13" height="13" rx="2"></rect>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
        </button>
    }
}
