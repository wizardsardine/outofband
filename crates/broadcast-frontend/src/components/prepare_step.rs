use yew::prelude::*;

use crate::guides::{GUIDES, Guide};
use crate::hooks::use_t;
use crate::i18n::{RichStyles, rich};
use crate::tokens::{
    BODY_COPY, EYEBROW_TRACK, FONT_MONO, NOTE_CARD_WARN, PROSE, PROSE_SIZE, TEXT_MUTED_6A,
    TEXT_PRIMARY, WARNING,
};

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
    // Last, and quiet: someone who followed the instructions above does not
    // need these, and someone who did not is the reader they are for.
    let guides_label_style = format!(
        "margin:26px 0 8px;font-size:11.5px;font-weight:500;letter-spacing:{EYEBROW_TRACK};text-transform:uppercase;color:{TEXT_MUTED_6A};font-family:{FONT_MONO}"
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
            <div style={guides_label_style}>{t.guides_heading}</div>
            { for GUIDES.iter().map(|g| guide_row(g, t.guides_multilingual)) }
        </div>
    }
}

/// One guide as one clickable line: format, what it covers, who made it,
/// and the language it is in. No wrapper text, so the row is the sentence.
///
/// The whole row is the anchor rather than just the title, because the
/// author and the language are the parts a reader is deciding on.
fn guide_row(g: &'static Guide, multilingual: &'static str) -> Html {
    let row_style = format!(
        "display:flex;align-items:baseline;gap:10px;padding:7px 0;font-size:14px;line-height:1.5;color:{PROSE};text-decoration:none;flex-wrap:wrap"
    );
    let meta_style = format!("color:{TEXT_MUTED_6A};font-size:13px");
    html! {
        <a key={g.href} href={g.href} target="_blank" rel="noopener noreferrer" style={row_style}>
            // Presentational: the row already says what it is in words, so a
            // screen reader gains nothing from "page facing up".
            <span aria-hidden="true" style="font-size:15px;line-height:1">{g.icon}</span>
            <span style={format!("color:{TEXT_PRIMARY};font-weight:600")}>{g.covers}</span>
            <span style={meta_style.clone()}>{g.author}</span>
            <span style={meta_style}>{g.lang.unwrap_or(multilingual)}</span>
        </a>
    }
}
