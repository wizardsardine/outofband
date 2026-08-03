//! Page copy, one catalogue per language.
//!
//! [`Strings`] is a struct of fields rather than a key-value map, so a
//! missing translation is a compile error instead of a blank space or an
//! English fallback discovered by a user. Adding a string means adding a
//! field, which will not build until every language has answered for it.
//!
//! What is deliberately **not** translated:
//!
//! - Format names and units (`PSBT · base64`, `sat/vB`, file extensions).
//!   They are identifiers, not prose.
//! - MARA's rejection text. It arrives from `/api/transactions` as English
//!   prose written by someone else; inventing a translation would put words
//!   in their mouth, and paraphrasing a rejection reason is how a user ends
//!   up solving the wrong problem.
//! - `tx-core`'s deep decode diagnostics ("not valid base64: …"), which
//!   embed upstream library messages. The structural failures a user can
//!   actually act on are translated here instead, keyed off the error
//!   variant rather than its text.

pub mod markup;

mod de;
mod en;
mod es;
mod fr;
mod pt_br;
mod ru;

pub use markup::{RichStyles, rich};

/// Languages the page carries. Order is the order the switcher lists them,
/// English first and the rest alphabetical by endonym.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Lang {
    En,
    De,
    Es,
    Fr,
    PtBr,
    Ru,
}

impl Lang {
    pub const ALL: [Lang; 6] = [Lang::En, Lang::De, Lang::Es, Lang::Fr, Lang::PtBr, Lang::Ru];

