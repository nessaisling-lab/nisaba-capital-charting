use iced::Color;

/// Whether the active theme is dark.
pub fn is_dark(theme: &iced::Theme) -> bool {
    *theme != iced::Theme::Light
}

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
