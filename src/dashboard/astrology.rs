//! Astrology UI components for the dashboard.
//!
//! - `NatalWheel` — canvas widget showing natal (gold) + transit (blue) planet positions
//! - `build_transits_section` — active transit aspects table
//! - `build_moon_line` — moon phase display row

use std::f32::consts::PI;

use iced::widget::canvas::{self};
use iced::widget::{column, row, scrollable, text, Column};
use iced::{Alignment, Color, Element, Length, Point, Rectangle};
use iced::mouse;

use pursuit_week4_automation::models::{DailyTransit, NatalPosition};
use crate::state::Message;

// ---------------------------------------------------------------------------
// Planet glyph abbreviations
// ---------------------------------------------------------------------------

fn planet_abbrev(name: &str) -> &'static str {
    match name {
        "Sun"     => "Su",
        "Moon"    => "Mo",
        "Mercury" => "Me",
        "Venus"   => "Ve",
        "Mars"    => "Ma",
        "Jupiter" => "Ju",
        "Saturn"  => "Sa",
        "Uranus"  => "Ur",
        "Neptune" => "Ne",
        "Pluto"   => "Pl",
        _         => "?",
    }
}

fn planet_glyph(name: &str) -> &'static str {
    match name {
        "Sun"     => "☉",
        "Moon"    => "☽",
        "Mercury" => "☿",
        "Venus"   => "♀",
        "Mars"    => "♂",
        "Jupiter" => "♃",
        "Saturn"  => "♄",
        "Uranus"  => "♅",
        "Neptune" => "♆",
        "Pluto"   => "♇",
        _         => "?",
    }
}

// ---------------------------------------------------------------------------
// NatalWheel canvas widget
// ---------------------------------------------------------------------------

