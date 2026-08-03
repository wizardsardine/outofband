use yew::prelude::*;

use crate::components::fee_card::format_rate;
use crate::hooks::fee::FeeSnapshot;
use crate::tokens::{
    ACCENT_TEAL, BODY_COPY, BORDER_STRONG, HAIRLINE, TEXT_MUTED_6A, TEXT_MUTED_7B,
};

#[derive(Properties, PartialEq)]
pub struct FaqProps {
    pub fee: Option<FeeSnapshot>,
}

#[function_component(Faq)]
pub fn faq(props: &FaqProps) -> Html {
    let open = use_state(|| None::<usize>);
    let entries = entries(props.fee.as_ref());
    let last = entries.len() - 1;

    let eyebrow_style = format!(
        "font-size:13px;font-weight:500;letter-spacing:1.6px;text-transform:uppercase;color:{TEXT_MUTED_7B};margin-bottom:26px"
    );
    let card_style = format!(
        "border:1px solid {BORDER_STRONG};border-radius:22px 2px 22px 2px;background:#0c0c0c;overflow:hidden"
    );

    html! {
        <div style="padding:76px 0 0">
            <div style={eyebrow_style}>{"Questions you should ask before using this"}</div>
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
    answer: String,
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
        "margin:0;padding:0 20px 10px;font-size:13.5px;line-height:1.65;color:{BODY_COPY};max-width:90ch;text-wrap:pretty"
    );
    let onclick = {
        let open = open.clone();
        Callback::from(move |_| open.set(if is_open { None } else { Some(index) }))
    };

    html! {
        <div style={wrap_style}>
            <button {onclick} style="width:100%;display:flex;align-items:center;justify-content:space-between;gap:20px;text-align:left;border:0;background:none;color:#f4f4f4;font-family:inherit;font-size:14px;font-weight:600;line-height:1.25;padding:6px 20px;cursor:pointer">
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

fn entries(fee: Option<&FeeSnapshot>) -> Vec<(&'static str, String)> {
    vec![
        (
            "Does this keep my transaction out of the public mempool?",
            "Yes. A normal broadcast gossips your transaction to every node on the network before \
             it is mined. If someone holds a key that can also spend those coins, that window is \
             all they need to replace you. Slipstream skips the gossip: the transaction goes to \
             one miner and appears in a block."
                .to_string(),
        ),
        (
            "Does anything I paste get sent to your server?",
            "Decoding, finalizing and fee maths all run in your browser. The only thing that ever \
             leaves this page is the finalized transaction hex, sent when you press Broadcast \
             through this site's own thin relay, which holds the Slipstream credential, enforces \
             rate limits, stores nothing, and logs txids only."
                .to_string(),
        ),
        (
            "What does MARA learn about me?",
            "The transaction itself. It reaches MARA through this site's relay, so MARA sees the \
             relay's IP, not yours. The relay operator sees your IP, as any website does. MARA \
             does not learn your PSBT metadata, your xpubs, your descriptor, or which other \
             transactions you queued here. Use Tor or a VPN if the relay operator seeing your IP \
             matters to you."
                .to_string(),
        ),
        (
            "Am I guaranteed to get confirmed?",
            "No. Accepted is not confirmed. MARA mines a share of blocks, not all of them, and \
             may drop your transaction for reasons it does not have to explain. Treat this as a \
             better chance, not a promise. If it has not confirmed after a few hours, broadcast \
             normally."
                .to_string(),
        ),
        ("Why is there a minimum fee rate at all?", floor_answer(fee)),
    ]
}

fn floor_answer(fee: Option<&FeeSnapshot>) -> String {
    let floor = match fee {
        Some(snapshot) => format_rate(snapshot.rate_sat_vb),
        None => "—".to_string(),
    };
    format!(
        "Slipstream mines only what clears its own fee floor, currently {floor} sat/vB. Anything \
         below is rejected outright rather than queued, so a transaction that misses the floor \
         needs to be rebuilt at a higher rate before it can be accepted."
    )
}
