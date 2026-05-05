//! Astrology UI components for the dashboard.
//!
//! - `NatalWheel` — canvas widget showing natal (gold) + transit (blue) planet positions
//! - `build_transits_section` — active transit aspects table
//! - `build_moon_line` — moon phase display row

use std::f32::consts::PI;

use iced::widget::canvas::{self};
use iced::widget::{column, row, text, Column};
use iced::{Alignment, Color, Element, Length, Point, Rectangle};
use iced::mouse;

use pursuit_week4_automation::models::{DailyTransit, NatalPosition};
use crate::state::Message;
use crate::view::shared::gutter_scroll;
use crate::theme;

// ---------------------------------------------------------------------------
// Planet glyph abbreviations
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn planet_abbrev(name: &str) -> &'static str {
    match name {
        "Sun"       => "Su",
        "Moon"      => "Mo",
        "Mercury"   => "Me",
        "Venus"     => "Ve",
        "Mars"      => "Ma",
        "Jupiter"   => "Ju",
        "Saturn"    => "Sa",
        "Uranus"    => "Ur",
        "Neptune"   => "Ne",
        "Pluto"     => "Pl",
        "NorthNode" => "NN",
        "SouthNode" => "SN",
        "Chiron"    => "Ch",
        _           => "?",
    }
}

fn planet_glyph(name: &str) -> &'static str {
    match name {
        "Sun"       => "☉",
        "Moon"      => "☽",
        "Mercury"   => "☿",
        "Venus"     => "♀",
        "Mars"      => "♂",
        "Jupiter"   => "♃",
        "Saturn"    => "♄",
        "Uranus"    => "♅",
        "Neptune"   => "♆",
        "Pluto"     => "♇",
        "NorthNode" => "☊",
        "SouthNode" => "☋",
        "Chiron"    => "⚷",
        _           => "?",
    }
}

// ---------------------------------------------------------------------------
// NatalWheel canvas widget (2D reference — superseded by NatalWheel3DProgram)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub struct NatalWheel {
    pub natal:    Vec<NatalPosition>,
    pub transits: Vec<DailyTransit>,
    pub time:     f32,  // shader_time for animated transit ring drift
}

/// Per-sign zodiac colors — element-based (fire/earth/air/water).
#[allow(dead_code)]
const SIGN_COLORS: [Color; 12] = [
    Color { r: 0.95, g: 0.30, b: 0.20, a: 0.30 }, // Aries       — fire (red)
    Color { r: 0.45, g: 0.75, b: 0.30, a: 0.30 }, // Taurus      — earth (green)
    Color { r: 0.90, g: 0.85, b: 0.30, a: 0.30 }, // Gemini      — air (yellow)
    Color { r: 0.30, g: 0.60, b: 0.95, a: 0.30 }, // Cancer      — water (blue)
    Color { r: 0.95, g: 0.55, b: 0.15, a: 0.30 }, // Leo         — fire (orange)
    Color { r: 0.55, g: 0.80, b: 0.40, a: 0.30 }, // Virgo       — earth (sage)
    Color { r: 0.85, g: 0.75, b: 0.40, a: 0.30 }, // Libra       — air (gold)
    Color { r: 0.70, g: 0.25, b: 0.30, a: 0.30 }, // Scorpio     — water (deep red)
    Color { r: 0.80, g: 0.40, b: 0.90, a: 0.30 }, // Sagittarius — fire (purple)
    Color { r: 0.40, g: 0.55, b: 0.45, a: 0.30 }, // Capricorn   — earth (dark green)
    Color { r: 0.35, g: 0.70, b: 0.90, a: 0.30 }, // Aquarius    — air (cyan)
    Color { r: 0.50, g: 0.40, b: 0.80, a: 0.30 }, // Pisces      — water (indigo)
];

