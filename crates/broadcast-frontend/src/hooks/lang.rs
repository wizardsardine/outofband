//! Which language the page is in: chosen, remembered, or guessed.
//!
//! Order of preference is a stored choice, then the browser's own list,
//! then English. An explicit choice outranks `navigator.languages` because
//! a reader who picked a language on this page meant it, and their browser
//! is often set to whatever the machine shipped with.

use yew::prelude::*;

use crate::i18n::{Lang, Strings};

/// Namespaced because this page is served from a domain that hosts other
/// things; a bare `lang` key is a collision waiting to happen.
const STORAGE_KEY: &str = "outofband.lang";

#[derive(Clone, PartialEq)]
pub struct LangHandle {
    pub lang: Lang,
    pub set: Callback<Lang>,
}

impl LangHandle {
    pub fn strings(&self) -> &'static Strings {
        self.lang.strings()
    }
}

/// Owns the language state. Called once, in `App`, and handed to the tree
/// through a context.
#[hook]
pub fn use_lang_state() -> LangHandle {
    let lang = use_state(initial_lang);

    // `<html lang>` drives screen-reader pronunciation and the browser's
    // offer to translate; the title is what a tab and a bookmark show.
    // Both live outside the yew tree, so they are set as an effect rather
    // than rendered.
    {
        let current = *lang;
        use_effect_with(current, move |current| {
            let current = *current;
            if let Some(document) = document() {
                if let Some(root) = document.document_element() {
                    let _ = root.set_attribute("lang", current.code());
                }
                document.set_title(current.strings().page_title);
            }
            || ()
        });
    }

    let set = {
        let lang = lang.clone();
        Callback::from(move |next: Lang| {
            store(next);
            lang.set(next);
        })
    };

    LangHandle { lang: *lang, set }
}

/// Reads the language from context. Every component below `App` uses this.
#[hook]
pub fn use_lang() -> LangHandle {
    // A component rendered outside the provider is a wiring mistake, not a
    // user-facing condition: fall back to English rather than panicking in
    // a browser where a panic takes the whole page down.
    use_context::<LangHandle>().unwrap_or_else(|| LangHandle {
        lang: Lang::En,
        set: Callback::noop(),
    })
}

/// The catalogue for the current language, for components that only read
/// copy and never switch it.
#[hook]
pub fn use_t() -> &'static Strings {
    use_lang().lang.strings()
}

/// No `reviewed()` filter here: an unreviewed language is offered, so it
/// is also detected. Landing a French reader on French with the notice
/// showing beats landing them on English they may not read.
fn initial_lang() -> Lang {
    stored().or_else(from_browser).unwrap_or(Lang::En)
}

fn document() -> Option<web_sys::Document> {
    web_sys::window()?.document()
}

fn local_storage() -> Option<web_sys::Storage> {
    // Throws rather than returning null under some privacy settings, which
    // `ok()?` turns back into "no storage" instead of a dead page.
    web_sys::window()?.local_storage().ok()?
}

fn stored() -> Option<Lang> {
    let raw = local_storage()?.get_item(STORAGE_KEY).ok()??;
    Lang::from_tag(&raw)
}

fn store(lang: Lang) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(STORAGE_KEY, lang.code());
    }
}

/// First entry of `navigator.languages` we carry. The list is in the user's
/// own order of preference, so the first match wins rather than the best.
fn from_browser() -> Option<Lang> {
    let navigator = web_sys::window()?.navigator();
    for value in navigator.languages().iter() {
        if let Some(tag) = value.as_string() {
            if let Some(lang) = Lang::from_tag(&tag) {
                return Some(lang);
            }
        }
    }
    // Older browsers, and any where `languages` came back empty.
    Lang::from_tag(&navigator.language()?)
}
