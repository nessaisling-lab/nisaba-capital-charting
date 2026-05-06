use iced::widget::canvas::{self, Action};
use iced::{Color, Point, Rectangle, Size};
use iced::mouse;
use pursuit_week4_automation::models::{LagrangeHistory, PriceRow};

use crate::state::Message;
use crate::helpers::format_shares;
use crate::theme;

// ---------------------------------------------------------------------------
// Price chart — Iced canvas widget with indicator overlays and hover tooltip
// ---------------------------------------------------------------------------

/// Represents an astro event marker to draw on the price chart.
#[derive(Debug, Clone)]
pub struct AstroMarker {
    pub bar_index: usize,
    pub label: String,
    pub favorable: bool, // true = green line, false = red line
}

pub struct PriceChart {
    pub data:          Vec<f32>,
    pub ticker:        String,
    pub sma20:         Vec<Option<f32>>,
    pub sma50:         Vec<Option<f32>>,
    pub bb_upper:      Vec<Option<f32>>,
    pub bb_lower:      Vec<Option<f32>>,
    pub rows_chrono:   Vec<PriceRow>,
    pub volumes:       Vec<i64>,
    pub astro_markers: Vec<AstroMarker>,
    pub draw_progress: f32,  // v9.0: 0→1 candle draw-in animation
    pub tooltip_dims:  (f32, f32, f32), // v11.3: (font_px, box_w, box_h)
    /// v11.6.K — cached static geometry. Cleared on data change so the
    /// expensive candle/SMA/BB/volume painting only runs once per data
    /// load instead of on every cursor move. Hover overlay still paints
    /// fresh each tick.
    pub cache:         std::sync::Arc<canvas::Cache>,
}

impl PriceChart {
    fn price_to_y(price: f32, min: f32, range: f32, pad_top: f32, h: f32) -> f32 {
        pad_top + h - ((price - min) / range) * h
    }

    fn draw_series(frame: &mut canvas::Frame, pts: &[Option<Point>], color: Color, width: f32) {
        let path = canvas::Path::new(|b| {
            let mut started = false;
            for p in pts {
                match p {
                    Some(pt) if started => b.line_to(*pt),
                    Some(pt) => { b.move_to(*pt); started = true; }
                    None => started = false,
                }
            }
        });
        frame.stroke(&path, canvas::Stroke {
            style: canvas::Style::Solid(color),
            width,
            ..canvas::Stroke::default()
        });
    }
}

impl canvas::Program<Message> for PriceChart {
    type State = Option<Point>;

    fn update(
        &self,
        state: &mut Option<Point>,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        match event {
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // v11.7.D — only invalidate when cursor crosses into a
                // new candle. Root cause of hover lag: every pixel-level
                // mouse move was triggering Action::capture() and an
                // entire-window redraw. With 1000Hz mice the window was
                // redrawing 1000× per second while sweeping the chart.
                // Snap to bar index — at most n redraws per pass instead.
                let new_pos = cursor.position_in(bounds);
                let bar_idx = |p: Option<Point>| -> Option<usize> {
                    let pos = p?;
                    let pad_left = 55.0_f32;
                    let pad_right = 15.0_f32;
                    let w = bounds.width - pad_left - pad_right;
                    if pos.x < pad_left || pos.x > pad_left + w { return None; }
                    let n = self.data.len();
                    if n < 2 { return None; }
                    let frac = ((pos.x - pad_left) / w).clamp(0.0, 1.0);
                    Some(((frac * (n - 1) as f32).round() as usize).min(n - 1))
                };
                let prev_bar = bar_idx(*state);
                let next_bar = bar_idx(new_pos);
                if prev_bar == next_bar {
                    *state = new_pos;
                    return None;
                }
                *state = new_pos;
                Some(Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorLeft) => {
                *state = None;
                Some(Action::capture())
            }
            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Option<Point>,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        state: &Option<Point>,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // v11.6.K — split static (cached) + hover (fresh) layers. Cursor
        // moves trigger only the lightweight hover repaint; candles, SMAs,
        // Bollinger bands, volume, and astro markers stay cached.
        if self.data.len() < 2 {
            let mut empty = canvas::Frame::new(renderer, bounds.size());
            empty.fill_rectangle(Point::ORIGIN, bounds.size(), theme::canvas_bg(_theme));
            return vec![empty.into_geometry()];
        }
        let static_geo = self.cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), theme::canvas_bg(_theme));

