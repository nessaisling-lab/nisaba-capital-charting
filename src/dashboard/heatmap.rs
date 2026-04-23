//! Sector Heat Map — canvas widget showing sectors colored by average astro score.
//!
//! Each sector is drawn as a rectangle whose fill color reflects its average
//! astrological score across all scored tickers. Larger sectors (more tickers)
//! get proportionally wider cells. Labels show sector name, avg score, and count.

use iced::widget::canvas::{self};
use iced::{Color, Point, Rectangle, Size};
use iced::mouse;

use crate::db::SectorSummary;
use crate::state::Message;
use crate::theme;

pub struct SectorHeatMap {
    pub sectors: Vec<SectorSummary>,
}

impl canvas::Program<Message> for SectorHeatMap {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        iced_theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let bg = theme::canvas_bg(iced_theme);
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), bg);

        if self.sectors.is_empty() {
            let label = canvas::Text {
                content: "No sector data yet. Run scraper with FMP key to populate sectors.".into(),
                position: Point::new(bounds.width / 2.0, bounds.height / 2.0),
                color: theme::fg_dim(iced_theme),
                size: iced::Pixels(12.0),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Center,
                ..Default::default()
            };
            frame.fill_text(label);
            return vec![frame.into_geometry()];
        }

        let total_tickers: i64 = self.sectors.iter().map(|s| s.ticker_count).sum();
        if total_tickers == 0 {
            return vec![frame.into_geometry()];
        }

        let padding = 2.0;
        let available_width = bounds.width - padding;
        let cell_height = bounds.height - padding * 2.0;

        let mut x = padding;

        for sector in &self.sectors {
            // Width proportional to ticker count (minimum 40px so label fits)
            let fraction = sector.ticker_count as f32 / total_tickers as f32;
            let cell_width = (available_width * fraction).max(40.0);

            // Color based on average astro score
            let score = sector.avg_astro.unwrap_or(50.0) as f32;
            let fill = score_to_color(score);

            frame.fill_rectangle(
                Point::new(x, padding),
                Size::new(cell_width - 1.0, cell_height),
                fill,
            );

            // Text labels (sector name + score + count)
            if cell_width > 50.0 {
                let text_color = if score > 55.0 {
                    Color::from_rgb(0.0, 0.0, 0.0) // dark text on bright bg
                } else {
                    Color::from_rgb(1.0, 1.0, 1.0) // light text on dark bg
                };

                // Truncate sector name to fit
                let max_chars = ((cell_width - 8.0) / 6.0) as usize;
                let name: String = sector.sector.chars().take(max_chars).collect();

                let name_label = canvas::Text {
                    content: name,
                    position: Point::new(x + cell_width / 2.0, padding + cell_height * 0.3),
                    color: text_color,
                    size: iced::Pixels(10.0),
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment: iced::alignment::Vertical::Center,
                    ..Default::default()
                };
                frame.fill_text(name_label);

                let score_label = canvas::Text {
                    content: format!("{score:.0}"),
                    position: Point::new(x + cell_width / 2.0, padding + cell_height * 0.55),
                    color: text_color,
                    size: iced::Pixels(14.0),
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment: iced::alignment::Vertical::Center,
                    ..Default::default()
                };
                frame.fill_text(score_label);

                let count_label = canvas::Text {
                    content: format!("({} tkrs)", sector.ticker_count),
                    position: Point::new(x + cell_width / 2.0, padding + cell_height * 0.78),
                    color: text_color,
                    size: iced::Pixels(9.0),
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment: iced::alignment::Vertical::Center,
                    ..Default::default()
                };
                frame.fill_text(count_label);
            }

            x += cell_width;
            if x >= bounds.width { break; }
        }

        vec![frame.into_geometry()]
    }
}

/// Map a 0-100 astro score to a heat map color.
/// Red (0) -> Orange (25) -> Yellow (50) -> Green (75) -> Bright Green (100)
fn score_to_color(score: f32) -> Color {
    let s = (score / 100.0).clamp(0.0, 1.0);
    if s < 0.5 {
        // Red to Yellow
        let t = s * 2.0;
        Color::from_rgb(0.85 - t * 0.15, t * 0.7, 0.1)
    } else {
        // Yellow to Green
        let t = (s - 0.5) * 2.0;
        Color::from_rgb(0.7 - t * 0.55, 0.7 + t * 0.15, 0.1 + t * 0.3)
    }
}
