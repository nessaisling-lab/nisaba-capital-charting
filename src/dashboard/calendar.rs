//! Astro Calendar — monthly view showing favorable/unfavorable days
//! based on astro scores for the selected ticker.
//!
//! Displays a 7-column calendar grid where each day cell is colored by
//! the astro score for that date. Green = favorable, red = unfavorable.

use chrono::{Datelike, NaiveDate, Weekday};
use iced::widget::canvas::{self};
use iced::{Color, Point, Rectangle, Size};
use iced::mouse;

use crate::state::Message;
use crate::theme;

/// Data for one day in the calendar.
#[derive(Debug, Clone)]
#[allow(dead_code)] // `label` populated from DB, displayed in future tooltip
pub struct CalendarDay {
    pub date: NaiveDate,
    pub astro_score: Option<f64>,
    pub label: Option<String>,
}

/// The calendar widget.
pub struct AstroCalendar {
    pub year: i32,
    pub month: u32,
    pub days: Vec<CalendarDay>,
}

impl AstroCalendar {
    /// v13.1.1 — Score → cell color mapping. Punchy red below 35 (the
    /// "Unfavorable" threshold matching the forecast list semantics) so
    /// the calendar visually flags sell-zone days. User v96 review:
    /// "I don't see the red for 25 here — that's a good example of that.
    /// Mark what is unfavorable here as well."
    ///
    /// Bands:
    ///   score >= 70 — saturated green (Optimal)
    ///   50-70      — green (Favorable)
    ///   35-50      — muted neutral
    ///   25-35      — orange (Unfavorable)
    ///   < 25       — vivid red (Misaligned)
    fn score_to_color(score: f64) -> Color {
        if score >= 70.0 {
            Color::from_rgb(0.20, 0.78, 0.35) // saturated green — Optimal
        } else if score >= 50.0 {
            let t = ((score - 50.0) / 20.0).clamp(0.0, 1.0) as f32;
            Color::from_rgb(
                0.50 * (1.0 - t) + 0.20 * t,
                0.70 * (1.0 - t) + 0.78 * t,
                0.20 * (1.0 - t) + 0.35 * t,
            )
        } else if score >= 35.0 {
            // Neutral muted band
            Color::from_rgb(0.55, 0.50, 0.30)
        } else if score >= 25.0 {
            // Unfavorable — orange warning
            Color::from_rgb(0.85, 0.45, 0.10)
        } else {
            // Misaligned — vivid red
            Color::from_rgb(0.92, 0.20, 0.20)
        }
    }
}

impl canvas::Program<Message> for AstroCalendar {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), theme::canvas_bg(_theme));

        let header_h = 20.0_f32;
        let cell_w = bounds.width / 7.0;
        let rows = 6; // max rows in a month
        let cell_h = (bounds.height - header_h) / rows as f32;

        // Day-of-week headers
        let dow = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        for (i, label) in dow.iter().enumerate() {
            frame.fill_text(canvas::Text {
                content: label.to_string(),
                position: Point::new(i as f32 * cell_w + cell_w / 2.0, 4.0),
                color: theme::fg_muted(_theme),
                size: iced::Pixels(9.0),
                align_x: iced::alignment::Horizontal::Center.into(),
                ..canvas::Text::default()
            });
        }

        // Find the first day of the month and its weekday offset
        let first = NaiveDate::from_ymd_opt(self.year, self.month, 1);
        let Some(first_date) = first else { return vec![frame.into_geometry()]; };
        let weekday_offset = match first_date.weekday() {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        };

        // Build a lookup from date -> score
        let score_map: std::collections::HashMap<NaiveDate, f64> = self.days.iter()
            .filter_map(|d| d.astro_score.map(|s| (d.date, s)))
            .collect();

        // Days in month
        let days_in_month = if self.month == 12 {
            NaiveDate::from_ymd_opt(self.year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(self.year, self.month + 1, 1)
        }.map(|d| d.pred_opt().map(|p| p.day()).unwrap_or(28)).unwrap_or(28);

        for day_num in 1..=days_in_month {
            let cell_index = weekday_offset + (day_num - 1) as usize;
            let col = cell_index % 7;
            let row_idx = cell_index / 7;
            let x = col as f32 * cell_w;
            let y = header_h + row_idx as f32 * cell_h;

            let Some(date) = NaiveDate::from_ymd_opt(self.year, self.month, day_num) else { continue };

            // Color cell by astro score
            if let Some(&score) = score_map.get(&date) {
                let color = Self::score_to_color(score);
                frame.fill_rectangle(
                    Point::new(x + 1.0, y + 1.0),
                    Size::new(cell_w - 2.0, cell_h - 2.0),
                    color,
                );
                // Score label
                frame.fill_text(canvas::Text {
                    content: format!("{score:.0}"),
                    position: Point::new(x + cell_w - 4.0, y + cell_h - 4.0),
                    color: Color::WHITE,
                    size: iced::Pixels(8.0),
                    align_x: iced::alignment::Horizontal::Right.into(),
                    align_y: iced::alignment::Vertical::Bottom,
                    ..canvas::Text::default()
                });
            }

            // Day number
            frame.fill_text(canvas::Text {
                content: format!("{day_num}"),
                position: Point::new(x + 3.0, y + 2.0),
                color: theme::label_color(_theme),
                size: iced::Pixels(10.0),
                ..canvas::Text::default()
            });
        }

        // Month/year title
        let month_name = match self.month {
            1 => "January", 2 => "February", 3 => "March", 4 => "April",
            5 => "May", 6 => "June", 7 => "July", 8 => "August",
            9 => "September", 10 => "October", 11 => "November", 12 => "December",
            _ => "?",
        };
        frame.fill_text(canvas::Text {
            content: format!("{month_name} {}", self.year),
            position: Point::new(bounds.width / 2.0, bounds.height - 2.0),
            color: theme::fg_muted(_theme),
            size: iced::Pixels(10.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Bottom,
            ..canvas::Text::default()
        });

        vec![frame.into_geometry()]
    }
}