impl canvas::Program<Message> for NatalWheel {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let bg        = theme::canvas_bg(theme);
        let dim       = theme::ring_dim(theme);
        let sign_col  = theme::sign_color(theme);

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), bg);

        let cx = bounds.width / 2.0;
        let cy = bounds.height / 2.0;
        let r_outer   = (cx.min(cy) - 8.0).max(10.0);
        let r_sign    = r_outer - 2.0;
        let r_natal   = r_outer * 0.70;
        let r_transit = r_outer * 0.88;
        let r_center  = r_outer * 0.52;

        // --------------- Colored zodiac ring segments (v7.5) ---------------
        const SIGN_ABBREVS: &[&str] = &[
            "Ari","Tau","Gem","Can","Leo","Vir",
            "Lib","Sco","Sag","Cap","Aqu","Pis",
        ];
        let steps = 30; // arc resolution per sign
        for i in 0..12 {
            // Draw filled arc segment for each sign
            let sign_start = i as f64 * 30.0;
            let arc = canvas::Path::new(|b| {
                let a0 = lon_to_angle(sign_start);
                b.move_to(Point::new(cx + r_natal * a0.cos(), cy + r_natal * a0.sin()));
                for s in 0..=steps {
                    let lon = sign_start + (s as f64 * 30.0 / steps as f64);
                    let a = lon_to_angle(lon);
                    b.line_to(Point::new(cx + r_outer * a.cos(), cy + r_outer * a.sin()));
                }
                for s in (0..=steps).rev() {
                    let lon = sign_start + (s as f64 * 30.0 / steps as f64);
                    let a = lon_to_angle(lon);
                    b.line_to(Point::new(cx + r_natal * a.cos(), cy + r_natal * a.sin()));
                }
            });
            frame.fill(&arc, SIGN_COLORS[i]);

            // Sign divider line
            let angle = lon_to_angle(sign_start);
            let line = canvas::Path::new(|b| {
                b.move_to(Point::new(cx + r_center * angle.cos(), cy + r_center * angle.sin()));
                b.line_to(Point::new(cx + r_outer  * angle.cos(), cy + r_outer  * angle.sin()));
            });
            frame.stroke(&line, canvas::Stroke {
                style: canvas::Style::Solid(dim),
                width: 0.8,
                ..canvas::Stroke::default()
            });

            // Sign label at midpoint
            let mid_angle = lon_to_angle(i as f64 * 30.0 + 15.0);
            let label_r = (r_sign + r_natal) / 2.0 + 6.0;
            frame.fill_text(canvas::Text {
                content: SIGN_ABBREVS[i].to_string(),
                position: Point::new(cx + label_r * mid_angle.cos(), cy + label_r * mid_angle.sin()),
                color: sign_col,
                size: iced::Pixels(9.0),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }

        // --------------- Ring strokes ---------------
        let outer_circle = canvas::Path::circle(Point::new(cx, cy), r_outer);
        frame.stroke(&outer_circle, canvas::Stroke {
            style: canvas::Style::Solid(dim),
            width: 1.5,
            ..canvas::Stroke::default()
        });
        let natal_circle = canvas::Path::circle(Point::new(cx, cy), r_natal);
        frame.stroke(&natal_circle, canvas::Stroke {
            style: canvas::Style::Solid(dim),
            width: 1.0,
            ..canvas::Stroke::default()
        });
        let inner_circle = canvas::Path::circle(Point::new(cx, cy), r_center);
        frame.stroke(&inner_circle, canvas::Stroke {
            style: canvas::Style::Solid(dim),
            width: 1.0,
            ..canvas::Stroke::default()
        });

        // --------------- Aspect lines (styled by type, v7.5) ---------------
        for natal_pos in &self.natal {
            for transit in &self.transits {
                let n_lon = natal_pos.longitude;
                let t_lon = transit.longitude;
                let mut diff = (n_lon - t_lon).abs() % 360.0;
                if diff > 180.0 { diff = 360.0 - diff; }

                let (color, width, draw) = if diff < 8.0 || diff > 172.0 {
                    (theme::ASPECT_CONJUNCTION, 1.5, true)    // conjunction — thick gold
                } else if (diff - 60.0).abs() < 6.0 {
                    (theme::ASPECT_SEXTILE, 0.8, true)        // sextile — thin green
                } else if (diff - 90.0).abs() < 8.0 {
                    (theme::ASPECT_SQUARE, 1.2, true)         // square — medium red
                } else if (diff - 120.0).abs() < 8.0 {
                    (theme::ASPECT_TRINE, 1.2, true)          // trine — medium blue
                } else {
                    (Color::TRANSPARENT, 0.0, false)
                };

                if draw {
                    let na = lon_to_angle(n_lon as f32);
                    let ta = lon_to_angle(t_lon as f32);
                    let aspect_line = canvas::Path::new(|b| {
                        b.move_to(Point::new(cx + r_center * na.cos(), cy + r_center * na.sin()));
                        b.line_to(Point::new(cx + r_center * ta.cos(), cy + r_center * ta.sin()));
                    });
                    frame.stroke(&aspect_line, canvas::Stroke {
                        style: canvas::Style::Solid(color),
                        width,
                        ..canvas::Stroke::default()
                    });
                }
            }
        }

        // --------------- Natal planets (gold with glow halo, v7.5) ---------------
        for pos in &self.natal {
            let angle = lon_to_angle(pos.longitude as f32);
            let px = cx + r_natal * angle.cos();
            let py = cy + r_natal * angle.sin();

            // Glow halo
            let halo = canvas::Path::circle(Point::new(px, py), 6.0);
            frame.fill(&halo, Color { a: 0.15, ..theme::NATAL_GOLD });

            let dot = canvas::Path::circle(Point::new(px, py), 3.5);
            frame.fill(&dot, theme::NATAL_GOLD);

            frame.fill_text(canvas::Text {
                content: planet_glyph(&pos.planet).to_string(),
                position: Point::new(px, py - 10.0),
                color: theme::NATAL_GOLD_DIM,
                size: iced::Pixels(10.0),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }

        // --------------- Transit planets (blue/red with glow, v7.5) ---------------
        // Slow drift: 0.5°/sec creates subtle "heavens in motion" (v7.6)
        let drift = (self.time * 0.5_f32).to_radians();
        for transit in &self.transits {
            let angle = lon_to_angle(transit.longitude as f32) + drift;
            let px = cx + r_transit * angle.cos();
            let py = cy + r_transit * angle.sin();

            let transit_color = if transit.retrograde {
                theme::RETROGRADE_RED
            } else {
                theme::TRANSIT_BLUE
            };

            // Glow halo
            let halo = canvas::Path::circle(Point::new(px, py), 5.0);
            frame.fill(&halo, Color { a: 0.12, ..transit_color });

            let dot = canvas::Path::circle(Point::new(px, py), 3.0);
            frame.fill(&dot, transit_color);

            let suffix = if transit.retrograde { "ℛ" } else { "" };
            frame.fill_text(canvas::Text {
                content: format!("{}{}", planet_glyph(&transit.planet), suffix),
                position: Point::new(px, py - 9.0),
                color: transit_color,
                size: iced::Pixels(10.0),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }

        // --------------- Center label ---------------
        frame.fill_text(canvas::Text {
            content: "Natal".to_string(),
            position: Point::new(cx, cy + 8.0),
            color: theme::NATAL_GOLD_LABEL,
            size: iced::Pixels(10.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            ..canvas::Text::default()
        });
        frame.fill_text(canvas::Text {
            content: "Transit".to_string(),
            position: Point::new(cx, cy - 5.0),
            color: theme::TRANSIT_BLUE_LABEL,
            size: iced::Pixels(10.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            ..canvas::Text::default()
        });

        vec![frame.into_geometry()]
    }
}

/// Convert ecliptic longitude to canvas angle.
/// 0° (Aries) = right (3 o'clock), increasing counter-clockwise.
#[allow(dead_code)]
fn lon_to_angle<T: Into<f64>>(lon: T) -> f32 {
    let lon = lon.into() as f32;
    // Ecliptic 0° = Aries = right on wheel, ascending counter-clockwise
    -(lon * PI / 180.0)
}

// ---------------------------------------------------------------------------
// Active transits table
// ---------------------------------------------------------------------------

pub fn build_transits_section<'a>(
    aspects: &'a [serde_json::Value],
    moon_phase: Option<&'a str>,
    moon_deg: Option<f64>,
    mercury_rx: bool,
) -> Element<'a, Message> {
    let header = text("Astrological Transits").size(theme::text_lg());

    // Moon phase line
    let moon_line = {
        let phase = moon_phase.unwrap_or("—");
        let deg   = moon_deg.map(|d| format!(" ({:.0}°)", d)).unwrap_or_default();
        let rx_note = if mercury_rx { "  •  ☿ Mercury Rx — caution" } else { "" };
        text(format!("Moon: {phase}{deg}{rx_note}")).size(theme::text_base())
    };

    if aspects.is_empty() {
        return column![
            header,
            moon_line,
            text("No active aspects — run the scraper to compute today's transits").size(theme::text_base()),
        ].spacing(4).into();
    }

    // Table header
    let col_hdr = row![
        text("Transit").size(theme::text_sm()).width(Length::Fixed(110.0)),
        text("Natal").size(theme::text_sm()).width(Length::Fixed(110.0)),
        text("Aspect").size(theme::text_sm()).width(Length::Fixed(90.0)),
        text("Orb").size(theme::text_sm()).width(Length::Fixed(45.0)),
        text("A/S").size(theme::text_sm()).width(Length::Fixed(30.0)),
        text("Effect").size(theme::text_sm()).width(Length::Fill),
    ].spacing(6);

    let rows: Vec<Element<Message>> = aspects.iter().take(15).filter_map(|obj| {
        let transit_planet = obj["transit_planet"].as_str()?;
        let transit_sign   = obj["transit_sign"].as_str()?;
        let natal_planet   = obj["natal_planet"].as_str()?;
        let natal_sign     = obj["natal_sign"].as_str()?;
        let aspect         = obj["aspect"].as_str()?;
        let symbol         = obj["aspect_symbol"].as_str().unwrap_or("");
        let orb            = obj["orb"].as_f64().unwrap_or(0.0);
        let effect         = obj["effect"].as_str().unwrap_or("—");
        let delta          = obj["score_delta"].as_f64().unwrap_or(0.0);
        let applying       = obj["applying"].as_bool().unwrap_or(true);
        let dignity        = obj["dignity"].as_str().unwrap_or("");

        let effect_color_hint = if delta > 4.0 { "+" } else if delta < -4.0 { "-" } else { " " };

        // Applying/separating indicator
        let apply_indicator = if applying { "A" } else { "S" };

        // Dignity suffix for transit planet
        let dignity_suffix = match dignity {
            "Domicile" | "Exalted" => "+",
            "Detriment" | "Fall"   => "-",
            _                      => "",
        };

        Some(row![
            text(format!("{} {}{} ({})", planet_glyph(transit_planet), transit_planet, dignity_suffix, transit_sign))
                .size(theme::text_sm()).width(Length::Fixed(110.0)),
            text(format!("{} {} ({})", planet_glyph(natal_planet), natal_planet, natal_sign))
                .size(theme::text_sm()).width(Length::Fixed(110.0)),
            text(format!("{} {}", symbol, aspect))
                .size(theme::text_sm()).width(Length::Fixed(90.0)),
            text(format!("{:.1}°", orb))
                .size(theme::text_sm()).width(Length::Fixed(45.0)),
            text(apply_indicator)
                .size(theme::text_sm()).width(Length::Fixed(30.0)),
            text(format!("{}{}", effect_color_hint, effect))
                .size(theme::text_sm()).width(Length::Fill),
        ].spacing(6).into())
    }).collect();

    column![
        header,
        moon_line,
        col_hdr,
        gutter_scroll(Column::with_children(rows).spacing(2), 160.0),
    ].spacing(4).into()
}

// ---------------------------------------------------------------------------
// Legend row (for the wheel)
// ---------------------------------------------------------------------------

pub fn build_wheel_legend<'a>() -> Element<'a, Message> {
    // v11.0: Zodiac sign band with element colors
    let zodiac_row = row![
        text("\u{2648}").size(theme::text_base()).color(SIGN_COLORS[0]),
        text("\u{2649}").size(theme::text_base()).color(SIGN_COLORS[1]),
        text("\u{264A}").size(theme::text_base()).color(SIGN_COLORS[2]),
        text("\u{264B}").size(theme::text_base()).color(SIGN_COLORS[3]),
        text("\u{264C}").size(theme::text_base()).color(SIGN_COLORS[4]),
        text("\u{264D}").size(theme::text_base()).color(SIGN_COLORS[5]),
        text("\u{264E}").size(theme::text_base()).color(SIGN_COLORS[6]),
        text("\u{264F}").size(theme::text_base()).color(SIGN_COLORS[7]),
        text("\u{2650}").size(theme::text_base()).color(SIGN_COLORS[8]),
        text("\u{2651}").size(theme::text_base()).color(SIGN_COLORS[9]),
        text("\u{2652}").size(theme::text_base()).color(SIGN_COLORS[10]),
        text("\u{2653}").size(theme::text_base()).color(SIGN_COLORS[11]),
    ].spacing(3);

    // Dot legend + planet symbols in one compact row
    let legend_row = row![
        text("\u{25CF}").size(theme::text_sm()).color(theme::NATAL_GOLD),
        text("Natal").size(theme::text_xs()),
        text(" \u{25CF}").size(theme::text_sm()).color(theme::TRANSIT_BLUE),
        text("Transit").size(theme::text_xs()),
        text(" \u{25CF}").size(theme::text_sm()).color(theme::RETROGRADE_RED),
        text("Rx").size(theme::text_xs()),
    ].spacing(2).align_y(Alignment::Center);

    let planet_row = row![
        text("\u{2609}Sun \u{263D}Moon \u{263F}Mer \u{2640}Ven \u{2642}Mar \u{2643}Jup \u{2644}Sat \u{2645}Ura \u{2646}Nep \u{2647}Plu")
            .size(theme::text_xs()),
    ];

    column![zodiac_row, legend_row, planet_row]
        .spacing(2)
        .align_x(Alignment::Center)
        .into()
}
