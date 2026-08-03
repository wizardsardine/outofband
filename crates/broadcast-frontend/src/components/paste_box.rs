use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use crate::tokens::{
    self, BORDER_STRONG, CARD_NESTED, FIELD_TEXT, NOTE_CARD_ERROR, SURFACE_PARSE_ERROR,
    TEXT_MUTED_6A, TEXT_MUTED_7B, TEXT_SECONDARY,
};

const PLACEHOLDER: &str = "cHNidP8BAHECAAAAAf8Zj1...\n\nor\n\n02000000000101ef51e1b804cc89d182d279655c3aa89e815b1b309fe287d9b2b55d57b90ec68a...\n\nor drop a file here";

#[derive(Properties, PartialEq)]
pub struct PasteBoxProps {
    pub raw_text: String,
    pub on_raw_text: Callback<String>,
    pub has_items: bool,
    pub parse_error: Option<String>,
    pub on_submit: Callback<()>,
    pub on_clear: Callback<()>,
}

#[function_component(PasteBox)]
pub fn paste_box(props: &PasteBoxProps) -> Html {
    // Same function on the same input as `use_queue`'s `on_submit`, so the
    // button's enabled state and what the submit path enforces cannot drift.
    let lines = tx_core::split_lines(&props.raw_text);
    let can_queue = crate::queue::paste_gate(&lines).allows_queueing();
    let detected = crate::queue::detected_label(&lines);

    let oninput = {
        let on_raw_text = props.on_raw_text.clone();
        Callback::from(move |e: InputEvent| {
            let target: HtmlTextAreaElement = e.target_unchecked_into();
            on_raw_text.emit(target.value());
        })
    };

    let onkeydown = {
        let on_submit = props.on_submit.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" && (e.ctrl_key() || e.meta_key()) {
                // Without this the browser still inserts a newline, and the
                // native input event that follows overwrites `raw_text` and
                // clears `parse_error` after the submit already ran.
                e.prevent_default();
                on_submit.emit(());
            }
        })
    };

    let onclick_queue = {
        let on_submit = props.on_submit.clone();
        Callback::from(move |_| on_submit.emit(()))
    };

    let onclick_clear = {
        let on_clear = props.on_clear.clone();
        Callback::from(move |_| on_clear.emit(()))
    };

    html! {
        <div style="padding:60px 0 0">
            <div style="display:flex;align-items:baseline;justify-content:space-between;gap:20px;flex-wrap:wrap;margin-bottom:18px">
                <div>
                    <h2 style="margin:0;font-size:26px;font-weight:600;letter-spacing:-.2px">{"Load transactions"}</h2>
                    <p style={format!("margin:7px 0 0;font-size:13.5px;line-height:1.55;color:{TEXT_MUTED_7B};max-width:70ch")}>
                        {"Paste or drop signed PSBTs and raw transactions. We hand each one to MARA Slipstream, which mines it without ever touching the public mempool."}
                    </p>
                </div>
                <span style={format!("font-family:'IBM Plex Mono',monospace;font-size:11px;letter-spacing:.6px;color:{TEXT_MUTED_6A}")}>{detected}</span>
            </div>

            <textarea
                value={props.raw_text.clone()}
                {oninput}
                {onkeydown}
                spellcheck="false"
                placeholder={PLACEHOLDER}
                style={textarea_style()}
            />

            <div style="display:flex;flex-wrap:wrap;gap:12px;align-items:center;margin-top:16px">
                <button
                    onclick={onclick_queue}
                    disabled={!can_queue}
                    class={if can_queue { "primary-btn" } else { "" }}
                    style={tokens::primary_button_style(can_queue, 30)}
                >{"Add to queue"}</button>
                <button style={choose_files_style()}>
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M12 16V4"></path>
                        <path d="m7 9 5-5 5 5"></path>
                        <path d="M4 16v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"></path>
                    </svg>
                    {"Choose files"}
                </button>
                <span style={format!("font-size:12.5px;color:{TEXT_MUTED_6A}")}>{"Files, folders or archives: .txt .psbt .txn .tar .tar.gz .zip"}</span>
                if props.has_items {
                    <button onclick={onclick_clear} style={format!("border:0;background:none;color:{TEXT_MUTED_6A};font-family:inherit;font-size:13px;cursor:pointer;padding:14px 4px")}>{"Clear queue"}</button>
                }
            </div>

            if let Some(error) = &props.parse_error {
                { parse_error_card(error) }
            }
        </div>
    }
}

fn parse_error_card(error: &str) -> Html {
    let (border, edge, text_color) = NOTE_CARD_ERROR;
    let style = format!(
        "margin-top:18px;border:1px solid {border};border-left:3px solid {edge};border-radius:2px;background:{SURFACE_PARSE_ERROR};padding:16px 20px"
    );
    html! {
        <div {style}>
            <div style={format!("font-size:11.5px;font-weight:600;letter-spacing:1.2px;text-transform:uppercase;color:{edge}")}>{"Could not parse"}</div>
            <p style={format!("margin:7px 0 0;font-size:13.5px;line-height:1.55;color:{text_color};font-family:'IBM Plex Mono',monospace")}>{error.to_string()}</p>
        </div>
    }
}

fn textarea_style() -> String {
    format!(
        "width:100%;box-sizing:border-box;height:214px;resize:vertical;border-radius:2px;color:{FIELD_TEXT};font-family:'IBM Plex Mono',monospace;font-size:13px;line-height:1.6;padding:18px 20px;word-break:break-all;border:1px solid {BORDER_STRONG};background:{CARD_NESTED}"
    )
}

fn choose_files_style() -> String {
    format!(
        "display:flex;align-items:center;gap:9px;border:1px solid {BORDER_STRONG};border-radius:2px;background:#000;color:{TEXT_SECONDARY};font-family:inherit;font-size:13px;padding:13px 18px;cursor:pointer"
    )
}
