//! Canvas-rendered decorative ornaments for the Grimoire UI (v7.3+).
//!
//! Widgets that transform the book layout from restyled containers
//! into a video game grimoire:
//! - BookSpine: vertical binding strip with cross-stitch marks
//! - PageHeaderOrnament: Renaissance-style flourish above page content
//! - PageBorderCorner: decorative corner brackets for the page frame
//! - TabSparkle: gold particle burst on tab hover (v7.6)

use iced::widget::canvas::{self};
use iced::{Color, Point, Rectangle};
use iced::mouse;

use crate::state::Message;
use crate::theme;

// ═══════════════════════════════════════════════════════════════════════════
// BookSpine — vertical binding strip (24px wide, full height)
// ═══════════════════════════════════════════════════════════════════════════

pub struct BookSpine;

impl canvas::Program<Message> for BookSpine {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;
        let cx = w / 2.0;

        // Background: deep leather spine
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), theme::GRIMOIRE_SPINE);

        // Center vertical line
        let center_line = canvas::Path::new(|b| {
            b.move_to(Point::new(cx, 0.0));
            b.line_to(Point::new(cx, h));
        });
        let p = theme::palette();
        frame.stroke(&center_line, canvas::Stroke {
            style: canvas::Style::Solid(Color { a: 0.45, ..p.rule }),
            width: 1.0,
            ..canvas::Stroke::default()
        });

        // Cross-stitch marks — alternating short horizontal lines
        let stitch_count = ((h / 40.0) as usize).max(4);
        let stitch_len = 6.0;
        for i in 0..stitch_count {
            let y = (i as f32 + 0.5) / stitch_count as f32 * h;
            let side = if i % 2 == 0 { -1.0 } else { 1.0 };

            // Horizontal stitch
            let stitch = canvas::Path::new(|b| {
                b.move_to(Point::new(cx - stitch_len * 0.5 * side, y - 2.0));
                b.line_to(Point::new(cx + stitch_len * 0.5 * side, y + 2.0));
            });
            frame.stroke(&stitch, canvas::Stroke {
                style: canvas::Style::Solid(theme::GRIMOIRE_STITCH),
                width: 1.5,
                ..canvas::Stroke::default()
            });

            // Cross stitch (opposite diagonal)
            let cross = canvas::Path::new(|b| {
                b.move_to(Point::new(cx - stitch_len * 0.5 * side, y + 2.0));
                b.line_to(Point::new(cx + stitch_len * 0.5 * side, y - 2.0));
            });
            frame.stroke(&cross, canvas::Stroke {
                style: canvas::Style::Solid(Color { a: 0.4, ..theme::GRIMOIRE_STITCH }),
                width: 1.0,
                ..canvas::Stroke::default()
            });
        }

        // Diamond endcaps at top and bottom
        let gold = Color { a: 0.55, ..p.gold };
        for &cap_y in &[12.0, h - 12.0] {
            let diamond = canvas::Path::new(|b| {
                b.move_to(Point::new(cx, cap_y - 5.0));
                b.line_to(Point::new(cx + 4.0, cap_y));
                b.line_to(Point::new(cx, cap_y + 5.0));
                b.line_to(Point::new(cx - 4.0, cap_y));
                b.line_to(Point::new(cx, cap_y - 5.0));
            });
            frame.fill(&diamond, gold);
        }

        vec![frame.into_geometry()]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PageHeaderOrnament — Renaissance flourish (full width, 32px tall)
// ═══════════════════════════════════════════════════════════════════════════

pub struct PageHeaderOrnament;

impl canvas::Program<Message> for PageHeaderOrnament {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let cy = bounds.height / 2.0;
        let p = theme::palette();
        let gold = Color { a: 0.7, ..p.gold };
        let gold_fill = Color { a: 0.35, ..p.gold };
        let rule_faint = Color { a: 0.4, ..p.rule };

        // Central lozenge (diamond)
        let loz_w = 14.0;
        let loz_h = 10.0;
        let lozenge = canvas::Path::new(|b| {
            b.move_to(Point::new(w / 2.0, cy - loz_h / 2.0));
            b.line_to(Point::new(w / 2.0 + loz_w / 2.0, cy));
            b.line_to(Point::new(w / 2.0, cy + loz_h / 2.0));
            b.line_to(Point::new(w / 2.0 - loz_w / 2.0, cy));
            b.line_to(Point::new(w / 2.0, cy - loz_h / 2.0));
        });
        frame.fill(&lozenge, gold_fill);
        frame.stroke(&lozenge, canvas::Stroke {
            style: canvas::Style::Solid(gold),
            width: 1.0,
            ..canvas::Stroke::default()
        });

        // Scrollwork: sine-wave curves extending from lozenge
        let scroll_len = (w * 0.25).min(120.0);
        let amplitude = 4.0;
        let segments = 20;

