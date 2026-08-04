use yew::prelude::*;

use crate::hooks::fee::FeeSnapshot;
use crate::hooks::use_t;
use crate::i18n::{RichStyles, rich};
use crate::tokens::{
    BORDER_STRONG, EYEBROW_TRACK, FEE_NUMBER_GRADIENT, FONT_MONO, NOTE_CARD_WARN, PROSE,
    PROSE_SIZE, RULE, TEXT_MUTED_7B, TEXT_SECONDARY, WARNING,
};

#[derive(Properties, PartialEq)]
pub struct FeeCardProps {
    pub mobile: bool,
    pub fee: Option<FeeSnapshot>,
}

#[function_component(FeeCard)]
pub fn fee_card(props: &FeeCardProps) -> Html {
    let t = use_t();
    let stale = props.fee.as_ref().is_some_and(|snapshot| snapshot.stale);
    let display = match &props.fee {
        Some(snapshot) => format_rate(snapshot.rate_sat_vb),
        None => "—".to_string(),
    };

    let card_style = fee_card_style(props.mobile);
    let num_style = fee_num_style(props.mobile, stale);
    let eyebrow_style = format!(
        "font-size:12px;font-weight:500;letter-spacing:{EYEBROW_TRACK};text-transform:uppercase;color:{TEXT_MUTED_7B}"
    );
    let unit_style = format!("font-family:{FONT_MONO};font-size:22px;color:{TEXT_SECONDARY}");
    let rule_style = format!("height:1px;background:{RULE};margin:26px 0 20px");
    let sentence_style =
        format!("margin:0;font-size:{PROSE_SIZE};line-height:1.6;color:{PROSE};text-wrap:pretty");
    let stale_note_style =
        format!("margin:10px 0 0;font-size:13px;line-height:1.5;color:{WARNING}");
    let (note_border, note_edge, note_text) = NOTE_CARD_WARN;
    let advice_card_style = format!(
        "border:1px solid {note_border};border-left:3px solid {note_edge};border-radius:2px;background:#100d06;padding:13px 15px;margin-top:16px"
    );
    let advice_text_style =
        format!("margin:0;font-size:14px;line-height:1.6;color:{note_text};text-wrap:pretty");
    let advice_styles = RichStyles::new(format!("color:{WARNING};font-weight:600"), "");

    html! {
        <div style={card_style}>
            <div style={eyebrow_style}>{t.fee_eyebrow}</div>
            <div style="display:flex;align-items:baseline;gap:14px;margin-top:12px">
                <span style={num_style}>{display}</span>
                <span style={unit_style}>{"sat/vB"}</span>
            </div>
            <div style={rule_style}></div>
            // "Not expected to be mined", not "not accepted": this number is
            // `effective_rate`, and admission is governed by the lower
            // `submit_fee_rate` the card never shows (section 2 of PLAN.md).
            // Below this rate a transaction can still be accepted, and then
            // never mined — which is the whole reason to say so here.
            <p style={sentence_style}>{t.fee_sentence}</p>
            // Attached to the number, not to the advice below it: what may
            // be out of date is the figure on this card.
            if stale {
                <p style={stale_note_style}>{t.fee_stale}</p>
            }
            // The floor is not a target. It moves while a transaction is
            // being built, so one pinned to the number shown at drafting
            // time can be under the floor by the time it is submitted.
            <div style={advice_card_style}>
                <p style={advice_text_style}>{ rich(t.fee_advice, &advice_styles) }</p>
            </div>
        </div>
    }
}

fn fee_card_style(mobile: bool) -> String {
    let padding = if mobile {
        "26px 24px 24px"
    } else {
        "36px 38px 32px"
    };
    format!(
        "border:1px solid {BORDER_STRONG};border-radius:44px 2px 44px 2px;background:#0c0c0c;box-shadow:2px 4px 10px 0 rgba(0,0,0,.15);padding:{padding}"
    )
}

fn fee_num_style(mobile: bool, stale: bool) -> String {
    let size = if mobile {
        "font-size:76px;letter-spacing:-2px"
    } else {
        "font-size:124px;letter-spacing:-4px"
    };
    let opacity = if stale { "opacity:.5;" } else { "" };
    format!(
        "font-family:{FONT_MONO};font-weight:700;line-height:.86;background:{FEE_NUMBER_GRADIENT};-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent;{opacity}{size}"
    )
}

fn format_rate(rate: f64) -> String {
    if rate.fract() == 0.0 {
        format!("{rate:.0}")
    } else {
        let formatted = format!("{rate:.2}");
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}
