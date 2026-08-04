//! Third-party walkthroughs of this page, linked from the prepare step.
//!
//! Not in `i18n`, because none of a row is translatable: wallet names,
//! author handles and a language's endonym read the same in every
//! catalogue. That is what makes the list identical on all nine language
//! versions of the page, which is the point. A reader on the French page
//! who speaks German should be offered the German video, and the only way
//! they can tell it is German is if the label says `Deutsch` rather than
//! `allemand`.
//!
//! Adding a guide is one entry here and no catalogue churn.

/// A guide, as one clickable row.
pub struct Guide {
    /// Format at a glance, so the row needs no word for "video".
    pub icon: &'static str,
    /// What the guide walks through, usually the wallets it covers.
    pub covers: &'static str,
    /// Who made it. Credit, and a signal of whether to trust it.
    pub author: &'static str,
    /// The language the guide is *in*, as its own endonym. Never the
    /// reader's language, and never translated.
    pub lang: &'static str,
    pub href: &'static str,
}

/// Written first, then video; within a format, the order they arrived.
pub static GUIDES: &[Guide] = &[
    Guide {
        icon: "📄",
        covers: "Sparrow, Liana",
        author: "ProfEduStream",
        lang: "English",
        href: "https://planb.academy/en/tutorials/wallet/desktop/slipstream-3d024596-70b8-4161-96c8-eb5bea8f3228",
    },
    Guide {
        icon: "▶️",
        covers: "Liana",
        author: "Chris",
        lang: "Deutsch",
        href: "https://www.youtube.com/watch?v=PnkJYQkQY3Y",
    },
];
