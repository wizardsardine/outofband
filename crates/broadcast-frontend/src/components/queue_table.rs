use yew::prelude::*;

use crate::components::queue_row::QueueRow;
use crate::hooks::fee::FeeSnapshot;
use crate::queue::QueueItem;
use crate::tokens::{BORDER_STRONG, CARD_NESTED, QUEUE_ROW_COLUMNS, TEXT_MUTED_6A};

#[derive(Properties, PartialEq)]
pub struct QueueTableProps {
    pub items: Vec<QueueItem>,
    pub fee: Option<FeeSnapshot>,
    pub mobile: bool,
    pub broadcasting: bool,
    pub on_remove: Callback<u64>,
    pub on_set_total: Callback<(u64, Option<u64>)>,
    pub on_retry: Callback<u64>,
}

#[function_component(QueueTable)]
pub fn queue_table(props: &QueueTableProps) -> Html {
    // Until the floor loads, compare against infinity: a row never reads
    // "Ready" before we actually know the rate it must clear.
    let floor = props
        .fee
        .as_ref()
        .map_or(f64::INFINITY, |fee| fee.rate_sat_vb);

    html! {
        <div style={format!("border:1px solid {BORDER_STRONG};border-radius:0 0 44px 2px;background:{CARD_NESTED};overflow:hidden")}>
            if !props.mobile {
                <div style={table_head_style()}>
                    <span></span>
                    <span>{"Source"}</span>
                    <span>{"Format"}</span>
                    <span style="text-align:right">{"vsize"}</span>
                    <span style="text-align:right">{"Fee rate"}</span>
                    <span style="text-align:right">{"Status"}</span>
                    <span></span>
                </div>
            }
            { for props.items.iter().map(|item| html! {
                <QueueRow
                    key={item.id}
                    item={item.clone()}
                    floor={floor}
                    mobile={props.mobile}
                    broadcasting={props.broadcasting}
                    on_remove={props.on_remove.clone()}
                    on_set_total={props.on_set_total.clone()}
                    on_retry={props.on_retry.clone()}
                />
            }) }
        </div>
    }
}

fn table_head_style() -> String {
    format!(
        "display:grid;grid-template-columns:{QUEUE_ROW_COLUMNS};gap:16px;align-items:center;padding:12px 26px;border-bottom:1px solid #1f1f1f;background:#060606;font-size:10.5px;font-weight:500;letter-spacing:1.2px;text-transform:uppercase;color:{TEXT_MUTED_6A}"
    )
}
