use yew::prelude::*;

use crate::components::{ContextBanner, DisclosureStrip, Hero};
use crate::hooks::{use_fee, use_mobile};

#[function_component(App)]
pub fn app() -> Html {
    let mobile = use_mobile();
    let fee = use_fee();

    html! {
        <div style="min-height:100vh;background:#000;color:#f4f4f4;font-family:'IBM Plex Sans',system-ui,sans-serif">
            <DisclosureStrip />
            <div style="padding:0 6%">
                <div style="max-width:1280px;margin:0 auto">
                    <ContextBanner />
                    <Hero mobile={mobile} fee={fee} />
                </div>
            </div>
        </div>
    }
}
