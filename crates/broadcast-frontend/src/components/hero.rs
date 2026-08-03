use yew::prelude::*;

use crate::components::fee_card::FeeCard;
use crate::hooks::fee::FeeSnapshot;
use crate::hooks::use_t;
use crate::tokens::{
    BORDER_STRONG, HAIRLINE, HEADLINE_GRADIENT_TEAL, HEADLINE_GRADIENT_VIOLET, TEXT_SECONDARY,
};

#[derive(Properties, PartialEq)]
pub struct HeroProps {
    pub mobile: bool,
    pub fee: Option<FeeSnapshot>,
}

#[function_component(Hero)]
pub fn hero(props: &HeroProps) -> Html {
    let t = use_t();
    let grid_style = hero_grid_style(props.mobile);
    let h1_style = h1_style(props.mobile);
    let chip_style = format!(
        "font-family:'IBM Plex Mono',monospace;font-size:11px;letter-spacing:.5px;color:{TEXT_SECONDARY};border:1px solid {BORDER_STRONG};border-radius:2px;padding:7px 11px"
    );
    let line_one_style = format!(
        "background:{HEADLINE_GRADIENT_TEAL};-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent"
    );
    let line_two_style = format!(
        "background:{HEADLINE_GRADIENT_VIOLET};-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent"
    );

    html! {
        <div style={grid_style}>
            <div>
                <h1 style={h1_style}>
                    <span style={line_one_style}>{t.hero_line_one}</span>
                    <br />
                    <span style={line_two_style}>{t.hero_line_two}</span>
                </h1>
                <div style="display:flex;flex-wrap:wrap;gap:9px;margin-top:32px">
                    <span style={chip_style.clone()}>{"PSBT · base64"}</span>
                    <span style={chip_style.clone()}>{"RAW TX · hex"}</span>
                    <span style={chip_style}>{".TXT .PSBT .TXN .TAR.GZ .ZIP"}</span>
                </div>
            </div>
            <FeeCard mobile={props.mobile} fee={props.fee.clone()} />
        </div>
    }
}

fn hero_grid_style(mobile: bool) -> String {
    if mobile {
        format!(
            "display:flex;flex-direction:column;gap:32px;padding:26px 0 44px;border-bottom:1px solid {HAIRLINE}"
        )
    } else {
        format!(
            "display:grid;grid-template-columns:minmax(0,1.05fr) minmax(0,.95fr);gap:60px;align-items:center;padding:20px 0 60px;border-bottom:1px solid {HAIRLINE}"
        )
    }
}

/// Sized down from the mockup's 64px: the headline now carries the
/// "without using the public mempool" clause, and at 64px that clause
/// wrapped into a block tall enough to push the fee card out of the fold.
fn h1_style(mobile: bool) -> String {
    let (letter_spacing, font_size) = if mobile {
        ("-.5px", "31px")
    } else {
        ("-1px", "48px")
    };
    format!(
        "margin:0;font-weight:700;line-height:1.06;letter-spacing:{letter_spacing};font-size:{font_size}"
    )
}
