//! Design tokens transcribed from `UI_MOCKUP.html` (PLAN.md section 5):
//! the palette, surface tints, note-card colour triples and gradients
//! for the whole frontend. Some tokens here are not yet consumed by any
//! component in the current phase; later phases draw on them instead of
//! re-deriving the values from the mockup.
#![allow(dead_code)]

pub const BORDER_STRONG: &str = "#2a2a2a";
pub const HAIRLINE: &str = "#1a1a1a";
pub const RULE: &str = "#1f1f1f";
pub const TEXT_MUTED_7B: &str = "#7b7b7b";
pub const TEXT_MUTED_6A: &str = "#6a6a6a";
pub const TEXT_DISABLED: &str = "#4a4a4a";
pub const TEXT_PLACEHOLDER: &str = "#454545";
pub const FIELD_TEXT: &str = "#e6e6e6";
pub const TEXT_SECONDARY: &str = "#a1a1a1";
pub const BODY_COPY: &str = "#909090";
pub const CARD_NESTED: &str = "#060606";
pub const ACCENT_TEAL: &str = "#5fe7e4";
pub const ERROR_RED: &str = "#ef445f";
pub const WARNING: &str = "#e0b341";

/// Accepted-row background tint.
pub const SURFACE_ACCEPTED: &str = "#07100f";
/// Drag-over highlight background; paired with an [`ACCENT_TEAL`] border.
pub const SURFACE_DRAG_OVER: &str = "#07171a";
/// "Could not parse" card background; paired with a [`SURFACE_PARSE_ERROR_BORDER`]
/// border and an [`ERROR_RED`] left edge.
pub const SURFACE_PARSE_ERROR: &str = "#120a0d";
pub const SURFACE_PARSE_ERROR_BORDER: &str = "#4a2230";

/// Row note card colour triples: (border, left edge, text).
pub const NOTE_CARD_ERROR: (&str, &str, &str) = ("#4a2230", "#ef445f", "#c9a8b0");
pub const NOTE_CARD_WARN: (&str, &str, &str) = ("#3d3520", "#e0b341", "#c9bb95");
pub const NOTE_CARD_OK: (&str, &str, &str) = ("#20343d", "#5fe7e4", "#9dc4cc");

pub const FEE_NUMBER_GRADIENT: &str =
    "linear-gradient(231.49deg,#61ffe1 10%,#5572f5 62%,#a341ff 100%)";
pub const HEADLINE_GRADIENT_TEAL: &str =
    "linear-gradient(237deg,#61ffe1 0%,#5fe7e4 50%,#5ccbe8 70%,#59a4ee 100%)";
pub const HEADLINE_GRADIENT_VIOLET: &str =
    "linear-gradient(237deg,#5433ff 0%,#6e38ff 40%,#853cff 60%,#a341ff 80%,#ac43ff 100%)";

pub const RESPONSIVE_BREAKPOINT_QUERY: &str = "(max-width: 860px)";
pub const FEE_POLL_INTERVAL_MS: u32 = 30_000;
