//! Theme system — "The Ledger" v7.0
//!
//! 24-stage circadian palette with smooth hourly interpolation.
//! Light mode: aged parchment tones (dawn gold → midday bright → dusk amber → night sepia).
//! Dark mode: leather-bound tones (dawn walnut → midday oak → dusk mahogany → night deep leather).
//!
//! A global RwLock<LedgerPalette> is updated every 30s on Tick and read by
//! all semantic color functions. Canvas widgets get the palette automatically.
#![allow(dead_code)]

use iced::Color;
use chrono::Timelike;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::RwLock;

// ═══════════════════════════════════════════════════════════════════════════
// Theme mode — user-selectable, persisted in settings
// ═══════════════════════════════════════════════════════════════════════════

/// User-selectable theme mode. Cycles: Auto → Parchment → Leather → Auto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    /// Circadian: Parchment by day (hours 6-18), Leather by night.
    Auto,
    /// Always light — aged parchment, still shifts through 24 stages.
    Parchment,
    /// Always dark — leather-bound, still shifts through 24 stages.
    Leather,
}

impl ThemeMode {
    pub fn next(self) -> Self {
        match self {
            Self::Auto      => Self::Parchment,
            Self::Parchment => Self::Leather,
            Self::Leather   => Self::Auto,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto      => "Auto",
            Self::Parchment => "Parchment",
            Self::Leather   => "Leather",
        }
    }
    /// Whether this mode resolves to dark at the given hour.
    pub fn is_dark(self, hour: u32) -> bool {
        match self {
            Self::Auto      => !(6..=18).contains(&hour),
            Self::Parchment => false,
            Self::Leather   => true,
        }
    }
}

impl Default for ThemeMode {
    fn default() -> Self { Self::Auto }
}

// ═══════════════════════════════════════════════════════════════════════════
// LedgerPalette — 11 semantic color channels
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy)]
pub struct LedgerPalette {
    pub bg:          Color,  // page background
    pub surface:     Color,  // card/panel background
    pub ink:         Color,  // primary text
    pub ink_soft:    Color,  // secondary text
    pub ink_faint:   Color,  // disabled/muted text
    pub rule:        Color,  // subtle dividers
    pub rule_strong: Color,  // bold dividers
    pub accent:      Color,  // primary action color
    pub gold:        Color,  // astro/highlight accent
    pub bullish:     Color,  // positive / green
    pub bearish:     Color,  // negative / red
}

// ═══════════════════════════════════════════════════════════════════════════
// Color interpolation
// ═══════════════════════════════════════════════════════════════════════════

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
}

