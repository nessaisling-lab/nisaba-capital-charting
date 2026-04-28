//! Canvas-rendered decorative ornaments for the Grimoire UI (v7.3).
//!
//! Three widgets that transform the book layout from restyled containers
//! into a video game grimoire:
//! - BookSpine: vertical binding strip with cross-stitch marks
//! - PageHeaderOrnament: Renaissance-style flourish above page content
//! - PageBorderCorner: decorative corner brackets for the page frame

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
            style: canvas::Style::Solid(Color { a: 0.3, ..p.rule }),
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
        let gold = Color { a: 0.4, ..p.gold };
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
        let gold = Color { a: 0.5, ..p.gold };
        let gold_fill = Color { a: 0.2, ..p.gold };
        let rule_faint = Color { a: 0.25, ..p.rule };

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
        let line_color = Color { a: 0.4, ..p.rule_strong };
        let dot_color = Color { a: 0.5, ..p.gold };
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