        // Price range — expand to fit OHLC wicks + BB bands
        let mut min = self.data.iter().cloned().fold(f32::INFINITY, f32::min);
        let mut max = self.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        // Include high/low from OHLC data so candlestick wicks don't clip
        for row in &self.rows_chrono {
            let h_f = row.high.to_string().parse::<f32>().unwrap_or(0.0);
            let l_f = row.low.to_string().parse::<f32>().unwrap_or(f32::INFINITY);
            if h_f > max { max = h_f; }
            if l_f < min { min = l_f; }
        }
        for &v in self.bb_upper.iter().filter_map(|x| x.as_ref()) { if v > max { max = v; } }
        for &v in self.bb_lower.iter().filter_map(|x| x.as_ref()) { if v < min { min = v; } }
        let range = max - min;
        if range == 0.0 { return; }

        let pad_left   = 55.0_f32;
        let pad_right  = 15.0_f32;
        let pad_top    = 15.0_f32;
        let pad_bottom = 30.0_f32;
        let vol_height = (bounds.height - pad_top - pad_bottom) * 0.18; // 18% for volume
        let w = bounds.width  - pad_left - pad_right;
        let h = bounds.height - pad_top  - pad_bottom - vol_height;
        let n = self.data.len();

        let x_of = |i: usize| pad_left + (i as f32 / (n - 1) as f32) * w;
        let y_of = |p: f32|   Self::price_to_y(p, min, range, pad_top, h);

