use yew::prelude::*;

use crate::hooks::use_t;
use crate::i18n::{RichStyles, Strings, rich};
use crate::tokens::{
    ACCENT_TEAL, BORDER_STRONG, EYEBROW_TRACK, HAIRLINE, PROSE, PROSE_SIZE, TEXT_MUTED_6A,
    TEXT_MUTED_7B,
};

#[function_component(Faq)]
pub fn faq() -> Html {
    let t = use_t();
    let open = use_state(|| None::<usize>);
    let entries = entries(t);
    let last = entries.len() - 1;

    let eyebrow_style = format!(
        "font-size:13px;font-weight:500;letter-spacing:{EYEBROW_TRACK};text-transform:uppercase;color:{TEXT_MUTED_7B};margin-bottom:26px"
    );
    let card_style = format!(
        "border:1px solid {BORDER_STRONG};border-radius:22px 2px 22px 2px;background:#0c0c0c;overflow:hidden"
    );

    html! {
        <div style="padding:76px 0 0">
            <div style={eyebrow_style}>{t.faq_eyebrow}</div>
            <div style={card_style}>
                { for entries.into_iter().enumerate().map(|(index, (question, answer))| {
                    faq_entry(question, answer, index == last, index, &open)
                }) }
            </div>
        </div>
    }
}

fn faq_entry(
    question: &'static str,
    answer: Html,
    is_last: bool,
    index: usize,
    open: &UseStateHandle<Option<usize>>,
) -> Html {
    let is_open = **open == Some(index);
    let wrap_style = format!(
        "border-bottom:1px solid {}",
        if is_last { "transparent" } else { HAIRLINE }
    );
    let icon_color = if is_open { ACCENT_TEAL } else { TEXT_MUTED_6A };
    let rotation = if is_open { 180 } else { 0 };
    let icon_style = format!(
        "display:flex;flex:none;color:{icon_color};transition:transform .2s ease-in-out;transform:rotate({rotation}deg)"
    );
    let answer_style = format!(
        "margin:0;padding:0 20px 12px;font-size:{PROSE_SIZE};line-height:1.65;color:{PROSE};max-width:90ch;text-wrap:pretty"
    );
    let onclick = {
        let open = open.clone();
        Callback::from(move |_| open.set(if is_open { None } else { Some(index) }))
    };

    html! {
        <div style={wrap_style}>
            <button {onclick} style="width:100%;display:flex;align-items:center;justify-content:space-between;gap:20px;text-align:left;border:0;background:none;color:#f4f4f4;font-family:inherit;font-size:15px;font-weight:600;line-height:1.25;padding:8px 20px;cursor:pointer">
                <span>{question}</span>
                <span style={icon_style}>
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"></path></svg>
                </span>
            </button>
            if is_open {
                <p style={answer_style}>{answer}</p>
            }
        </div>
    }
}

/// Answers are `Html`, not `String`, so one can carry a link. Nothing here
/// interpolates the live fee rate: the fee card owns that number, and the
/// FAQ repeating it gave two places to read the same figure from.
fn entries(t: &Strings) -> Vec<(&'static str, Html)> {
    // Only the trust question is marked up, and only with a link, so the
    // emphasis styles go unused here.
    let styles = RichStyles::plain();
    vec![
        (t.faq_q_why, rich(t.faq_a_why, &styles)),
        (t.faq_q_where, rich(t.faq_a_where, &styles)),
        (t.faq_q_mara_learns, rich(t.faq_a_mara_learns, &styles)),
        (t.faq_q_guaranteed, rich(t.faq_a_guaranteed, &styles)),
        (t.faq_q_rate_differs, rich(t.faq_a_rate_differs, &styles)),
        (t.faq_q_cpfp, rich(t.faq_a_cpfp, &styles)),
        (t.faq_q_risks, rich(t.faq_a_risks, &styles)),
        (t.faq_q_domain, rich(t.faq_a_domain, &styles)),
    ]
}
