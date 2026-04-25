//! Theme system — circadian auto-switching, Catppuccin Mocha/Latte palettes,
//! and semantic color functions for canvas widgets.
//!
//! Full palette kept as a design system. Not all colors/functions are
//! referenced yet — they are adopted incrementally across views.
#![allow(dead_code)]

use iced::Color;

// ---------------------------------------------------------------------------
// Circadian theme system — 4 phases, 4 user modes
// ---------------------------------------------------------------------------

/// User-selectable theme mode. Cycles: Auto → Latte → Mocha → TokyoNight → Auto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    /// Circadian: CatppuccinLatte by day, CatppuccinMocha by night.
    Auto,
    /// Catppuccin Latte (warm light theme).
    AlwaysLight,
    /// Catppuccin Mocha (rich dark theme).
    AlwaysDark,
    /// TokyoNight (cool dark theme, blue-tinted).
    TokyoNight,
}

impl ThemeMode {
    pub fn next(self) -> Self {
        match self {
            Self::Auto => Self::AlwaysLight,
            Self::AlwaysLight => Self::AlwaysDark,
            Self::AlwaysDark => Self::TokyoNight,
            Self::TokyoNight => Self::Auto,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::AlwaysLight => "Latte",
            Self::AlwaysDark => "Mocha",
            Self::TokyoNight => "Tokyo",
        }
    }
}

impl Default for ThemeMode {
    fn default() -> Self { Self::Auto }
}

/// 4 circadian phases based on time of day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircadianPhase {
    Dawn,   // 05:00 - 08:59  warm cream tones
    Day,    // 09:00 - 16:59  bright, high contrast
    Dusk,   // 17:00 - 20:59  amber/warm tones
    Night,  // 21:00 - 04:59  deep navy
}

impl CircadianPhase {
    /// Determine phase from local hour (0-23).
    pub fn from_hour(hour: u32) -> Self {
        match hour {
            5..=8   => Self::Dawn,
            9..=16  => Self::Day,
            17..=20 => Self::Dusk,
            _       => Self::Night,
        }
    }

    /// Get the current phase based on system clock.
    pub fn current() -> Self {
        let hour = chrono::Local::now().hour();
        Self::from_hour(hour)
    }
}

/// Resolve the active phase from the user's mode selection.
pub fn active_phase(mode: ThemeMode) -> CircadianPhase {
    match mode {
        ThemeMode::Auto => CircadianPhase::current(),
        ThemeMode::AlwaysLight => CircadianPhase::Day,
        ThemeMode::AlwaysDark | ThemeMode::TokyoNight => CircadianPhase::Night,
    }
}

/// Map a circadian phase to an Iced Theme.
pub fn iced_theme(mode: ThemeMode) -> iced::Theme {
    match mode {
        ThemeMode::Auto => match active_phase(mode) {
            CircadianPhase::Dawn | CircadianPhase::Day => iced::Theme::CatppuccinLatte,
            CircadianPhase::Dusk | CircadianPhase::Night => iced::Theme::CatppuccinMocha,
        },
        ThemeMode::AlwaysLight => iced::Theme::CatppuccinLatte,
        ThemeMode::AlwaysDark  => iced::Theme::CatppuccinMocha,
        ThemeMode::TokyoNight  => iced::Theme::TokyoNight,
    }
}

/// Whether the active theme is dark (checks palette background luminance).
pub fn is_dark(theme: &iced::Theme) -> bool {
    let bg = theme.palette().background;
    (bg.r + bg.g + bg.b) / 3.0 < 0.5
}

use chrono::Timelike;
use std::sync::atomic::{AtomicU32, Ordering};

// ---------------------------------------------------------------------------
// Type scale — 1.25x (major third) based on 15px body
// 11 → 14 → 15 → 18 → 21 → 28
//
// Runtime-adjustable via set_font_scale(). Presets:
//   Compact 0.85 | Default 1.0 | Large 1.15 | XL 1.35
// ---------------------------------------------------------------------------

// Scale stored as integer (100 = 1.0x) for atomic access
static FONT_SCALE_X100: AtomicU32 = AtomicU32::new(100);

/// Set the global font scale factor (1.0 = default).
pub fn set_font_scale(scale: f32) {
    FONT_SCALE_X100.store((scale * 100.0) as u32, Ordering::Relaxed);
}

/// Get the current font scale factor.
pub fn font_scale() -> f32 {
    FONT_SCALE_X100.load(Ordering::Relaxed) as f32 / 100.0
}

fn s(base: f32) -> f32 { base * font_scale() }

pub fn text_xs()   -> f32 { s(11.0) }  // sparkline dates, legend labels
pub fn text_sm()   -> f32 { s(14.0) }  // table headers, gauge labels, captions
pub fn text_base() -> f32 { s(15.0) }  // body text, data values
pub fn text_md()   -> f32 { s(18.0) }  // secondary section headings
pub fn text_lg()   -> f32 { s(21.0) }  // primary section headings
pub fn text_2xl()  -> f32 { s(28.0) }  // page title


