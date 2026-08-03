use yew::prelude::*;

use crate::hooks::use_t;
use crate::i18n::{RichStyles, rich};
use crate::tokens::{ACCENT_TEAL, BORDER_STRONG, PROSE, PROSE_SIZE, TEXT_PRIMARY, WARNING};

/// Who this tool is for, and more importantly who it is not for. The
/// second line is the one that matters: someone whose keys are already
/// recoverable should be reading the disclosure, not queueing transactions
/// here, so it is set apart in the warning colour instead of running on
/// from the first sentence.
#[function_component(ContextBanner)]
pub fn context_banner() -> Html {
    let t = use_t();
    let style = format!(
        "border:1px solid {BORDER_STRONG};border-left:3px solid {ACCENT_TEAL};border-radius:2px;background:#0c0c0c;padding:18px 26px;margin-top:16px"
    );
    let text_style = format!(
        "margin:0;font-size:{PROSE_SIZE};line-height:1.6;color:{PROSE};max-width:96ch;text-wrap:pretty"
    );
    let warning_row_style = "display:flex;gap:10px;margin:12px 0 0;max-width:96ch";
    let warning_text_style = format!(
        "margin:0;font-size:{PROSE_SIZE};line-height:1.6;color:{WARNING};text-wrap:pretty;min-width:0"
    );
    let audience_styles = RichStyles::new(format!("color:{TEXT_PRIMARY};font-weight:600"), "");
    let warning_styles = RichStyles::new("font-weight:600", "font-weight:700");

    html! {
        <div style={style}>
            <p style={text_style}>{ rich(t.context_audience, &audience_styles) }</p>
            <div style={warning_row_style}>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke={WARNING} stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="flex:none;margin-top:4px">
                    <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"></path>
                    <path d="M12 9v4"></path>
                    <path d="M12 17h.01"></path>
                </svg>
                <p style={warning_text_style}>
                    { rich(t.context_not_recommended, &warning_styles) }
                </p>
            </div>
        </div>
    }
}
