use yew::prelude::*;

use crate::hooks::fee::FeeSnapshot;
use crate::queue::{self, QueueItem};
use crate::tokens::{
    self, ACCENT_TEAL_BRIGHT, BORDER_STRONG, ERROR_RED, RULE, TEXT_MUTED_7B, TEXT_PRIMARY,
    TEXT_SECONDARY,
};

#[derive(Properties, PartialEq)]
pub struct StatStripProps {
    pub items: Vec<QueueItem>,
    pub fee: Option<FeeSnapshot>,
    pub mobile: bool,
    pub broadcasting: bool,
    pub on_broadcast: Callback<()>,
}

#[function_component(StatStrip)]
pub fn stat_strip(props: &StatStripProps) -> Html {
    let floor = props
        .fee
        .as_ref()
        .map_or(f64::INFINITY, |fee| fee.rate_sat_vb);
    let stats = queue::stats(&props.items, floor);
    let submittable = props
        .items
        .iter()
        .filter(|item| item.is_submittable())
        .count();
    let disabled = submittable == 0 || props.broadcasting;
    let label = queue::send_label(submittable, props.broadcasting);

    let onclick = {
        let on_broadcast = props.on_broadcast.clone();
        Callback::from(move |_| on_broadcast.emit(()))
    };

    html! {
        <div style={strip_grid_style(props.mobile)}>
            { stat_cell(0, props.mobile, "In queue", &stats.total.to_string(), TEXT_PRIMARY) }
            { stat_cell(1, props.mobile, "Clear the floor", &stats.at_or_above_floor.to_string(), ACCENT_TEAL_BRIGHT) }
            { stat_cell(2, props.mobile, "Below floor", &stats.below_floor.to_string(), ERROR_RED) }
            { stat_cell(3, props.mobile, "Fee unknown", &stats.fee_unknown.to_string(), TEXT_SECONDARY) }
            <div style={cta_style(props.mobile)}>
                <button
                    {onclick}
                    {disabled}
                    class={if disabled { "" } else { "primary-btn" }}
                    style={broadcast_button_style(!disabled, props.mobile)}
                >{label}</button>
            </div>
        </div>
    }
}

fn stat_cell(index: usize, mobile: bool, label: &str, value: &str, value_color: &str) -> Html {
    html! {
        <div style={cell_style(index, mobile)}>
            <div style={format!("font-size:11px;font-weight:500;letter-spacing:1.4px;text-transform:uppercase;color:{TEXT_MUTED_7B}")}>{label.to_string()}</div>
            <div style={format!("font-family:'IBM Plex Mono',monospace;font-size:28px;font-weight:600;margin-top:6px;color:{value_color}")}>{value.to_string()}</div>
        </div>
    }
}

fn cell_style(index: usize, mobile: bool) -> String {
    let per_row = if mobile { 2 } else { 4 };
    let mut style = "padding:20px 26px".to_string();
    if index % per_row != per_row - 1 {
        style.push_str(&format!(";border-right:1px solid {RULE}"));
    }
    if mobile && index < 2 {
        style.push_str(&format!(";border-bottom:1px solid {RULE}"));
    }
    style
}

fn strip_grid_style(mobile: bool) -> String {
    let columns = if mobile {
        "repeat(2,minmax(0,1fr))"
    } else {
        "repeat(4,minmax(0,1fr)) minmax(190px,1.4fr)"
    };
    format!(
        "display:grid;grid-template-columns:{columns};border:1px solid {BORDER_STRONG};border-bottom:0;border-radius:44px 2px 0 0;background:#0c0c0c;overflow:hidden"
    )
}

fn cta_style(mobile: bool) -> String {
    let base = "padding:20px 26px;display:flex;align-items:center;";
    if mobile {
        format!("{base}grid-column:1 / -1;border-top:1px solid {RULE}")
    } else {
        format!("{base}justify-content:flex-end")
    }
}

fn broadcast_button_style(enabled: bool, mobile: bool) -> String {
    let mut style = format!(
        "{};white-space:nowrap",
        tokens::primary_button_style(enabled, 26)
    );
    if mobile {
        style.push_str(";width:100%;justify-content:center");
    }
    style
}