        // Grid lines + Y labels
        for i in 0..=4 {
            let t  = i as f32 / 4.0;
            let y  = pad_top + h - t * h;
            let pr = min + t * range;
            let grid = canvas::Path::new(|b| {
                b.move_to(Point::new(pad_left, y));
                b.line_to(Point::new(pad_left + w, y));
            });
            frame.stroke(&grid, canvas::Stroke {
                style: canvas::Style::Solid(theme::grid_line(_theme)),
                width: 1.0,
                ..canvas::Stroke::default()
            });
            frame.fill_text(canvas::Text {
                content: if range < 5.0 { format!("{pr:.2}") }
                         else if range < 50.0 { format!("{pr:.1}") }
                         else { format!("{pr:.0}") },
                position: Point::new(pad_left - 5.0, y),
                color: theme::fg_muted(_theme),
                size: iced::Pixels(10.0),
                align_x: iced::alignment::Horizontal::Right.into(),
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }

        // Bollinger Bands (behind everything)
        if self.bb_upper.len() == n && self.bb_lower.len() == n {
            let upper_pts: Vec<Option<Point>> = self.bb_upper.iter().enumerate()
                .map(|(i, &v)| v.map(|p| Point::new(x_of(i), y_of(p)))).collect();
            let lower_pts: Vec<Option<Point>> = self.bb_lower.iter().enumerate()
                .map(|(i, &v)| v.map(|p| Point::new(x_of(i), y_of(p)))).collect();
            Self::draw_series(frame, &upper_pts, theme::BB_BLUE, 1.0);
            Self::draw_series(frame, &lower_pts, theme::BB_BLUE, 1.0);
        }

        // SMA 50 (yellow)
        if self.sma50.len() == n {
            let pts: Vec<Option<Point>> = self.sma50.iter().enumerate()
                .map(|(i, &v)| v.map(|p| Point::new(x_of(i), y_of(p)))).collect();
            Self::draw_series(frame, &pts, theme::SMA50_YELLOW, 1.2);
        }

        // SMA 20 (orange)
        if self.sma20.len() == n {
            let pts: Vec<Option<Point>> = self.sma20.iter().enumerate()
                .map(|(i, &v)| v.map(|p| Point::new(x_of(i), y_of(p)))).collect();
            Self::draw_series(frame, &pts, theme::SMA20_ORANGE, 1.2);
        }

        // Candlestick chart — OHLC bodies + wicks
        let bar_w = (w / n as f32).clamp(2.0, 12.0);
        let body_w = (bar_w * 0.7).max(1.5);
        let price_pts: Vec<Point> = self.data.iter().enumerate()
            .map(|(i, &p)| Point::new(x_of(i), y_of(p))).collect();

        for (i, row) in self.rows_chrono.iter().enumerate() {
            let open_f  = row.open.to_string().parse::<f32>().unwrap_or(0.0);
            let high_f  = row.high.to_string().parse::<f32>().unwrap_or(0.0);
            let low_f   = row.low.to_string().parse::<f32>().unwrap_or(0.0);
            let close_f = row.close.to_string().parse::<f32>().unwrap_or(0.0);

            let cx = x_of(i);
            let bullish = close_f >= open_f;
            let color = if bullish { theme::bullish(_theme) } else { theme::bearish(_theme) };

            // Per-candle stagger: leftmost candles appear first (v9.0 draw-in)
            let total = self.rows_chrono.len().max(1) as f32;
            let candle_delay = (i as f32 / total) * 0.6;  // stagger over 60% of progress
            let candle_t = crate::animation::ease_out_cubic(
                ((self.draw_progress - candle_delay) / 0.4).clamp(0.0, 1.0)
            );
            if candle_t < 0.001 { continue; }  // skip unborn candles

            // Midpoint for grow-from-center effect
            let mid_y = y_of((high_f + low_f) * 0.5);

            // Wick: thin vertical line from high to low (scaled by candle_t)
            let wick_top = mid_y + (y_of(high_f) - mid_y) * candle_t;
            let wick_bot = mid_y + (y_of(low_f) - mid_y) * candle_t;
            let wick = canvas::Path::new(|b| {
                b.move_to(Point::new(cx, wick_top));
                b.line_to(Point::new(cx, wick_bot));
            });
            frame.stroke(&wick, canvas::Stroke {
                style: canvas::Style::Solid(Color { a: candle_t, ..color }),
                width: 1.0,
                ..canvas::Stroke::default()
            });

            // Body: filled rectangle from open to close (scaled by candle_t)
            let raw_top = y_of(open_f.max(close_f));
            let raw_bot = y_of(open_f.min(close_f));
            let body_top = mid_y + (raw_top - mid_y) * candle_t;
            let body_bot = mid_y + (raw_bot - mid_y) * candle_t;
            let body_h = (body_bot - body_top).max(1.0); // min 1px for doji
            frame.fill_rectangle(
                Point::new(cx - body_w / 2.0, body_top),
                Size::new(body_w, body_h),
                Color { a: candle_t, ..color },
            );
        }

        // Ticker label
        frame.fill_text(canvas::Text {
            content: self.ticker.clone(),
            position: Point::new(pad_left + 6.0, pad_top + 4.0),
            color: theme::fg_dim(_theme),
            size: iced::Pixels(11.0),
            ..canvas::Text::default()
        });

        // Legend
        let legend = [
            ("— SMA20", theme::SMA20_ORANGE),
            ("— SMA50", theme::SMA50_YELLOW),
            ("— BB",    theme::BB_BLUE),
        ];
        for (i, (label, color)) in legend.iter().enumerate() {
            frame.fill_text(canvas::Text {
                content: label.to_string(),
                position: Point::new(bounds.width - 130.0 + i as f32 * 43.0, pad_top + 4.0),
                color: *color,
                size: iced::Pixels(9.0),
                ..canvas::Text::default()
            });
        }

        // High / low callouts
        let max_i = self.data.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal)).map(|(i, _)| i).unwrap_or(0);
        let min_i = self.data.iter().enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal)).map(|(i, _)| i).unwrap_or(0);
        for idx in [max_i, min_i] {
            let p = price_pts[idx];
            frame.fill_text(canvas::Text {
                content: format!("{:.2}", self.data[idx]),
                position: Point::new(p.x, p.y - 10.0),
                color: theme::label_color(_theme),
                size: iced::Pixels(9.0),
                align_x: iced::alignment::Horizontal::Center.into(),
                ..canvas::Text::default()
            });
        }

        // Current price label (right edge, highlighted)
        if let Some(&last_price) = self.data.last() {
            let last_y = y_of(last_price);
            let label = if range < 5.0 { format!("{last_price:.2}") }
                        else if range < 50.0 { format!("{last_price:.1}") }
                        else { format!("{last_price:.0}") };
            // Background pill
            let pill_w = 48.0_f32;
            let pill_h = 14.0_f32;
            frame.fill_rectangle(
                Point::new(pad_left + w + 2.0, last_y - pill_h / 2.0),
                iced::Size::new(pill_w, pill_h),
                theme::ACCENT_BLUE,
            );
            frame.fill_text(canvas::Text {
                content: label,
                position: Point::new(pad_left + w + 4.0, last_y),
                color: Color::WHITE,
                size: iced::Pixels(10.0),
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }

        // Volume bars (bottom strip)
        if self.volumes.len() == n {
            let max_vol = self.volumes.iter().cloned().max().unwrap_or(1) as f32;
            let vol_top = pad_top + h;
            let bar_w = (w / n as f32).max(1.0);
            for (i, &vol) in self.volumes.iter().enumerate() {
                let bar_h = (vol as f32 / max_vol) * vol_height;
                let bx = x_of(i) - bar_w / 2.0;
                let by = vol_top + vol_height - bar_h;
                let vol_color = if i > 0 && self.data[i] >= self.data[i - 1] {
                    Color::from_rgba(0.3, 0.7, 0.4, 0.5) // green = up day
                } else {
                    Color::from_rgba(0.7, 0.3, 0.3, 0.5) // red = down day
                };
                frame.fill_rectangle(
                    Point::new(bx, by),
                    Size::new(bar_w.min(6.0), bar_h),
                    vol_color,
                );
            }
        }

        // Astro event markers (vertical dashed lines)
        for marker in &self.astro_markers {
            if marker.bar_index < n {
                let mx = x_of(marker.bar_index);
                let marker_color = if marker.favorable {
                    Color::from_rgba(0.3, 0.8, 0.4, 0.6)
                } else {
                    Color::from_rgba(0.8, 0.3, 0.3, 0.6)
                };
                // Draw dashed vertical line (4px on, 4px off)
                let segments = ((h / 8.0) as usize).max(1);
                for s in 0..segments {
                    let sy = pad_top + s as f32 * 8.0;
                    let ey = (sy + 4.0).min(pad_top + h);
                    let seg = canvas::Path::new(|b| {
                        b.move_to(Point::new(mx, sy));
                        b.line_to(Point::new(mx, ey));
                    });
                    frame.stroke(&seg, canvas::Stroke {
                        style: canvas::Style::Solid(marker_color),
                        width: 1.0,
                        ..canvas::Stroke::default()
                    });
                }
                // Small label at top
                frame.fill_text(canvas::Text {
                    content: marker.label.clone(),
                    position: Point::new(mx + 2.0, pad_top - 2.0),
                    color: marker_color,
                    size: iced::Pixels(7.0),
                    ..canvas::Text::default()
                });
            }
        }

        }); // close cache.draw closure (v11.6.K)

        // ── Hover overlay — fresh frame each tick (cheap) ─────────
        // Recompute layout vars used by hover (cache closure consumed them)
        let mut hover_frame = canvas::Frame::new(renderer, bounds.size());
        let pad_left   = 55.0_f32;
        let pad_right  = 15.0_f32;
        let pad_top    = 15.0_f32;
        let pad_bottom = 30.0_f32;
        let vol_height = (bounds.height - pad_top - pad_bottom) * 0.18;
        let w = bounds.width  - pad_left - pad_right;
        let h = bounds.height - pad_top  - pad_bottom - vol_height;
        let n = self.data.len();
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for &v in &self.data { if v < min { min = v; } if v > max { max = v; } }
        for row in &self.rows_chrono {
            let h_f = row.high.to_string().parse::<f32>().unwrap_or(0.0);
            let l_f = row.low.to_string().parse::<f32>().unwrap_or(f32::INFINITY);
            if h_f > max { max = h_f; }
            if l_f < min { min = l_f; }
        }
        for &v in self.bb_upper.iter().filter_map(|x| x.as_ref()) { if v > max { max = v; } }
        for &v in self.bb_lower.iter().filter_map(|x| x.as_ref()) { if v < min { min = v; } }
        let range = max - min;
        if range > 0.0 {
        let x_of = |i: usize| pad_left + (i as f32 / (n - 1) as f32) * w;
        let y_of = |p: f32| Self::price_to_y(p, min, range, pad_top, h);
        let frame = &mut hover_frame;

        if let Some(pos) = state {
            if pos.x >= pad_left && pos.x <= pad_left + w && n > 1 {
                let frac = ((pos.x - pad_left) / w).clamp(0.0, 1.0);
                let bar_i = ((frac * (n - 1) as f32).round() as usize).min(n - 1);
                let bar_x = x_of(bar_i);
                let bar_y = y_of(self.data[bar_i]);

                // Vertical crosshair
                let vline = canvas::Path::new(|b| {
                    b.move_to(Point::new(bar_x, pad_top));
                    b.line_to(Point::new(bar_x, pad_top + h));
                });
                frame.stroke(&vline, canvas::Stroke {
                    style: canvas::Style::Solid(theme::fg_dim(_theme)),
                    width: 1.0,
                    ..canvas::Stroke::default()
                });

                // Horizontal crosshair
                let hline = canvas::Path::new(|b| {
                    b.move_to(Point::new(pad_left, bar_y));
                    b.line_to(Point::new(pad_left + w, bar_y));
                });
                frame.stroke(&hline, canvas::Stroke {
                    style: canvas::Style::Solid(theme::ring_dim(_theme)),
                    width: 1.0,
                    ..canvas::Stroke::default()
                });

                // OHLCV tooltip — v11.3: size driven by user setting
                // v11.5.F1+F2: spell out Opening/High/Low/Closing + Volume hint
                if let Some(row) = self.rows_chrono.get(bar_i) {
                    let label = format!(
                        "{}\nOpening:  {:.2}\nHigh:     {:.2}\nLow:      {:.2}\nClosing:  {:.2}\nVolume:   {} shares",
                        row.date,
                        row.open, row.high, row.low, row.close,
                        format_shares(row.volume),
                    );
                    let (font_px, box_w, box_h) = self.tooltip_dims;
                    let tip_x = if bar_x < pad_left + w / 2.0 { bar_x + 8.0 } else { bar_x - (box_w + 4.0) };
                    let tip_y = pad_top + 4.0;
                    let p = theme::palette();
                    // Dark bg card for contrast in both Parchment and Leather (v9.1)
                    let tip_bg = Color { r: 0.12, g: 0.10, b: 0.08, a: 0.95 };
                    frame.fill_rectangle(
                        Point::new(tip_x - 2.0, tip_y - 2.0),
                        iced::Size::new(box_w, box_h),
                        tip_bg,
                    );
                    let border = canvas::Path::new(|b| {
                        b.rectangle(Point::new(tip_x - 2.0, tip_y - 2.0), iced::Size::new(box_w, box_h));
                    });
                    frame.stroke(&border, canvas::Stroke {
                        style: canvas::Style::Solid(Color { a: 0.4, ..p.gold }),
                        width: 1.0,
                        ..canvas::Stroke::default()
                    });
                    frame.fill_text(canvas::Text {
                        content: label,
                        position: Point::new(tip_x + 4.0, tip_y + 4.0),
                        color: Color { r: 0.95, g: 0.90, b: 0.80, a: 1.0 },
                        size: iced::Pixels(font_px),
                        ..canvas::Text::default()
                    });
                }
            }
        }
        } // close `if range > 0.0`

        vec![static_geo, hover_frame.into_geometry()]
    }
}

