use iced::widget::canvas::Canvas;
use iced::widget::{column, container, horizontal_rule, row, scrollable, text, Column, Row};
use iced::{Alignment, Element, Length};

use crate::charts::{AstroMarker, LagrangeSparkline, PriceChart};
use crate::indicators::{compute_lagrange_score, compute_ticker_score, Indicators};
use crate::patterns;
use crate::signals::generate_signal_bullets;
use crate::state::{Dashboard, Message};
use crate::theme;

use super::shared::make_gauge;

impl Dashboard {
    pub(crate) fn view_overview(&self) -> Element<'_, Message> {
        // ── Chart data ──────────────────────────────────────
        let rows_chrono: Vec<_> = self.rows.iter().rev().cloned().collect();
        let max_bars = self.chart_timeframe.max_bars();
        let visible_rows: Vec<_> = if rows_chrono.len() > max_bars {
            rows_chrono[rows_chrono.len() - max_bars..].to_vec()
        } else {
            rows_chrono.clone()
        };

        let chart_data: Vec<f32> = visible_rows
            .iter()
            .map(|r| r.close.to_string().parse::<f32>().unwrap_or(0.0))
            .collect();
        let volumes: Vec<i64> = visible_rows.iter().map(|r| r.volume).collect();

        let (sma20, sma50, bb_upper, bb_lower) = match &self.indicators {
            Some(ind) => {
                let trim = |v: &[Option<f32>]| -> Vec<Option<f32>> {
                    if v.len() > max_bars {
                        v[v.len() - max_bars..].to_vec()
                    } else {
                        v.to_vec()
                    }
                };
                (
                    trim(&ind.sma20),
                    trim(&ind.sma50),
                    trim(&ind.bb_upper),
                    trim(&ind.bb_lower),
                )
            }
            None => (vec![], vec![], vec![], vec![]),
        };

