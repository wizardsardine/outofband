use yew::prelude::*;

use crate::hooks::use_t;
use crate::i18n::{RichStyles, rich};
use crate::tokens::{BODY_COPY, NOTE_CARD_WARN, PROSE, PROSE_SIZE, TEXT_PRIMARY, WARNING};

/// Sits directly above the paste box: the step before loading anything is
/// getting a PSBT out of the wallet without broadcasting it, and in Liana
/// that means Export rather than the broadcast button one click away from
/// it. Pressing the wrong one puts the transaction in the public mempool,
/// which is the single thing this page exists to avoid, so the instruction
/// carries the warning palette rather than sitting in body copy.
#[function_component(PrepareStep)]
pub fn prepare_step() -> Html {
    let t = use_t();
    let (note_border, note_edge, note_text) = NOTE_CARD_WARN;
    let card_style = format!(
        "border:1px solid {note_border};border-left:3px solid {note_edge};border-radius:2px;background:#100d06;padding:18px 22px"
    );
    let note_style = format!(
        "margin:0;font-size:{PROSE_SIZE};line-height:1.6;color:{note_text};max-width:88ch;text-wrap:pretty;min-width:0"
    );
    let secondary_style = format!(
        "margin:12px 0 0;font-size:14px;line-height:1.6;color:{BODY_COPY};max-width:88ch;text-wrap:pretty"
    );
    // Body copy, not the muted grey an aside would get: this paragraph is
    // the instruction for every wallet that is not Liana, so it carries the
    // same weight as the card beside it.
    let aside_style = format!(
        "margin:18px 0 0;font-size:{PROSE_SIZE};line-height:1.6;color:{PROSE};max-width:88ch;text-wrap:pretty"
    );
    // Muted and last: a note about what this page will carry later, not an
    // instruction for today. It still earns its place, because "wait and
    // check back" is a real option here and a better one than guessing at a
    // wallet's signing flow when guessing wrong means a public broadcast.
    let pending_style = format!(
        "margin:14px 0 0;font-size:14px;line-height:1.6;color:{BODY_COPY};max-width:88ch;text-wrap:pretty"
    );

    // Inside the warn card the emphasis is amber on amber, so UI names take
    // white to stay legible against it; outside, plain white is the accent.
    let card_styles = RichStyles::new(
        format!("color:{TEXT_PRIMARY};font-weight:600"),
        format!("color:{WARNING};font-weight:700"),
    );
    let prose_styles = RichStyles::new(format!("color:{TEXT_PRIMARY};font-weight:600"), "");

    html! {
        <div style="padding:60px 0 0">
            <h2 style="margin:0 0 18px;font-size:26px;font-weight:600;letter-spacing:-.2px">{t.prepare_heading}</h2>
            <div style={card_style}>
                <div style="display:flex;gap:11px">
                    <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke={WARNING} stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="flex:none;margin-top:3px">
                        <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"></path>
                        <path d="M12 9v4"></path>
                        <path d="M12 17h.01"></path>
                    </svg>
                    <p style={note_style}>{ rich(t.prepare_liana, &card_styles) }</p>
                </div>
                <p style={secondary_style}>{ rich(t.prepare_drafts, &card_styles) }</p>
            </div>
            <p style={aside_style}>{ rich(t.prepare_other_wallets, &prose_styles) }</p>
            <p style={pending_style}>{t.prepare_tutorials}</p>
        </div>
    }
}
