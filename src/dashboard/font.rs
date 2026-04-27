//! Typography system — "The Ledger" v7.0
//!
//! Four-role type hierarchy inspired by Renaissance book typography:
//!   DISPLAY  — Fraunces: ornate display serif for headings & titles
//!   BODY     — Source Serif 4: readable workhorse serif for body text
//!   INTER    — Inter: clean sans for numeric values & UI controls
//!   MONO     — JetBrains Mono: tabular data columns
//!
//! Fonts are embedded at compile time via `include_bytes!` and loaded into
//! Iced's internal fontdb at startup. Variable fonts (Fraunces, Source Serif 4)
//! provide all weight variants from a single file.
//!
//! All constants are part of the type system design palette and are adopted
//! incrementally across views.
#![allow(dead_code)]

use iced::Font;
use iced::font::{Family, Weight, Style, Stretch};

// ---------------------------------------------------------------------------
// Raw font bytes (embedded at compile time)
// ---------------------------------------------------------------------------

pub const FRAUNCES_BYTES: &[u8] =
    include_bytes!("../../assets/fonts/Fraunces-Variable.ttf");

pub const SOURCE_SERIF_BYTES: &[u8] =
    include_bytes!("../../assets/fonts/SourceSerif4-Variable.ttf");

pub const INTER_REGULAR_BYTES: &[u8] =
    include_bytes!("../../assets/fonts/Inter-Regular.ttf");

pub const INTER_SEMIBOLD_BYTES: &[u8] =
    include_bytes!("../../assets/fonts/Inter-SemiBold.ttf");

pub const MONO_REGULAR_BYTES: &[u8] =
    include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");

// ---------------------------------------------------------------------------
// Font constants — use these throughout the dashboard
// ---------------------------------------------------------------------------

/// Fraunces SemiBold — page titles, section headings, tab labels.
/// Ornate display serif with optical size axis for the Renaissance book feel.
pub const DISPLAY: Font = Font {
    family: Family::Name("Fraunces"),
    weight: Weight::Semibold,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// Source Serif 4 Regular — body text, descriptions, labels.
/// Readable workhorse serif for sustained reading.
pub const BODY: Font = Font {
    family: Family::Name("Source Serif 4"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// Source Serif 4 SemiBold — emphasized body text, bold labels.
pub const BODY_BOLD: Font = Font {
    family: Family::Name("Source Serif 4"),
    weight: Weight::Semibold,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// Inter Regular — numeric values, prices, scores, percentages.
pub const INTER: Font = Font {
    family: Family::Name("Inter"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// Inter SemiBold — numeric emphasis, bold data values.
pub const INTER_BOLD: Font = Font {
    family: Family::Name("Inter"),
    weight: Weight::Semibold,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// JetBrains Mono — tabular data columns, code-like content.
pub const MONO: Font = Font {
    family: Family::Name("JetBrains Mono"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};
