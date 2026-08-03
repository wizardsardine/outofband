use yew::prelude::*;

use crate::tokens::{HAIRLINE, TEXT_MUTED_6A};

#[function_component(Footer)]
pub fn footer() -> Html {
    let row_style = format!(
        "display:flex;flex-wrap:wrap;gap:20px;justify-content:space-between;align-items:center;padding:56px 0 64px;margin-top:60px;border-top:1px solid {HAIRLINE}"
    );
    let attribution_style = format!("font-size:13px;color:{TEXT_MUTED_6A}");

    html! {
        <div style={row_style}>
            <span style={attribution_style}>
                {"Built by "}
                <a href="https://wizardsardine.com" target="_blank" rel="noopener noreferrer">{"Wizardsardine"}</a>
            </span>
            <div style="display:flex;gap:22px;font-size:13px">
                <a href="https://wizardsardine.com/blog/coldcard-rng-vulnerability/" target="_blank" rel="noopener noreferrer">{"Disclosure"}</a>
                <a href="https://slipstream.mara.com/" target="_blank" rel="noopener noreferrer">{"Slipstream terms"}</a>
            </div>
        </div>
    }
}
