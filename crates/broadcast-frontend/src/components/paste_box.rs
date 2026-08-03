use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{DragEvent, HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::hooks::use_lang;
use crate::i18n::Strings;
use crate::tokens::{
    self, ACCENT_TEAL, BODY_COPY, BORDER_STRONG, CARD_NESTED, FIELD_TEXT, NOTE_CARD_ERROR, PROSE,
    PROSE_SIZE, SURFACE_DRAG_OVER, SURFACE_PARSE_ERROR, TEXT_MUTED_6A, TEXT_SECONDARY,
};

/// Sample data on the outside, translated connective tissue in between: a
/// reader does not need "cHNidP8…" localised, but does need to be told a
/// file can be dropped here.
fn placeholder(t: &Strings) -> String {
    let (or, drop) = (t.load_placeholder_or, t.load_placeholder_drop);
    format!(
        "cHNidP8BAHECAAAAAf8Zj1...\n\n{or}\n\n02000000000101ef51e1b804cc89d182d279655c3aa89e815b1b309fe287d9b2b55d57b90ec68a...\n\n{drop}"
    )
}

/// Advertises the accepted extensions in the file picker; detection itself
/// is always by content, never by what the picker filtered on.
const FILE_ACCEPT: &str = ".txt,.psbt,.txn,.hex,.raw,.tar,.gz,.tgz,.zip";

#[derive(Properties, PartialEq)]
pub struct PasteBoxProps {
    pub raw_text: String,
    pub on_raw_text: Callback<String>,
    pub has_items: bool,
    pub parse_error: Option<String>,
    pub on_submit: Callback<()>,
    pub on_clear: Callback<()>,
    pub broadcasting: bool,
    pub on_files: Callback<web_sys::FileList>,
}

#[function_component(PasteBox)]
pub fn paste_box(props: &PasteBoxProps) -> Html {
    let lang = use_lang().lang;
    let t = lang.strings();
    // Same function on the same input as `use_queue`'s `on_submit`, so the
    // button's enabled state and what the submit path enforces cannot drift.
    let lines = tx_core::split_lines(&props.raw_text);
    let can_queue = crate::queue::paste_gate(&lines).allows_queueing();
    let detected = crate::queue::detected_label(&lines, lang);

    let file_input_ref = use_node_ref();
    let drag_over = use_state(|| false);

    // Window-level, not just the textarea's: a file dropped anywhere on the
    // page must never navigate the tab away, and the page at large is a
    // drop target per PLAN.md section 5.
    {
        let on_files = props.on_files.clone();
        let drag_over = drag_over.clone();
        use_effect_with((), move |()| {
            let Some(window) = web_sys::window() else {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            };

            let dragover_listener = {
                let drag_over = drag_over.clone();
                Closure::<dyn Fn(web_sys::Event)>::new(move |event: web_sys::Event| {
                    event.prevent_default();
                    if !*drag_over {
                        drag_over.set(true);
                    }
                })
            };
            let dragleave_listener = {
                let drag_over = drag_over.clone();
                Closure::<dyn Fn(web_sys::Event)>::new(move |_event: web_sys::Event| {
                    drag_over.set(false);
                })
            };
            let drop_listener = {
                let drag_over = drag_over.clone();
                let on_files = on_files.clone();
                Closure::<dyn Fn(web_sys::Event)>::new(move |event: web_sys::Event| {
                    event.prevent_default();
                    drag_over.set(false);
                    let files = event
                        .dyn_ref::<DragEvent>()
                        .and_then(DragEvent::data_transfer)
                        .and_then(|data_transfer| data_transfer.files());
                    if let Some(files) = files {
                        on_files.emit(files);
                    }
                })
            };

            let _ = window.add_event_listener_with_callback(
                "dragover",
                dragover_listener.as_ref().unchecked_ref(),
            );
            let _ = window.add_event_listener_with_callback(
                "dragleave",
                dragleave_listener.as_ref().unchecked_ref(),
            );
            let _ = window
                .add_event_listener_with_callback("drop", drop_listener.as_ref().unchecked_ref());

            Box::new(move || {
                let _ = window.remove_event_listener_with_callback(
                    "dragover",
                    dragover_listener.as_ref().unchecked_ref(),
                );
                let _ = window.remove_event_listener_with_callback(
                    "dragleave",
                    dragleave_listener.as_ref().unchecked_ref(),
                );
                let _ = window.remove_event_listener_with_callback(
                    "drop",
                    drop_listener.as_ref().unchecked_ref(),
                );
                drop(dragover_listener);
                drop(dragleave_listener);
                drop(drop_listener);
            }) as Box<dyn FnOnce()>
        });
    }

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

    let onclick_choose_files = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let onchange_file_input = {
        let on_files = props.on_files.clone();
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_: Event| {
            let Some(input) = file_input_ref.cast::<HtmlInputElement>() else {
                return;
            };
            if let Some(files) = input.files() {
                on_files.emit(files);
            }
            // Reset so choosing the same file again still fires `onchange`.
            input.set_value("");
        })
    };

    html! {
        <div style="padding:60px 0 0">
            <div style="display:flex;align-items:baseline;justify-content:space-between;gap:20px;flex-wrap:wrap;margin-bottom:18px">
                <div>
                    <h2 style="margin:0;font-size:26px;font-weight:600;letter-spacing:-.2px">{t.load_heading}</h2>
                    <p style={format!("margin:8px 0 0;font-size:{PROSE_SIZE};line-height:1.55;color:{PROSE};max-width:70ch")}>
                        {t.load_blurb}
                    </p>
                </div>
                <span style={format!("font-family:'IBM Plex Mono',monospace;font-size:11px;letter-spacing:.6px;color:{TEXT_MUTED_6A}")}>{detected}</span>
            </div>

            <textarea
                value={props.raw_text.clone()}
                {oninput}
                {onkeydown}
                spellcheck="false"
                placeholder={placeholder(t)}
                style={textarea_style(*drag_over)}
            />
            <input
                type="file"
                multiple={true}
                accept={FILE_ACCEPT}
                ref={file_input_ref}
                onchange={onchange_file_input}
                style="display:none"
            />

            <div style="display:flex;flex-wrap:wrap;gap:12px;align-items:center;margin-top:16px">
                <button
                    onclick={onclick_queue}
                    disabled={!can_queue}
                    class={if can_queue { "primary-btn" } else { "" }}
                    style={tokens::primary_button_style(can_queue, 30)}
                >{t.btn_add_to_queue}</button>
                <button onclick={onclick_choose_files} style={choose_files_style()}>
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M12 16V4"></path>
                        <path d="m7 9 5-5 5 5"></path>
                        <path d="M4 16v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"></path>
                    </svg>
                    {t.btn_choose_files}
                </button>
                <span style={format!("font-size:13.5px;color:{BODY_COPY}")}>{t.load_accepts}</span>
                if props.has_items {
                    <button
                        onclick={onclick_clear}
                        disabled={props.broadcasting}
                        style={clear_button_style(props.broadcasting)}
                    >{t.btn_clear_queue}</button>
                }
            </div>

            if let Some(error) = &props.parse_error {
                { parse_error_card(error, t) }
            }
        </div>
    }
}

