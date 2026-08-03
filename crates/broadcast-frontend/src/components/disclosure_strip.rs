use yew::prelude::*;

use crate::components::LangPicker;
use crate::hooks::use_t;
use crate::tokens::{NOTE_CARD_WARN, TEXT_PRIMARY, WARNING};

/// The disclosure is the first thing on the page and stays there: someone
/// who lands here from a link needs to read the vulnerability write-up
/// before deciding whether this tool is for them at all. It is dressed in
/// the warning palette rather than the muted chrome of the mockup, so it
/// reads as an advisory and not as a nav bar.
#[function_component(DisclosureStrip)]
pub fn disclosure_strip() -> Html {
    let t = use_t();
    let (border, _, _) = NOTE_CARD_WARN;
    let bar_style = format!(
        "position:sticky;top:0;z-index:20;background:#14100a;border-bottom:1px solid {border};box-shadow:0 1px 0 0 rgba(224,179,65,.18)"
    );
    let label_style = format!(
        "font-family:'IBM Plex Mono',monospace;font-size:11px;font-weight:700;letter-spacing:1.8px;text-transform:uppercase;color:{WARNING};white-space:nowrap"
    );
    let link_style = format!(
        "display:flex;align-items:center;gap:8px;min-width:0;font-size:15px;font-weight:600;line-height:1.35;color:{TEXT_PRIMARY};text-decoration:none"
    );
    let link_text_style = format!(
        "min-width:0;overflow-wrap:anywhere;text-decoration:underline;text-decoration-color:{WARNING};text-decoration-thickness:1.5px;text-underline-offset:4px"
    );

    html! {
        <div style={bar_style}>
            <div style="padding:0 6%">
                <div style="max-width:1280px;margin:0 auto;display:flex;align-items:center;gap:10px 14px;min-height:52px;padding:10px 0;box-sizing:border-box;flex-wrap:wrap">
                    <span style="display:flex;align-items:center;gap:9px;flex:none">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke={WARNING} stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="flex:none">
                            <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"></path>
                            <path d="M12 9v4"></path>
                            <path d="M12 17h.01"></path>
                        </svg>
                        <span style={label_style}>{t.disclosure_label}</span>
                    </span>
                    <a class="disclosure-link" href="https://wizardsardine.com/blog/coldcard-rng-vulnerability/" target="_blank" rel="noopener noreferrer" style={link_style}>
                        <span style={link_text_style}>{t.disclosure_link}</span>
                        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="flex:none">
                            <path d="M7 17 17 7"></path>
                            <path d="M9 7h8v8"></path>
                        </svg>
                    </a>
                    // `margin-left:auto` rather than a spacer: when the bar
                    // wraps on a narrow screen the picker drops to its own
                    // line instead of being pinned to a stretched gap.
                    <span style="margin-left:auto;flex:none">
                        <LangPicker />
                    </span>
                </div>
            </div>
        </div>
    }
}