pub struct NatalWheel {
    pub natal:    Vec<NatalPosition>,
    pub transits: Vec<DailyTransit>,
}

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

        let is_dark = *theme != iced::Theme::Light;
        let bg        = if is_dark { Color::from_rgb(0.06, 0.06, 0.10) } else { Color::from_rgb(0.93, 0.93, 0.96) };
        let _fg       = if is_dark { Color::WHITE }                       else { Color::from_rgb(0.08, 0.08, 0.08) };
        let dim       = if is_dark { Color::from_rgba(1.0,1.0,1.0,0.20)} else { Color::from_rgba(0.0,0.0,0.0,0.15) };
        let sign_col  = if is_dark { Color::from_rgba(1.0,1.0,1.0,0.30)} else { Color::from_rgba(0.0,0.0,0.0,0.35) };

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), bg);

        let cx = bounds.width / 2.0;
        let cy = bounds.height / 2.0;
        let r_outer  = (cx.min(cy) - 8.0).max(10.0);
        let r_sign   = r_outer - 2.0;
        let r_natal  = r_outer * 0.72;
        let r_transit = r_outer * 0.90;
        let r_center = r_outer * 0.55;

        // --------------- Zodiac ring ---------------
        // Outer circle
        let outer_circle = canvas::Path::circle(Point::new(cx, cy), r_outer);
        frame.stroke(&outer_circle, canvas::Stroke {
            style: canvas::Style::Solid(dim),
            width: 1.5,
            ..canvas::Stroke::default()
        });

        // Inner circle (boundary between natal + transit rings)
        let inner_circle = canvas::Path::circle(Point::new(cx, cy), r_center);
        frame.stroke(&inner_circle, canvas::Stroke {
            style: canvas::Style::Solid(dim),
            width: 1.0,
            ..canvas::Stroke::default()
        });

        // Natal ring boundary
        let natal_circle = canvas::Path::circle(Point::new(cx, cy), r_natal);
        frame.stroke(&natal_circle, canvas::Stroke {
            style: canvas::Style::Solid(dim),
            width: 0.5,
            ..canvas::Stroke::default()
        });

        // 12 sign sector dividers + sign abbreviations
        const SIGN_ABBREVS: &[&str] = &[
            "Ari","Tau","Gem","Can","Leo","Vir",
            "Lib","Sco","Sag","Cap","Aqu","Pis",
        ];
        for i in 0..12 {
            let angle = lon_to_angle(i as f64 * 30.0);
            let line = canvas::Path::new(|b| {
                b.move_to(Point::new(cx + r_center * angle.cos(), cy + r_center * angle.sin()));
                b.line_to(Point::new(cx + r_outer  * angle.cos(), cy + r_outer  * angle.sin()));
            });
            frame.stroke(&line, canvas::Stroke {
                style: canvas::Style::Solid(dim),
                width: 0.8,
                ..canvas::Stroke::default()
            });

            // Sign label at midpoint of sector
            let mid_angle = lon_to_angle(i as f64 * 30.0 + 15.0);
            let label_r = (r_sign + r_outer) / 2.0 - 4.0;
            frame.fill_text(canvas::Text {
                content: SIGN_ABBREVS[i].to_string(),
                position: Point::new(cx + label_r * mid_angle.cos(), cy + label_r * mid_angle.sin()),
                color: sign_col,
                size: iced::Pixels(7.5),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }

        // --------------- Aspect lines (natal to transit) ---------------
        for natal_pos in &self.natal {
            for transit in &self.transits {
                let n_lon = natal_pos.longitude;
                let t_lon = transit.longitude;
                let mut diff = (n_lon - t_lon).abs() % 360.0;
                if diff > 180.0 { diff = 360.0 - diff; }

                // Only draw aspects within orb
                let (color, draw) = if diff < 8.0 || diff > 172.0 {
                    (Color::from_rgba(1.0, 0.9, 0.3, 0.18), true)   // conjunction/opposition
                } else if (diff - 60.0).abs() < 6.0 {
                    (Color::from_rgba(0.3, 1.0, 0.5, 0.15), true)   // sextile
                } else if (diff - 90.0).abs() < 8.0 {
                    (Color::from_rgba(1.0, 0.3, 0.3, 0.15), true)   // square
                } else if (diff - 120.0).abs() < 8.0 {
                    (Color::from_rgba(0.3, 0.7, 1.0, 0.18), true)   // trine
                } else {
                    (Color::TRANSPARENT, false)
                };

                if draw {
                    let na = lon_to_angle(n_lon as f32);
                    let ta = lon_to_angle(t_lon as f32);
                    let aspect_line = canvas::Path::new(|b| {
                        b.move_to(Point::new(cx + r_natal  * na.cos(), cy + r_natal  * na.sin()));
                        b.line_to(Point::new(cx + r_transit * ta.cos(), cy + r_transit * ta.sin()));
                    });
                    frame.stroke(&aspect_line, canvas::Stroke {
                        style: canvas::Style::Solid(color),
                        width: 0.8,
                        ..canvas::Stroke::default()
                    });
                }
            }
        }

        // --------------- Natal planets (gold, inner ring) ---------------
        for pos in &self.natal {
            let angle = lon_to_angle(pos.longitude as f32);
            let px = cx + r_natal * angle.cos();
            let py = cy + r_natal * angle.sin();

            let dot = canvas::Path::circle(Point::new(px, py), 2.5);
            frame.fill(&dot, Color::from_rgb(0.95, 0.80, 0.20));

            frame.fill_text(canvas::Text {
                content: planet_abbrev(&pos.planet).to_string(),
                position: Point::new(px, py - 7.0),
                color: Color::from_rgba(0.95, 0.80, 0.20, 0.90),
                size: iced::Pixels(7.0),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }

        // --------------- Transit planets (blue, outer ring) ---------------
        for transit in &self.transits {
            let angle = lon_to_angle(transit.longitude as f32);
            let px = cx + r_transit * angle.cos();
            let py = cy + r_transit * angle.sin();

            let transit_color = if transit.retrograde {
                Color::from_rgba(1.0, 0.5, 0.5, 0.90)  // red-ish when retrograde
            } else {
                Color::from_rgba(0.35, 0.70, 1.0, 0.90) // blue normally
            };

            let dot = canvas::Path::circle(Point::new(px, py), 2.5);
            frame.fill(&dot, transit_color);

            let suffix = if transit.retrograde { "ℛ" } else { "" };
            frame.fill_text(canvas::Text {
                content: format!("{}{}", planet_glyph(&transit.planet), suffix),
                position: Point::new(px, py - 7.0),
                color: transit_color,
                size: iced::Pixels(8.0),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }

        // --------------- Center label ---------------
        frame.fill_text(canvas::Text {
            content: "Natal".to_string(),
            position: Point::new(cx, cy + 6.0),
            color: Color::from_rgba(0.95, 0.80, 0.20, 0.50),
            size: iced::Pixels(8.0),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            ..canvas::Text::default()
        });
        frame.fill_text(canvas::Text {
            content: "Transit".to_string(),
            position: Point::new(cx, cy - 4.0),
            color: Color::from_rgba(0.35, 0.70, 1.0, 0.50),
            size: iced::Pixels(8.0),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            ..canvas::Text::default()
        });

        vec![frame.into_geometry()]
    }
}

/// Convert ecliptic longitude to canvas angle.
/// 0° (Aries) = right (3 o'clock), increasing counter-clockwise.
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
    let header = text("Astrological Transits").size(14);

    // Moon phase line
    let moon_line = {
        let phase = moon_phase.unwrap_or("—");
        let deg   = moon_deg.map(|d| format!(" ({:.0}°)", d)).unwrap_or_default();
        let rx_note = if mercury_rx { "  •  ☿ Mercury Rx — caution" } else { "" };
        text(format!("Moon: {phase}{deg}{rx_note}")).size(11)
    };

    if aspects.is_empty() {
        return column![
            header,
            moon_line,
            text("No active aspects — run the scraper to compute today's transits").size(11),
        ].spacing(4).into();
    }

    // Table header
    let col_hdr = row![
        text("Transit").size(10).width(Length::Fixed(110.0)),
        text("Natal").size(10).width(Length::Fixed(110.0)),
        text("Aspect").size(10).width(Length::Fixed(80.0)),
        text("Orb").size(10).width(Length::Fixed(45.0)),
        text("Effect").size(10).width(Length::Fill),
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

        let effect_color_hint = if delta > 4.0 { "+" } else if delta < -4.0 { "-" } else { " " };

        Some(row![
            text(format!("{} {} ({})", planet_glyph(transit_planet), transit_planet, transit_sign))
                .size(10).width(Length::Fixed(110.0)),
            text(format!("{} {} ({})", planet_glyph(natal_planet), natal_planet, natal_sign))
                .size(10).width(Length::Fixed(110.0)),
            text(format!("{} {}", symbol, aspect))
                .size(10).width(Length::Fixed(80.0)),
            text(format!("{:.1}°", orb))
                .size(10).width(Length::Fixed(45.0)),
            text(format!("{}{}", effect_color_hint, effect))
                .size(10).width(Length::Fill),
        ].spacing(6).into())
    }).collect();

    column![
        header,
        moon_line,
        col_hdr,
        scrollable(
            Column::with_children(rows).spacing(2)
        ).height(Length::Fixed(160.0)),
    ].spacing(4).into()
}

// ---------------------------------------------------------------------------
// Legend row (for the wheel)
// ---------------------------------------------------------------------------

pub fn build_wheel_legend<'a>() -> Element<'a, Message> {
    row![
        text("●").size(11).color(Color::from_rgb(0.95, 0.80, 0.20)),
        text("Natal (IPO)").size(10),
        iced::widget::Space::with_width(Length::Fixed(12.0)),
        text("●").size(11).color(Color::from_rgb(0.35, 0.70, 1.0)),
        text("Today's transits").size(10),
        iced::widget::Space::with_width(Length::Fixed(12.0)),
        text("●").size(11).color(Color::from_rgb(1.0, 0.5, 0.5)),
        text("Retrograde").size(10),
    ]
    .spacing(4)
    .align_y(Alignment::Center)
    .into()
}