fn parse_error_card(error: &str, t: &Strings) -> Html {
    let (border, edge, text_color) = NOTE_CARD_ERROR;
    let style = format!(
        "margin-top:18px;border:1px solid {border};border-left:3px solid {edge};border-radius:2px;background:{SURFACE_PARSE_ERROR};padding:16px 20px"
    );
    html! {
        <div {style}>
            <div style={format!("font-size:11.5px;font-weight:600;letter-spacing:1.2px;text-transform:uppercase;color:{edge}")}>{t.parse_error_title}</div>
            <p style={format!("margin:7px 0 0;font-size:13.5px;line-height:1.55;color:{text_color};font-family:'IBM Plex Mono',monospace")}>{error.to_string()}</p>
        </div>
    }
}

/// Solid teal border over a tinted background while a file is dragged over
/// the page (not dashed — PLAN.md section 5 calls out the mockup's unused
/// dashed `dropStyle` as a departure).
fn textarea_style(drag_over: bool) -> String {
    let (border, background) = if drag_over {
        (ACCENT_TEAL, SURFACE_DRAG_OVER)
    } else {
        (BORDER_STRONG, CARD_NESTED)
    };
    format!(
        "width:100%;box-sizing:border-box;height:214px;resize:vertical;border-radius:2px;color:{FIELD_TEXT};font-family:'IBM Plex Mono',monospace;font-size:13px;line-height:1.6;padding:18px 20px;word-break:break-all;border:1px solid {border};background:{background};transition:border-color .2s ease-in-out,background .2s ease-in-out"
    )
}

fn choose_files_style() -> String {
    format!(
        "display:flex;align-items:center;gap:9px;border:1px solid {BORDER_STRONG};border-radius:2px;background:#000;color:{TEXT_SECONDARY};font-family:inherit;font-size:13px;padding:13px 18px;cursor:pointer"
    )
}

fn clear_button_style(disabled: bool) -> String {
    let cursor = if disabled { "not-allowed" } else { "pointer" };
    format!(
        "border:0;background:none;color:{TEXT_MUTED_6A};font-family:inherit;font-size:13px;cursor:{cursor};padding:14px 4px"
    )
}