        let mut astro_markers: Vec<AstroMarker> = self
            .lagrange_history
            .iter()
            .filter_map(|lh| {
                let idx = visible_rows.iter().position(|r| r.date == lh.score_date)?;
                let astro = lh.astro_score.unwrap_or(50.0);
                if astro >= 75.0 {
                    Some(AstroMarker {
                        bar_index: idx,
                        label: "★".to_string(),
                        favorable: true,
                    })
                } else if astro <= 25.0 {
                    Some(AstroMarker {
                        bar_index: idx,
                        label: "⚠".to_string(),
                        favorable: false,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Retrograde station markers (Rx = red, Direct = green)
        let planet_glyph = |p: &str| match p {
            "Mercury" => "☿",
            "Venus"   => "♀",
            "Mars"    => "♂",
            "Jupiter" => "♃",
            "Saturn"  => "♄",
            _         => "?",
        };
        for ev in &self.retrograde_events {
            if let Some(idx) = visible_rows.iter().position(|r| r.date == ev.fetch_date) {
                let is_direct = ev.station == "D";
                astro_markers.push(AstroMarker {
                    bar_index: idx,
                    label: format!("{}{}", planet_glyph(&ev.planet), ev.station),
                    favorable: is_direct,
                });
            }
        }

        let chart: Element<Message> = if self.rows.is_empty() {
            container(
                column![
                    text(format!("{} — Awaiting Data", self.selected_ticker))
                        .size(theme::text_lg()),
                    text("No price history loaded for this ticker yet.").size(theme::text_base()),
                    text("Run the scraper to fetch OHLCV data, then refresh.").size(theme::text_sm()),
                ]
                .spacing(6)
                .align_x(Alignment::Center),
            )
            .center_x(Length::Fill)
            .padding([40, 20])
            .into()
        } else {
            Canvas::new(PriceChart {
                data: chart_data.clone(),
                ticker: self.selected_ticker.clone(),
                sma20,
                sma50,
                bb_upper,
                bb_lower,
                rows_chrono: visible_rows.clone(),
                volumes,
                astro_markers,
            })
            .width(Length::Fill)
            .height(Length::Fixed(250.0))
            .into()
        };

        // Sparkline
        let sparkline_strip = column![
            text("Lagrange Score — 90-day history").size(theme::text_base()),
            Canvas::new(LagrangeSparkline {
                history: self.lagrange_history.clone()
            })
            .width(Length::Fill)
            .height(Length::Fixed(60.0)),
        ]
        .spacing(2);

        // Indicators
        let indicator_row = match &self.indicators {
            None => row![text(if self.rows.is_empty() {
                "Indicators: —"
            } else {
                "Indicators: loading..."
            })
            .size(theme::text_base())]
            .spacing(20),
            Some(ind) => {
                let rsi_val = Indicators::last(&ind.rsi_vals);
                let sma20_val = Indicators::last(&ind.sma20);
                let sma50_val = Indicators::last(&ind.sma50);
                let macd_val = Indicators::last(&ind.macd_line);
                let sig_val = Indicators::last(&ind.macd_sig);

                let rsi_str = rsi_val
                    .map(|v| format!("RSI(14): {v:.1}"))
                    .unwrap_or_else(|| "RSI: —".into());
                let macd_str = match (macd_val, sig_val) {
                    (Some(m), Some(s)) => format!("MACD: {m:+.2}  Signal: {s:+.2}"),
                    _ => "MACD: —".into(),
                };
                let sma_str = match (sma20_val, sma50_val) {
                    (Some(a), Some(b)) => format!("SMA20: ${a:.2}  SMA50: ${b:.2}"),
                    _ => "SMA: —".into(),
                };
                let analyst_str = match &self.analyst_rating {
                    Some(r) => {
                        let total = r.strong_buy + r.buy + r.hold + r.sell + r.strong_sell;
                        if total == 0 {
                            "Analysts: —".into()
                        } else {
                            format!(
                                "Analysts: {}SB {}B {}H {}S {}SS",
                                r.strong_buy, r.buy, r.hold, r.sell, r.strong_sell
                            )
                        }
                    }
                    None => "Analysts: —".into(),
                };
                let sentiment_str = match &self.sentiment {
                    Some(s) => {
                        let label = s.sentiment_label.as_deref().unwrap_or("—");
                        let score = s
                            .sentiment_score
                            .as_ref()
                            .map(|v| format!(" ({v:.2})"))
                            .unwrap_or_default();
                        format!("Sentiment: {label}{score}")
                    }
                    None => "Sentiment: —".into(),
                };
                let short_str = match &self.short_interest {
                    Some(si) => si
                        .short_pct
                        .as_ref()
                        .map(|p| format!("Short%: {p:.1}%"))
                        .unwrap_or_else(|| "Short%: —".into()),
                    None => "Short%: —".into(),
                };

                row![
                    text(rsi_str).size(theme::text_base()),
                    text(macd_str).size(theme::text_base()),
                    text(sma_str).size(theme::text_base()),
                    text(analyst_str).size(theme::text_base()),
                    text(sentiment_str).size(theme::text_base()),
                    text(short_str).size(theme::text_base()),
                ]
                .spacing(24)
            }
        };

        // Timeframe selector
        let timeframe_bar: Row<Message> = crate::state::ChartTimeframe::all().iter().fold(
            row![text("Timeframe:").size(theme::text_sm())]
                .spacing(4)
                .align_y(Alignment::Center),
            |r, &tf| {
                let label = if tf == self.chart_timeframe {
                    format!("[{}]", tf.label())
                } else {
                    tf.label().to_string()
                };
                r.push(
                    iced::widget::button(text(label).size(theme::text_sm()))
                        .on_press(Message::SetTimeframe(tf)),
                )
            },
        );

        // Patterns
        let chart_data_chrono: Vec<f32> = self
            .rows
            .iter()
            .rev()
            .map(|r| r.close.to_string().parse::<f32>().unwrap_or(0.0))
            .collect();
        let patterns_section: Element<'_, Message> = if let Some(ref ind) = self.indicators {
            let detected = patterns::detect_patterns(&chart_data_chrono, ind);
            if detected.is_empty() {
                text("Patterns: none detected")
                    .size(theme::text_sm())
                    .into()
            } else {
                let items: Vec<Element<Message>> = detected
                    .iter()
                    .map(|p| {
                        let color = if p.is_bullish() {
                            theme::ZONE_OPTIMAL
                        } else {
                            theme::ZONE_MISALIGNED
                        };
                        let icon = if p.is_bullish() { "▲" } else { "▼" };
                        text(format!("  {icon} {}", p.label()))
                            .size(theme::text_sm())
                            .color(color)
                            .into()
                    })
                    .collect();
                column![
                    text("Technical Patterns").size(theme::text_base()),
                    Column::with_children(items).spacing(2),
                ]
                .spacing(3)
                .into()
            }
        } else {
            text("Patterns: —").size(theme::text_sm()).into()
        };

        // ── Gauges ──────────────────────────────────────────
        let crypto_gauge = make_gauge(
            "Crypto / Risk Sentiment".to_string(),
            self.fear_greed.clone(),
            match &self.fear_greed_err {
                Some(_) => "unavailable".to_string(),
                None => "loading...".to_string(),
            },
        );
        let equities_gauge = make_gauge(
            "Equities Sentiment".to_string(),
            self.market_fg.clone(),
            match &self.market_fg_err {
                Some(e) => format!("err: {}", &e[..e.len().min(60)]),
                None => "loading...".to_string(),
            },
        );
        let ticker_gauge = match &self.indicators {
            Some(ind) => {
                let (score, label) = compute_ticker_score(ind, &self.rows, &self.sentiment);
                make_gauge(
                    format!("{} Score", self.selected_ticker),
                    Some((score, label)),
                    String::new(),
                )
            }
            None => make_gauge(
                format!("{} Score", self.selected_ticker),
                None,
                if self.rows.is_empty() {
                    "no data".to_string()
                } else {
                    "loading...".to_string()
                },
            ),
        };
        let astro_gauge = make_gauge(
            format!("{} Astrology", self.selected_ticker),
            self.astro_score.as_ref().and_then(|s| {
                let score = s.astro_score? as f32;
                let label = s.astro_label.clone().unwrap_or_default();
                Some((score, label))
            }),
            "run scraper".to_string(),
        );
        let lagrange_gauge = match &self.indicators {
            Some(ind) => {
                let (score, label, _) = compute_lagrange_score(
                    ind,
                    &self.rows,
                    &self.sentiment,
                    &self.astro_score,
                    &self.macro_data,
                    &self.short_interest,
                );
                make_gauge(
                    format!("{} Lagrange Score", self.selected_ticker),
                    Some((score, label)),
                    String::new(),
                )
            }
            None => make_gauge(
                format!("{} Lagrange Score", self.selected_ticker),
                None,
                if self.rows.is_empty() {
                    "no data".to_string()
                } else {
                    "loading...".to_string()
                },
            ),
        };

        let gauges_row = scrollable(
            row![
                crypto_gauge,
                equities_gauge,
                ticker_gauge,
                astro_gauge,
                lagrange_gauge
            ]
            .spacing(16),
        )
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default(),
        ));

        // ── Signal Intelligence ─────────────────────────────
        let signal_section = if self.rows.is_empty() {
            column![
                text("Signal Intelligence").size(theme::text_lg()),
                text("No price data yet — run the scraper to fetch OHLCV history for this ticker.")
                    .size(theme::text_base()),
            ]
            .spacing(4)
        } else {
            match &self.indicators {
                None => column![
                    text("Signal Intelligence").size(theme::text_lg()),
                    text("Loading indicators...").size(theme::text_base()),
                ]
                .spacing(4),
                Some(ind) => {
                    let (lagrange, lagrange_label, _) = compute_lagrange_score(
                        ind,
                        &self.rows,
                        &self.sentiment,
                        &self.astro_score,
                        &self.macro_data,
                        &self.short_interest,
                    );
                    let bullets = generate_signal_bullets(
                        &self.selected_ticker,
                        ind,
                        &self.rows,
                        &self.sentiment,
                        &self.astro_score,
                        &self.macro_data,
                        &self.short_interest,
                        &self.analyst_rating,
                        &self.earnings,
                    );
                    let bullet_items: Vec<Element<Message>> = bullets
                        .iter()
                        .map(|b| text(format!("  • {b}")).size(theme::text_base()).into())
                        .collect();
                    let verdict = text(format!(
                        "  Lagrange Score: {lagrange:.0}/100 — {lagrange_label}"
                    ))
                    .size(theme::text_base());
                    column![
                        text(format!("Signal Intelligence: {}", self.selected_ticker))
                            .size(theme::text_lg()),
                        horizontal_rule(1),
                        Column::with_children(bullet_items).spacing(4),
                        horizontal_rule(1),
                        verdict,
                    ]
                    .spacing(6)
                }
            }
        };

        // ── Scored Universe / Ranking ───────────────────────
        let watchlist_section = if self.watchlist.is_empty() {
            column![
                text("Scored Universe").size(theme::text_lg()),
                text("Loading...").size(theme::text_base()),
            ]
            .spacing(4)
        } else {
            let sort_label = if self.sort_watchlist_by_score {
                "Sort: Score ▼"
            } else {
                "Sort: Ticker A–Z"
            };
            let panel_header = row![
                text(format!(
                    "Scored Universe  —  {} tickers",
                    self.watchlist.len()
                ))
                .size(theme::text_lg())
                .width(Length::Fill),
                iced::widget::button(text(sort_label).size(theme::text_base()))
                    .on_press(Message::ToggleWatchlistSort),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            let hdr = row![
                text("#").size(theme::text_base()).width(Length::Fixed(24.0)),
                text("Ticker")
                    .size(theme::text_base())
                    .width(Length::Fixed(64.0)),
                text("Score")
                    .size(theme::text_base())
                    .width(Length::Fixed(90.0)),
                text("Astro")
                    .size(theme::text_base())
                    .width(Length::Fixed(100.0)),
                text("Sentiment")
                    .size(theme::text_base())
                    .width(Length::Fixed(120.0)),
                text("Short%")
                    .size(theme::text_base())
                    .width(Length::Fill),
            ]
            .spacing(8);

            let rank_rows: Vec<Element<Message>> = self
                .watchlist
                .iter()
                .enumerate()
                .map(|(i, w)| {
                    let score = w.quick_score();
                    let astro_str = match (w.astro_score, w.astro_label.as_deref()) {
                        (Some(s), Some(l)) => format!("{s:.0} {l}"),
                        (Some(s), None) => format!("{s:.0}"),
                        _ => "—".into(),
                    };
                    let sent_str = match (w.sentiment_score.as_ref(), w.sentiment_label.as_deref())
                    {
                        (Some(s), Some(l)) => format!("{l} ({s:.2})"),
                        (Some(s), None) => s.to_string(),
                        _ => "—".into(),
                    };
                    let short_str = w
                        .short_pct
                        .as_ref()
                        .map(|p| format!("{p:.1}%"))
                        .unwrap_or_else(|| "—".into());
                    let (zone_color, score_zone) = match score as u32 {
                        0..=24 => (theme::ZONE_MISALIGNED, "Mis"),
                        25..=44 => (theme::ZONE_UNFAVORABLE, "Unf"),
                        45..=55 => (theme::ZONE_NEUTRAL, "Neu"),
                        56..=75 => (theme::ZONE_FAVORABLE, "Fav"),
                        _ => (theme::ZONE_OPTIMAL, "Opt"),
                    };
                    let ticker_btn = iced::widget::button(
                        text(w.ticker.clone()).size(theme::text_base()),
                    )
                    .on_press(Message::TickerSelected(w.ticker.clone()));
                    row![
                        text(format!("{}", i + 1))
                            .size(theme::text_base())
                            .width(Length::Fixed(24.0)),
                        ticker_btn,
                        text(format!("{score:.0} {score_zone}"))
                            .size(theme::text_base())
                            .color(zone_color)
                            .width(Length::Fixed(90.0)),
                        text(astro_str)
                            .size(theme::text_base())
                            .width(Length::Fixed(100.0)),
                        text(sent_str)
                            .size(theme::text_base())
                            .width(Length::Fixed(120.0)),
                        text(short_str)
                            .size(theme::text_base())
                            .width(Length::Fill),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .into()
                })
                .collect();

            column![
                panel_header,
                horizontal_rule(1),
                hdr,
                horizontal_rule(1),
                scrollable(Column::with_children(rank_rows).spacing(4))
                    .height(Length::Fixed(240.0)),
            ]
            .spacing(5)
        };

        // ── Polymarket ──────────────────────────────────────
        let polymarket_section: Element<'_, Message> = if self.polymarket.is_empty() {
            column![
                text("Prediction Markets (Polymarket)").size(theme::text_md()),
                text("No prediction market data yet. Run the scraper to fetch top markets.")
                    .size(theme::text_sm()),
            ]
            .spacing(4)
            .into()
        } else {
            let pm_items: Vec<Element<Message>> = self
                .polymarket
                .iter()
                .take(8)
                .map(|m| {
                    let yes_pct = m
                        .outcome_yes
                        .as_ref()
                        .map(|d| {
                            format!(
                                "{:.0}%",
                                d.to_string().parse::<f64>().unwrap_or(0.0) * 100.0
                            )
                        })
                        .unwrap_or_else(|| "—".into());
                    let vol = m
                        .volume
                        .as_ref()
                        .map(|d| {
                            let v = d.to_string().parse::<f64>().unwrap_or(0.0);
                            if v >= 999_500.0 {
                                format!("${:.1}M", v / 1_000_000.0)
                            } else if v >= 1_000.0 {
                                format!("${:.0}K", v / 1_000.0)
                            } else {
                                format!("${v:.0}")
                            }
                        })
                        .unwrap_or_else(|| "—".into());
                    let cat = m.category.as_deref().unwrap_or("—");
                    row![
                        text(yes_pct)
                            .size(theme::text_base())
                            .width(Length::Fixed(48.0)),
                        text(cat.to_string())
                            .size(theme::text_xs())
                            .width(Length::Fixed(70.0)),
                        text(m.question.clone())
                            .size(theme::text_sm())
                            .width(Length::Fill),
                        text(vol)
                            .size(theme::text_xs())
                            .width(Length::Fixed(70.0)),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into()
                })
                .collect();
            column![
                text("Prediction Markets (Polymarket)").size(theme::text_md()),
                scrollable(Column::with_children(pm_items).spacing(3))
                    .height(Length::Fixed(140.0)),
            ]
            .spacing(4)
            .into()
        };

        // ── Two-column layout assembly ──────────────────────
        let macro_strip = self.build_macro_strip();

        let left_col: Column<Message> = column![
            timeframe_bar,
            chart,
            sparkline_strip,
            indicator_row,
            patterns_section,
            macro_strip,
        ]
        .spacing(8);

        let right_col: Column<Message> = column![
            container(signal_section).padding([10, 14]),
            horizontal_rule(1),
            container(watchlist_section).padding([10, 14]),
        ]
        .spacing(8);

        let two_col = row![
            container(left_col).width(Length::FillPortion(3)),
            container(right_col).width(Length::FillPortion(2)),
        ]
        .spacing(12);

        column![
            gauges_row,
            horizontal_rule(1),
            two_col,
            horizontal_rule(1),
            polymarket_section,
        ]
        .spacing(10)
        .into()
    }
}
