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

        // 8 particles at deterministic positions seeded per tab (v9.0: up from 5)
        let particles: [(f32, f32, f32); 8] = [
            (0.10, 0.25, 2.5),
            (0.30, 0.65, 1.8),
            (0.50, 0.20, 3.0),
            (0.65, 0.75, 2.0),
            (0.80, 0.35, 3.5),
            (0.20, 0.80, 2.2),
            (0.90, 0.50, 1.5),
            (0.45, 0.45, 4.0),
        ];

        for (i, &(fx, fy, size)) in particles.iter().enumerate() {
            // Faster initial burst: 0.08 stagger (was 0.12), 4× fade-in speed
            let delay = i as f32 * 0.08;
            let particle_alpha = ((self.alpha - delay) * 4.0).clamp(0.0, 1.0);
            if particle_alpha < 0.01 { continue; }

            // Offset positions slightly by seed for variety across tabs
            let seed_f = (self.seed.wrapping_mul(2654435761_u32.wrapping_add(i as u32))) as f32 / u32::MAX as f32;
            let px = (fx + seed_f * 0.2 - 0.1).clamp(0.05, 0.95) * w;
            // Gravity drift: particles drift downward as alpha progresses
            let gravity = self.alpha * 2.5;  // pixels of downward drift
            let py = ((fy + seed_f * 0.15 - 0.075).clamp(0.05, 0.95) * h) + gravity;

            let dot = canvas::Path::circle(Point::new(px, py.min(h - 1.0)), size);
            frame.fill(&dot, Color { a: particle_alpha * 0.55, ..p.gold });
        }

        vec![frame.into_geometry()]
    }
}
