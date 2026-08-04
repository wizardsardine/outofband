use yew::prelude::*;

use crate::hooks::use_t;
use crate::tokens::{BODY_COPY, BORDER_STRONG, ERROR_RED, FONT_MONO, TEXT_MUTED_7B, TEXT_PRIMARY};

/// PSBTs refused at load time because `tx-core` could not finalize them
/// (PLAN.md section 1): the one thing this UI refuses on the user's behalf,
/// since an unsigned transaction cannot succeed under any fee. One modal
/// covers every refusal from a single load operation.
#[derive(Properties, PartialEq)]
pub struct FinalizationModalProps {
    pub refused: Vec<(String, String)>,
    pub on_close: Callback<()>,
}

#[function_component(FinalizationModal)]
pub fn finalization_modal(props: &FinalizationModalProps) -> Html {
    let t = use_t();
    let onclick_close = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    html! {
        <div style={backdrop_style()}>
            <div style={card_style()}>
                <div style={format!("font-size:17px;font-weight:600;letter-spacing:-.1px;color:{TEXT_PRIMARY}")}>
                    {t.modal_title}
                </div>
                <ul style="list-style:none;margin:18px 0;padding:0;display:flex;flex-direction:column;gap:10px">
                    { for props.refused.iter().map(refused_row) }
                </ul>
                <p style={format!("margin:0 0 22px;font-size:14.5px;line-height:1.55;color:{BODY_COPY}")}>
                    {t.modal_instruction}
                </p>
                <button onclick={onclick_close} style={close_button_style()}>{t.btn_close}</button>
            </div>
        </div>
    }
}

fn refused_row((name, reason): &(String, String)) -> Html {
    html! {
        <li style={format!("font-family:{FONT_MONO};font-size:13px;color:{TEXT_PRIMARY}")}>
            <div style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{name.clone()}</div>
            <div style={format!("margin-top:3px;color:{TEXT_MUTED_7B};font-size:11.5px;line-height:1.45")}>{reason.clone()}</div>
        </li>
    }
}

fn backdrop_style() -> String {
    "position:fixed;inset:0;background:rgba(0,0,0,.72);display:flex;align-items:flex-start;justify-content:center;z-index:60;padding:24px;box-sizing:border-box;overflow-y:auto".to_string()
}

fn card_style() -> String {
    format!(
        "background:#0c0c0c;border:1px solid {BORDER_STRONG};border-left:3px solid {ERROR_RED};border-radius:2px;padding:28px 32px;max-width:520px;width:100%;max-height:calc(100dvh - 48px);margin:auto 0;box-sizing:border-box;overflow-y:auto"
    )
}

fn close_button_style() -> String {
    format!(
        "border:1px solid {ERROR_RED};border-radius:2px;background:#000;color:{ERROR_RED};font-family:inherit;font-size:13.5px;font-weight:600;letter-spacing:.6px;text-transform:uppercase;padding:12px 28px;cursor:pointer"
    )
}
