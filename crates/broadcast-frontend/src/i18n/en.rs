//! English. The source text every other catalogue translates, so a change
//! here is a change five translators need to hear about.

use super::{Plurals, Strings};

pub static STRINGS: Strings = Strings {
    page_title: "Outofband: direct-to-miner broadcast",
    lang_picker_label: "Language",

    disclosure_label: "Security disclosure",
    disclosure_link: "Coldcard RNG vulnerability, and why you might need this tool",

    context_audience: "This tool is important for some types of users only: mainly *Liana*, \
        *Miniscript*, or some types of *multisig* wallets.",
    context_not_recommended: "This tool is !NOT! recommended for wallets critically at risk, as \
        described in [this blog post ›](https://wizardsardine.com/blog/coldcard-rng-vulnerability/)",

    hero_line_one: "Get your transactions",
    hero_line_two: "mined directly, without using the public mempool.",

    fee_eyebrow: "Minimum accepted fee rate",
    fee_sentence: "Anything below this rate is not expected to be mined. Clearing it is not a \
        guarantee either.",
    fee_stale: "This rate may be out of date.",
    fee_advice: "This floor is *dynamic*. It moves with demand, so build your transaction at a \
        fee rate *significantly higher* than the floor, or it may stop clearing before it is mined.",

    prepare_heading: "Prepare your transaction",
    prepare_liana: "For Liana users, prepare your transaction normally and sign it, but !DO NOT! \
        press the *Broadcast* button. Once it is signed, click *Export* instead. That file is what \
        you load in the next step.",
    prepare_drafts: "If you already signed it but did not save the PSBT file, you can find the \
        transaction again under *Drafts and Approvals*.",
    prepare_other_wallets: "Build and sign your transaction normally, but *do not broadcast it to \
        the Bitcoin network*. Not all software lets you sign without broadcasting, so check its \
        documentation before you sign if you are unsure. Liana users, just read the card above.",
    prepare_tutorials: "Links to tutorials from educators will be added to this page once they \
        are ready. If you are unsure how to do this with your wallet, you can wait and check back \
        here later.",

    load_heading: "Load transactions",
    load_blurb: "Paste or drop signed PSBTs and raw transactions. Your browser sends each one \
        straight to MARA Slipstream, which mines it without ever touching the public mempool.",
    load_placeholder_or: "or",
    load_placeholder_drop: "or drop a file here",
    load_accepts: "Files, folders or archives: .txt .psbt .txn .tar .tar.gz .zip",
    btn_add_to_queue: "Add to queue",
    btn_choose_files: "Choose files",
    btn_clear_queue: "Clear queue",
    parse_error_title: "Could not parse",
    detected_psbt: "DETECTED · PSBT",
    detected_raw: "DETECTED · RAW TRANSACTION",
    detected_unrecognised: "UNRECOGNISED FORMAT",

    stat_in_queue: "In queue",
    stat_clear_floor: "Clear the floor",
    stat_below_floor: "Below floor",
    stat_fee_unknown: "Fee unknown",
    btn_send: "Send",
    btn_send_batch: "Send batch ({n})",
    btn_sending: "Sending…",

    col_source: "Source",
    col_format: "Format",
    col_vsize: "vsize",
    col_fee_rate: "Fee rate",
    col_status: "Status",
    btn_retry: "Retry",
    row_remove_title: "Remove from queue",
    row_missing_value: "This transaction's input amounts are not known, so the fee cannot be \
        derived locally. Enter the total value being spent to check it before submitting.",
    row_total_placeholder: "Total input value (sats)",

    status_invalid: "Invalid",
    status_accepted: "Accepted",
    status_sending: "Sending…",
    status_rejected: "Rejected",
    status_rate_limited: "Rate limited",
    status_failed: "Failed",
    status_fee_unknown: "Fee unknown",
    status_ready: "Ready",
    status_below_floor: "Below floor",

    modal_title: "PSBT cannot be finalized",
    modal_instruction: "Correct the PSBT, then load it again.",
    btn_close: "Close",

    faq_eyebrow: "Questions you should ask before using this",
    faq_q_why: "Why is this tool helpful?",
    faq_a_why: "A normal transaction broadcast gossips your transaction to every Bitcoin node on \
        the network before it is mined. If someone holds a key that can also spend those coins, \
        they can replace the transaction and steal its coins before it gets mined. Slipstream \
        (what this tool uses) skips the gossip: the transaction goes to one miner without being \
        sent to the rest of the network. It will be slower to mine, but you will be safe from \
        transaction replacement.",
    faq_q_where: "Where does what I paste actually go?",
    faq_a_where: "Decoding, finalizing and fee maths all run in your browser, and this site has \
        no server in the middle. The only thing that ever leaves this page (and your computer) is \
        the finalized transaction hex, sent straight from your browser to MARA Slipstream when \
        you press Send.",
    faq_q_mara_learns: "What does MARA learn about me?",
    faq_a_mara_learns: "The transaction details, and your IP address. Your browser talks to MARA \
        directly, so the connection is yours and MARA sees the address you are browsing from. \
        MARA does not learn your PSBT metadata, your xpubs, your descriptor, or which other \
        transactions you queued here. Use Tor or a VPN if MARA seeing your IP matters to you.",
    faq_q_guaranteed: "Am I guaranteed to get my transaction mined?",
    faq_a_guaranteed: "No. Accepted by Slipstream is not confirmed. MARA mines only a portion of \
        blocks, not all of them, and may drop your transaction for reasons it does not have to \
        explain. Treat this as a better chance, not a promise. If it has not confirmed after a \
        while (hours), retry.",
    faq_q_rate_differs: "Why is the fee rate different from Mempool.space?",
    faq_a_rate_differs: "Slipstream gets paid for the service through a higher fee rate. This \
        website does not take any share of the payment, or any compensation.",
    faq_q_cpfp: "Can a second transaction pay the fee for a low-fee one (CPFP)?",
    faq_a_cpfp: "No. Slipstream looks at every transaction on its own, so one that does not pay \
        enough will not get mined here, even if you send another one paying extra to cover it. \
        You can still send several related transactions, as long as each pays enough by itself: \
        name the files 01, 02, 03 and they go out in that order. If you need one transaction to \
        pay for another, contact MARA directly.",
    faq_q_risks: "What are the risks of using this service?",
    faq_a_risks: "MARA, the company behind Slipstream, has to be trusted not to perform the \
        attack on your transaction itself. This is an acceptable risk compared to broadcasting it \
        publicly and letting ANYONE perform the attack.",
    faq_q_domain: "Why is this page on the Wizardsardine domain?",
    faq_a_domain: "We (Wizardsardine) are a security company, maintaining the Liana wallet \
        ([lianawallet.com](https://lianawallet.com)). Liana users did not have an easy way to \
        broadcast to Slipstream before this tool, so this is a service to them, open to the rest \
        of the community.",

    footer_built_by: "Built by [Wizardsardine](https://wizardsardine.com)",
    footer_disclosure: "Disclosure",
    footer_slipstream_terms: "Slipstream terms",

    msg_nothing_pasted: "Nothing pasted yet.",
    msg_unrecognised: "Not valid base64 PSBT or hex transaction data.",
    msg_too_large: "Transaction is {bytes} bytes as hex, over the {limit} MiB payload limit.",
    msg_no_inputs: "Transaction has no inputs.",
    msg_no_outputs: "Transaction has no outputs.",
    msg_rate_limited_note: "Slipstream is rate limiting this browser. Wait a few minutes and send \
        again.",
    msg_retrying_now: "Rate limited, retrying now…",
    msg_unreachable: "Could not reach MARA Slipstream. Check your connection, or whether a VPN, \
        proxy, or browser extension is blocking slipstream.mara.com.",
    msg_unexpected_response: "Unexpected response from Slipstream (HTTP {status})",

    msg_missing_utxo: Plurals::two(
        "Missing UTXO data for {n} input. Fee and final scripts could not be validated.",
        "Missing UTXO data for {n} inputs. Fee and final scripts could not be validated.",
    ),
    msg_retrying_in: Plurals::two(
        "Rate limited, retrying in {n} second…",
        "Rate limited, retrying in {n} seconds…",
    ),
    msg_duplicate: Plurals::two(
        "That transaction is already in the queue.",
        "Those transactions are already in the queue.",
    ),
};