// ---------------------------------------------------------------------------
// Catppuccin Mocha palette constants (for canvas / custom drawing)
// ---------------------------------------------------------------------------

pub const MOCHA_BASE:     Color = Color::from_rgb(0.118, 0.118, 0.180); // #1e1e2e
pub const MOCHA_MANTLE:   Color = Color::from_rgb(0.094, 0.094, 0.145); // #181825
pub const MOCHA_SURFACE0: Color = Color::from_rgb(0.192, 0.196, 0.267); // #313244
pub const MOCHA_SURFACE1: Color = Color::from_rgb(0.271, 0.278, 0.353); // #45475a
pub const MOCHA_OVERLAY0: Color = Color::from_rgb(0.424, 0.439, 0.525); // #6c7086
pub const MOCHA_SUBTEXT0: Color = Color::from_rgb(0.651, 0.678, 0.784); // #a6adc8
pub const MOCHA_SUBTEXT1: Color = Color::from_rgb(0.729, 0.761, 0.871); // #bac2de
pub const MOCHA_TEXT:     Color = Color::from_rgb(0.804, 0.839, 0.957); // #cdd6f4

pub const MOCHA_BLUE:     Color = Color::from_rgb(0.537, 0.706, 0.980); // #89b4fa
pub const MOCHA_GREEN:    Color = Color::from_rgb(0.651, 0.890, 0.631); // #a6e3a1
pub const MOCHA_RED:      Color = Color::from_rgb(0.953, 0.545, 0.659); // #f38ba8
pub const MOCHA_YELLOW:   Color = Color::from_rgb(0.976, 0.886, 0.686); // #f9e2af
pub const MOCHA_PEACH:    Color = Color::from_rgb(0.980, 0.702, 0.529); // #fab387
pub const MOCHA_MAUVE:    Color = Color::from_rgb(0.796, 0.651, 0.969); // #cba6f7
pub const MOCHA_SKY:      Color = Color::from_rgb(0.537, 0.863, 0.922); // #89dceb
pub const MOCHA_TEAL:     Color = Color::from_rgb(0.580, 0.886, 0.835); // #94e2d5
pub const MOCHA_LAVENDER: Color = Color::from_rgb(0.706, 0.745, 0.996); // #b4befe

// Catppuccin Latte palette (light mode canvas)
pub const LATTE_BASE:     Color = Color::from_rgb(0.937, 0.945, 0.961); // #eff1f5
pub const LATTE_SURFACE0: Color = Color::from_rgb(0.800, 0.816, 0.855); // #ccd0da
pub const LATTE_OVERLAY0: Color = Color::from_rgb(0.604, 0.620, 0.694); // #9ca0b0
pub const LATTE_SUBTEXT0: Color = Color::from_rgb(0.424, 0.431, 0.522); // #6c6f85
pub const LATTE_SUBTEXT1: Color = Color::from_rgb(0.361, 0.373, 0.467); // #5c5f77
pub const LATTE_TEXT:     Color = Color::from_rgb(0.298, 0.310, 0.412); // #4c4f69

// ---------------------------------------------------------------------------
// Semantic colors — adapt to active theme palette
// ---------------------------------------------------------------------------

pub fn canvas_bg(theme: &iced::Theme) -> Color {
    if is_dark(theme) { MOCHA_BASE } else { LATTE_BASE }
}

pub fn surface(theme: &iced::Theme) -> Color {
    if is_dark(theme) { MOCHA_SURFACE0 } else { LATTE_SURFACE0 }
}

pub fn fg(theme: &iced::Theme) -> Color {
    if is_dark(theme) { MOCHA_TEXT } else { LATTE_TEXT }
}

pub fn fg_dim(theme: &iced::Theme) -> Color {
    if is_dark(theme) { MOCHA_SUBTEXT0 } else { LATTE_SUBTEXT0 }
}

pub fn fg_muted(theme: &iced::Theme) -> Color {
    if is_dark(theme) { MOCHA_OVERLAY0 } else { LATTE_OVERLAY0 }
}

pub fn label_color(theme: &iced::Theme) -> Color {
    if is_dark(theme) { MOCHA_SUBTEXT1 } else { LATTE_SUBTEXT1 }
}

pub fn grid_line(theme: &iced::Theme) -> Color {
    if is_dark(theme) {
        Color { a: 0.15, ..MOCHA_SURFACE1 }
    } else {
        Color { a: 0.30, ..LATTE_SURFACE0 }
    }
}

pub fn sign_color(theme: &iced::Theme) -> Color {
    if is_dark(theme) { MOCHA_OVERLAY0 } else { LATTE_OVERLAY0 }
}