    /// BCP 47 tag, for `<html lang>` and for persistence.
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::De => "de",
            Lang::Es => "es",
            Lang::Fr => "fr",
            Lang::PtBr => "pt-BR",
            Lang::Ru => "ru",
        }
    }

    /// The language's name in itself. A reader looking for their own
    /// language is not helped by seeing it named in English.
    pub fn endonym(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::De => "Deutsch",
            Lang::Es => "Español",
            Lang::Fr => "Français",
            Lang::PtBr => "Português",
            Lang::Ru => "Русский",
        }
    }

    /// Whether a native speaker has signed the translation off.
    ///
    /// This page tells people not to press a broadcast button, and a
    /// mistranslation there costs someone their coins. Unreviewed locales
    /// stay out of the switcher: the machinery ships, the languages appear
    /// one at a time as somebody vouches for them. Flip a flag here in the
    /// same commit that records who reviewed it.
    pub fn reviewed(self) -> bool {
        match self {
            Lang::En => true,
            Lang::De | Lang::Es | Lang::Fr | Lang::PtBr | Lang::Ru => false,
        }
    }

    /// Languages the switcher offers. Always contains at least English.
    pub fn offered() -> Vec<Lang> {
        Lang::ALL.into_iter().filter(|l| l.reviewed()).collect()
    }

    /// Matches a browser tag such as `pt-BR`, `pt`, or `es-419`.
    ///
    /// Region is honoured when we carry that region and ignored otherwise,
    /// so `pt-PT` still lands on the Portuguese catalogue rather than
    /// falling through to English. Case-insensitive: `navigator.languages`
    /// is not required to normalise it.
    pub fn from_tag(tag: &str) -> Option<Lang> {
        let tag = tag.trim().to_ascii_lowercase();
        if tag.is_empty() {
            return None;
        }
        if let Some(exact) = Lang::ALL
            .into_iter()
            .find(|l| l.code().eq_ignore_ascii_case(&tag))
        {
            return Some(exact);
        }
        let primary = tag.split(['-', '_']).next().unwrap_or(&tag);
        Lang::ALL
            .into_iter()
            .find(|l| l.code().split('-').next() == Some(primary))
    }

    pub fn strings(self) -> &'static Strings {
        match self {
            Lang::En => &en::STRINGS,
            Lang::De => &de::STRINGS,
            Lang::Es => &es::STRINGS,
            Lang::Fr => &fr::STRINGS,
            Lang::PtBr => &pt_br::STRINGS,
            Lang::Ru => &ru::STRINGS,
        }
    }

    /// Which plural form `n` takes. Only the three counted messages use
    /// this; everything else is written to avoid needing it.
    pub fn plural(self, n: u64) -> Plural {
        match self {
            // Russian: 1, 21, 31… one; 2-4, 22-24… few; the rest many.
            Lang::Ru => {
                let (tens, units) = (n % 100, n % 10);
                if units == 1 && tens != 11 {
                    Plural::One
                } else if (2..=4).contains(&units) && !(12..=14).contains(&tens) {
                    Plural::Few
                } else {
                    Plural::Many
                }
            }
            // French counts 0 as singular; the rest of the Latin set does not.
            Lang::Fr => {
                if n <= 1 {
                    Plural::One
                } else {
                    Plural::Other
                }
            }
            Lang::En | Lang::De | Lang::Es | Lang::PtBr => {
                if n == 1 {
                    Plural::One
                } else {
                    Plural::Other
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Plural {
    One,
    Few,
    Many,
    Other,
}

/// The forms one counted message takes. Languages with two forms repeat
/// `other` into `few`/`many`; spelling that out beats a fallback chain that
/// silently picks the wrong one.
pub struct Plurals {
    pub one: &'static str,
    pub few: &'static str,
    pub many: &'static str,
    pub other: &'static str,
}

impl Plurals {
    /// Two-form languages: everything that is not exactly one.
    pub const fn two(one: &'static str, other: &'static str) -> Self {
        Self {
            one,
            few: other,
            many: other,
            other,
        }
    }

    pub fn pick(&self, lang: Lang, n: u64) -> &'static str {
        match lang.plural(n) {
            Plural::One => self.one,
            Plural::Few => self.few,
            Plural::Many => self.many,
            Plural::Other => self.other,
        }
    }
}

/// Every string the page can show. Field order follows the page top to
/// bottom, then the messages that only appear once something goes wrong.
pub struct Strings {
    // Document
    pub page_title: &'static str,
    pub lang_picker_label: &'static str,

    // Disclosure strip
    pub disclosure_label: &'static str,
    pub disclosure_link: &'static str,

    // Context banner
    pub context_audience: &'static str,
    pub context_not_recommended: &'static str,

    // Hero
    pub hero_line_one: &'static str,
    pub hero_line_two: &'static str,

    // Fee card
    pub fee_eyebrow: &'static str,
    pub fee_sentence: &'static str,
    pub fee_stale: &'static str,
    pub fee_advice: &'static str,

    // Prepare step
    pub prepare_heading: &'static str,
    pub prepare_liana: &'static str,
    pub prepare_drafts: &'static str,
    pub prepare_other_wallets: &'static str,
    pub prepare_tutorials: &'static str,

    // Load section
    pub load_heading: &'static str,
    pub load_blurb: &'static str,
    pub load_placeholder_or: &'static str,
    pub load_placeholder_drop: &'static str,
    pub load_accepts: &'static str,
    pub btn_add_to_queue: &'static str,
    pub btn_choose_files: &'static str,
    pub btn_clear_queue: &'static str,
    pub parse_error_title: &'static str,
    pub detected_psbt: &'static str,
    pub detected_raw: &'static str,
    pub detected_unrecognised: &'static str,

    // Stat strip
    pub stat_in_queue: &'static str,
    pub stat_clear_floor: &'static str,
    pub stat_below_floor: &'static str,
    pub stat_fee_unknown: &'static str,
    pub btn_send: &'static str,
    /// `{n}` is the number of submittable rows.
    pub btn_send_batch: &'static str,
    pub btn_sending: &'static str,

    // Queue table
    pub col_source: &'static str,
    pub col_format: &'static str,
    pub col_vsize: &'static str,
    pub col_fee_rate: &'static str,
    pub col_status: &'static str,
    pub btn_retry: &'static str,
    pub row_remove_title: &'static str,
    pub row_missing_value: &'static str,
    pub row_total_placeholder: &'static str,

    // Row status
    pub status_invalid: &'static str,
    pub status_accepted: &'static str,
    pub status_sending: &'static str,
    pub status_rejected: &'static str,
    pub status_rate_limited: &'static str,
    pub status_failed: &'static str,
    pub status_fee_unknown: &'static str,
    pub status_ready: &'static str,
    pub status_below_floor: &'static str,

    // Finalization modal
    pub modal_title: &'static str,
    pub modal_instruction: &'static str,
    pub btn_close: &'static str,

    // FAQ
    pub faq_eyebrow: &'static str,
    pub faq_q_why: &'static str,
    pub faq_a_why: &'static str,
    pub faq_q_where: &'static str,
    pub faq_a_where: &'static str,
    pub faq_q_mara_learns: &'static str,
    pub faq_a_mara_learns: &'static str,
    pub faq_q_guaranteed: &'static str,
    pub faq_a_guaranteed: &'static str,
    pub faq_q_rate_differs: &'static str,
    pub faq_a_rate_differs: &'static str,
    pub faq_q_cpfp: &'static str,
    pub faq_a_cpfp: &'static str,
    pub faq_q_risks: &'static str,
    pub faq_a_risks: &'static str,
    pub faq_q_domain: &'static str,
    pub faq_a_domain: &'static str,

    // Footer
    pub footer_built_by: &'static str,
    pub footer_disclosure: &'static str,
    pub footer_slipstream_terms: &'static str,

    // Messages
    pub msg_nothing_pasted: &'static str,
    pub msg_unrecognised: &'static str,
    /// `{bytes}` and `{limit}`.
    pub msg_too_large: &'static str,
    pub msg_no_inputs: &'static str,
    pub msg_no_outputs: &'static str,
    pub msg_rate_limited_note: &'static str,
    pub msg_retrying_now: &'static str,
    pub msg_unreachable: &'static str,
    /// `{status}` is the HTTP status code.
    pub msg_unexpected_response: &'static str,

    // Counted messages
    /// `{n}` inputs whose UTXO data is missing.
    pub msg_missing_utxo: Plurals,
    /// `{n}` seconds left on a rate-limit backoff.
    pub msg_retrying_in: Plurals,
    /// `{n}` transactions already queued.
    pub msg_duplicate: Plurals,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_a_distinct_code_and_endonym() {
        for lang in Lang::ALL {
            let same_code = Lang::ALL.iter().filter(|l| l.code() == lang.code()).count();
            assert_eq!(same_code, 1, "{lang:?} shares its code");
            let same_name = Lang::ALL
                .iter()
                .filter(|l| l.endonym() == lang.endonym())
                .count();
            assert_eq!(same_name, 1, "{lang:?} shares its endonym");
        }
    }

    #[test]
    fn every_code_round_trips_through_from_tag() {
        for lang in Lang::ALL {
            assert_eq!(Lang::from_tag(lang.code()), Some(lang), "{lang:?}");
        }
    }

    #[test]
    fn a_region_we_do_not_carry_still_finds_its_language() {
        // pt-PT is not pt-BR, but Portuguese copy beats English copy.
        assert_eq!(Lang::from_tag("pt-PT"), Some(Lang::PtBr));
        assert_eq!(Lang::from_tag("es-419"), Some(Lang::Es));
        assert_eq!(Lang::from_tag("de-CH"), Some(Lang::De));
        assert_eq!(Lang::from_tag("EN-GB"), Some(Lang::En));
        assert_eq!(Lang::from_tag("fr_CA"), Some(Lang::Fr));
    }

    #[test]
    fn an_unknown_or_empty_tag_matches_nothing() {
        assert_eq!(Lang::from_tag("ja"), None);
        assert_eq!(Lang::from_tag(""), None);
        assert_eq!(Lang::from_tag("   "), None);
        // Not a prefix match on the primary subtag: "eng" is not "en".
        assert_eq!(Lang::from_tag("eng"), None);
    }

    #[test]
    fn english_is_always_offered() {
        assert!(Lang::offered().contains(&Lang::En));
    }

    #[test]
    fn russian_plurals_follow_the_slavic_rule() {
        for (n, want) in [
            (1, Plural::One),
            (2, Plural::Few),
            (4, Plural::Few),
            (5, Plural::Many),
            (11, Plural::Many),
            (12, Plural::Many),
            (14, Plural::Many),
            (21, Plural::One),
            (22, Plural::Few),
            (25, Plural::Many),
            (0, Plural::Many),
        ] {
            assert_eq!(Lang::Ru.plural(n), want, "ru n={n}");
        }
    }

    #[test]
    fn french_counts_zero_as_singular_and_english_does_not() {
        assert_eq!(Lang::Fr.plural(0), Plural::One);
        assert_eq!(Lang::En.plural(0), Plural::Other);
        assert_eq!(Lang::En.plural(1), Plural::One);
    }

    #[test]
    fn two_form_plurals_never_return_an_empty_string() {
        // Guards the `Plurals::two` shorthand: few/many must be populated,
        // not left blank for languages that never ask for them.
        let p = Plurals::two("one", "many");
        for lang in Lang::ALL {
            for n in [0, 1, 2, 5, 21, 22] {
                assert!(!p.pick(lang, n).is_empty(), "{lang:?} n={n}");
            }
        }
    }

    #[test]
    fn placeholders_survive_translation() {
        // A translator dropping `{n}` turns a count into a sentence that
        // says nothing. Check every catalogue keeps the placeholders its
        // English original has.
        for lang in Lang::ALL {
            let s = lang.strings();
            assert!(s.btn_send_batch.contains("{n}"), "{lang:?} btn_send_batch");
            assert!(
                s.msg_unexpected_response.contains("{status}"),
                "{lang:?} msg_unexpected_response"
            );
            assert!(s.msg_too_large.contains("{bytes}"), "{lang:?} bytes");
            assert!(s.msg_too_large.contains("{limit}"), "{lang:?} limit");
            for form in [
                s.msg_missing_utxo.one,
                s.msg_missing_utxo.few,
                s.msg_missing_utxo.many,
                s.msg_missing_utxo.other,
                s.msg_retrying_in.one,
                s.msg_retrying_in.few,
                s.msg_retrying_in.many,
                s.msg_retrying_in.other,
            ] {
                assert!(form.contains("{n}"), "{lang:?} counted form: {form}");
            }
        }
    }

    #[test]
    fn no_catalogue_string_is_empty() {
        // An empty field compiles and renders as a hole in the page.
        for lang in Lang::ALL {
            let s = lang.strings();
            for (name, value) in [
                ("page_title", s.page_title),
                ("disclosure_label", s.disclosure_label),
                ("disclosure_link", s.disclosure_link),
                ("context_audience", s.context_audience),
                ("context_not_recommended", s.context_not_recommended),
                ("hero_line_one", s.hero_line_one),
                ("hero_line_two", s.hero_line_two),
                ("fee_eyebrow", s.fee_eyebrow),
                ("fee_sentence", s.fee_sentence),
                ("fee_advice", s.fee_advice),
                ("prepare_heading", s.prepare_heading),
                ("prepare_liana", s.prepare_liana),
                ("prepare_other_wallets", s.prepare_other_wallets),
                ("load_heading", s.load_heading),
                ("load_blurb", s.load_blurb),
                ("btn_add_to_queue", s.btn_add_to_queue),
                ("btn_send", s.btn_send),
                ("faq_eyebrow", s.faq_eyebrow),
                ("faq_a_why", s.faq_a_why),
                ("faq_a_domain", s.faq_a_domain),
                ("msg_unreachable", s.msg_unreachable),
            ] {
                assert!(!value.trim().is_empty(), "{lang:?} {name} is empty");
            }
        }
    }

    #[test]
    fn no_user_visible_string_carries_an_em_dash() {
        // PLAN.md section 4: em dashes are out of the page copy, and a
        // translation is the easiest place for one to slip back in.
        for lang in Lang::ALL {
            let s = lang.strings();
            for value in [
                s.page_title,
                s.context_audience,
                s.context_not_recommended,
                s.hero_line_two,
                s.fee_sentence,
                s.fee_advice,
                s.prepare_liana,
                s.prepare_other_wallets,
                s.prepare_tutorials,
                s.load_blurb,
                s.faq_a_why,
                s.faq_a_cpfp,
                s.faq_a_risks,
                s.faq_a_domain,
                s.msg_unreachable,
                s.msg_retrying_now,
            ] {
                assert!(!value.contains('—'), "{lang:?} em dash in: {value}");
            }
        }
    }
}
