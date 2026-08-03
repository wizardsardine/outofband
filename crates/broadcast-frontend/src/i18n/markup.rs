//! A three-marker inline syntax for catalogue strings, so a translator gets
//! one contiguous sentence to work with instead of a row of fragments.
//!
//! Splitting "…mainly **Liana**, **Miniscript**, or some types of
//! **multisig** wallets" into six catalogue entries would force every
//! language into English word order, which is the usual way a segmented
//! catalogue goes wrong. Instead the emphasis travels inside the string and
//! the translator moves it with the words it belongs to:
//!
//! - `*text*` — emphasised, in the caller's [`RichStyles::emphasis`]
//! - `!text!` — the warning voice, in [`RichStyles::shout`] (`DO NOT`)
//! - `[text](https://…)` — a link, opened in a new tab
//!
//! Markers do not nest, and an unclosed one renders literally rather than
//! swallowing the rest of the sentence: a translator's typo should look
//! wrong, not delete a paragraph.

use yew::prelude::*;

/// Emphasis styling is per call site, not per marker: the same `*text*`
/// is white-on-black in the context banner and amber inside a warn card.
#[derive(Clone, PartialEq)]
pub struct RichStyles {
    pub emphasis: String,
    pub shout: String,
}

impl RichStyles {
    pub fn new(emphasis: impl Into<String>, shout: impl Into<String>) -> Self {
        Self {
            emphasis: emphasis.into(),
            shout: shout.into(),
        }
    }

    /// For strings whose only markup is a link.
    pub fn plain() -> Self {
        Self::new(String::new(), String::new())
    }
}

/// Renders one catalogue string into a fragment of styled nodes.
pub fn rich(text: &str, styles: &RichStyles) -> Html {
    let mut nodes: Vec<Html> = Vec::new();
    let mut literal = String::new();
    let mut rest = text;

    while let Some(index) = rest.find(['*', '!', '[']) {
        let (before, from_marker) = rest.split_at(index);
        literal.push_str(before);

        let mut chars = from_marker.chars();
        let marker = chars.next().expect("find() located a marker char");
        let after_marker = chars.as_str();

        let parsed = match marker {
            '*' => close_at(after_marker, '*').map(|(inner, tail)| {
                (
                    html! { <strong style={styles.emphasis.clone()}>{inner.to_string()}</strong> },
                    tail,
                )
            }),
            '!' => close_at(after_marker, '!').map(|(inner, tail)| {
                (
                    html! { <strong style={styles.shout.clone()}>{inner.to_string()}</strong> },
                    tail,
                )
            }),
            _ => parse_link(after_marker),
        };

        match parsed {
            Some((node, tail)) => {
                push_literal(&mut nodes, &mut literal);
                nodes.push(node);
                rest = tail;
            }
            // Unclosed marker: keep it as text and carry on past it, so one
            // bad character costs one character rather than the paragraph.
            None => {
                literal.push(marker);
                rest = after_marker;
            }
        }
    }

    literal.push_str(rest);
    push_literal(&mut nodes, &mut literal);
    Html::from_iter(nodes)
}

fn push_literal(nodes: &mut Vec<Html>, literal: &mut String) {
    if !literal.is_empty() {
        nodes.push(html! { {literal.clone()} });
        literal.clear();
    }
}

/// Splits at the next `close`, returning the text before it and the text
/// after. A marker that never closes yields `None`.
fn close_at(rest: &str, close: char) -> Option<(&str, &str)> {
    let end = rest.find(close)?;
    Some((&rest[..end], &rest[end + close.len_utf8()..]))
}

/// `text](href)`, the remainder of a `[text](href)` link after its `[`.
fn parse_link(rest: &str) -> Option<(Html, &str)> {
    let (label, after_label) = close_at(rest, ']')?;
    let mut chars = after_label.chars();
    if chars.next()? != '(' {
        return None;
    }
    let (href, tail) = close_at(chars.as_str(), ')')?;
    let node = html! {
        <a href={href.to_string()} target="_blank" rel="noopener noreferrer">{label.to_string()}</a>
    };
    Some((node, tail))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Renders to a string the way the tests want to read it: markup
    /// stripped, text preserved, so a catalogue string can be checked for
    /// what it says as well as for how it is marked up.
    fn text_of(html: &Html) -> String {
        format!("{html:?}")
    }

    #[test]
    fn plain_text_survives_untouched() {
        let out = rich("No markers here.", &RichStyles::plain());
        assert!(text_of(&out).contains("No markers here."));
    }

    #[test]
    fn an_unclosed_marker_does_not_swallow_the_rest() {
        // The bug this guards against: `find('*')` succeeding, no close, and
        // the remainder vanishing from the page.
        let out = rich("half *open sentence", &RichStyles::plain());
        let rendered = text_of(&out);
        assert!(rendered.contains("open sentence"), "{rendered}");
    }

    #[test]
    fn a_link_needs_both_halves() {
        // `[label]` with no `(href)` is text, not a dropped fragment.
        let out = rich("see [the docs] please", &RichStyles::plain());
        let rendered = text_of(&out);
        assert!(rendered.contains("the docs"), "{rendered}");
        assert!(rendered.contains("please"), "{rendered}");
    }

    #[test]
    fn markers_split_the_sentence_into_parts() {
        let styles = RichStyles::new("font-weight:600", "font-weight:700");
        let out = rich("a *b* c !d! e [f](https://x.test) g", &styles);
        let rendered = text_of(&out);
        for fragment in ["a ", "b", " c ", "d", " e ", "f", " g"] {
            assert!(
                rendered.contains(fragment),
                "missing {fragment:?}: {rendered}"
            );
        }
    }

    #[test]
    fn multibyte_text_around_markers_does_not_panic() {
        // close_at() slices by byte index; accented and CJK copy must not
        // land mid-character.
        let out = rich(
            "café *München* 東京 [ü](https://x.test)",
            &RichStyles::plain(),
        );
        let rendered = text_of(&out);
        assert!(rendered.contains("München"), "{rendered}");
        assert!(rendered.contains("東京"), "{rendered}");
    }
}