// ---------------------------------------------------------------------------
// Lagrange Score Sparkline — 90-day history strip below the price chart
// ---------------------------------------------------------------------------

pub struct LagrangeSparkline {
    pub history: Vec<LagrangeHistory>,
}

impl<Message> canvas::Program<Message> for LagrangeSparkline {
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

        if self.history.is_empty() {
            frame.fill_text(canvas::Text {
                content: "No Lagrange history yet — run the scraper".to_string(),
                position: Point::new(8.0, bounds.height / 2.0 - 6.0),
                color: theme::fg_muted(_theme),
                size: iced::Pixels(10.0),
                ..canvas::Text::default()
            });
            return vec![frame.into_geometry()];
        }

        let n = self.history.len();
        let pad = 4.0f32;
        let inner_w = bounds.width - pad * 2.0;
        let inner_h = bounds.height - pad * 2.0;

        // Score range 0-100; draw horizontal zone bands first
        let zones: &[(f32, f32, Color)] = &[
            (0.0,  24.0, theme::SPARK_ZONE_MIS),
            (25.0, 44.0, theme::SPARK_ZONE_UNF),
            (45.0, 55.0, theme::SPARK_ZONE_NEU),
            (56.0, 75.0, theme::SPARK_ZONE_FAV),
            (76.0,100.0, theme::SPARK_ZONE_OPT),
        ];
        for (lo, hi, color) in zones {
            let y_hi = pad + inner_h - (hi / 100.0) * inner_h;
            let y_lo = pad + inner_h - (lo / 100.0) * inner_h;
            frame.fill_rectangle(
                Point::new(pad, y_hi),
                Size::new(inner_w, y_lo - y_hi),
                *color,
            );
        }

