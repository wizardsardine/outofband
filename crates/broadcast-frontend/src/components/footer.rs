use yew::prelude::*;

use crate::hooks::use_t;
use crate::i18n::{RichStyles, rich};
use crate::tokens::{BODY_COPY, HAIRLINE};

#[function_component(Footer)]
pub fn footer() -> Html {
    let t = use_t();
    let row_style = format!(
        "display:flex;flex-wrap:wrap;gap:20px;justify-content:space-between;align-items:center;padding:56px 0 64px;margin-top:60px;border-top:1px solid {HAIRLINE}"
    );
    let attribution_style = format!("font-size:13.5px;color:{BODY_COPY}");

    html! {
        <div style={row_style}>
            <span style={attribution_style}>
                { rich(t.footer_built_by, &RichStyles::plain()) }
            </span>
            <div style="display:flex;gap:22px;align-items:center;font-size:13.5px">
                <a href="https://wizardsardine.com/blog/coldcard-rng-vulnerability/" target="_blank" rel="noopener noreferrer">{t.footer_disclosure}</a>
                <a href="https://slipstream.mara.com/" target="_blank" rel="noopener noreferrer">{t.footer_slipstream_terms}</a>
            </div>
        </div>
    }
}
