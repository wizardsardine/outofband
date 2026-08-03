use yew::prelude::*;

use crate::tokens::{ACCENT_TEAL, BORDER_STRONG, TEXT_SECONDARY};

#[function_component(ContextBanner)]
pub fn context_banner() -> Html {
    let style = format!(
        "border:1px solid {BORDER_STRONG};border-left:3px solid {ACCENT_TEAL};border-radius:2px;background:#0c0c0c;padding:18px 26px;margin-top:16px"
    );
    let text_style = format!(
        "margin:0;font-size:14.5px;line-height:1.6;color:{TEXT_SECONDARY};max-width:96ch;text-wrap:pretty"
    );
    html! {
        <div style={style}>
            <p style={text_style}>
                {"This tool exists because of a specific failure: keys generated with predictable randomness can be recovered by anyone who notices. If that is your situation, moving the coins is a race, and the public mempool is where you lose it. "}
                <a href="https://wizardsardine.com/blog/coldcard-rng-vulnerability/" target="_blank" rel="noopener noreferrer" style="font-weight:600">{"Read the disclosure ›"}</a>
            </p>
        </div>
    }
}
