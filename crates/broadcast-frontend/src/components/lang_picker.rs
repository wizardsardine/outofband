use web_sys::HtmlSelectElement;
use yew::prelude::*;

use crate::hooks::use_lang;
use crate::i18n::Lang;
use crate::tokens::{BODY_COPY, BORDER_STRONG, TEXT_SECONDARY};

/// A `<select>`, not a row of links or flags.
///
/// Flags name countries, not languages, and there is no flag for Spanish or
/// Portuguese that does not tell most of their speakers the page is for
/// somebody else. A native control also comes with keyboard handling, a
/// touch-friendly picker and screen-reader support already correct, which a
/// hand-rolled dropdown on a page this small does not earn.
///
/// Renders nothing while English is the only reviewed language: a picker
/// with one option is furniture.
#[function_component(LangPicker)]
pub fn lang_picker() -> Html {
    let handle = use_lang();
    let offered = Lang::offered();

    if offered.len() < 2 {
        return Html::default();
    }

    let onchange = {
        let set = handle.set.clone();
        Callback::from(move |event: Event| {
            let select: HtmlSelectElement = event.target_unchecked_into();
            if let Some(lang) = Lang::from_tag(&select.value()) {
                set.emit(lang);
            }
        })
    };

    let select_style = format!(
        "appearance:none;border:1px solid {BORDER_STRONG};border-radius:2px;background:#000;\
         color:{TEXT_SECONDARY};font-family:inherit;font-size:13.5px;padding:7px 30px 7px 10px;\
         cursor:pointer"
    );
    // The chevron the native arrow would have drawn, kept because
    // `appearance:none` removes it along with the platform styling that
    // clashes with the page.
    let wrap_style = "position:relative;display:inline-flex;align-items:center";
    let chevron_style =
        format!("position:absolute;right:9px;display:flex;pointer-events:none;color:{BODY_COPY}");

    html! {
        <span style={wrap_style}>
            <select
                {onchange}
                value={handle.lang.code()}
                aria-label={handle.strings().lang_picker_label}
                style={select_style}
            >
                { for offered.into_iter().map(|lang| html! {
                    <option value={lang.code()} selected={lang == handle.lang}>
                        {lang.endonym()}
                    </option>
                }) }
            </select>
            <span style={chevron_style}>
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"></path></svg>
            </span>
        </span>
    }
}
