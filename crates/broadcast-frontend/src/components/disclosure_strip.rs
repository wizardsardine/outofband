use yew::prelude::*;

#[function_component(DisclosureStrip)]
pub fn disclosure_strip() -> Html {
    html! {
        <div style="position:sticky;top:0;z-index:20;background:rgba(0,0,0,.94);border-bottom:1px solid #1a1a1a">
            <div style="padding:0 6%">
                <div style="max-width:1280px;margin:0 auto;display:flex;align-items:center;gap:10px 14px;min-height:42px;padding:7px 0;box-sizing:border-box;flex-wrap:wrap">
                    <span style="font-family:'IBM Plex Mono',monospace;font-size:10.5px;font-weight:500;letter-spacing:1.6px;text-transform:uppercase;color:#7b7b7b;white-space:nowrap">{"Security disclosure"}</span>
                    <span style="width:1px;height:14px;background:#2a2a2a"></span>
                    <a href="https://wizardsardine.com/blog/coldcard-rng-vulnerability/" target="_blank" style="display:flex;align-items:center;gap:8px;min-width:0;font-size:13.5px;line-height:1.35;color:#b0def0;text-decoration:none">
                        <span style="min-width:0;overflow-wrap:anywhere;text-decoration:underline;text-decoration-color:#4a6470;text-underline-offset:3px">{"Coldcard RNG vulnerability, and why you might need this tool"}</span>
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="flex:none">
                            <path d="M7 17 17 7"></path>
                            <path d="M9 7h8v8"></path>
                        </svg>
                    </a>
                </div>
            </div>
        </div>
    }
}
