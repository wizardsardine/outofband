use yew::prelude::*;

use crate::hooks::use_lang;
use crate::i18n::Lang;
use crate::tokens::{BORDER_STRONG, PROSE, TEXT_PRIMARY, TEXT_SECONDARY};

/// Shown while an unreviewed translation is active, directly under the
/// disclosure strip so it is read before anything it qualifies.
///
/// Deliberately **not** in the warning palette. The amber on this page means
/// "this could cost you coins", and spending it on a provenance note would
/// blunt the two places that actually need it. This is a caveat about who
/// wrote the words, so it reads as chrome with an escape hatch: the notice
/// in the reader's language, and a button back to the authoritative text.
///
/// Renders nothing for English (the source) or for any language a human has
/// signed off, so it disappears one language at a time as review happens.
#[function_component(TranslationNotice)]
pub fn translation_notice() -> Html {
    let handle = use_lang();
    if handle.lang.reviewed() {
        return Html::default();
    }
    let t = handle.strings();

    let onclick = {
        let set = handle.set.clone();
        Callback::from(move |_| set.emit(Lang::En))
    };

    let bar_style = format!("background:#0c0c0c;border-bottom:1px solid {BORDER_STRONG}");
    let text_style = format!(
        "margin:0;flex:1 1 240px;min-width:0;font-size:13.5px;line-height:1.5;color:{PROSE};text-wrap:pretty"
    );
    let label_style = format!(
        "font-family:'IBM Plex Mono',monospace;font-size:10.5px;font-weight:600;\
         letter-spacing:1.6px;text-transform:uppercase;color:{TEXT_SECONDARY};white-space:nowrap"
    );
    let button_style = format!(
        "flex:none;border:1px solid {BORDER_STRONG};border-radius:2px;background:#000;\
         color:{TEXT_PRIMARY};font-family:inherit;font-size:13px;font-weight:600;\
         padding:7px 14px;cursor:pointer"
    );

    html! {
        <div style={bar_style}>
            <div style="padding:0 6%">
                <div style="max-width:1280px;margin:0 auto;display:flex;align-items:center;gap:10px 16px;padding:11px 0;flex-wrap:wrap">
                    <span style="display:flex;align-items:center;gap:9px;flex:none">
                        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke={TEXT_SECONDARY} stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" style="flex:none">
                            <path d="m5 8 6 6"></path>
                            <path d="m4 14 6-6 2-3"></path>
                            <path d="M2 5h12"></path>
                            <path d="M7 2h1"></path>
                            <path d="m22 22-5-10-5 10"></path>
                            <path d="M14 18h6"></path>
                        </svg>
                        <span style={label_style}>{"AI translation"}</span>
                    </span>
                    <p style={text_style}>{t.ai_notice}</p>
                    <button {onclick} style={button_style}>{t.ai_notice_action}</button>
                </div>
            </div>
        </div>
    }
}