        // Grid lines at 25 / 45 / 55 / 75
        for level in [25.0f32, 45.0, 55.0, 75.0] {
            let y = pad + inner_h - (level / 100.0) * inner_h;
            let line = canvas::Path::new(|b| {
                b.move_to(Point::new(pad, y));
                b.line_to(Point::new(pad + inner_w, y));
            });
            frame.stroke(&line, canvas::Stroke {
                style: canvas::Style::Solid(theme::grid_line(_theme)),
                width: 0.5,
                ..Default::default()
            });
        }

        // Score line (handle n=1 by centering the single point)
        let denom = if n > 1 { (n - 1) as f32 } else { 1.0 };
        let pts: Vec<Point> = self.history.iter().enumerate().map(|(i, row)| {
            let x = if n == 1 { pad + inner_w / 2.0 } else { pad + (i as f32 / denom) * inner_w };
            let y = pad + inner_h - (row.score / 100.0) * inner_h;
            Point::new(x, y.max(pad))
        }).collect();

        // v11.6.F — color the score line by the most-recent zone so the
        // line itself reflects current state (in addition to the dot).
        let last_score = self.history.last().map(|r| r.score).unwrap_or(50.0);
        let line_color = if last_score >= 76.0 {
            theme::ZONE_OPTIMAL
        } else if last_score >= 56.0 {
            theme::ZONE_FAVORABLE
        } else if last_score >= 45.0 {
            theme::SPARKLINE_BLUE
        } else if last_score >= 25.0 {
            theme::ZONE_UNFAVORABLE
        } else {
            theme::ZONE_MISALIGNED
        };