        for side in [-1.0_f32, 1.0] {
            let start_x = w / 2.0 + side * (loz_w / 2.0 + 4.0);
            let scroll = canvas::Path::new(|b| {
                for i in 0..=segments {
                    let t = i as f32 / segments as f32;
                    let x = start_x + side * t * scroll_len;
                    let y = cy + amplitude * (t * std::f32::consts::PI * 2.0).sin() * (1.0 - t);
                    if i == 0 { b.move_to(Point::new(x, y)); }
                    else { b.line_to(Point::new(x, y)); }
                }
            });
            frame.stroke(&scroll, canvas::Stroke {
                style: canvas::Style::Solid(gold),
                width: 1.0,
                ..canvas::Stroke::default()
            });

            // Terminal dot
            let end_x = start_x + side * scroll_len;
            frame.fill(&canvas::Path::circle(Point::new(end_x, cy), 2.0), gold);

            // Extending rule to edge
            let edge_x = if side < 0.0 { 12.0 } else { w - 12.0 };
            let rule = canvas::Path::new(|b| {
                b.move_to(Point::new(end_x + side * 6.0, cy));
                b.line_to(Point::new(edge_x, cy));
            });
            frame.stroke(&rule, canvas::Stroke {
                style: canvas::Style::Solid(rule_faint),
                width: 0.5,
                ..canvas::Stroke::default()
            });

            // Edge terminal dot
            frame.fill(&canvas::Path::circle(Point::new(edge_x, cy), 1.5), gold_fill);
        }

        vec![frame.into_geometry()]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PageBorderCorner — decorative corner bracket (20×20px)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub struct PageBorderCorner {
    pub corner: Corner,
}

impl canvas::Program<Message> for PageBorderCorner {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let s = bounds.width.min(bounds.height);
        let p = theme::palette();
        let line_color = Color { a: 0.55, ..p.rule_strong };
        let dot_color = Color { a: 0.65, ..p.gold };
        let line_len = s * 0.7;

        // Corner vertex and direction vectors based on corner type
        let (vx, vy, dx, dy) = match self.corner {
            Corner::TopLeft     => (2.0,     2.0,      1.0,  1.0),
            Corner::TopRight    => (s - 2.0, 2.0,     -1.0,  1.0),
            Corner::BottomLeft  => (2.0,     s - 2.0,  1.0, -1.0),
            Corner::BottomRight => (s - 2.0, s - 2.0, -1.0, -1.0),
        };

        // Two perpendicular lines from vertex
        let bracket = canvas::Path::new(|b| {
            // Horizontal arm
            b.move_to(Point::new(vx, vy));
            b.line_to(Point::new(vx + dx * line_len, vy));
            // Vertical arm
            b.move_to(Point::new(vx, vy));
            b.line_to(Point::new(vx, vy + dy * line_len));
        });
        frame.stroke(&bracket, canvas::Stroke {
            style: canvas::Style::Solid(line_color),
            width: 1.5,
            ..canvas::Stroke::default()
        });

        // Dots at line terminals
        frame.fill(
            &canvas::Path::circle(Point::new(vx + dx * line_len, vy), 1.5),
            dot_color,
        );
        frame.fill(
            &canvas::Path::circle(Point::new(vx, vy + dy * line_len), 1.5),
            dot_color,
        );

        // Small vertex ornament — tiny diamond
        let d = 2.5;
        let vertex_diamond = canvas::Path::new(|b| {
            b.move_to(Point::new(vx, vy - d));
            b.line_to(Point::new(vx + d, vy));
            b.line_to(Point::new(vx, vy + d));
            b.line_to(Point::new(vx - d, vy));
            b.line_to(Point::new(vx, vy - d));
        });
        frame.fill(&vertex_diamond, dot_color);

        vec![frame.into_geometry()]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ShootingStar — v11.8.F. Streaking comet with fading trail. Replaces the
// particle-puff sparkles inside the fetching pill per user request: "Like
// a shooting star kind of motif. Something of that nature."
// ═══════════════════════════════════════════════════════════════════════════

pub struct ShootingStar {
    pub time: f32,
}

impl canvas::Program<Message> for ShootingStar {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let p = theme::palette();
        let w = bounds.width;
        let h = bounds.height;

