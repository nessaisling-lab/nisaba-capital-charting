//! Custom typography system — Inter (UI text) + JetBrains Mono (numbers).
//!
//! Fonts are embedded at compile time via `include_bytes!` and loaded into
//! Iced's internal fontdb at startup. The `INTER` font is used for all UI
//! labels, headings, and body text. `MONO` is used for prices, scores,
//! percentages, and any numeric data columns.

use iced::Font;
use iced::font::{Family, Weight, Style, Stretch};

// ---------------------------------------------------------------------------
// Raw font bytes (embedded at compile time)
// ---------------------------------------------------------------------------

pub const INTER_REGULAR_BYTES: &[u8] =
    include_bytes!("../../assets/fonts/Inter-Regular.ttf");

pub const INTER_SEMIBOLD_BYTES: &[u8] =
    include_bytes!("../../assets/fonts/Inter-SemiBold.ttf");

pub const MONO_REGULAR_BYTES: &[u8] =
    include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");

// ---------------------------------------------------------------------------
// Font constants — use these throughout the dashboard
// ---------------------------------------------------------------------------

/// Inter Regular — body text, labels, table headers.
pub const INTER: Font = Font {
    family: Family::Name("Inter"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// Inter SemiBold — section headings, emphasis.
pub const INTER_BOLD: Font = Font {
    family: Family::Name("Inter"),
    weight: Weight::Semibold,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// JetBrains Mono — prices, scores, percentages, numeric columns.
#[allow(dead_code)]
pub const MONO: Font = Font {
    family: Family::Name("JetBrains Mono"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};