        let line = canvas::Path::new(|b| {
            for (i, &pt) in pts.iter().enumerate() {
                if i == 0 { b.move_to(pt); } else { b.line_to(pt); }
            }
        });
        frame.stroke(&line, canvas::Stroke {
            style: canvas::Style::Solid(line_color),
            width: 1.8,
            ..Default::default()
        });

        // v11.6.F — zone tags on right edge (Opt/Fav/Neu/Unf/Mis)
        let tag_x = pad + inner_w + 1.0;
        if tag_x < bounds.width - 4.0 {
            for (level, label, color) in [
                (88.0_f32, "Opt", theme::ZONE_OPTIMAL),
                (66.0,    "Fav", theme::ZONE_FAVORABLE),
                (50.0,    "Neu", theme::SPARKLINE_BLUE),
                (35.0,    "Unf", theme::ZONE_UNFAVORABLE),
                (12.0,    "Mis", theme::ZONE_MISALIGNED),
            ] {
                let y = pad + inner_h - (level / 100.0) * inner_h;
                frame.fill_text(canvas::Text {
                    content: label.to_string(),
                    position: Point::new(tag_x.min(bounds.width - 22.0), y - 5.0),
                    color: Color { a: 0.7, ..color },
                    size: iced::Pixels(8.0),
                    ..canvas::Text::default()
                });
            }
        }

        // Dot + label at last point
        if let Some(&last) = pts.last() {
            frame.fill(&canvas::Path::circle(last, 3.0), theme::SPARKLINE_BLUE);
            if let Some(row) = self.history.last() {
                frame.fill_text(canvas::Text {
                    content: format!("{:.0}", row.score),
                    position: Point::new(last.x + 4.0, last.y - 8.0),
                    color: theme::SPARKLINE_BLUE,
                    size: iced::Pixels(9.0),
                    ..canvas::Text::default()
                });
            }
        }

