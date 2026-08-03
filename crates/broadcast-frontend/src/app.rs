use yew::prelude::*;

use crate::components::{
    ContextBanner, DisclosureStrip, Faq, Footer, Hero, PasteBox, QueueTable, StatStrip,
};
use crate::hooks::{use_fee, use_mobile, use_queue};

#[function_component(App)]
pub fn app() -> Html {
    let mobile = use_mobile();
    let fee = use_fee();
    let queue = use_queue();
    let has_items = !queue.items.is_empty();

    html! {
        <div style="min-height:100vh;background:#000;color:#f4f4f4;font-family:'IBM Plex Sans',system-ui,sans-serif">
            <DisclosureStrip />
            <div style="padding:0 6%">
                <div style="max-width:1280px;margin:0 auto">
                    <ContextBanner />
                    <Hero mobile={mobile} fee={fee.clone()} />
                    <PasteBox
                        raw_text={queue.raw_text.clone()}
                        on_raw_text={queue.on_raw_text.clone()}
                        has_items={has_items}
                        parse_error={queue.parse_error.clone()}
                        on_submit={queue.on_submit.clone()}
                        on_clear={queue.on_clear.clone()}
                    />
                    if has_items {
                        <div style="padding:44px 0 0">
                            <StatStrip
                                items={queue.items.clone()}
                                fee={fee.clone()}
                                mobile={mobile}
                                broadcasting={queue.broadcasting}
                                on_broadcast={Callback::noop()}
                            />
                            <QueueTable
                                items={queue.items.clone()}
                                fee={fee.clone()}
                                mobile={mobile}
                                on_remove={queue.on_remove.clone()}
                                on_set_total={queue.on_set_total.clone()}
                            />
                        </div>
                    }
                    <Faq fee={fee} />
                    <Footer />
                </div>
            </div>
        </div>
    }
}
