//! Phosphor Icons integration for Iced 0.13.
//!
//! Phosphor is a flexible icon family with 6 weights (thin, light, regular,
//! bold, fill, duotone). We embed the Regular and Bold TTFs at compile time
//! and expose icon codepoints as char constants.
//!
//! Many constants are defined but not yet referenced — they form a curated
//! design-system palette for incremental adoption across views.
#![allow(dead_code)]

use iced::widget::text;
use iced::{Element, Font};
use iced::font::{Family, Weight, Stretch, Style};

/// Phosphor icon font bytes — Regular weight.
pub const PHOSPHOR_BYTES: &[u8] = include_bytes!("../../assets/fonts/Phosphor.ttf");

/// Phosphor icon font bytes — Bold weight.
pub const PHOSPHOR_BOLD_BYTES: &[u8] = include_bytes!("../../assets/fonts/Phosphor-Bold.ttf");

/// Phosphor Regular font handle for Iced 0.13.
pub const PHOSPHOR: Font = Font {
    family: Family::Name("Phosphor"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// Phosphor Bold font handle for Iced 0.13.
pub const PHOSPHOR_BOLD: Font = Font {
    family: Family::Name("Phosphor Bold"),
    weight: Weight::Bold,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

// ---------------------------------------------------------------------------
// Icon codepoints — Phosphor Regular
// ---------------------------------------------------------------------------

// Tab bar
pub const STARS: char        = '\u{e6a4}';  // ph-star-four — Astrology tab
pub const SPEEDOMETER: char  = '\u{ee74}';  // ph-speedometer — Overview tab
pub const GLOBE: char        = '\u{e288}';  // ph-globe — Universe tab
pub const BAR_CHART: char    = '\u{e150}';  // ph-chart-bar — Fundamentals tab
pub const NEWSPAPER: char    = '\u{e344}';  // ph-newspaper — Research tab
pub const BRIEFCASE: char    = '\u{e0ee}';  // ph-briefcase — Portfolio tab
pub const GEAR: char         = '\u{e270}';  // ph-gear — Settings tab

// Actions
pub const SEARCH: char       = '\u{e30c}';  // ph-magnifying-glass
pub const ARROW_REPEAT: char = '\u{e094}';  // ph-arrows-clockwise (Refresh)
pub const DOWNLOAD: char     = '\u{e20a}';  // ph-download
pub const FILTER: char       = '\u{e266}';  // ph-funnel
pub const PLUS_LG: char      = '\u{e3d4}';  // ph-plus
pub const TRASH: char        = '\u{e4a6}';  // ph-trash
pub const X_LG: char         = '\u{e4f6}';  // ph-x
pub const CHECK: char        = '\u{e182}';  // ph-check

// Navigation
pub const CHEVRON_LEFT: char  = '\u{e138}';  // ph-caret-left
pub const CHEVRON_RIGHT: char = '\u{e13a}';  // ph-caret-right
pub const CHEVRON_DOWN: char  = '\u{e136}';  // ph-caret-down
pub const CHEVRON_UP: char    = '\u{e13c}';  // ph-caret-up

// Indicators
pub const ARROW_UP: char       = '\u{e08e}';  // ph-arrow-up
pub const ARROW_DOWN: char     = '\u{e03e}';  // ph-arrow-down
pub const CARET_UP: char       = '\u{e13c}';  // ph-caret-up
pub const CARET_DOWN: char     = '\u{e136}';  // ph-caret-down
pub const SORT_DOWN: char      = '\u{e446}';  // ph-sort-descending
pub const SORT_UP: char        = '\u{e444}';  // ph-sort-ascending

// Status
pub const BELL: char           = '\u{e0ce}';  // ph-bell
pub const CLOCK: char          = '\u{e19a}';  // ph-clock
pub const INFO_CIRCLE: char    = '\u{e2ce}';  // ph-info
pub const EXCLAMATION_TRI: char = '\u{e4e0}';  // ph-warning
pub const EYE: char            = '\u{e220}';  // ph-eye
pub const EYE_SLASH: char      = '\u{e222}';  // ph-eye-slash
pub const STAR: char           = '\u{e46a}';  // ph-star
pub const MOON_STARS: char     = '\u{e58e}';  // ph-moon-stars
pub const CALENDAR: char       = '\u{e108}';  // ph-calendar
pub const LIGHTNING: char      = '\u{e2de}';  // ph-lightning
pub const GRAPH_UP: char       = '\u{e4ae}';  // ph-trend-up
pub const GRAPH_DOWN: char     = '\u{e4ac}';  // ph-trend-down
pub const ACTIVITY: char       = '\u{e000}';  // ph-activity
pub const WALLET: char         = '\u{e68a}';  // ph-wallet
pub const KEY: char            = '\u{e2d6}';  // ph-key
pub const BOOK_OPEN: char      = '\u{e0dc}';  // ph-book-open — Encyclopedia tab

// ---------------------------------------------------------------------------
// Helper — returns an iced 0.13 Text element with the Phosphor icon font
// ---------------------------------------------------------------------------

/// Create an icon `Text` element at the given size (Regular weight).
pub fn icon<'a, M: 'a>(codepoint: char, size: f32) -> Element<'a, M> {
    text(codepoint.to_string())
        .font(PHOSPHOR)
        .size(size)
        .into()
}

/// Create an icon `Text` element at the given size (Bold weight).
pub fn icon_bold<'a, M: 'a>(codepoint: char, size: f32) -> Element<'a, M> {
    text(codepoint.to_string())
        .font(PHOSPHOR_BOLD)
        .size(size)
        .into()
}