        // v11.6.F — date axis labels every ~25% of width (5 anchors).
        if n >= 2 {
            let anchors = [0.0, 0.25, 0.50, 0.75, 1.0];
            for (i, t) in anchors.iter().enumerate() {
                let row_idx = ((n - 1) as f32 * t).round() as usize;
                let Some(row) = self.history.get(row_idx) else { continue };
                let x = pad + t * inner_w;
                // shift label off the edges to avoid clipping
                let x_pos = if i == 0 { x } else if i == anchors.len() - 1 { x - 52.0 } else { x - 28.0 };
                frame.fill_text(canvas::Text {
                    content: row.score_date.format("%m-%d").to_string(),
                    position: Point::new(x_pos.max(pad), bounds.height - 2.0),
                    color: theme::fg_dim(_theme),
                    size: iced::Pixels(8.0),
                    ..canvas::Text::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }
}

// ---------------------------------------------------------------------------
// Equity Curve — Paper Trail performance chart (portfolio vs SPY benchmark)
// ---------------------------------------------------------------------------

/// Normalizes a value series to percentage change from the first value.
fn normalize_to_pct(values: &[f64]) -> Vec<f64> {
    if values.len() < 2 {
        return vec![];
    }
    let base = values[0];
    if base <= 0.0 {
        return vec![0.0; values.len()];
    }
    values.iter().map(|v| (v - base) / base * 100.0).collect()
}

pub struct EquityCurve {
    pub portfolio_values: Vec<f64>,
    pub spy_values:       Vec<f64>,
}

impl EquityCurve {
    fn draw_line(frame: &mut canvas::Frame, pts: &[Point], color: Color, width: f32) {
        if pts.len() < 2 { return; }
        let path = canvas::Path::new(|b| {
            b.move_to(pts[0]);
            for p in &pts[1..] {
                b.line_to(*p);
            }
        });
        frame.stroke(&path, canvas::Stroke {
            style: canvas::Style::Solid(color),
            width,
            ..canvas::Stroke::default()
        });
    }
}

impl canvas::Program<Message> for EquityCurve {
    type State = Option<Point>;

    fn update(
        &self,
        state: &mut Option<Point>,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        match event {
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                *state = cursor.position_in(bounds);
                Some(Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorLeft) => {
                *state = None;
                Some(Action::capture())
            }
            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Option<Point>,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) { mouse::Interaction::Crosshair }
        else { mouse::Interaction::default() }
    }

    fn draw(
        &self,
        state: &Option<Point>,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), theme::canvas_bg(_theme));

        let port_pct = normalize_to_pct(&self.portfolio_values);
        let spy_pct  = normalize_to_pct(&self.spy_values);

        if port_pct.len() < 2 {
            frame.fill_text(canvas::Text {
                content: "Not enough data for equity curve yet".to_string(),
                position: Point::new(8.0, bounds.height / 2.0 - 6.0),
                color: theme::fg_muted(_theme),
                size: iced::Pixels(10.0),
                ..canvas::Text::default()
            });
            return vec![frame.into_geometry()];
        }

        let pad_left   = 55.0_f32;
        let pad_right  = 15.0_f32;
        let pad_top    = 20.0_f32;
        let pad_bottom = 20.0_f32;
        let w = bounds.width  - pad_left - pad_right;
        let h = bounds.height - pad_top  - pad_bottom;
        let n = port_pct.len();

        // Find Y range across both series
        let mut y_min: f64 = port_pct.iter().cloned().fold(f64::INFINITY, f64::min);
        let mut y_max: f64 = port_pct.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        for &v in &spy_pct {
            if v < y_min { y_min = v; }
            if v > y_max { y_max = v; }
        }
        // Ensure 0% baseline is always visible
        if y_min > 0.0 { y_min = -1.0; }
        if y_max < 0.0 { y_max = 1.0; }
        // Add 5% padding
        let pad_y = (y_max - y_min) * 0.05;
        y_min -= pad_y;
        y_max += pad_y;
        let y_range = y_max - y_min;
        if y_range < 0.001 { return vec![frame.into_geometry()]; }

        let x_of = |i: usize| -> f32 {
            pad_left + (i as f32 / (n - 1).max(1) as f32) * w
        };
        let y_of = |pct: f64| -> f32 {
            pad_top + ((y_max - pct) / y_range) as f32 * h
        };

        // Grid lines + Y labels (percentage)
        let grid_count = 5;
        for i in 0..=grid_count {
            let t = i as f64 / grid_count as f64;
            let pct_val = y_min + t * y_range;
            let y = y_of(pct_val);
            let grid = canvas::Path::new(|b| {
                b.move_to(Point::new(pad_left, y));
                b.line_to(Point::new(pad_left + w, y));
            });
            frame.stroke(&grid, canvas::Stroke {
                style: canvas::Style::Solid(theme::grid_line(_theme)),
                width: 1.0,
                ..canvas::Stroke::default()
            });
            frame.fill_text(canvas::Text {
                content: format!("{pct_val:+.1}%"),
                position: Point::new(pad_left - 5.0, y),
                color: theme::fg_muted(_theme),
                size: iced::Pixels(9.0),
                align_x: iced::alignment::Horizontal::Right.into(),
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }

        // 0% baseline (emphasized)
        let zero_y = y_of(0.0);
        if zero_y >= pad_top && zero_y <= pad_top + h {
            let baseline = canvas::Path::new(|b| {
                b.move_to(Point::new(pad_left, zero_y));
                b.line_to(Point::new(pad_left + w, zero_y));
            });
            frame.stroke(&baseline, canvas::Stroke {
                style: canvas::Style::Solid(Color { a: 0.3, ..theme::palette().ink }),
                width: 1.5,
                ..canvas::Stroke::default()
            });
        }

        // SPY line (draw first, behind portfolio)
        let spy_color = Color::from_rgba(0.5, 0.5, 0.6, 0.7);
        if spy_pct.len() >= 2 {
            let spy_n = spy_pct.len().min(n);
            let spy_pts: Vec<Point> = (0..spy_n)
                .map(|i| Point::new(x_of(i), y_of(spy_pct[i])))
                .collect();
            Self::draw_line(&mut frame, &spy_pts, spy_color, 1.5);

            // End label
            if let Some(&last_spy) = spy_pct.last() {
                let lx = x_of(spy_n.saturating_sub(1));
                let ly = y_of(last_spy);
                frame.fill_text(canvas::Text {
                    content: format!("SPY {last_spy:+.1}%"),
                    position: Point::new(lx + 4.0, ly),
                    color: spy_color,
                    size: iced::Pixels(9.0),
                    align_y: iced::alignment::Vertical::Center,
                    ..canvas::Text::default()
                });
            }
        }

        // Portfolio line (green)
        let port_color = Color::from_rgb(0.2, 0.8, 0.4);
        let port_pts: Vec<Point> = (0..n)
            .map(|i| Point::new(x_of(i), y_of(port_pct[i])))
            .collect();
        Self::draw_line(&mut frame, &port_pts, port_color, 2.0);

        // End label for portfolio
        if let Some(&last_port) = port_pct.last() {
            let lx = x_of(n - 1);
            let ly = y_of(last_port);
            frame.fill_text(canvas::Text {
                content: format!("Portfolio {last_port:+.1}%"),
                position: Point::new(lx + 4.0, ly - 12.0),
                color: port_color,
                size: iced::Pixels(9.0),
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }

        // Title
        frame.fill_text(canvas::Text {
            content: "Equity Curve (% return)".to_string(),
            position: Point::new(pad_left + 6.0, 4.0),
            color: theme::fg_dim(_theme),
            size: iced::Pixels(11.0),
            ..canvas::Text::default()
        });

        // Legend
        frame.fill_text(canvas::Text {
            content: "Portfolio".to_string(),
            position: Point::new(bounds.width - 120.0, 4.0),
            color: port_color,
            size: iced::Pixels(9.0),
            ..canvas::Text::default()
        });
        frame.fill_text(canvas::Text {
            content: "SPY".to_string(),
            position: Point::new(bounds.width - 60.0, 4.0),
            color: spy_color,
            size: iced::Pixels(9.0),
            ..canvas::Text::default()
        });

        // Hover crosshair + tooltip
        if let Some(pos) = state {
            if pos.x >= pad_left && pos.x <= pad_left + w && n > 1 {
                let frac = ((pos.x - pad_left) / w).clamp(0.0, 1.0);
                let bar_i = ((frac * (n - 1) as f32).round() as usize).min(n - 1);
                let bar_x = x_of(bar_i);

                // Vertical crosshair
                let vline = canvas::Path::new(|b| {
                    b.move_to(Point::new(bar_x, pad_top));
                    b.line_to(Point::new(bar_x, pad_top + h));
                });
                frame.stroke(&vline, canvas::Stroke {
                    style: canvas::Style::Solid(theme::fg_dim(_theme)),
                    width: 1.0,
                    ..canvas::Stroke::default()
                });

                // Tooltip
                let port_val = port_pct.get(bar_i).copied().unwrap_or(0.0);
                let spy_val = spy_pct.get(bar_i).copied().unwrap_or(0.0);
                let alpha_val = port_val - spy_val;
                let label = format!(
                    "Day {}\nPortfolio: {:+.2}%\nSPY: {:+.2}%\nAlpha: {:+.2}%",
                    bar_i + 1, port_val, spy_val, alpha_val
                );
                let tip_x = if bar_x < pad_left + w / 2.0 { bar_x + 8.0 } else { bar_x - 110.0 };
                let tip_bg = if theme::is_dark(_theme) {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.78)
                } else {
                    Color::from_rgba(0.15, 0.15, 0.20, 0.88)
                };
                frame.fill_rectangle(
                    Point::new(tip_x, pad_top + 4.0),
                    Size::new(105.0, 50.0),
                    tip_bg,
                );
                frame.fill_text(canvas::Text {
                    content: label,
                    position: Point::new(tip_x + 4.0, pad_top + 8.0),
                    color: Color::WHITE,
                    size: iced::Pixels(9.0),
                    ..canvas::Text::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }
}
