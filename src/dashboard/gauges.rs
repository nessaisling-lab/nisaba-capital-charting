use iced::widget::canvas::{self};
use iced::{Color, Point, Rectangle};
use iced::mouse;

use crate::state::Message;
use crate::theme;

// ---------------------------------------------------------------------------
// Fear & Greed gauge — semicircular canvas widget
// ---------------------------------------------------------------------------

pub struct FearGreedGauge {
    pub score: f32,
    pub label: String,
}

impl canvas::Program<Message> for FearGreedGauge {
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
        let fg        = theme::fg(theme);
        let fg_dim    = theme::fg_dim(theme);
        let label_col = theme::label_color(theme);

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), bg);

        let cx = bounds.width / 2.0;
        let cy = bounds.height - 10.0;
        let r = (cy - 4.0).min(bounds.width / 2.0 - 12.0);
        let arc_w = 13.0_f32;
        let r_mid = r - arc_w / 2.0;

        // score → angle: 0 = π (left), 100 = 0 (right), arc passes through top
        let to_angle = |s: f32| -> f32 {
            std::f32::consts::PI * (1.0 - s / 100.0)
        };

        // Five color zones
        const ZONES: &[(f32, f32, (f32, f32, f32))] = &[
            (0.0,  20.0, theme::GAUGE_EXTREME_FEAR),
            (20.0, 40.0, theme::GAUGE_FEAR),
            (40.0, 60.0, theme::GAUGE_NEUTRAL),
            (60.0, 80.0, theme::GAUGE_GREED),
            (80.0, 100.0, theme::GAUGE_EXTREME_GREED),
        ];

        // v11.3 — outer gold gilt trace (subtle Renaissance frame)
        let p_pal = theme::palette();
        let gilt = Color { a: 0.45, ..p_pal.gold };
        let outer_arc = canvas::Path::new(|b| {
            let steps = 80_usize;
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let a = std::f32::consts::PI * (1.0 - t);
                let x = cx + (r + 3.0) * a.cos();
                let y = cy - (r + 3.0) * a.sin();
                if i == 0 { b.move_to(Point::new(x, y)); } else { b.line_to(Point::new(x, y)); }
            }
        });
        frame.stroke(&outer_arc, canvas::Stroke {
            style: canvas::Style::Solid(gilt),
            width: 0.8,
            ..canvas::Stroke::default()
        });

        for &(s0, s1, (r_c, g_c, b_c)) in ZONES {
            let steps = 24_usize;
            let path = canvas::Path::new(|b| {
                for i in 0..=steps {
                    let t = i as f32 / steps as f32;
                    let a = to_angle(s0 + t * (s1 - s0));
                    let x = cx + r_mid * a.cos();
                    let y = cy - r_mid * a.sin();
                    if i == 0 { b.move_to(Point::new(x, y)); } else { b.line_to(Point::new(x, y)); }
                }
            });
            frame.stroke(&path, canvas::Stroke {
                style: canvas::Style::Solid(Color::from_rgb(r_c, g_c, b_c)),
                width: arc_w,
                ..canvas::Stroke::default()
            });
        }

        // v11.3 — sundial tick marks (compass-rose feel)
        // Major ticks every 25 pts (5 in total), minor every 5 pts.
        for i in 0..=20 {
            let pct = i as f32 * 5.0;
            let a = to_angle(pct);
            let is_major = i % 5 == 0;
            let inner_r = if is_major { r - arc_w - 6.0 } else { r - arc_w - 3.0 };
            let outer_r = r - arc_w + 1.0;
            let p1 = Point::new(cx + inner_r * a.cos(), cy - inner_r * a.sin());
            let p2 = Point::new(cx + outer_r * a.cos(), cy - outer_r * a.sin());
            let tick = canvas::Path::new(|b| { b.move_to(p1); b.line_to(p2); });
            frame.stroke(&tick, canvas::Stroke {
                style: canvas::Style::Solid(Color { a: if is_major { 0.65 } else { 0.30 }, ..p_pal.gold }),
                width: if is_major { 1.4 } else { 0.7 },
                ..canvas::Stroke::default()
            });
        }

        // Needle — gold backbone, dark accent on top
        let score = self.score.clamp(0.0, 100.0);
        let na = to_angle(score);
        let tip = Point::new(cx + (r - 3.0) * na.cos(), cy - (r - 3.0) * na.sin());
        let needle_back = canvas::Path::new(|b| {
            b.move_to(Point::new(cx, cy));
            b.line_to(tip);
        });
        frame.stroke(&needle_back, canvas::Stroke {
            style: canvas::Style::Solid(p_pal.gold),
            width: 3.0,
            ..canvas::Stroke::default()
        });
        frame.stroke(&needle_back, canvas::Stroke {
            style: canvas::Style::Solid(fg),
            width: 1.2,
            ..canvas::Stroke::default()
        });

        // Center cap — 4-pointed star (sundial gnomon look)
        let star = canvas::Path::new(|b| {
            let r_outer = 6.0;
            let r_inner = 2.2;
            for k in 0..8 {
                let a = std::f32::consts::PI * (k as f32) / 4.0;
                let rr = if k % 2 == 0 { r_outer } else { r_inner };
                let x = cx + rr * a.cos();
                let y = cy - rr * a.sin();
                if k == 0 { b.move_to(Point::new(x, y)); } else { b.line_to(Point::new(x, y)); }
            }
            b.close();
        });
        frame.fill(&star, p_pal.gold);
        let star_outline = canvas::Path::circle(Point::new(cx, cy), 1.2);
        frame.fill(&star_outline, fg);

        // Score number
        frame.fill_text(canvas::Text {
            content: format!("{:.0}", score),
            position: Point::new(cx, cy - 22.0),
            color: fg,
            size: iced::Pixels(15.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Center,
            ..canvas::Text::default()
        });

        // Label
        frame.fill_text(canvas::Text {
            content: self.label.clone(),
            position: Point::new(cx, cy - 8.0),
            color: label_col,
            size: iced::Pixels(8.5),
            align_x: iced::alignment::Horizontal::Center.into(),
            ..canvas::Text::default()
        });

        // End labels
        frame.fill_text(canvas::Text {
            content: "0".into(),
            position: Point::new(6.0, cy + 2.0),
            color: fg_dim,
            size: iced::Pixels(8.5),
            ..canvas::Text::default()
        });
        frame.fill_text(canvas::Text {
            content: "100".into(),
            position: Point::new(bounds.width - 22.0, cy + 2.0),
            color: fg_dim,
            size: iced::Pixels(8.5),
            ..canvas::Text::default()
        });

        vec![frame.into_geometry()]
    }
}
