use iced::Color;

// ---------------------------------------------------------------------------
// Circadian theme system — 4 phases, 3 user modes
// ---------------------------------------------------------------------------

/// User-selectable theme mode. Cycles: Auto → Light → Dark → Auto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    /// Automatically select phase based on local time of day.
    Auto,
    /// Always use the Day (light) phase.
    AlwaysLight,
    /// Always use the Night (dark) phase.
    AlwaysDark,
}

impl ThemeMode {
    pub fn next(self) -> Self {
        match self {
            Self::Auto => Self::AlwaysLight,
            Self::AlwaysLight => Self::AlwaysDark,
            Self::AlwaysDark => Self::Auto,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::AlwaysLight => "Light",
            Self::AlwaysDark => "Dark",
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
        ThemeMode::AlwaysDark => CircadianPhase::Night,
    }
}

/// Map a circadian phase to an Iced Theme.
pub fn iced_theme(mode: ThemeMode) -> iced::Theme {
    match active_phase(mode) {
        CircadianPhase::Dawn | CircadianPhase::Day => iced::Theme::Light,
        CircadianPhase::Dusk | CircadianPhase::Night => iced::Theme::Dark,
    }
}

/// Whether the active theme is dark.
pub fn is_dark(theme: &iced::Theme) -> bool {
    *theme != iced::Theme::Light
}

use chrono::Timelike;

// ---------------------------------------------------------------------------
// Type scale — 1.2x (minor third) based on 12px body
// 8 → 10 → 12 → 14 → 17 → 20 → 24
// ---------------------------------------------------------------------------

pub const TEXT_XS: f32 = 8.0;      // sparkline dates, legend labels
pub const TEXT_SM: f32 = 10.0;     // table headers, gauge labels, captions
pub const TEXT_BASE: f32 = 12.0;   // body text, data values
pub const TEXT_MD: f32 = 14.0;     // secondary section headings
pub const TEXT_LG: f32 = 17.0;     // primary section headings
pub const TEXT_2XL: f32 = 24.0;    // page title

// ---------------------------------------------------------------------------
// Semantic background colors
// ---------------------------------------------------------------------------

pub fn canvas_bg(theme: &iced::Theme) -> Color {
    if is_dark(theme) {
        Color::from_rgb(0.06, 0.06, 0.10)
    } else {
        Color::from_rgb(0.93, 0.93, 0.96)
    }
}

// ---------------------------------------------------------------------------
// Foreground / text colors
// ---------------------------------------------------------------------------

pub fn fg(theme: &iced::Theme) -> Color {
    if is_dark(theme) {
        Color::WHITE
    } else {
        Color::from_rgb(0.08, 0.08, 0.08)
    }
}

pub fn fg_dim(theme: &iced::Theme) -> Color {
    if is_dark(theme) {
        Color::from_rgba(1.0, 1.0, 1.0, 0.40)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.45)
    }
}

pub fn fg_muted(theme: &iced::Theme) -> Color {
    if is_dark(theme) {
        Color::from_rgba(1.0, 1.0, 1.0, 0.50)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.50)
    }
}

pub fn label_color(theme: &iced::Theme) -> Color {
    if is_dark(theme) {
        Color::from_rgba(1.0, 1.0, 1.0, 0.65)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.60)
    }
}

pub fn grid_line(theme: &iced::Theme) -> Color {
    if is_dark(theme) {
        Color::from_rgba(1.0, 1.0, 1.0, 0.08)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.08)
    }
}

pub fn sign_color(theme: &iced::Theme) -> Color {
    if is_dark(theme) {
        Color::from_rgba(1.0, 1.0, 1.0, 0.30)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.35)
    }
}

pub fn ring_dim(theme: &iced::Theme) -> Color {
    if is_dark(theme) {
        Color::from_rgba(1.0, 1.0, 1.0, 0.20)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.15)
    }
}

// ---------------------------------------------------------------------------
// Chart accent colors (theme-independent — these read well on both bgs)
// ---------------------------------------------------------------------------

pub const ACCENT_BLUE: Color = Color::from_rgb(0.2, 0.65, 1.0);
pub const ACCENT_BLUE_FILL: Color = Color {
    r: 0.2, g: 0.65, b: 1.0, a: 0.15,
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