fn lerp_palette(a: &LedgerPalette, b: &LedgerPalette, t: f32) -> LedgerPalette {
    LedgerPalette {
        bg:          lerp_color(a.bg, b.bg, t),
        surface:     lerp_color(a.surface, b.surface, t),
        ink:         lerp_color(a.ink, b.ink, t),
        ink_soft:    lerp_color(a.ink_soft, b.ink_soft, t),
        ink_faint:   lerp_color(a.ink_faint, b.ink_faint, t),
        rule:        lerp_color(a.rule, b.rule, t),
        rule_strong: lerp_color(a.rule_strong, b.rule_strong, t),
        accent:      lerp_color(a.accent, b.accent, t),
        gold:        lerp_color(a.gold, b.gold, t),
        bullish:     lerp_color(a.bullish, b.bullish, t),
        bearish:     lerp_color(a.bearish, b.bearish, t),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 8 anchor palettes — 4 parchment (light) + 4 leather (dark)
//
// Parchment colors derived from the Berkshire Hathaway redesign CSS variables.
// Leather colors designed to complement: warm walnut/oak/mahogany family.
// ═══════════════════════════════════════════════════════════════════════════

// Helper: convert hex to Color at compile time
const fn hex(r: u8, g: u8, b: u8) -> Color {
    Color { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0 }
}

// ── Parchment (light mode) anchors ──────────────────────────────────────

const PARCHMENT_DAWN: LedgerPalette = LedgerPalette {
    bg:          hex(0xf6, 0xf4, 0xef),  // #f6f4ef — cool cream
    surface:     hex(0xec, 0xe9, 0xdf),  // #ece9df
    ink:         hex(0x1a, 0x1a, 0x1c),  // #1a1a1c
    ink_soft:    hex(0x5a, 0x5a, 0x60),  // #5a5a60
    ink_faint:   hex(0x8a, 0x82, 0x72),  // #8a8272
    rule:        hex(0xc9, 0xc5, 0xb8),  // #c9c5b8
    rule_strong: hex(0x1a, 0x1a, 0x1c),  // #1a1a1c
    accent:      hex(0x1e, 0x3a, 0x5f),  // #1e3a5f — dawn blue
    gold:        hex(0x7a, 0x68, 0x42),  // #7a6842
    bullish:     hex(0x2a, 0x7a, 0x3a),  // muted forest green
    bearish:     hex(0x8b, 0x2a, 0x1a),  // muted brick red
};

const PARCHMENT_DAY: LedgerPalette = LedgerPalette {
    bg:          hex(0xfa, 0xf7, 0xf0),  // #faf7f0 — warm paper
    surface:     hex(0xf2, 0xed, 0xe1),  // #f2ede1
    ink:         hex(0x1a, 0x16, 0x12),  // #1a1612
    ink_soft:    hex(0x5c, 0x55, 0x4a),  // #5c554a
    ink_faint:   hex(0x8a, 0x82, 0x72),  // #8a8272
    rule:        hex(0xd4, 0xcd, 0xb8),  // #d4cdb8 — hairline
    rule_strong: hex(0x1a, 0x16, 0x12),  // #1a1612
    accent:      hex(0x0d, 0x23, 0x40),  // #0d2340 — deep navy
    gold:        hex(0x8b, 0x6b, 0x3a),  // #8b6b3a — gold leaf
    bullish:     hex(0x28, 0x6e, 0x35),  // slightly brighter green for day
    bearish:     hex(0x8b, 0x2a, 0x1a),  // #8b2a1a
};

const PARCHMENT_DUSK: LedgerPalette = LedgerPalette {
    bg:          hex(0xf5, 0xeb, 0xda),  // #f5ebda — amber cream
    surface:     hex(0xeb, 0xe0, 0xc9),  // #ebe0c9
    ink:         hex(0x2a, 0x1f, 0x15),  // #2a1f15
    ink_soft:    hex(0x6b, 0x55, 0x42),  // #6b5542
    ink_faint:   hex(0x8a, 0x78, 0x62),  // warm faint
    rule:        hex(0xc9, 0xb9, 0x98),  // #c9b998
    rule_strong: hex(0x2a, 0x1f, 0x15),  // #2a1f15
    accent:      hex(0x5a, 0x3a, 0x2a),  // #5a3a2a — warm brown
    gold:        hex(0x8b, 0x3a, 0x1a),  // #8b3a1a — ember
    bullish:     hex(0x3a, 0x6e, 0x2a),  // warm green
    bearish:     hex(0x8b, 0x3a, 0x1a),  // #8b3a1a
};

const PARCHMENT_NIGHT: LedgerPalette = LedgerPalette {
    bg:          hex(0xe8, 0xdc, 0xc8),  // #e8dcc8 — deep parchment
    surface:     hex(0xde, 0xd2, 0xba),  // warm surface
    ink:         hex(0x2c, 0x24, 0x16),  // #2c2416 — sepia ink
    ink_soft:    hex(0x5c, 0x50, 0x3a),  // sepia soft
    ink_faint:   hex(0x7a, 0x6e, 0x58),  // sepia faint
    rule:        hex(0xb8, 0xaa, 0x8e),  // warm rule
    rule_strong: hex(0x2c, 0x24, 0x16),  // #2c2416
    accent:      hex(0x3a, 0x2a, 0x1a),  // deep sepia
    gold:        hex(0x8b, 0x6b, 0x3a),  // gold
    bullish:     hex(0x3a, 0x6a, 0x2a),  // muted green
    bearish:     hex(0x8b, 0x2a, 0x1a),  // muted red
};

// ── Leather (dark mode) anchors ─────────────────────────────────────────

const LEATHER_DAWN: LedgerPalette = LedgerPalette {
    bg:          hex(0x1e, 0x17, 0x14),  // #1e1714 — warm walnut
    surface:     hex(0x28, 0x20, 0x1a),  // walnut surface
    ink:         hex(0xd8, 0xcc, 0xb4),  // warm cream text
    ink_soft:    hex(0xa0, 0x94, 0x7e),  // muted cream
    ink_faint:   hex(0x6b, 0x63, 0x52),  // dim
    rule:        hex(0x32, 0x2a, 0x22),  // subtle
    rule_strong: hex(0xd8, 0xcc, 0xb4),  // cream rule
    accent:      hex(0x8a, 0xa0, 0xc0),  // cool steel blue
    gold:        hex(0xc0, 0x9a, 0x5a),  // warm gold
    bullish:     hex(0x5a, 0xaa, 0x5a),  // soft green
    bearish:     hex(0xc0, 0x5a, 0x4a),  // soft red
};

const LEATHER_DAY: LedgerPalette = LedgerPalette {
    bg:          hex(0x1a, 0x14, 0x10),  // #1a1410 — deep oak
    surface:     hex(0x24, 0x1c, 0x16),  // oak surface
    ink:         hex(0xe8, 0xdc, 0xc8),  // #e8dcc8 — bright cream
    ink_soft:    hex(0xa8, 0x9f, 0x88),  // #a89f88
    ink_faint:   hex(0x6b, 0x63, 0x52),  // #6b6352
    rule:        hex(0x2e, 0x2a, 0x22),  // #2e2a22
    rule_strong: hex(0xe8, 0xdc, 0xc8),  // #e8dcc8
    accent:      hex(0x7a, 0x9e, 0xc8),  // muted blue
    gold:        hex(0xc9, 0xa8, 0x6a),  // #c9a86a
    bullish:     hex(0x6a, 0xb8, 0x6a),  // clear green
    bearish:     hex(0xc8, 0x5a, 0x4a),  // clear red
};

const LEATHER_DUSK: LedgerPalette = LedgerPalette {
    bg:          hex(0x1f, 0x16, 0x10),  // #1f1610 — mahogany
    surface:     hex(0x2a, 0x1e, 0x16),  // mahogany surface
    ink:         hex(0xdc, 0xcc, 0xb0),  // warm cream
    ink_soft:    hex(0x9a, 0x8a, 0x72),  // amber soft
    ink_faint:   hex(0x68, 0x5a, 0x48),  // amber faint
    rule:        hex(0x30, 0x28, 0x1e),  // warm rule
    rule_strong: hex(0xdc, 0xcc, 0xb0),  // cream
    accent:      hex(0xb0, 0x80, 0x50),  // warm amber accent
    gold:        hex(0xd4, 0xa8, 0x6a),  // #d4a86a — warm gold
    bullish:     hex(0x5a, 0xa8, 0x5a),  // warm green
    bearish:     hex(0xc8, 0x5a, 0x3a),  // warm red
};

const LEATHER_NIGHT: LedgerPalette = LedgerPalette {
    bg:          hex(0x14, 0x11, 0x0c),  // #14110c — deepest leather (BH redesign night)
    surface:     hex(0x1d, 0x1a, 0x14),  // #1d1a14
    ink:         hex(0xe8, 0xdf, 0xc8),  // #e8dfc8
    ink_soft:    hex(0xa8, 0x9f, 0x88),  // #a89f88
    ink_faint:   hex(0x6b, 0x63, 0x52),  // #6b6352
    rule:        hex(0x2e, 0x2a, 0x22),  // #2e2a22
    rule_strong: hex(0xe8, 0xdf, 0xc8),  // #e8dfc8
    accent:      hex(0xc9, 0xa8, 0x6a),  // #c9a86a — warm gold accent
    gold:        hex(0xd4, 0xa8, 0x6a),  // #d4a86a
    bullish:     hex(0x5a, 0xa8, 0x5a),  // soft green
    bearish:     hex(0xc0, 0x5a, 0x4a),  // soft red
};

// ═══════════════════════════════════════════════════════════════════════════
// Palette computation — hour + mode → interpolated palette
// ═══════════════════════════════════════════════════════════════════════════

/// Compute the palette for a given hour (0-23) and dark/light mode.
///
/// Interpolation curve:
///   5..8:  lerp(dawn → day)
///   8..17: hold day
///  17..20: lerp(day → dusk)
///  20..23: lerp(dusk → night)
///  23..5:  hold night
pub fn compute_palette(hour: u32, dark: bool) -> LedgerPalette {
    let (dawn, day, dusk, night) = if dark {
        (&LEATHER_DAWN, &LEATHER_DAY, &LEATHER_DUSK, &LEATHER_NIGHT)
    } else {
        (&PARCHMENT_DAWN, &PARCHMENT_DAY, &PARCHMENT_DUSK, &PARCHMENT_NIGHT)
    };

    match hour {
        5..=7  => lerp_palette(dawn, day, (hour - 5) as f32 / 3.0),
        8..=16 => *day,
        17..=19 => lerp_palette(day, dusk, (hour - 17) as f32 / 3.0),
        20..=22 => lerp_palette(dusk, night, (hour - 20) as f32 / 3.0),
        _ => *night,  // 23, 0..4
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Global palette cache — RwLock, updated on Tick
// ═══════════════════════════════════════════════════════════════════════════

static PALETTE: RwLock<Option<LedgerPalette>> = RwLock::new(None);

/// Read the current palette. Falls back to Parchment Day if uninitialized.
pub fn palette() -> LedgerPalette {
    PALETTE
        .read()
        .ok()
        .and_then(|guard| *guard)
        .unwrap_or(PARCHMENT_DAY)
}

/// Update the global palette cache. Called from Tick subscription and theme changes.
pub fn update_palette(hour: u32, dark: bool) {
    let p = compute_palette(hour, dark);
    if let Ok(mut guard) = PALETTE.write() {
        *guard = Some(p);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Iced theme construction
// ═══════════════════════════════════════════════════════════════════════════

/// Build an iced::Theme from the current palette.
pub fn iced_theme(mode: ThemeMode, hour: u32) -> iced::Theme {
    let dark = mode.is_dark(hour);
    update_palette(hour, dark);
    let p = compute_palette(hour, dark);
    iced::Theme::custom(
        String::from("Ledger"),
        iced::theme::Palette {
            background: p.bg,
            text:       p.ink,
            primary:    p.accent,
            success:    p.bullish,
            danger:     p.bearish,
        },
    )
}

/// Whether the active theme is dark (checks palette background luminance).
pub fn is_dark(theme: &iced::Theme) -> bool {
    let bg = theme.palette().background;
    (bg.r + bg.g + bg.b) / 3.0 < 0.5
}

/// Get the current system clock hour.
pub fn current_hour() -> u32 {
    chrono::Local::now().hour()
}

// ═══════════════════════════════════════════════════════════════════════════
// Type scale — 1.25x (major third) based on 15px body
// 11 → 14 → 15 → 18 → 21 → 28
//
// Runtime-adjustable via set_font_scale(). Presets:
//   Compact 0.85 | Default 1.0 | Large 1.15 | XL 1.35
// ═══════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════
// Semantic color functions — read from the global palette cache
//
// The `_theme` parameter is kept for API compatibility (canvas widgets pass
// it in draw()). The function bodies read from the global cache instead.
// ═══════════════════════════════════════════════════════════════════════════

pub fn canvas_bg(_theme: &iced::Theme) -> Color {
    palette().bg
}

pub fn surface(_theme: &iced::Theme) -> Color {
    palette().surface
}

pub fn fg(_theme: &iced::Theme) -> Color {
    palette().ink
}

pub fn fg_dim(_theme: &iced::Theme) -> Color {
    palette().ink_soft
}

pub fn fg_muted(_theme: &iced::Theme) -> Color {
    palette().ink_faint
}

pub fn label_color(_theme: &iced::Theme) -> Color {
    palette().ink_soft
}

pub fn grid_line(_theme: &iced::Theme) -> Color {
    Color { a: 0.25, ..palette().rule }
}

pub fn sign_color(_theme: &iced::Theme) -> Color {
    palette().ink_faint
}

pub fn ring_dim(_theme: &iced::Theme) -> Color {
    Color { a: 0.25, ..palette().rule }
}

/// Accent color for primary actions and highlights.
pub fn accent(_theme: &iced::Theme) -> Color {
    palette().accent
}

/// Bullish / positive color.
pub fn bullish(_theme: &iced::Theme) -> Color {
    palette().bullish
}

/// Bearish / negative color.
pub fn bearish(_theme: &iced::Theme) -> Color {
    palette().bearish
}

/// Astro / gold accent.
pub fn gold(_theme: &iced::Theme) -> Color {
    palette().gold
}

// ═══════════════════════════════════════════════════════════════════════════
// Chart accent colors — domain-specific, do NOT shift with circadian
//
// These encode data meaning (bullish/bearish, score zones, astrology) and
// must remain constant so the user can learn their significance.
// ═══════════════════════════════════════════════════════════════════════════

pub const ACCENT_BLUE: Color = Color { r: 0.537, g: 0.706, b: 0.980, a: 1.0 };
pub const ACCENT_BLUE_FILL: Color = Color {
    r: 0.537, g: 0.706, b: 0.980, a: 0.15,
};
pub const SMA20_ORANGE: Color = Color { r: 1.0, g: 0.55, b: 0.1, a: 0.85 };
pub const SMA50_YELLOW: Color = Color { r: 1.0, g: 0.85, b: 0.2, a: 0.7 };
pub const BB_BLUE: Color = Color { r: 0.4, g: 0.7, b: 1.0, a: 0.35 };
pub const SPARKLINE_BLUE: Color = Color { r: 0.4, g: 0.8, b: 1.0, a: 1.0 };

// Natal wheel
pub const NATAL_GOLD: Color = Color { r: 0.95, g: 0.80, b: 0.20, a: 1.0 };
pub const NATAL_GOLD_DIM: Color = Color { r: 0.95, g: 0.80, b: 0.20, a: 0.90 };
pub const NATAL_GOLD_LABEL: Color = Color { r: 0.95, g: 0.80, b: 0.20, a: 0.50 };
pub const TRANSIT_BLUE: Color = Color { r: 0.35, g: 0.70, b: 1.0, a: 0.90 };
pub const TRANSIT_BLUE_LABEL: Color = Color { r: 0.35, g: 0.70, b: 1.0, a: 0.50 };
pub const RETROGRADE_RED: Color = Color { r: 1.0, g: 0.5, b: 0.5, a: 0.90 };

// Score zones
pub const ZONE_MISALIGNED: Color = Color { r: 0.9, g: 0.2, b: 0.2, a: 1.0 };
pub const ZONE_UNFAVORABLE: Color = Color { r: 0.85, g: 0.45, b: 0.1, a: 1.0 };
pub const ZONE_NEUTRAL: Color = Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 };
pub const ZONE_FAVORABLE: Color = Color { r: 0.2, g: 0.65, b: 0.9, a: 1.0 };
pub const ZONE_OPTIMAL: Color = Color { r: 0.0, g: 0.78, b: 0.35, a: 1.0 };

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