pub fn ring_dim(theme: &iced::Theme) -> Color {
    if is_dark(theme) {
        Color { a: 0.25, ..MOCHA_SURFACE1 }
    } else {
        Color { a: 0.20, ..LATTE_SURFACE0 }
    }
}

/// Accent color (blue) for primary actions and highlights.
pub fn accent(theme: &iced::Theme) -> Color {
    if is_dark(theme) { MOCHA_BLUE } else { Color::from_rgb(0.118, 0.400, 0.961) } // Latte Blue
}

/// Bullish / positive color.
pub fn bullish(_theme: &iced::Theme) -> Color { MOCHA_GREEN }

/// Bearish / negative color.
pub fn bearish(_theme: &iced::Theme) -> Color { MOCHA_RED }

/// Astro / gold accent.
pub fn gold(_theme: &iced::Theme) -> Color { MOCHA_YELLOW }

// ---------------------------------------------------------------------------
// Chart accent colors — Catppuccin-influenced, readable on both light/dark
// ---------------------------------------------------------------------------

pub const ACCENT_BLUE: Color = MOCHA_BLUE;
pub const ACCENT_BLUE_FILL: Color = Color {
    r: 0.537, g: 0.706, b: 0.980, a: 0.15,
};
pub const SMA20_ORANGE: Color = Color { r: 1.0, g: 0.55, b: 0.1, a: 0.85 };
pub const SMA50_YELLOW: Color = Color { r: 1.0, g: 0.85, b: 0.2, a: 0.7 };
pub const BB_BLUE: Color = Color { r: 0.4, g: 0.7, b: 1.0, a: 0.35 };
pub const SPARKLINE_BLUE: Color = Color::from_rgb(0.4, 0.8, 1.0);

// Natal wheel
pub const NATAL_GOLD: Color = Color::from_rgb(0.95, 0.80, 0.20);
pub const NATAL_GOLD_DIM: Color = Color { r: 0.95, g: 0.80, b: 0.20, a: 0.90 };
pub const NATAL_GOLD_LABEL: Color = Color { r: 0.95, g: 0.80, b: 0.20, a: 0.50 };
pub const TRANSIT_BLUE: Color = Color { r: 0.35, g: 0.70, b: 1.0, a: 0.90 };
pub const TRANSIT_BLUE_LABEL: Color = Color { r: 0.35, g: 0.70, b: 1.0, a: 0.50 };
pub const RETROGRADE_RED: Color = Color { r: 1.0, g: 0.5, b: 0.5, a: 0.90 };

// Score zones
pub const ZONE_MISALIGNED: Color = Color::from_rgb(0.9, 0.2, 0.2);
pub const ZONE_UNFAVORABLE: Color = Color::from_rgb(0.85, 0.45, 0.1);
pub const ZONE_NEUTRAL: Color = Color::from_rgb(0.6, 0.6, 0.6);
pub const ZONE_FAVORABLE: Color = Color::from_rgb(0.2, 0.65, 0.9);
pub const ZONE_OPTIMAL: Color = Color::from_rgb(0.0, 0.78, 0.35);

// Gauge zones
pub const GAUGE_EXTREME_FEAR: (f32, f32, f32) = (0.85, 0.12, 0.12);
pub const GAUGE_FEAR: (f32, f32, f32) = (0.95, 0.48, 0.08);
pub const GAUGE_NEUTRAL: (f32, f32, f32) = (0.90, 0.86, 0.08);
pub const GAUGE_GREED: (f32, f32, f32) = (0.52, 0.86, 0.10);
pub const GAUGE_EXTREME_GREED: (f32, f32, f32) = (0.10, 0.76, 0.10);

// Sparkline zone bands
pub const SPARK_ZONE_MIS: Color = Color { r: 0.8, g: 0.1, b: 0.1, a: 0.18 };
pub const SPARK_ZONE_UNF: Color = Color { r: 0.8, g: 0.4, b: 0.0, a: 0.18 };
pub const SPARK_ZONE_NEU: Color = Color { r: 0.8, g: 0.8, b: 0.0, a: 0.18 };
pub const SPARK_ZONE_FAV: Color = Color { r: 0.2, g: 0.7, b: 0.2, a: 0.18 };
pub const SPARK_ZONE_OPT: Color = Color { r: 0.0, g: 0.9, b: 0.4, a: 0.18 };

// Aspect colors (natal wheel)
pub const ASPECT_CONJUNCTION: Color = Color { r: 1.0, g: 0.9, b: 0.3, a: 0.18 };
pub const ASPECT_SEXTILE: Color = Color { r: 0.3, g: 1.0, b: 0.5, a: 0.15 };
pub const ASPECT_SQUARE: Color = Color { r: 1.0, g: 0.3, b: 0.3, a: 0.15 };
pub const ASPECT_TRINE: Color = Color { r: 0.3, g: 0.7, b: 1.0, a: 0.18 };