        // Two stars phased 0.5 cycle apart so there's always at least one
        // streak visible. Each completes a left→right pass over 1.6s,
        // then dwells off-screen briefly before re-entering.
        let cycle = 1.6_f32;
        for phase_offset in [0.0_f32, cycle * 0.5] {
            let local = ((self.time + phase_offset) % cycle) / cycle; // 0..1
            // Star x slides 0..1.2 (extra 0.2 so trail can exit cleanly)
            let star_x = local * 1.2 * w;
            // Slight downward arc — y dips through the bar
            let arc = ((local - 0.5) * 4.0).powi(2) - 1.0; // parabola, min -1 at 0.5
            let star_y = h * 0.5 + arc * h * 0.15;

            // Trail: 8 segments fading from full alpha back along travel
            let trail_segments = 8;
            let trail_len = w * 0.18;
            for seg in 0..trail_segments {
                let t = seg as f32 / trail_segments as f32;
                let seg_x = star_x - t * trail_len;
                let seg_y = star_y; // trail follows the head's y
                let seg_alpha = (1.0 - t) * 0.85;
                if seg_x < 0.0 || seg_x > w { continue; }
                let dot = canvas::Path::circle(
                    Point::new(seg_x, seg_y),
                    1.4 - 1.0 * t,
                );
                frame.fill(&dot, Color { a: seg_alpha, ..p.gold });
            }

            // Bright head
            if star_x >= 0.0 && star_x <= w {
                let head_color = Color { r: 1.0, g: 0.97, b: 0.85, a: 1.0 };
                let head = canvas::Path::circle(
                    Point::new(star_x, star_y),
                    1.8,
                );
                frame.fill(&head, head_color);
                // 4-point cross-flare for sparkle
                let r = 4.0;
                let cross = canvas::Path::new(|b| {
                    b.move_to(Point::new(star_x - r, star_y));
                    b.line_to(Point::new(star_x + r, star_y));
                    b.move_to(Point::new(star_x, star_y - r));
                    b.line_to(Point::new(star_x, star_y + r));
                });
                frame.stroke(&cross, canvas::Stroke {
                    style: canvas::Style::Solid(Color { a: 0.7, ..head_color }),
                    width: 1.0,
                    line_cap: canvas::LineCap::Round,
                    ..Default::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TabSparkle — gold particle burst on tab hover (20×16px)
// v9.0: 8 particles, varied sizes, gravity drift, faster burst + longer fade
// ═══════════════════════════════════════════════════════════════════════════

pub struct TabSparkle {
    pub alpha: f32,    // 0.0–1.0 driven by tab_hover_progress
    pub seed:  u32,    // per-tab seed for particle positions
}

impl canvas::Program<Message> for TabSparkle {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        if self.alpha < 0.01 {
            return vec![];
        }
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let p = theme::palette();
        let w = bounds.width;
        let h = bounds.height;

        // v11.6.G — 12 particles (up from 8), bigger sizes, gold + soft-white
        // mix, brighter alpha. User: "make it like a little star or sparkly
        // animation so you can see it more visibly."
        let particles: [(f32, f32, f32, bool); 12] = [
            (0.08, 0.20, 3.5, true),  (0.22, 0.55, 2.4, false),
            (0.36, 0.18, 4.2, true),  (0.50, 0.62, 2.8, false),
            (0.62, 0.30, 3.0, true),  (0.74, 0.78, 2.2, false),
            (0.85, 0.40, 4.5, true),  (0.92, 0.20, 2.0, false),
            (0.18, 0.82, 3.2, false), (0.40, 0.45, 5.0, true),
            (0.58, 0.85, 2.6, false), (0.78, 0.55, 3.8, true),
        ];

        let soft_white = Color { r: 1.0, g: 0.95, b: 0.82, a: 1.0 };

        for (i, &(fx, fy, size, is_gold)) in particles.iter().enumerate() {
            let delay = i as f32 * 0.05;  // tighter stagger (was 0.08)
            let particle_alpha = ((self.alpha - delay) * 5.0).clamp(0.0, 1.0); // 5× fade-in (was 4×)
            if particle_alpha < 0.01 { continue; }

            let seed_f = (self.seed.wrapping_mul(2654435761_u32.wrapping_add(i as u32))) as f32 / u32::MAX as f32;
            let px = (fx + seed_f * 0.2 - 0.1).clamp(0.05, 0.95) * w;
            let gravity = self.alpha * 2.5;
            let py = ((fy + seed_f * 0.15 - 0.075).clamp(0.05, 0.95) * h) + gravity;

            // 4-pointed star sparkle (cross of two lines) for the bigger
            // gold particles; circles for the soft-white sub-particles.
            let center = Point::new(px, py.min(h - 1.0));
            let base_color = if is_gold { p.gold } else { soft_white };
            let final_alpha = particle_alpha * if is_gold { 0.85 } else { 0.65 };
            let color = Color { a: final_alpha, ..base_color };

            if is_gold && size >= 3.0 {
                let r = size;
                let cross = canvas::Path::new(|b| {
                    b.move_to(Point::new(center.x - r, center.y));
                    b.line_to(Point::new(center.x + r, center.y));
                    b.move_to(Point::new(center.x, center.y - r));
                    b.line_to(Point::new(center.x, center.y + r));
                });
                frame.stroke(&cross, canvas::Stroke {
                    style: canvas::Style::Solid(color),
                    width: 1.4,
                    line_cap: canvas::LineCap::Round,
                    ..Default::default()
                });
                let dot = canvas::Path::circle(center, size * 0.4);
                frame.fill(&dot, color);
            } else {
                let dot = canvas::Path::circle(center, size * 0.6);
                frame.fill(&dot, color);
            }
        }

        vec![frame.into_geometry()]
    }
}
