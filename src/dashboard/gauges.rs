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

        // Needle
        let score = self.score.clamp(0.0, 100.0);
        let na = to_angle(score);
        let needle_path = canvas::Path::new(|b| {
            b.move_to(Point::new(cx, cy));
            b.line_to(Point::new(cx + (r - 3.0) * na.cos(), cy - (r - 3.0) * na.sin()));
        });
        frame.stroke(&needle_path, canvas::Stroke {
            style: canvas::Style::Solid(fg),
            width: 2.0,
            ..canvas::Stroke::default()
        });

        // Center dot
        let dot = canvas::Path::circle(Point::new(cx, cy), 4.0);
        frame.fill(&dot, fg);

        // Score number
        frame.fill_text(canvas::Text {
            content: format!("{:.0}", score),
            position: Point::new(cx, cy - 22.0),
            color: fg,
            size: iced::Pixels(15.0),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Center,
            ..canvas::Text::default()
        });

        // Label
        frame.fill_text(canvas::Text {
            content: self.label.clone(),
            position: Point::new(cx, cy - 8.0),
            color: label_col,
            size: iced::Pixels(8.5),
            horizontal_alignment: iced::alignment::Horizontal::Center,
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
