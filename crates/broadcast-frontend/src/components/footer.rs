use yew::prelude::*;

use crate::hooks::use_t;
use crate::i18n::{RichStyles, rich};
use crate::tokens::{BODY_COPY, COMMIT, HAIRLINE, SOURCE_REPO_URL, TEXT_MUTED_6A};

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
            <div style="display:flex;flex-wrap:wrap;gap:22px;align-items:center;font-size:13.5px">
                <a href={SOURCE_REPO_URL} target="_blank" rel="noopener noreferrer">{t.footer_source}</a>
                { commit_link() }
                <a href="https://wizardsardine.com/blog/coldcard-rng-vulnerability/" target="_blank" rel="noopener noreferrer">{t.footer_disclosure}</a>
                <a href="https://slipstream.mara.com/" target="_blank" rel="noopener noreferrer">{t.footer_slipstream_terms}</a>
            </div>
        </div>
    }
}

/// The running revision, linked to that commit. Rendered as plain text when
/// the build could not determine one, since a link would 404.
fn commit_link() -> Html {
    let style =
        format!("font-family:'IBM Plex Mono',monospace;font-size:12px;color:{TEXT_MUTED_6A}");
    if COMMIT == "unknown" {
        return html! { <span style={style}>{COMMIT}</span> };
    }
    let href = format!(
        "{SOURCE_REPO_URL}/commit/{}",
        COMMIT.trim_end_matches("-dirty")
    );
    html! {
        <a {href} target="_blank" rel="noopener noreferrer" style={style}>{COMMIT}</a>
    }
}
