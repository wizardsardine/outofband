use yew::prelude::*;

use crate::tokens::{NOTE_CARD_WARN, TEXT_MUTED_7B, TEXT_PRIMARY, TEXT_SECONDARY, WARNING};

/// Sits directly above the paste box: the step before loading anything is
/// getting a PSBT out of the wallet without broadcasting it, and in Liana
/// that means Export rather than the broadcast button one click away from
/// it. Pressing the wrong one puts the transaction in the public mempool,
/// which is the single thing this page exists to avoid, so the instruction
/// carries the warning palette rather than sitting in body copy.
#[function_component(PrepareStep)]
pub fn prepare_step() -> Html {
    let (note_border, note_edge, note_text) = NOTE_CARD_WARN;
    let card_style = format!(
        "border:1px solid {note_border};border-left:3px solid {note_edge};border-radius:2px;background:#100d06;padding:18px 22px"
    );
    let note_style = format!(
        "margin:0;font-size:14px;line-height:1.6;color:{note_text};max-width:88ch;text-wrap:pretty;min-width:0"
    );
    let secondary_style = format!(
        "margin:12px 0 0;font-size:13.5px;line-height:1.6;color:{TEXT_MUTED_7B};max-width:88ch;text-wrap:pretty"
    );
    // Body copy, not the muted grey an aside would get: this paragraph is
    // the instruction for every wallet that is not Liana, so it carries the
    // same weight as the card beside it.
    let aside_style = format!(
        "margin:18px 0 0;font-size:14px;line-height:1.6;color:{TEXT_SECONDARY};max-width:88ch;text-wrap:pretty"
    );
    let ui_name_style = format!("color:{TEXT_PRIMARY};font-weight:600");
    let emphasis_style = format!("color:{TEXT_PRIMARY};font-weight:600");
    let shout_style = format!("color:{WARNING};font-weight:700");

    html! {
        <div style="padding:60px 0 0">
            <h2 style="margin:0 0 18px;font-size:26px;font-weight:600;letter-spacing:-.2px">{"Prepare your transaction"}</h2>
            <div style={card_style}>
                <div style="display:flex;gap:11px">
                    <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke={WARNING} stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="flex:none;margin-top:3px">
                        <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"></path>
                        <path d="M12 9v4"></path>
                        <path d="M12 17h.01"></path>
                    </svg>
                    <p style={note_style}>
                        {"For Liana users, prepare your transaction normally and sign it, but "}
                        <strong style={shout_style}>{"DO NOT"}</strong>
                        {" press the broadcast button. Once it is signed, click "}
                        <strong style={ui_name_style.clone()}>{"Export"}</strong>
                        {" instead. That file is what you load in the next step."}
                    </p>
                </div>
                <p style={secondary_style}>
                    {"If you already signed it but did not save the PSBT file, you can find the \
                      transaction again under "}
                    <strong style={ui_name_style}>{"Drafts and Approvals"}</strong>
                    {"."}
                </p>
            </div>
            // Outside the card and in body copy rather than the warning
            // palette: this is the instruction for everyone, and a second
            // amber block beside the first would flatten both. The one
            // sentence that must not be skimmed is picked out instead.
            <p style={aside_style}>
                {"Build and sign your transaction normally, but "}
                <strong style={emphasis_style}>{"do not broadcast it to the Bitcoin network"}</strong>
                {". Not all software lets you sign without broadcasting, so check its \
                  documentation before you sign if you are unsure. Liana users, just read the card \
                  above."}
            </p>
        </div>
    }
}
