//! Third-party walkthroughs of this page, linked from the prepare step.
//!
//! Every language sees every guide. A reader on the French page who also
//! reads German should be offered the German video, so the list is not
//! filtered by the page's language; instead each row says which language
//! the guide itself is in.
//!
//! That label is the guide's own endonym, never translated: `Deutsch` tells
//! a French, Japanese and Russian reader the same true thing, where
//! `allemand` only works for one of them. So a row carries no translatable
//! text at all, and adding a guide is one entry here. The exception is a
//! guide published in many languages, which has no single endonym to name
//! and borrows [`Strings::guides_multilingual`] instead.
//!
//! [`Strings::guides_multilingual`]: crate::i18n::Strings::guides_multilingual

/// A guide, as one clickable row.
pub struct Guide {
    /// Format at a glance, so the row needs no word for "video".
    pub icon: &'static str,
    /// What the guide walks through, usually the wallets it covers.
    pub covers: &'static str,
    /// Who made it. Credit, and a signal of whether to trust it.
    pub author: &'static str,
    /// The language the guide is *in*, as its own endonym. `None` for a
    /// guide its author publishes in several languages, which the page
    /// labels from the catalogue rather than naming them all.
    pub lang: Option<&'static str>,
    pub href: &'static str,
}

/// Written first, then video; within a format, the order they arrived.
pub static GUIDES: &[Guide] = &[
    Guide {
        icon: "📄",
        covers: "Sparrow, Liana",
        author: "ProfEduStream",
        // Published in several languages on planb.academy. The link keeps
        // its `/en/` path because that is the one confirmed to exist; the
        // site offers the reader its own locale from there.
        lang: None,
        href: "https://planb.academy/en/tutorials/wallet/desktop/slipstream-3d024596-70b8-4161-96c8-eb5bea8f3228",
    },
    Guide {
        icon: "▶️",
        covers: "Liana",
        author: "Chris",
        lang: Some("Deutsch"),
        href: "https://www.youtube.com/watch?v=PnkJYQkQY3Y",
    },
    Guide {
        icon: "▶️",
        covers: "Liana",
        author: "BTC Andres",
        lang: Some("Español"),
        href: "https://www.youtube.com/watch?v=mQYk4ccqkTk",
    },
];
