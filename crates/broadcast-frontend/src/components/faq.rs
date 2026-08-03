use yew::prelude::*;

use crate::tokens::{
    ACCENT_TEAL, BODY_COPY, BORDER_STRONG, HAIRLINE, TEXT_MUTED_6A, TEXT_MUTED_7B,
};

#[function_component(Faq)]
pub fn faq() -> Html {
    let open = use_state(|| None::<usize>);
    let entries = entries();
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

/// Answers are `Html`, not `String`, so one can carry a link. Nothing here
/// interpolates the live fee rate: the fee card owns that number, and the
/// FAQ repeating it gave two places to read the same figure from.
fn entries() -> Vec<(&'static str, Html)> {
    vec![
        (
            "Why is this tool helpful?",
            html! {
                {"A normal transaction broadcast gossips your transaction to every Bitcoin node on \
                  the network before it is mined. If someone holds a key that can also spend those \
                  coins, they can replace the transaction and steal its coins before it gets \
                  mined. Slipstream (what this tool uses) skips the gossip: the transaction goes \
                  to one miner without being sent to the rest of the network. It will be slower to \
                  mine, but you will be safe from transaction replacement."}
            },
        ),
        (
            "Where does what I paste actually go?",
            html! {
                {"Decoding, finalizing and fee maths all run in your browser, and this site has no \
                  server in the middle. The only thing that ever leaves this page (and your \
                  computer) is the finalized transaction hex, sent straight from your browser to \
                  MARA Slipstream when you press Send."}
            },
        ),
        (
            "What does MARA learn about me?",
            html! {
                {"The transaction details, and your IP address. Your browser talks to MARA \
                  directly, so the connection is yours and MARA sees the address you are browsing \
                  from. MARA does not learn your PSBT metadata, your xpubs, your descriptor, or \
                  which other transactions you queued here. Use Tor or a VPN if MARA seeing your \
                  IP matters to you."}
            },
        ),
        (
            "Am I guaranteed to get my transaction mined?",
            html! {
                {"No. Accepted by Slipstream is not confirmed. MARA mines only a portion of \
                  blocks, not all of them, and may drop your transaction for reasons it does not \
                  have to explain. Treat this as a better chance, not a promise. If it has not \
                  confirmed after a while (hours), retry."}
            },
        ),
        (
            "Why is the fee rate different from Mempool.space?",
            html! {
                {"Slipstream gets paid for the service through a higher fee rate. This website \
                  does not take any share of the payment, or any compensation."}
            },
        ),
        // "Will not get mined", not "is rejected": a parent under the
        // displayed floor but over `submit_fee_rate` is accepted and then
        // never mined (section 2 of PLAN.md), so only the mining claim holds
        // for every low-fee parent a reader might have in mind.
        (
            "Can a second transaction pay the fee for a low-fee one (CPFP)?",
            html! {
                {"No. Slipstream looks at every transaction on its own, so one that does not pay \
                  enough will not get mined here — even if you send another one paying extra to \
                  cover it. You can still send several related transactions, as long as each pays \
                  enough by itself: name the files 01, 02, 03 and they go out in that order. If you \
                  need one transaction to pay for another, contact MARA directly."}
            },
        ),
        (
            "What are the risks of using this service?",
            html! {
                {"MARA, the company behind Slipstream, has to be trusted not to perform the attack \
                  on your transaction itself. This is an acceptable risk compared to broadcasting \
                  it publicly and letting ANYONE perform the attack."}
            },
        ),
        (
            "Why is this page on the Wizardsardine domain?",
            html! {
                <>
                    {"We (Wizardsardine) are a security company, maintaining the Liana wallet ("}
                    <a href="https://lianawallet.com" target="_blank" rel="noopener noreferrer">{"lianawallet.com"}</a>
                    {"). Liana users did not have an easy way to broadcast to Slipstream before \
                      this tool, so this is a service to them, open to the rest of the community."}
                </>
            },
        ),
    ]
}
