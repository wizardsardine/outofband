use yew::prelude::*;

use crate::components::{
    ContextBanner, DisclosureStrip, Faq, FinalizationModal, Footer, Hero, PasteBox, PrepareStep,
    QueueTable, StatStrip, TranslationNotice,
};
use crate::hooks::{LangHandle, use_fee, use_file_load, use_lang_state, use_mobile, use_queue};

#[function_component(App)]
pub fn app() -> Html {
    let lang = use_lang_state();
    let mobile = use_mobile();
    let fee = use_fee();
    let queue = use_queue(lang.lang);
    let has_items = !queue.items.is_empty();
    let on_files = use_file_load(queue.on_files_loaded.clone());

    html! {
        <ContextProvider<LangHandle> context={lang}>
        <div style="min-height:100vh;background:#000;color:#f4f4f4;font-family:'IBM Plex Sans',system-ui,sans-serif">
            <DisclosureStrip />
            <TranslationNotice />
            <div style="padding:0 6%">
                <div style="max-width:1280px;margin:0 auto">
                    <ContextBanner />
                    <Hero mobile={mobile} fee={fee.clone()} />
                    <PrepareStep />
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
                    // Part of loading, so it reads as the result of the step
                    // above it rather than as a separate section: what you
                    // put in the box appears here, and the broadcast control
                    // sits with the rows it acts on.
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
                    if !queue.refused_psbts.is_empty() {
                        <FinalizationModal
                            refused={queue.refused_psbts.clone()}
                            on_close={queue.on_dismiss_refused.clone()}
                        />
                    }
                    <Faq />
                    <Footer />
                </div>
            </div>
        </div>
        </ContextProvider<LangHandle>>
    }
}
