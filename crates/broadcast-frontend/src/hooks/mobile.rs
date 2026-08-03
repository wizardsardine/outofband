use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::MediaQueryList;
use yew::prelude::*;

use crate::tokens::RESPONSIVE_BREAKPOINT_QUERY;

/// Tracks the single responsive breakpoint via `window.matchMedia`, held in
/// state rather than read once, so the layout forks live as the viewport
/// crosses 860px.
#[hook]
pub fn use_mobile() -> bool {
    let mobile = use_state(|| media_query().map(|mql| mql.matches()).unwrap_or(false));

    {
        let mobile = mobile.clone();
        use_effect_with((), move |()| {
            let Some(mql) = media_query() else {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            };
            let listener = Closure::<dyn Fn(web_sys::MediaQueryListEvent)>::new(
                move |event: web_sys::MediaQueryListEvent| {
                    mobile.set(event.matches());
                },
            );
            mql.set_onchange(Some(listener.as_ref().unchecked_ref()));
            Box::new(move || {
                mql.set_onchange(None);
                drop(listener);
            }) as Box<dyn FnOnce()>
        });
    }

    *mobile
}

fn media_query() -> Option<MediaQueryList> {
    web_sys::window()?
        .match_media(RESPONSIVE_BREAKPOINT_QUERY)
        .ok()
        .flatten()
}
