use iced::widget::canvas::{self};
use iced::{Color, Point, Rectangle};
use iced::mouse;

use crate::state::Message;

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

        let is_dark = *theme != iced::Theme::Light;
        let bg        = if is_dark { Color::from_rgb(0.06, 0.06, 0.10) } else { Color::from_rgb(0.91, 0.92, 0.94) };
        let fg        = if is_dark { Color::WHITE }                       else { Color::from_rgb(0.08, 0.08, 0.08) };
        let fg_dim    = if is_dark { Color::from_rgba(1.0,1.0,1.0,0.40)} else { Color::from_rgba(0.0,0.0,0.0,0.45) };
        let label_col = if is_dark { Color::from_rgba(1.0,1.0,1.0,0.65)} else { Color::from_rgba(0.0,0.0,0.0,0.60) };

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
            (0.0,  20.0, (0.85, 0.12, 0.12)), // Extreme Fear — red
            (20.0, 40.0, (0.95, 0.48, 0.08)), // Fear — orange
            (40.0, 60.0, (0.90, 0.86, 0.08)), // Neutral — yellow
            (60.0, 80.0, (0.52, 0.86, 0.10)), // Greed — yellow-green
            (80.0, 100.0,(0.10, 0.76, 0.10)), // Extreme Greed — green
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
