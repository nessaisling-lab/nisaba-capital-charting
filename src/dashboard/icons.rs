//! Bootstrap Icons integration via `iced_fonts`.
//!
//! We re-use the font bytes from `iced_fonts` but define our own `icon()`
//! helper that returns Iced 0.13-compatible `Text` widgets. This avoids
//! type incompatibility between iced_fonts 0.3 (which targets iced_core 0.14)
//! and our iced 0.13 application.
//!
//! Many constants are defined but not yet referenced — they form a curated
//! design-system palette for incremental adoption across views.
#![allow(dead_code)]

use iced::widget::text;
use iced::{Element, Font};
use iced::font::{Family, Weight, Stretch, Style};

/// Bootstrap icon font bytes (from `iced_fonts` crate).
pub const BOOTSTRAP_BYTES: &[u8] = iced_fonts::BOOTSTRAP_FONT_BYTES;

/// Bootstrap icon font handle for Iced 0.13.
pub const BOOTSTRAP: Font = Font {
    family: Family::Name("bootstrap-icons"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

// ---------------------------------------------------------------------------
// Icon codepoints — only the icons we actually use
// ---------------------------------------------------------------------------

// Tab bar
pub const STARS: char        = '\u{f589}';  // Astrology tab
pub const SPEEDOMETER: char  = '\u{f57f}';  // Overview tab
pub const GLOBE: char        = '\u{f3ee}';  // Universe tab
pub const BAR_CHART: char    = '\u{f17e}';  // Fundamentals tab
pub const NEWSPAPER: char    = '\u{f4a3}';  // Research tab
pub const BRIEFCASE: char    = '\u{f1cc}';  // Portfolio tab
pub const GEAR: char         = '\u{f3e5}';  // Settings tab

// Actions
pub const SEARCH: char       = '\u{f52a}';
pub const ARROW_REPEAT: char = '\u{f130}';  // Refresh
pub const DOWNLOAD: char     = '\u{f30a}';
pub const FILTER: char       = '\u{f3ca}';
pub const PLUS_LG: char      = '\u{f64d}';
pub const TRASH: char        = '\u{f5de}';
pub const X_LG: char         = '\u{f659}';
pub const CHECK: char        = '\u{f26e}';

// Navigation
pub const CHEVRON_LEFT: char  = '\u{f284}';
pub const CHEVRON_RIGHT: char = '\u{f285}';
pub const CHEVRON_DOWN: char  = '\u{f282}';
pub const CHEVRON_UP: char    = '\u{f286}';

// Indicators
pub const ARROW_UP: char       = '\u{f148}';
pub const ARROW_DOWN: char     = '\u{f128}';
pub const CARET_UP: char       = '\u{f238}';
pub const CARET_DOWN: char     = '\u{f22c}';
pub const SORT_DOWN: char      = '\u{f575}';
pub const SORT_UP: char        = '\u{f57b}';

// Status
pub const BELL: char           = '\u{f18a}';
pub const CLOCK: char          = '\u{f293}';
pub const INFO_CIRCLE: char    = '\u{f431}';
pub const EXCLAMATION_TRI: char = '\u{f33b}';
pub const EYE: char            = '\u{f341}';
pub const STAR: char           = '\u{f588}';
pub const MOON_STARS: char     = '\u{f496}';
pub const CALENDAR: char       = '\u{f1f6}';
pub const LIGHTNING: char      = '\u{f46f}';
pub const GRAPH_UP: char       = '\u{f3f2}';
pub const GRAPH_DOWN: char     = '\u{f3f1}';
pub const ACTIVITY: char       = '\u{f66b}';
pub const WALLET: char         = '\u{f614}';
pub const KEY: char            = '\u{f449}';

// ---------------------------------------------------------------------------
// Helper — returns an iced 0.13 Text element with the Bootstrap icon font
// ---------------------------------------------------------------------------

/// Create an icon `Text` element at the given size.
pub fn icon<'a, M: 'a>(codepoint: char, size: f32) -> Element<'a, M> {
    text(codepoint.to_string())
        .font(BOOTSTRAP)
        .size(size)
        .into()
}
