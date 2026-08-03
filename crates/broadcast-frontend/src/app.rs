use yew::prelude::*;

use crate::components::{
    ContextBanner, DisclosureStrip, Faq, FinalizationModal, Footer, Hero, PasteBox, QueueTable,
    StatStrip,
};
use crate::hooks::{use_fee, use_file_load, use_mobile, use_queue};

#[function_component(App)]
pub fn app() -> Html {
    let mobile = use_mobile();
    let fee = use_fee();
    let queue = use_queue();
    let has_items = !queue.items.is_empty();
    let on_files = use_file_load(queue.on_files_loaded.clone());

    html! {
        <div style="min-height:100vh;background:#000;color:#f4f4f4;font-family:'IBM Plex Sans',system-ui,sans-serif">
            <DisclosureStrip />
            <div style="padding:0 6%">
                <div style="max-width:1280px;margin:0 auto">
                    <ContextBanner />
                    <Hero mobile={mobile} fee={fee.clone()} />
                    // Above the paste box once it has rows: the queue is what
                    // the user came back to act on, and pushing it below a
                    // tall textarea buries it.
                    if has_items {
                        <div style="padding:44px 0 0">
                            <StatStrip
                                items={queue.items.clone()}
                                fee={fee.clone()}
                                mobile={mobile}
                                broadcasting={queue.broadcasting}
                                on_broadcast={queue.on_broadcast.clone()}
                            />
                            <QueueTable
                                items={queue.items.clone()}
                                fee={fee.clone()}
                                mobile={mobile}
                                broadcasting={queue.broadcasting}
                                on_remove={queue.on_remove.clone()}
                                on_set_total={queue.on_set_total.clone()}
                                on_send={queue.on_send.clone()}
                            />
                        </div>
                    }
                    <PasteBox
                        raw_text={queue.raw_text.clone()}
                        on_raw_text={queue.on_raw_text.clone()}
                        has_items={has_items}
                        parse_error={queue.parse_error.clone()}
                        on_submit={queue.on_submit.clone()}
                        on_clear={queue.on_clear.clone()}
                        broadcasting={queue.broadcasting}
                        on_files={on_files}
                    />
                    if !queue.refused_psbts.is_empty() {
                        <FinalizationModal
                            refused={queue.refused_psbts.clone()}
                            on_close={queue.on_dismiss_refused.clone()}
                        />
                    }
                    <Faq fee={fee} />
                    <Footer />
                </div>
            </div>
        </div>
    }
}
