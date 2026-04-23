use iced::widget::canvas::Canvas;
use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input, Column, Row};
use iced::{Alignment, Element, Length};

use crate::agents::{AgentPersona, AgentVerdict};
use crate::calendar::AstroCalendar;
use crate::charts::AstroMarker;
use crate::state::ChartTimeframe;
use crate::astrology::{build_transits_section, build_wheel_legend, NatalWheel};
use crate::charts::{LagrangeSparkline, PriceChart};
use crate::heatmap::SectorHeatMap;
use crate::patterns;
use crate::gauges::FearGreedGauge;
use crate::helpers::{describe_8k_items, format_market_value, format_market_value_i64, format_shares};
use crate::indicators::{compute_lagrange_score, compute_ticker_score, Indicators};
use crate::signals::generate_signal_bullets;
use crate::state::{Dashboard, Message};
use crate::tabs::Tab;
use crate::theme;

impl Dashboard {
    pub fn view(&self) -> Element<'_, Message> {
        // Ticker selector buttons (pinned watchlist)
        let ticker_buttons: Row<Message> = self.tickers.iter().fold(row![].spacing(6), |r, ticker| {
            let btn = button(text(ticker).size(theme::TEXT_BASE)).on_press(Message::TickerSelected(ticker.clone()));
            r.push(btn)
        });

        // Search bar — type any ticker symbol and press Enter or Search
        let search_bar = row![
            text_input("Search any ticker…", &self.ticker_search_input)
                .id(crate::update::SEARCH_INPUT_ID)
                .on_input(Message::TickerSearchInput)
                .on_submit(Message::TickerSearchSubmit)
                .width(iced::Length::Fixed(200.0))
                .size(theme::TEXT_BASE),
            button(text("Go").size(theme::TEXT_BASE))
                .on_press(Message::TickerSearchSubmit),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

        // Autocomplete suggestion dropdown — shown when suggestions exist
        let autocomplete: Element<Message> = if self.autocomplete_suggestions.is_empty() {
            // Zero-height placeholder so layout doesn't jump
            row![].into()
        } else {
            let items: Vec<Element<Message>> = self.autocomplete_suggestions.iter()
                .map(|(ticker, name)| {
                    button(
                        text(format!("{ticker}  —  {name}")).size(theme::TEXT_BASE)
                    )
                    .on_press(Message::AutocompleteSelected(ticker.clone()))
                    .width(Length::Fixed(340.0))
                    .into()
                })
                .collect();
            column(items).spacing(2).into()
        };

        // Recently viewed — last 10 tickers as quick-click buttons
        let recently_viewed_row: Row<Message> = if self.recently_viewed.is_empty() {
            row![text("Recently viewed: —").size(theme::TEXT_BASE)].spacing(4)
        } else {
            let label = text("Recently:").size(theme::TEXT_BASE);
            let buttons = self.recently_viewed.iter().fold(
                row![label].spacing(6),
                |r, t| r.push(
                    button(text(t).size(theme::TEXT_BASE))
                        .on_press(Message::TickerSelected(t.clone()))
                ),
            );
            buttons
        };

        // Timeframe selector buttons
        let timeframe_bar: Row<Message> = ChartTimeframe::all().iter().fold(
            row![text("Timeframe:").size(theme::TEXT_SM)].spacing(4).align_y(Alignment::Center),
            |r, &tf| {
                let label = if tf == self.chart_timeframe {
                    format!("[{}]", tf.label())
                } else {
                    tf.label().to_string()
                };
                r.push(button(text(label).size(theme::TEXT_SM)).on_press(Message::SetTimeframe(tf)))
            },
        );

        // Close prices in chronological order, filtered by timeframe
        let rows_chrono: Vec<_> = self.rows.iter().rev().cloned().collect();
        let max_bars = self.chart_timeframe.max_bars();
        let visible_rows: Vec<_> = if rows_chrono.len() > max_bars {
            rows_chrono[rows_chrono.len() - max_bars..].to_vec()
        } else {
            rows_chrono.clone()
        };

        let chart_data: Vec<f32> = visible_rows.iter()
            .map(|r| r.close.to_string().parse::<f32>().unwrap_or(0.0))
            .collect();
        let volumes: Vec<i64> = visible_rows.iter().map(|r| r.volume).collect();

        let (sma20, sma50, bb_upper, bb_lower) = match &self.indicators {
            Some(ind) => {
                // Trim indicators to match visible window
                let trim = |v: &[Option<f32>]| -> Vec<Option<f32>> {
                    if v.len() > max_bars { v[v.len() - max_bars..].to_vec() } else { v.to_vec() }
                };
                (trim(&ind.sma20), trim(&ind.sma50), trim(&ind.bb_upper), trim(&ind.bb_lower))
            }
            None => (vec![], vec![], vec![], vec![]),
        };

        // Build astro markers: mark days where astro score crossed thresholds
        // Use lagrange_history which has dates aligned with price data
        let astro_markers: Vec<AstroMarker> = self.lagrange_history.iter().filter_map(|lh| {
            let idx = visible_rows.iter().position(|r| r.date == lh.score_date)?;
            let astro = lh.astro_score.unwrap_or(50.0);
            // Only mark significant days (score in top or bottom quintile)
            if astro >= 75.0 {
                Some(AstroMarker { bar_index: idx, label: "★".to_string(), favorable: true })
            } else if astro <= 25.0 {
                Some(AstroMarker { bar_index: idx, label: "⚠".to_string(), favorable: false })
            } else {
                None
            }
        }).collect();

        let chart: Element<Message> = if self.rows.is_empty() {
            container(
                column![
                    text(format!("{} — Awaiting Data", self.selected_ticker)).size(theme::TEXT_LG),
                    text("No price history loaded for this ticker yet.").size(theme::TEXT_BASE),
                    text("Run the scraper to fetch OHLCV data, then refresh.").size(theme::TEXT_SM),
                ].spacing(6).align_x(Alignment::Center),
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

        // Lagrange Score sparkline strip
        let sparkline_strip = column![
            text("Lagrange Score — 90-day history").size(theme::TEXT_BASE),
            Canvas::new(LagrangeSparkline { history: self.lagrange_history.clone() })
                .width(Length::Fill)
                .height(Length::Fixed(60.0)),
        ].spacing(2);

        // Indicator summary row
        let indicator_row = match &self.indicators {
            None => row![
                text(if self.rows.is_empty() { "Indicators: —" } else { "Indicators: loading..." }).size(theme::TEXT_BASE)
            ].spacing(20),
            Some(ind) => {
                let rsi_val  = Indicators::last(&ind.rsi_vals);
                let sma20_val = Indicators::last(&ind.sma20);
                let sma50_val = Indicators::last(&ind.sma50);
                let macd_val  = Indicators::last(&ind.macd_line);
                let sig_val   = Indicators::last(&ind.macd_sig);

                let rsi_str = rsi_val.map(|v| format!("RSI(14): {v:.1}")).unwrap_or_else(|| "RSI: —".into());
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
                        let score = s.sentiment_score
                            .as_ref()
                            .map(|v| format!(" ({v:.2})"))
                            .unwrap_or_default();
                        format!("Sentiment: {label}{score}")
                    }
                    None => "Sentiment: —".into(),
                };

                let short_str = match &self.short_interest {
                    Some(si) => si.short_pct.as_ref()
                        .map(|p| format!("Short%: {p:.1}%"))
                        .unwrap_or_else(|| "Short%: —".into()),
                    None => "Short%: —".into(),
                };

                row![
                    text(rsi_str).size(theme::TEXT_BASE),
                    text(macd_str).size(theme::TEXT_BASE),
                    text(sma_str).size(theme::TEXT_BASE),
                    text(analyst_str).size(theme::TEXT_BASE),
                    text(sentiment_str).size(theme::TEXT_BASE),
                    text(short_str).size(theme::TEXT_BASE),
                ]
                .spacing(24)
            }
        };

        // Earnings calendar
        let earnings_section = if self.earnings.is_empty() {
            column![
                text(format!("{} Earnings", self.selected_ticker)).size(theme::TEXT_MD),
                text("No earnings dates found for this ticker.").size(theme::TEXT_BASE),
                text("The scraper fetches earnings dates from Finnhub.").size(theme::TEXT_SM),
            ].spacing(4)
        } else {
            let today = chrono::Utc::now().date_naive();
            let hdr = row![
                text("Date").width(Length::Fixed(90.0)),
                text("Ticker").width(Length::Fixed(60.0)),
                text("EPS Est").width(Length::Fixed(80.0)),
                text("EPS Actual").width(Length::Fixed(90.0)),
                text("Rev Est").width(Length::Fill),
            ].spacing(8);
            let items: Vec<Element<Message>> = self.earnings.iter().map(|e| {
                let is_upcoming = e.earnings_date >= today;
                let eps_est = e.eps_estimate
                    .as_ref().map(|v| format!("${v:.2}")).unwrap_or_else(|| "—".into());
                let eps_act = e.eps_actual
                    .as_ref().map(|v| format!("${v:.2}")).unwrap_or_else(|| "—".into());
                let rev_est = e.revenue_estimate
                    .map(format_market_value_i64).unwrap_or_else(|| "—".into());
                let date_str = if is_upcoming {
                    format!(">> {}", e.earnings_date)
                } else {
                    e.earnings_date.to_string()
                };
                row![
                    text(date_str).width(Length::Fixed(90.0)),
                    text(&e.ticker).width(Length::Fixed(60.0)),
                    text(eps_est).width(Length::Fixed(80.0)),
                    text(eps_act).width(Length::Fixed(90.0)),
                    text(rev_est).width(Length::Fill),
                ].spacing(8).into()
            }).collect();
            column![
                text("Earnings Calendar").size(theme::TEXT_MD),
                hdr,
                scrollable(Column::with_children(items).spacing(4)).height(Length::Fixed(130.0)),
            ].spacing(4)
        };

        // Institutional holdings (13F)
        let holdings_section = if self.holdings.is_empty() {
            column![
                text("Top Institutional Holders").size(theme::TEXT_MD),
                text("No institutional holdings loaded yet.").size(theme::TEXT_BASE),
                text("The scraper fetches 13F filings from SEC EDGAR.").size(theme::TEXT_SM),
            ].spacing(4)
        } else {
            let hdr = row![
                text("Institution").width(Length::FillPortion(4)),
                text("Shares Held").width(Length::FillPortion(3)),
                text("Market Value").width(Length::FillPortion(3)),
                text("Period").width(Length::FillPortion(2)),
            ].spacing(8);
            let holding_rows: Vec<Element<Message>> = self.holdings.iter().map(|h| {
                row![
                    text(&h.institution_name).width(Length::FillPortion(4)),
                    text(format_shares(h.shares_held)).width(Length::FillPortion(3)),
                    text(format_market_value(&h.market_value)).width(Length::FillPortion(3)),
                    text(h.report_period.to_string()).width(Length::FillPortion(2)),
                ].spacing(8).into()
            }).collect();
            column![
                text("Top Institutional Holders").size(theme::TEXT_MD),
                hdr,
                scrollable(Column::with_children(holding_rows).spacing(4)).height(Length::Fixed(120.0)),
            ].spacing(4)
        };

        // 8-K filings
        let filings_section = if self.filings_8k.is_empty() {
            column![
                text("Recent 8-K Filings").size(theme::TEXT_MD),
                text("No recent filings loaded yet.").size(theme::TEXT_BASE),
                text("Material events (earnings, M&A, leadership changes) appear here.").size(theme::TEXT_SM),
            ].spacing(4)
        } else {
            let filing_rows: Vec<Element<Message>> = self.filings_8k.iter().map(|f| {
                let desc = f.items.as_deref().map(describe_8k_items).unwrap_or_else(|| "—".into());
                let url = f.edgar_url.clone();
                row![
                    text(f.filed_date.to_string()).size(theme::TEXT_BASE).width(Length::Fixed(90.0)),
                    text(desc).size(theme::TEXT_BASE).width(Length::Fill),
                    button(text("Copy").size(theme::TEXT_SM)).on_press(Message::CopyText(url.clone())),
                    button(text("Open").size(theme::TEXT_SM)).on_press(Message::OpenUrl(url)),
                ].spacing(8).align_y(Alignment::Center).into()
            }).collect();
            column![
                text("Recent 8-K Filings").size(theme::TEXT_MD),
                scrollable(Column::with_children(filing_rows).spacing(4)).height(Length::Fixed(100.0)),
            ].spacing(4)
        };

        // News headlines
        let news_section = if self.news.is_empty() {
            column![
                text("Recent News").size(theme::TEXT_MD),
                text("No headlines loaded yet.").size(theme::TEXT_BASE),
                text("News articles are fetched from Finnhub during scraper runs.").size(theme::TEXT_SM),
            ].spacing(4)
        } else {
            let news_items: Vec<Element<Message>> = self.news.iter().map(|n| {
                let source = n.source.as_deref().unwrap_or("—");
                let date = n.published_at.format("%b %d").to_string();
                let url = n.url.clone();
                let copy_text = format!("{} — {}", n.headline, n.url);
                row![
                    text(format!("[{date}]")).size(theme::TEXT_BASE).width(Length::Fixed(52.0)),
                    text(source.to_string()).size(theme::TEXT_BASE).width(Length::Fixed(72.0)),
                    text(&n.headline).size(theme::TEXT_BASE).width(Length::Fill),
                    button(text("Copy").size(theme::TEXT_SM)).on_press(Message::CopyText(copy_text)),
                    button(text("Open").size(theme::TEXT_SM)).on_press(Message::OpenUrl(url)),
                ].spacing(6).align_y(Alignment::Center).into()
            }).collect();
            column![
                text("Recent News").size(theme::TEXT_MD),
                scrollable(Column::with_children(news_items).spacing(4)).height(Length::Fixed(120.0)),
            ].spacing(4)
        };

        // Insider trades
        let insider_section = if self.insider_trades.is_empty() {
            column![
                text("Recent Insider Trades").size(theme::TEXT_MD),
                text("No Form 4 insider transactions loaded yet.").size(theme::TEXT_BASE),
                text("Insider buys and sells are fetched from SEC EDGAR.").size(theme::TEXT_SM),
            ].spacing(4)
        } else {
            let hdr = row![
                text("Date").width(Length::FillPortion(2)),
                text("Insider").width(Length::FillPortion(4)),
                text("Title").width(Length::FillPortion(3)),
                text("Type").width(Length::FillPortion(1)),
                text("Shares").width(Length::FillPortion(2)),
                text("Price").width(Length::FillPortion(2)),
            ].spacing(8);
            let trade_rows: Vec<Element<Message>> = self.insider_trades.iter().map(|t| {
                row![
                    text(t.transaction_date.to_string()).width(Length::FillPortion(2)),
                    text(&t.insider_name).width(Length::FillPortion(4)),
                    text(t.insider_title.as_deref().unwrap_or("—")).width(Length::FillPortion(3)),
                    text(if t.transaction_type == "A" { "Buy" } else { "Sell" }).width(Length::FillPortion(1)),
                    text(format!("{:.0}", t.shares)).width(Length::FillPortion(2)),
                    text(format!("${:.2}", t.price_per_share)).width(Length::FillPortion(2)),
                ].spacing(8).into()
            }).collect();
            column![
                text("Recent Insider Trades").size(theme::TEXT_MD),
                hdr,
                scrollable(Column::with_children(trade_rows).spacing(4)).height(Length::Fixed(130.0)),
            ].spacing(4)
        };

        // Price table
        let price_header = row![
            text("Date").width(Length::FillPortion(2)),
            text("Open").width(Length::FillPortion(2)),
            text("High").width(Length::FillPortion(2)),
            text("Low").width(Length::FillPortion(2)),
            text("Close").width(Length::FillPortion(2)),
            text("Volume").width(Length::FillPortion(3)),
        ].spacing(10);

        let data_rows: Column<Message> = if self.rows.is_empty() {
            column![text(&self.status).size(theme::TEXT_MD)]
        } else {
            let price_rows: Vec<Element<Message>> = self.rows.iter().map(|r| {
                row![
                    text(r.date.to_string()).width(Length::FillPortion(2)),
                    text(format!("{:.2}", r.open)).width(Length::FillPortion(2)),
                    text(format!("{:.2}", r.high)).width(Length::FillPortion(2)),
                    text(format!("{:.2}", r.low)).width(Length::FillPortion(2)),
                    text(format!("{:.2}", r.close)).width(Length::FillPortion(2)),
                    text(format_shares(r.volume)).width(Length::FillPortion(3)),
                ].spacing(10).into()
            }).collect();
            Column::with_children(price_rows).spacing(4)
        };

        let refresh_label = if self.refreshing { "Refreshing..." } else { "Refresh Now" };
        let theme_label   = format!("Theme: {}", self.theme_mode.label());

        // Gauge helper closure
        let make_gauge = |title: String, data: Option<(f32, String)>, fallback: String| -> Element<Message> {
            match data {
                Some((score, label)) => column![
                    text(title).size(theme::TEXT_SM),
                    Canvas::new(FearGreedGauge { score, label })
                        .width(Length::Fixed(148.0))
                        .height(Length::Fixed(82.0)),
                ].align_x(Alignment::Center).spacing(2).into(),
                None => column![
                    text(title).size(theme::TEXT_SM),
                    text(fallback).size(theme::TEXT_SM),
                ].align_x(Alignment::Center).spacing(4).into(),
            }
        };

        let crypto_gauge = make_gauge(
            "Crypto / Risk Sentiment".to_string(),
            self.fear_greed.clone(),
            match &self.fear_greed_err {
                Some(_) => "unavailable".to_string(),
                None    => "loading...".to_string(),
            },
        );

        let equities_gauge = make_gauge(
            "Equities Sentiment".to_string(),
            self.market_fg.clone(),
            match &self.market_fg_err {
                Some(e) => format!("err: {}", &e[..e.len().min(60)]),
                None    => "loading...".to_string(),
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
                if self.rows.is_empty() { "no data".to_string() } else { "loading...".to_string() },
            ),
        };

        // Astrological F&G gauge
        let astro_gauge = make_gauge(
            format!("{} Astrology", self.selected_ticker),
            self.astro_score.as_ref().and_then(|s| {
                let score = s.astro_score? as f32;
                let label = s.astro_label.clone().unwrap_or_default();
                Some((score, label))
            }),
            "run scraper".to_string(),
        );

        // Lagrange Score gauge — blends all signals
        let lagrange_gauge = match &self.indicators {
            Some(ind) => {
                let (score, label, _) = compute_lagrange_score(
                    ind, &self.rows, &self.sentiment,
                    &self.astro_score, &self.macro_data, &self.short_interest,
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
                if self.rows.is_empty() { "no data".to_string() } else { "loading...".to_string() },
            ),
        };

        let gauges_row = scrollable(
            row![crypto_gauge, equities_gauge, ticker_gauge, astro_gauge, lagrange_gauge]
                .spacing(16)
        )
        .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default()));

        let tab_subtitle = match self.active_tab {
            Tab::Astrology => "Astrology & Timing",
            Tab::Overview => "Daily Price Data",
            Tab::Universe => "Universe Explorer",
            Tab::Fundamentals => "Fundamentals & Agents",
            Tab::Research => "Research & Filings",
            Tab::Portfolio => "Portfolio & Positions",
            Tab::Settings => "Settings",
        };
        let header = row![
            text(format!("{} — {}", self.selected_ticker, tab_subtitle)).size(theme::TEXT_2XL),
            iced::widget::Space::with_width(Length::Fill),
            button(text(theme_label).size(theme::TEXT_SM)).on_press(Message::ToggleTheme),
        ].align_y(Alignment::Center);

        // Signal Intelligence panel
        let signal_section = if self.rows.is_empty() {
            column![
                text("Signal Intelligence").size(theme::TEXT_LG),
                text("No price data yet — run the scraper to fetch OHLCV history for this ticker.").size(theme::TEXT_BASE),
            ].spacing(4)
        } else {
            match &self.indicators { None => column![
                text("Signal Intelligence").size(theme::TEXT_LG),
                text("Loading indicators...").size(theme::TEXT_BASE),
            ].spacing(4),
            Some(ind) => {
                let (lagrange, lagrange_label, _) = compute_lagrange_score(
                    ind, &self.rows, &self.sentiment,
                    &self.astro_score, &self.macro_data, &self.short_interest,
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
                let bullet_items: Vec<Element<Message>> = bullets.iter()
                    .map(|b| text(format!("  • {b}")).size(theme::TEXT_BASE).into())
                    .collect();
                let verdict = text(format!(
                    "  Lagrange Score: {lagrange:.0}/100 — {lagrange_label}"
                )).size(theme::TEXT_BASE);
                column![
                    text(format!("Signal Intelligence: {}", self.selected_ticker)).size(theme::TEXT_LG),
                    horizontal_rule(1),
                    Column::with_children(bullet_items).spacing(4),
                    horizontal_rule(1),
                    verdict,
                ].spacing(6)
            }
        } // close else { match ...
        };

        // Scored Universe / Ranking panel
        let watchlist_section = if self.watchlist.is_empty() {
            column![
                text("Scored Universe").size(theme::TEXT_LG),
                text("Loading...").size(theme::TEXT_BASE),
            ].spacing(4)
        } else {
            let sort_label = if self.sort_watchlist_by_score { "Sort: Score ▼" } else { "Sort: Ticker A–Z" };
            let panel_header = row![
                text(format!("Scored Universe  —  {} tickers", self.watchlist.len()))
                    .size(theme::TEXT_LG)
                    .width(Length::Fill),
                button(text(sort_label).size(theme::TEXT_BASE))
                    .on_press(Message::ToggleWatchlistSort),
            ].spacing(8).align_y(Alignment::Center);

            let hdr = row![
                text("#").size(theme::TEXT_BASE).width(Length::Fixed(24.0)),
                text("Ticker").size(theme::TEXT_BASE).width(Length::Fixed(64.0)),
                text("Score").size(theme::TEXT_BASE).width(Length::Fixed(90.0)),
                text("Astro").size(theme::TEXT_BASE).width(Length::Fixed(100.0)),
                text("Sentiment").size(theme::TEXT_BASE).width(Length::Fixed(120.0)),
                text("Short%").size(theme::TEXT_BASE).width(Length::Fill),
            ].spacing(8);

            let rank_rows: Vec<Element<Message>> = self.watchlist.iter().enumerate().map(|(i, w)| {
                let score = w.quick_score();
                let astro_str = match (w.astro_score, w.astro_label.as_deref()) {
                    (Some(s), Some(l)) => format!("{s:.0} {l}"),
                    (Some(s), None)    => format!("{s:.0}"),
                    _                  => "—".into(),
                };
                let sent_str = match (w.sentiment_score.as_ref(), w.sentiment_label.as_deref()) {
                    (Some(s), Some(l)) => format!("{l} ({s:.2})"),
                    (Some(s), None)    => s.to_string(),
                    _                  => "—".into(),
                };
                let short_str = w.short_pct.as_ref()
                    .map(|p| format!("{p:.1}%"))
                    .unwrap_or_else(|| "—".into());
                let (zone_color, score_zone) = match score as u32 {
                    0..=24  => (theme::ZONE_MISALIGNED, "Mis"),
                    25..=44 => (theme::ZONE_UNFAVORABLE, "Unf"),
                    45..=55 => (theme::ZONE_NEUTRAL, "Neu"),
                    56..=75 => (theme::ZONE_FAVORABLE, "Fav"),
                    _       => (theme::ZONE_OPTIMAL, "Opt"),
                };
                let ticker_btn = button(text(w.ticker.clone()).size(theme::TEXT_BASE))
                    .on_press(Message::TickerSelected(w.ticker.clone()));
                row![
                    text(format!("{}", i + 1)).size(theme::TEXT_BASE).width(Length::Fixed(24.0)),
                    ticker_btn,
                    text(format!("{score:.0} {score_zone}")).size(theme::TEXT_BASE).color(zone_color).width(Length::Fixed(90.0)),
                    text(astro_str).size(theme::TEXT_BASE).width(Length::Fixed(100.0)),
                    text(sent_str).size(theme::TEXT_BASE).width(Length::Fixed(120.0)),
                    text(short_str).size(theme::TEXT_BASE).width(Length::Fill),
                ].spacing(8).align_y(Alignment::Center).into()
            }).collect();

            column![
                panel_header,
                horizontal_rule(1),
                hdr,
                horizontal_rule(1),
                scrollable(Column::with_children(rank_rows).spacing(4))
                    .height(Length::Fixed(240.0)),
            ].spacing(5)
        };

        // Macro strip — latest value for key FRED series, formatted to 2 decimal places
        let macro_find = |id: &str| -> String {
            self.macro_data.iter()
                .find(|m| m.series_id == id)
                .and_then(|m| m.value.as_ref())
                .and_then(|v| v.to_string().parse::<f64>().ok())
                .map(|v| format!("{v:.2}"))
                .unwrap_or_else(|| "—".into())
        };
        let macro_strip_us = row![
            text(format!("Fed Funds: {}%",   macro_find("FEDFUNDS"))).size(theme::TEXT_BASE),
            text(format!("CPI YoY: {}%",       macro_find("CPIAUCSL_YOY"))).size(theme::TEXT_BASE),
            text(format!("Unemploy: {}%",     macro_find("UNRATE"))).size(theme::TEXT_BASE),
            text(format!("10Y: {}%",          macro_find("GS10"))).size(theme::TEXT_BASE),
            text(format!("2Y: {}%",           macro_find("GS2"))).size(theme::TEXT_BASE),
            text(format!("Spread: {}%",       macro_find("T10Y2Y"))).size(theme::TEXT_BASE),
            text(format!("VIX: {}",           macro_find("VIXCLS"))).size(theme::TEXT_BASE),
            text(format!("WTI Oil: ${}",      macro_find("DCOILWTICO"))).size(theme::TEXT_BASE),
        ].spacing(20);
        // International macro (DBnomics) — shows "—" until first scraper run fetches data
        let macro_strip_intl = row![
            text(format!("Euribor 3M: {}%",  macro_find("DBNOMICS:ECB/FM/M.U2.EUR.RT.MM.EURIBOR3MD_.HSTA"))).size(theme::TEXT_BASE),
            text(format!("PBoC: {}%",         macro_find("DBNOMICS:BIS/WS_CBPOL/M.CN"))).size(theme::TEXT_BASE),
            text(format!("EU CPI: {}%",       macro_find("DBNOMICS:Eurostat/prc_hicp_manr/M.RCH_A.CP00.EA"))).size(theme::TEXT_BASE),
            text(format!("OECD CLI: {}",      macro_find("DBNOMICS:OECD/MEI_CLI/LOLITOAA.USA.M"))).size(theme::TEXT_BASE),
            text(format!("US Credit/GDP: {}%",macro_find("DBNOMICS:BIS/WS_TC/Q.US.P.A.M.770.A"))).size(theme::TEXT_BASE),
        ].spacing(20);
        let macro_strip = column![macro_strip_us, macro_strip_intl].spacing(4);

        // Astrology section: natal wheel + transits table side by side
        let moon_phase  = self.astro_score.as_ref().and_then(|s| s.moon_phase.as_deref());
        let moon_deg    = self.astro_score.as_ref().and_then(|s| s.moon_phase_deg);
        let mercury_rx  = self.astro_score.as_ref().and_then(|s| s.mercury_rx).unwrap_or(false);

        let astrology_section: Element<Message> = if self.natal_positions.is_empty() {
            column![
                text(format!("{} Astrology", self.selected_ticker)).size(theme::TEXT_LG),
                horizontal_rule(1),
                text("No birth chart yet for this ticker.").size(theme::TEXT_BASE),
                text("The scraper enriches ~50 tickers per day via SEC EDGAR.").size(theme::TEXT_SM),
                text("Once an IPO date is found, the natal chart is computed automatically.").size(theme::TEXT_SM),
            ]
            .spacing(6)
            .into()
        } else {
            let natal_wheel = Canvas::new(NatalWheel {
                natal:    self.natal_positions.clone(),
                transits: self.daily_transits.clone(),
            })
            .width(Length::Fixed(300.0))
            .height(Length::Fixed(300.0));

            let wheel_col = column![
                text(format!("{} Birth Chart", self.selected_ticker)).size(theme::TEXT_LG),
                natal_wheel,
                build_wheel_legend(),
            ].spacing(4).align_x(Alignment::Center);

            let transits_col = column![
                build_transits_section(&self.astro_aspects, moon_phase, moon_deg, mercury_rx),
            ].width(Length::Fill);

            // Horoscope reading section (if available)
            let horoscope_section: Element<Message> = if let Some(ref h) = self.horoscope {
                let key_transit_items: Vec<Element<Message>> = h.key_transits.iter().map(|t| {
                    row![
                        text(&t.transit_desc).size(theme::TEXT_SM).width(Length::Fixed(180.0)),
                        text(&t.strength).size(theme::TEXT_XS).width(Length::Fixed(120.0)),
                        text(&t.financial_implication).size(theme::TEXT_XS).width(Length::Fill),
                    ].spacing(8).into()
                }).collect();

                let mercury_line: Element<Message> = if let Some(ref warn) = h.mercury_warning {
                    text(format!("Mercury: {warn}")).size(theme::TEXT_SM)
                        .color(theme::ZONE_UNFAVORABLE).into()
                } else {
                    text("Mercury: Direct — clear communications").size(theme::TEXT_SM).into()
                };

                column![
                    horizontal_rule(1),
                    text("Horoscope Reading").size(theme::TEXT_LG),
                    text(&h.overall_outlook).size(theme::TEXT_BASE),
                    row![
                        text(format!("Theme: {}", h.dominant_theme)).size(theme::TEXT_SM),
                        text(format!("Confidence: {:.0}/100", h.confidence)).size(theme::TEXT_SM),
                    ].spacing(20),
                    text("Key Transits:").size(theme::TEXT_SM),
                    Column::with_children(key_transit_items).spacing(2),
                    row![
                        text(&h.moon_guidance).size(theme::TEXT_SM),
                        mercury_line,
                    ].spacing(20),
                    text(format!("Timing: {}", h.timing_window)).size(theme::TEXT_SM),
                ].spacing(6).into()
            } else {
                text("Horoscope reading not yet generated for today. Run the scraper to compute.")
                    .size(theme::TEXT_SM).into()
            };

            column![
                row![wheel_col, transits_col]
                    .spacing(20)
                    .align_y(Alignment::Start),
                horoscope_section,
            ].spacing(12).into()
        };

        // News and 8-K side by side
        let news_filings_row = row![
            column![filings_section].width(Length::FillPortion(1)),
            column![news_section].width(Length::FillPortion(1)),
        ].spacing(20);

        // Portfolio panel (with P&L when data available)
        let portfolio_section = if !self.portfolio_pnl.is_empty() {
            // Enhanced P&L view with current prices and astro scores
            let hdr = row![
                text("Ticker").size(theme::TEXT_SM).width(Length::Fixed(60.0)),
                text("Shares").size(theme::TEXT_SM).width(Length::Fixed(60.0)),
                text("Avg Cost").size(theme::TEXT_SM).width(Length::Fixed(72.0)),
                text("Last").size(theme::TEXT_SM).width(Length::Fixed(72.0)),
                text("P&L").size(theme::TEXT_SM).width(Length::Fixed(88.0)),
                text("P&L %").size(theme::TEXT_SM).width(Length::Fixed(60.0)),
                text("Astro").size(theme::TEXT_SM).width(Length::Fill),
            ].spacing(6);

            let mut total_cost = 0.0_f64;
            let mut total_value = 0.0_f64;

            let pos_rows: Vec<Element<Message>> = self.portfolio_pnl.iter().map(|p| {
                let cost_basis = p.shares as f64 * p.avg_cost as f64;
                let last_price = p.last_close.as_ref()
                    .and_then(|v| v.to_string().parse::<f64>().ok())
                    .unwrap_or(0.0);
                let mkt_value = p.shares as f64 * last_price;
                let pnl = mkt_value - cost_basis;
                let pnl_pct = if cost_basis > 0.0 { pnl / cost_basis * 100.0 } else { 0.0 };

                total_cost += cost_basis;
                total_value += mkt_value;

                let pnl_color = if pnl > 0.0 { theme::ZONE_OPTIMAL } else if pnl < 0.0 { theme::ZONE_MISALIGNED } else { theme::ZONE_NEUTRAL };
                let astro_label = match (&p.astro_score, &p.astro_label) {
                    (Some(s), Some(l)) => format!("{s:.0} {l}"),
                    _ => "---".to_string(),
                };

                row![
                    text(&p.ticker).size(theme::TEXT_SM).width(Length::Fixed(60.0)),
                    text(format!("{:.1}", p.shares)).size(theme::TEXT_SM).width(Length::Fixed(60.0)),
                    text(format!("${:.2}", p.avg_cost)).size(theme::TEXT_SM).width(Length::Fixed(72.0)),
                    text(if last_price > 0.0 { format!("${last_price:.2}") } else { "---".to_string() }).size(theme::TEXT_SM).width(Length::Fixed(72.0)),
                    text(format!("{:+.0}", pnl)).size(theme::TEXT_SM).color(pnl_color).width(Length::Fixed(88.0)),
                    text(format!("{:+.1}%", pnl_pct)).size(theme::TEXT_SM).color(pnl_color).width(Length::Fixed(60.0)),
                    text(astro_label).size(theme::TEXT_SM).width(Length::Fill),
                ].spacing(6).into()
            }).collect();

            let total_pnl = total_value - total_cost;
            let total_pnl_pct = if total_cost > 0.0 { total_pnl / total_cost * 100.0 } else { 0.0 };
            let total_color = if total_pnl > 0.0 { theme::ZONE_OPTIMAL } else if total_pnl < 0.0 { theme::ZONE_MISALIGNED } else { theme::ZONE_NEUTRAL };

            column![
                text("Portfolio").size(theme::TEXT_MD),
                horizontal_rule(1),
                hdr,
                Column::with_children(pos_rows).spacing(2),
                horizontal_rule(1),
                row![
                    text(format!("Cost: ${total_cost:.0}")).size(theme::TEXT_SM),
                    text(format!("Value: ${total_value:.0}")).size(theme::TEXT_SM),
                    text(format!("P&L: {:+.0} ({:+.1}%)", total_pnl, total_pnl_pct)).size(theme::TEXT_BASE).color(total_color),
                ].spacing(16),
            ].spacing(4)
        } else if self.portfolio.is_empty() {
            column![
                text("Portfolio").size(theme::TEXT_MD),
                text("No positions tracked yet.").size(theme::TEXT_BASE),
                text("Add rows to portfolio_positions via portfolio_seed.sql.").size(theme::TEXT_SM),
            ].spacing(4)
        } else {
            // Fallback: basic view without prices (shouldn't normally hit this)
            let hdr = row![
                text("Ticker").size(theme::TEXT_BASE).width(Length::Fixed(64.0)),
                text("Shares").size(theme::TEXT_BASE).width(Length::Fixed(72.0)),
                text("Avg Cost").size(theme::TEXT_BASE).width(Length::Fixed(88.0)),
                text("Cost Basis").size(theme::TEXT_BASE).width(Length::Fill),
            ].spacing(8);

            let pos_rows: Vec<Element<Message>> = self.portfolio.iter().map(|p| {
                let cost_basis = p.shares * p.avg_cost;
                row![
                    text(&p.ticker).size(theme::TEXT_BASE).width(Length::Fixed(64.0)),
                    text(format!("{:.2}", p.shares)).size(theme::TEXT_BASE).width(Length::Fixed(72.0)),
                    text(format!("${:.2}", p.avg_cost)).size(theme::TEXT_BASE).width(Length::Fixed(88.0)),
                    text(format!("${cost_basis:.0}")).size(theme::TEXT_BASE).width(Length::Fill),
                ].spacing(8).into()
            }).collect();

            let total_basis: f32 = self.portfolio.iter().map(|p| p.shares * p.avg_cost).sum();
            column![
                text("Portfolio").size(theme::TEXT_MD),
                horizontal_rule(1),
                hdr,
                Column::with_children(pos_rows).spacing(2),
                horizontal_rule(1),
                text(format!("Total cost basis: ${total_basis:.0}")).size(theme::TEXT_BASE),
            ].spacing(4)
        };

        // Alerts panel
        let alerts_section = {
            let unread = self.unread_alert_count;
            let heading = if unread > 0 {
                format!("Lagrange Alerts  [{unread} new]")
            } else {
                "Lagrange Alerts".to_string()
            };
            if self.alerts.is_empty() {
                column![
                    text(heading).size(theme::TEXT_LG),
                    text("No alerts yet — fires when a ticker enters Optimal or Misaligned zone").size(theme::TEXT_BASE),
                ].spacing(4)
            } else {
                let hdr = row![
                    text("Date").size(theme::TEXT_BASE).width(Length::Fixed(90.0)),
                    text("Ticker").size(theme::TEXT_BASE).width(Length::Fixed(64.0)),
                    text("Score").size(theme::TEXT_BASE).width(Length::Fixed(56.0)),
                    text("Zone").size(theme::TEXT_BASE).width(Length::Fixed(110.0)),
                    text("Was").size(theme::TEXT_BASE).width(Length::Fill),
                    text("").size(theme::TEXT_BASE).width(Length::Fixed(80.0)),
                ].spacing(8);
                let alert_rows: Vec<Element<Message>> = self.alerts.iter().map(|a| {
                    let zone_color = if a.label == "Optimal" {
                        theme::ZONE_OPTIMAL
                    } else {
                        theme::ZONE_MISALIGNED
                    };
                    let prev = a.prev_label.as_deref().unwrap_or("—");
                    let read_btn: Element<Message> = if a.is_read {
                        text("✓ read").size(theme::TEXT_SM).width(Length::Fixed(80.0)).into()
                    } else {
                        button(text("Mark Read").size(theme::TEXT_SM))
                            .on_press(Message::MarkAlertRead(a.id))
                            .into()
                    };
                    row![
                        text(a.alert_date.to_string()).size(theme::TEXT_BASE).width(Length::Fixed(90.0)),
                        text(&a.ticker).size(theme::TEXT_BASE).width(Length::Fixed(64.0)),
                        text(format!("{:.1}", a.score)).size(theme::TEXT_BASE).width(Length::Fixed(56.0)),
                        text(&a.label).size(theme::TEXT_BASE).color(zone_color).width(Length::Fixed(110.0)),
                        text(prev.to_string()).size(theme::TEXT_BASE).width(Length::Fill),
                        read_btn,
                    ].spacing(8).align_y(Alignment::Center).into()
                }).collect();
                column![
                    text(heading).size(theme::TEXT_LG),
                    horizontal_rule(1),
                    hdr,
                    horizontal_rule(1),
                    scrollable(Column::with_children(alert_rows).spacing(4))
                        .height(Length::Fixed(160.0)),
                ].spacing(4)
            }
        };

        // ── Tab bar ──────────────────────────────────────────────────
        let tab_bar: Row<Message> = Tab::all().iter().fold(row![].spacing(4), |r, &tab| {
            let label = if tab == self.active_tab {
                format!("[{}]", tab.label())
            } else {
                tab.label().to_string()
            };
            let btn: Element<Message> = button(text(label).size(theme::TEXT_BASE))
                .on_press(Message::TabSelected(tab))
                .into();
            r.push(btn)
        });

        // ── Tab content dispatch ────────────────────────────────────
        let tab_content: Element<Message> = match self.active_tab {
            Tab::Astrology => {
                // ── Backtest Section ─────────────────────────────
                let backtest_section: Element<'_, Message> = {
                    let config_row = row![
                        text("Buy when astro >").size(theme::TEXT_SM),
                        text_input("65", &self.backtest_buy_input)
                            .on_input(Message::BacktestBuyInput)
                            .width(Length::Fixed(50.0))
                            .size(theme::TEXT_SM),
                        text("Sell when astro <").size(theme::TEXT_SM),
                        text_input("35", &self.backtest_sell_input)
                            .on_input(Message::BacktestSellInput)
                            .width(Length::Fixed(50.0))
                            .size(theme::TEXT_SM),
                        button(text("Run Backtest").size(theme::TEXT_SM)).on_press(Message::RunBacktest),
                    ].spacing(8).align_y(Alignment::Center);

                    if let Some(ref bt) = self.backtest_result {
                        let strat_color = if bt.total_return_pct > bt.buy_hold_return_pct {
                            theme::ZONE_OPTIMAL
                        } else {
                            theme::ZONE_MISALIGNED
                        };
                        let acc_color = if bt.signal_accuracy_pct >= 55.0 {
                            theme::ZONE_OPTIMAL
                        } else if bt.signal_accuracy_pct >= 45.0 {
                            theme::ZONE_NEUTRAL
                        } else {
                            theme::ZONE_MISALIGNED
                        };

                        let metrics = column![
                            row![
                                text(format!("Strategy: {:.1}%", bt.total_return_pct)).size(theme::TEXT_BASE).color(strat_color),
                                text("  vs  ").size(theme::TEXT_SM),
                                text(format!("Buy & Hold: {:.1}%", bt.buy_hold_return_pct)).size(theme::TEXT_BASE),
                            ].spacing(4),
                            row![
                                text(format!("Trades: {}", bt.num_trades)).size(theme::TEXT_SM),
                                text(format!("Win Rate: {:.0}%", bt.win_rate_pct)).size(theme::TEXT_SM),
                                text(format!("Max DD: {:.1}%", bt.max_drawdown_pct)).size(theme::TEXT_SM),
                                text(format!("Final: ${:.0}", bt.final_capital)).size(theme::TEXT_SM),
                            ].spacing(12),
                            text(format!("Astro Signal Accuracy (30d): {:.1}%", bt.signal_accuracy_pct))
                                .size(theme::TEXT_BASE)
                                .color(acc_color),
                        ].spacing(4);

                        // Recent trades table (last 10)
                        let trade_rows: Vec<Element<Message>> = bt.trades.iter().rev().take(10).map(|t| {
                            let color = if t.return_pct > 0.0 { theme::ZONE_OPTIMAL } else { theme::ZONE_MISALIGNED };
                            text(format!(
                                "  {} @ ${:.2}  ->  {} @ ${:.2}  ({:+.1}%)",
                                t.buy_date, t.buy_price, t.sell_date, t.sell_price, t.return_pct
                            )).size(theme::TEXT_SM).color(color).into()
                        }).collect();

                        column![
                            text(format!("Backtest: {} ({} days)", bt.ticker, bt.days_tested)).size(theme::TEXT_MD),
                            config_row,
                            horizontal_rule(1),
                            metrics,
                            horizontal_rule(1),
                            text("Recent Trades (last 10)").size(theme::TEXT_SM),
                            Column::with_children(trade_rows).spacing(1),
                        ].spacing(6).into()
                    } else {
                        column![
                            text("Astro Backtest").size(theme::TEXT_MD),
                            config_row,
                            text("Press 'Run Backtest' to test the astro signal for this ticker.").size(theme::TEXT_SM),
                        ].spacing(6).into()
                    }
                };

                // ── Strategy Builder Section ──────────────────
                let strategy_section: Element<'_, Message> = {
                    use crate::strategy::Condition;

                    let buy_logic_btn = button(
                        text(self.strategy.buy_logic.label()).size(theme::TEXT_SM)
                    ).on_press(Message::StrategyToggleBuyLogic);

                    let buy_conds: Vec<Element<Message>> = self.strategy.buy_conditions.iter().enumerate().map(|(i, c)| {
                        row![
                            text(c.label()).size(theme::TEXT_SM),
                            button(text("✕").size(theme::TEXT_SM)).on_press(Message::StrategyRemoveBuyCond(i)),
                        ].spacing(4).into()
                    }).collect();

                    let sell_logic_btn = button(
                        text(self.strategy.sell_logic.label()).size(theme::TEXT_SM)
                    ).on_press(Message::StrategyToggleSellLogic);

                    let sell_conds: Vec<Element<Message>> = self.strategy.sell_conditions.iter().enumerate().map(|(i, c)| {
                        row![
                            text(c.label()).size(theme::TEXT_SM),
                            button(text("✕").size(theme::TEXT_SM)).on_press(Message::StrategyRemoveSellCond(i)),
                        ].spacing(4).into()
                    }).collect();

                    // Quick-add buttons for common conditions
                    let buy_add = row![
                        button(text("+Astro>70").size(theme::TEXT_SM)).on_press(Message::StrategyAddBuyCond(Condition::AstroAbove(70.0))),
                        button(text("+RSI<70").size(theme::TEXT_SM)).on_press(Message::StrategyAddBuyCond(Condition::RsiBelow(70.0))),
                        button(text("+Astro>80").size(theme::TEXT_SM)).on_press(Message::StrategyAddBuyCond(Condition::AstroAbove(80.0))),
                        button(text("+P>SMA50").size(theme::TEXT_SM)).on_press(Message::StrategyAddBuyCond(Condition::PriceAboveSma50)),
                    ].spacing(4);

                    let sell_add = row![
                        button(text("+Astro<30").size(theme::TEXT_SM)).on_press(Message::StrategyAddSellCond(Condition::AstroBelow(30.0))),
                        button(text("+RSI>80").size(theme::TEXT_SM)).on_press(Message::StrategyAddSellCond(Condition::RsiAbove(80.0))),
                        button(text("+Astro<20").size(theme::TEXT_SM)).on_press(Message::StrategyAddSellCond(Condition::AstroBelow(20.0))),
                        button(text("+P<SMA50").size(theme::TEXT_SM)).on_press(Message::StrategyAddSellCond(Condition::PriceBelowSma50)),
                    ].spacing(4);

                    let mut strat_col = column![
                        text("Strategy Builder").size(theme::TEXT_MD),
                        row![text("BUY when").size(theme::TEXT_SM), buy_logic_btn].spacing(6).align_y(Alignment::Center),
                        Column::with_children(buy_conds).spacing(2),
                        buy_add,
                        row![text("SELL when").size(theme::TEXT_SM), sell_logic_btn].spacing(6).align_y(Alignment::Center),
                        Column::with_children(sell_conds).spacing(2),
                        sell_add,
                        button(text("Run Strategy Backtest").size(theme::TEXT_SM)).on_press(Message::RunStrategy),
                    ].spacing(6);

                    if let Some(ref sr) = self.strategy_result {
                        let color = if sr.total_return_pct > sr.buy_hold_return_pct {
                            theme::ZONE_OPTIMAL
                        } else {
                            theme::ZONE_MISALIGNED
                        };
                        strat_col = strat_col.push(horizontal_rule(1));
                        strat_col = strat_col.push(
                            row![
                                text(format!("Strategy: {:.1}%", sr.total_return_pct)).size(theme::TEXT_BASE).color(color),
                                text(format!("vs B&H: {:.1}%", sr.buy_hold_return_pct)).size(theme::TEXT_BASE),
                                text(format!("Trades: {}", sr.num_trades)).size(theme::TEXT_SM),
                                text(format!("Win: {:.0}%", sr.win_rate_pct)).size(theme::TEXT_SM),
                            ].spacing(12)
                        );
                    }

                    strat_col.into()
                };

                // ── Astro Calendar ────────────────────────────
                let calendar_section: Element<'_, Message> = {
                    let nav = row![
                        button(text("◀").size(theme::TEXT_SM)).on_press(Message::CalendarPrevMonth),
                        text("Astro Calendar").size(theme::TEXT_MD),
                        button(text("▶").size(theme::TEXT_SM)).on_press(Message::CalendarNextMonth),
                    ].spacing(8).align_y(Alignment::Center);

                    let legend = row![
                        text("■").size(theme::TEXT_SM).color(iced::Color::from_rgb(0.3, 0.8, 0.4)),
                        text("Favorable (>50)").size(theme::TEXT_XS),
                        text("■").size(theme::TEXT_SM).color(iced::Color::from_rgb(0.5, 0.7, 0.2)),
                        text("Neutral (~50)").size(theme::TEXT_XS),
                        text("■").size(theme::TEXT_SM).color(iced::Color::from_rgb(0.8, 0.3, 0.3)),
                        text("Unfavorable (<50)").size(theme::TEXT_XS),
                    ].spacing(6).align_y(Alignment::Center);

                    column![
                        nav,
                        Canvas::new(AstroCalendar {
                            year: self.calendar_year,
                            month: self.calendar_month,
                            days: self.calendar_days.clone(),
                        })
                        .width(Length::Fill)
                        .height(Length::Fixed(180.0)),
                        legend,
                    ].spacing(6).into()
                };

                // Astrology tab: natal wheel + transits + calendar + backtest + strategy
                column![
                    astrology_section,
                    horizontal_rule(1),
                    container(calendar_section).padding([10, 14]),
                    horizontal_rule(1),
                    container(backtest_section).padding([10, 14]),
                    horizontal_rule(1),
                    container(strategy_section).padding([10, 14]),
                ].spacing(10).into()
            }
            Tab::Overview => {
                // Overview tab: two-column layout
                // Left (3/5): chart + sparkline + indicators + macro
                // Right (2/5): signals + watchlist
                // Pattern recognition
                let chart_data_chrono: Vec<f32> = self.rows.iter().rev()
                    .map(|r| r.close.to_string().parse::<f32>().unwrap_or(0.0))
                    .collect();
                let patterns_section: Element<'_, Message> = if let Some(ref ind) = self.indicators {
                    let detected = patterns::detect_patterns(&chart_data_chrono, ind);
                    if detected.is_empty() {
                        text("Patterns: none detected").size(theme::TEXT_SM).into()
                    } else {
                        let items: Vec<Element<Message>> = detected.iter().map(|p| {
                            let color = if p.is_bullish() { theme::ZONE_OPTIMAL } else { theme::ZONE_MISALIGNED };
                            let icon = if p.is_bullish() { "▲" } else { "▼" };
                            text(format!("  {icon} {}", p.label())).size(theme::TEXT_SM).color(color).into()
                        }).collect();
                        column![
                            text("Technical Patterns").size(theme::TEXT_BASE),
                            Column::with_children(items).spacing(2),
                        ].spacing(3).into()
                    }
                } else {
                    text("Patterns: —").size(theme::TEXT_SM).into()
                };

                let left_col: Column<Message> = column![
                    timeframe_bar,
                    chart,
                    sparkline_strip,
                    indicator_row,
                    patterns_section,
                    macro_strip,
                ].spacing(8);

                let right_col: Column<Message> = column![
                    container(signal_section).padding([10, 14]),
                    horizontal_rule(1),
                    container(watchlist_section).padding([10, 14]),
                ].spacing(8);

                let two_col = row![
                    container(left_col).width(Length::FillPortion(3)),
                    container(right_col).width(Length::FillPortion(2)),
                ].spacing(12);

                // Polymarket prediction markets
                let polymarket_section: Element<'_, Message> = if self.polymarket.is_empty() {
                    column![
                        text("Prediction Markets (Polymarket)").size(theme::TEXT_MD),
                        text("No prediction market data yet. Run the scraper to fetch top markets.").size(theme::TEXT_SM),
                    ].spacing(4).into()
                } else {
                    let pm_items: Vec<Element<Message>> = self.polymarket.iter().take(8).map(|m| {
                        let yes_pct = m.outcome_yes.as_ref()
                            .map(|d| format!("{:.0}%", d.to_string().parse::<f64>().unwrap_or(0.0) * 100.0))
                            .unwrap_or_else(|| "—".into());
                        let vol = m.volume.as_ref()
                            .map(|d| {
                                let v = d.to_string().parse::<f64>().unwrap_or(0.0);
                                if v >= 1_000_000.0 { format!("${:.1}M", v / 1_000_000.0) }
                                else if v >= 1_000.0 { format!("${:.0}K", v / 1_000.0) }
                                else { format!("${v:.0}") }
                            })
                            .unwrap_or_else(|| "—".into());
                        let cat = m.category.as_deref().unwrap_or("—");
                        row![
                            text(yes_pct).size(theme::TEXT_BASE).width(Length::Fixed(48.0)),
                            text(cat.to_string()).size(theme::TEXT_XS).width(Length::Fixed(70.0)),
                            text(m.question.clone()).size(theme::TEXT_SM).width(Length::Fill),
                            text(vol).size(theme::TEXT_XS).width(Length::Fixed(70.0)),
                        ].spacing(6).align_y(Alignment::Center).into()
                    }).collect();
                    column![
                        text("Prediction Markets (Polymarket)").size(theme::TEXT_MD),
                        scrollable(Column::with_children(pm_items).spacing(3)).height(Length::Fixed(140.0)),
                    ].spacing(4).into()
                };

                column![
                    gauges_row,
                    horizontal_rule(1),
                    two_col,
                    horizontal_rule(1),
                    polymarket_section,
                ].spacing(10).into()
            }
            Tab::Universe => {
                // Universe Explorer — full scored universe with filters + pagination
                let page_size = 50usize;
                let total = self.universe_total;
                let max_page = if total == 0 { 0 } else { ((total as usize).saturating_sub(1)) / page_size };
                let page_label = format!(
                    "Page {} of {} ({total} tickers)",
                    self.universe_page + 1,
                    max_page + 1,
                );

                // Zone filter buttons
                let zone_options: Vec<(&str, Option<String>)> = vec![
                    ("All", None),
                    ("Optimal", Some("Optimal".into())),
                    ("Favorable", Some("Favorable".into())),
                    ("Neutral", Some("Neutral".into())),
                    ("Unfavorable", Some("Unfavorable".into())),
                    ("Misaligned", Some("Misaligned".into())),
                ];
                let zone_bar: Row<Message> = zone_options.into_iter().fold(
                    row![text("Zone:").size(theme::TEXT_SM)].spacing(4).align_y(Alignment::Center),
                    |r, (label, val)| {
                        let is_active = self.universe_filter_zone == val;
                        let display = if is_active { format!("[{label}]") } else { label.to_string() };
                        r.push(
                            button(text(display).size(theme::TEXT_SM))
                                .on_press(Message::UniverseFilterZone(val))
                        )
                    },
                );

                // Sector filter dropdown (as buttons, "All" + each sector)
                let sector_bar: Row<Message> = {
                    let mut r = row![text("Sector:").size(theme::TEXT_SM)].spacing(4).align_y(Alignment::Center);
                    let is_all = self.universe_filter_sector.is_none();
                    let all_label = if is_all { "[All]".to_string() } else { "All".to_string() };
                    r = r.push(
                        button(text(all_label).size(theme::TEXT_XS))
                            .on_press(Message::UniverseFilterSector(None))
                    );
                    for sector in &self.universe_sectors {
                        let is_active = self.universe_filter_sector.as_deref() == Some(sector.as_str());
                        let label = if is_active { format!("[{sector}]") } else { sector.clone() };
                        let val = Some(sector.clone());
                        r = r.push(
                            button(text(label).size(theme::TEXT_XS))
                                .on_press(Message::UniverseFilterSector(val))
                        );
                    }
                    r
                };

                // Pagination controls
                let pagination = row![
                    button(text("◀ Prev").size(theme::TEXT_SM)).on_press(Message::UniversePrevPage),
                    text(page_label).size(theme::TEXT_SM),
                    button(text("Next ▶").size(theme::TEXT_SM)).on_press(Message::UniverseNextPage),
                ].spacing(8).align_y(Alignment::Center);

                // Universe table
                let universe_table: Element<'_, Message> = if self.universe_rows.is_empty() {
                    text("No scored tickers yet. Run the scraper to compute astro scores.").size(theme::TEXT_BASE).into()
                } else {
                    let hdr = row![
                        text("#").size(theme::TEXT_SM).width(Length::Fixed(30.0)),
                        text("Ticker").size(theme::TEXT_SM).width(Length::Fixed(64.0)),
                        text("Company").size(theme::TEXT_SM).width(Length::FillPortion(3)),
                        text("Sector").size(theme::TEXT_SM).width(Length::FillPortion(2)),
                        text("Astro").size(theme::TEXT_SM).width(Length::Fixed(56.0)),
                        text("Score").size(theme::TEXT_SM).width(Length::Fixed(56.0)),
                        text("Zone").size(theme::TEXT_SM).width(Length::Fixed(90.0)),
                        text("Fin").size(theme::TEXT_SM).width(Length::Fixed(44.0)),
                        text("Macro").size(theme::TEXT_SM).width(Length::Fixed(44.0)),
                        text("Short").size(theme::TEXT_SM).width(Length::Fixed(44.0)),
                        text("Conc").size(theme::TEXT_SM).width(Length::Fixed(50.0)),
                    ].spacing(6);

                    let offset = self.universe_page * page_size;
                    let rows: Vec<Element<Message>> = self.universe_rows.iter().enumerate().map(|(i, u)| {
                        let zone_color = match u.label.as_str() {
                            "Optimal"     => theme::ZONE_OPTIMAL,
                            "Favorable"   => theme::ZONE_FAVORABLE,
                            "Neutral"     => theme::ZONE_NEUTRAL,
                            "Unfavorable" => theme::ZONE_UNFAVORABLE,
                            _             => theme::ZONE_MISALIGNED,
                        };
                        let astro_str = u.astro_score.map(|s| format!("{s:.0}")).unwrap_or_else(|| "---".into());
                        let fin_str = u.fin_score.map(|s| format!("{s:.0}")).unwrap_or_else(|| "---".into());
                        let macro_str = u.macro_score.map(|s| format!("{s:.0}")).unwrap_or_else(|| "---".into());
                        let short_str = u.short_score.map(|s| format!("{s:.0}")).unwrap_or_else(|| "---".into());
                        let conc = u.concordance.as_deref().unwrap_or("---");
                        let company = u.company_name.as_deref().unwrap_or("—");
                        let sector = u.sector.as_deref().unwrap_or("—");

                        let ticker_btn = button(text(u.ticker.clone()).size(theme::TEXT_SM))
                            .on_press(Message::TickerSelected(u.ticker.clone()));

                        row![
                            text(format!("{}", offset + i + 1)).size(theme::TEXT_SM).width(Length::Fixed(30.0)),
                            ticker_btn,
                            text(company.to_string()).size(theme::TEXT_XS).width(Length::FillPortion(3)),
                            text(sector.to_string()).size(theme::TEXT_XS).width(Length::FillPortion(2)),
                            text(astro_str).size(theme::TEXT_SM).width(Length::Fixed(56.0)),
                            text(format!("{:.0}", u.score)).size(theme::TEXT_SM).width(Length::Fixed(56.0)),
                            text(u.label.clone()).size(theme::TEXT_SM).color(zone_color).width(Length::Fixed(90.0)),
                            text(fin_str).size(theme::TEXT_SM).width(Length::Fixed(44.0)),
                            text(macro_str).size(theme::TEXT_SM).width(Length::Fixed(44.0)),
                            text(short_str).size(theme::TEXT_SM).width(Length::Fixed(44.0)),
                            text(conc.to_string()).size(theme::TEXT_XS).width(Length::Fixed(50.0)),
                        ].spacing(6).align_y(Alignment::Center).into()
                    }).collect();

                    column![
                        hdr,
                        horizontal_rule(1),
                        scrollable(Column::with_children(rows).spacing(3))
                            .height(Length::Fixed(400.0)),
                    ].spacing(4).into()
                };

                // Sector heat map
                let sector_heatmap = Canvas::new(SectorHeatMap {
                    sectors: self.sector_summaries.clone(),
                })
                .width(Length::Fill)
                .height(Length::Fixed(70.0));

                column![
                    text(format!("Universe Explorer — {} scored tickers", total)).size(theme::TEXT_LG),
                    horizontal_rule(1),
                    text("Sector Heat Map (by avg astro score)").size(theme::TEXT_SM),
                    sector_heatmap,
                    horizontal_rule(1),
                    scrollable(sector_bar).direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default())),
                    zone_bar,
                    pagination,
                    universe_table,
                    horizontal_rule(1),
                    container(alerts_section).padding([10, 14]),
                ].spacing(8).into()
            }
            Tab::Fundamentals => {
                // Fundamentals tab: valuation metrics + earnings + price table
                let fundamentals_section: Element<'_, Message> = if let Some(ref f) = self.fundamentals {
                    let fr = |v: Option<f64>| -> String {
                        v.map(|x| format!("{x:.2}")).unwrap_or_else(|| "---".to_string())
                    };
                    let fm = |v: Option<i64>| -> String {
                        v.map(format_market_value_i64).unwrap_or_else(|| "---".to_string())
                    };
                    let fp = |v: Option<f64>| -> String {
                        v.map(|x| format!("{:.1}%", x * 100.0)).unwrap_or_else(|| "---".to_string())
                    };

                    // Left column: Valuation
                    let val_col: Column<Message> = column![
                        text("Valuation").size(theme::TEXT_MD),
                        row![text("Market Cap").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fm(f.market_cap)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("P/E Ratio").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fr(f.pe_ratio)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("P/B Ratio").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fr(f.pb_ratio)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("P/S Ratio").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fr(f.ps_ratio)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("EV/EBITDA").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fr(f.ev_ebitda)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("PEG Ratio").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fr(f.peg_ratio)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("P/FCF").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fr(f.price_to_fcf)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("EPS").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fr(f.eps)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("Div Yield").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fp(f.dividend_yield)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                    ].spacing(4);

                    // Right column: Profitability + Balance Sheet
                    let prof_col: Column<Message> = column![
                        text("Profitability & Health").size(theme::TEXT_MD),
                        row![text("ROE").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fp(f.roe)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("ROA").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fp(f.roa)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("Net Margin").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fp(f.net_margin)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("Op Margin").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fp(f.operating_margin)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("Debt/Equity").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fr(f.debt_equity)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("Current Ratio").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fr(f.current_ratio)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("Revenue").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fm(f.revenue)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("Net Income").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fm(f.net_income)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                        row![text("FCF").size(theme::TEXT_SM).width(Length::FillPortion(2)), text(fm(f.fcf)).size(theme::TEXT_SM).width(Length::FillPortion(3))].spacing(4),
                    ].spacing(4);

                    let grid = row![val_col, prof_col].spacing(20);
                    let header_text = format!(
                        "Fundamentals — {} (as of {})",
                        f.ticker, f.fetch_date
                    );
                    column![
                        text(header_text).size(theme::TEXT_LG),
                        grid,
                    ].spacing(8).into()
                } else {
                    column![
                        text("Fundamental Metrics").size(theme::TEXT_LG),
                        text("No fundamental data yet. Run the scraper with an FMP API key to fetch valuation ratios, profitability metrics, and balance sheet data.")
                            .size(theme::TEXT_SM),
                        text("Source: Financial Modeling Prep /v3/key-metrics-ttm + /v3/ratios-ttm")
                            .size(theme::TEXT_XS),
                    ].spacing(6).into()
                };

                // DCF Calculator section
                let dcf_section: Element<'_, Message> = {
                    let input_row = row![
                        column![
                            text("Growth %").size(theme::TEXT_XS),
                            text_input("10", &self.dcf_growth_rate)
                                .on_input(Message::DcfGrowthRateInput)
                                .on_submit(Message::DcfCompute)
                                .size(theme::TEXT_SM)
                                .width(Length::Fixed(60.0)),
                        ].spacing(2),
                        column![
                            text("Years").size(theme::TEXT_XS),
                            text_input("5", &self.dcf_growth_years)
                                .on_input(Message::DcfGrowthYearsInput)
                                .on_submit(Message::DcfCompute)
                                .size(theme::TEXT_SM)
                                .width(Length::Fixed(50.0)),
                        ].spacing(2),
                        column![
                            text("Terminal %").size(theme::TEXT_XS),
                            text_input("2.5", &self.dcf_terminal_growth)
                                .on_input(Message::DcfTerminalGrowthInput)
                                .on_submit(Message::DcfCompute)
                                .size(theme::TEXT_SM)
                                .width(Length::Fixed(60.0)),
                        ].spacing(2),
                        column![
                            text("WACC %").size(theme::TEXT_XS),
                            text_input("10", &self.dcf_discount_rate)
                                .on_input(Message::DcfDiscountRateInput)
                                .on_submit(Message::DcfCompute)
                                .size(theme::TEXT_SM)
                                .width(Length::Fixed(60.0)),
                        ].spacing(2),
                        button(text("Compute").size(theme::TEXT_SM))
                            .on_press(Message::DcfCompute),
                    ].spacing(8).align_y(Alignment::End);

                    if let Some(ref dcf) = self.dcf_result {
                        let margin_color = if dcf.margin_of_safety_pct > 0.0 {
                            theme::ZONE_OPTIMAL
                        } else {
                            theme::ZONE_MISALIGNED
                        };
                        let margin_label = if dcf.margin_of_safety_pct > 0.0 {
                            "UNDERVALUED"
                        } else {
                            "OVERVALUED"
                        };
                        let result_row = row![
                            column![
                                text(format!("Intrinsic Value: ${:.2}", dcf.intrinsic_per_share)).size(theme::TEXT_MD),
                                text(format!("Enterprise Value: {}", format_market_value_i64(dcf.enterprise_value as i64))).size(theme::TEXT_SM),
                            ].spacing(2),
                            column![
                                text(format!("Margin of Safety: {:.1}%", dcf.margin_of_safety_pct))
                                    .size(theme::TEXT_MD)
                                    .color(margin_color),
                                text(margin_label).size(theme::TEXT_SM).color(margin_color),
                            ].spacing(2),
                        ].spacing(20);

                        column![
                            text("DCF Intrinsic Value Calculator").size(theme::TEXT_MD),
                            input_row,
                            result_row,
                        ].spacing(6).into()
                    } else {
                        column![
                            text("DCF Intrinsic Value Calculator").size(theme::TEXT_MD),
                            input_row,
                            text("Requires FCF + shares outstanding data. Run scraper with FMP key.").size(theme::TEXT_XS),
                        ].spacing(6).into()
                    }
                };

                // Agent Personas section
                let agent_buttons: Row<Message> = AgentPersona::all().iter().fold(
                    row![text("Ask the Council:").size(theme::TEXT_BASE)].spacing(6).align_y(Alignment::Center),
                    |r, &persona| {
                        let label = if self.active_agent == Some(persona) {
                            format!("[{}]", persona.short_name())
                        } else {
                            persona.short_name().to_string()
                        };
                        r.push(
                            button(text(label).size(theme::TEXT_BASE))
                                .on_press(Message::AgentSelected(persona))
                        )
                    },
                );

                let agent_section: Element<'_, Message> = if let Some(ref analysis) = self.agent_analysis {
                    let verdict_color = match analysis.verdict {
                        AgentVerdict::StrongBuy | AgentVerdict::Buy => theme::ZONE_OPTIMAL,
                        AgentVerdict::Hold | AgentVerdict::InsufficientData => theme::ZONE_NEUTRAL,
                        AgentVerdict::Sell | AgentVerdict::StrongSell => theme::ZONE_MISALIGNED,
                    };

                    let metrics_rows: Vec<Element<Message>> = analysis.key_metrics.iter().map(|(metric, value, assessment)| {
                        row![
                            text(metric.clone()).size(theme::TEXT_SM).width(Length::Fixed(110.0)),
                            text(value.clone()).size(theme::TEXT_SM).width(Length::Fixed(80.0)),
                            text(assessment.clone()).size(theme::TEXT_XS).width(Length::Fill),
                        ].spacing(8).into()
                    }).collect();

                    column![
                        text(format!("{} — {}", analysis.persona.name(), analysis.persona.philosophy())).size(theme::TEXT_SM),
                        horizontal_rule(1),
                        text(analysis.headline.clone()).size(theme::TEXT_BASE),
                        text(format!("Verdict: {}", analysis.verdict.label()))
                            .size(theme::TEXT_MD)
                            .color(verdict_color),
                        text(analysis.analysis.clone()).size(theme::TEXT_SM),
                        horizontal_rule(1),
                        Column::with_children(metrics_rows).spacing(3),
                        horizontal_rule(1),
                        text(format!("On the stars: {}", analysis.astro_take)).size(theme::TEXT_XS),
                    ].spacing(6).into()
                } else {
                    column![
                        text("Select an agent to get their investment analysis:").size(theme::TEXT_SM),
                        text("Buffett — moat, FCF, owner earnings").size(theme::TEXT_XS),
                        text("Graham — margin of safety, deep value").size(theme::TEXT_XS),
                        text("Lynch — PEG ratio, know what you own").size(theme::TEXT_XS),
                        text("Munger — quality, mental models, durability").size(theme::TEXT_XS),
                    ].spacing(3).into()
                };

                column![
                    fundamentals_section,
                    horizontal_rule(1),
                    dcf_section,
                    horizontal_rule(1),
                    agent_buttons,
                    agent_section,
                    horizontal_rule(1),
                    // Comparative Analysis section
                    {
                        let compare_input_row = row![
                            text("Compare:").size(theme::TEXT_BASE),
                            text_input("Add ticker (max 4)…", &self.compare_input)
                                .on_input(Message::CompareInput)
                                .on_submit(Message::CompareAdd)
                                .size(theme::TEXT_SM)
                                .width(Length::Fixed(140.0)),
                            button(text("Add").size(theme::TEXT_SM)).on_press(Message::CompareAdd),
                        ].spacing(6).align_y(Alignment::Center);

                        let chip_row: Row<Message> = self.compare_tickers.iter().fold(
                            row![].spacing(6),
                            |r, t| r.push(
                                button(text(format!("{t} ✕")).size(theme::TEXT_SM))
                                    .on_press(Message::CompareRemove(t.clone()))
                            ),
                        );

                        let compare_table: Element<'_, Message> = if self.compare_data.is_empty() {
                            text("Add tickers above to compare side by side.").size(theme::TEXT_SM).into()
                        } else {
                            let hdr = row![
                                text("Metric").size(theme::TEXT_SM).width(Length::Fixed(100.0)),
                            ].spacing(8);
                            let hdr = self.compare_data.iter().fold(hdr, |r, d| {
                                r.push(text(d.ticker.clone()).size(theme::TEXT_SM).width(Length::FillPortion(1)))
                            });

                            let metric_row = |label: &str, f: &dyn Fn(&crate::db::CompareRow) -> String| -> Element<'_, Message> {
                                let r = row![
                                    text(label.to_string()).size(theme::TEXT_XS).width(Length::Fixed(100.0)),
                                ];
                                let r = self.compare_data.iter().fold(r, |r, d| {
                                    r.push(text(f(d)).size(theme::TEXT_XS).width(Length::FillPortion(1)))
                                });
                                r.spacing(8).into()
                            };

                            let fr = |v: Option<f64>| v.map(|x| format!("{x:.2}")).unwrap_or_else(|| "---".into());
                            let fp = |v: Option<f64>| v.map(|x| format!("{:.1}%", x * 100.0)).unwrap_or_else(|| "---".into());
                            let fm = |v: Option<i64>| v.map(|x| format_market_value_i64(x)).unwrap_or_else(|| "---".into());

                            column![
                                hdr,
                                horizontal_rule(1),
                                metric_row("P/E",         &|d| fr(d.pe_ratio)),
                                metric_row("P/B",         &|d| fr(d.pb_ratio)),
                                metric_row("P/S",         &|d| fr(d.ps_ratio)),
                                metric_row("EV/EBITDA",   &|d| fr(d.ev_ebitda)),
                                metric_row("PEG",         &|d| fr(d.peg_ratio)),
                                metric_row("ROE",         &|d| fp(d.roe)),
                                metric_row("Net Margin",  &|d| fp(d.net_margin)),
                                metric_row("Debt/Equity", &|d| fr(d.debt_equity)),
                                metric_row("FCF",         &|d| fm(d.fcf)),
                                metric_row("Market Cap",  &|d| fm(d.market_cap)),
                                metric_row("Astro Score", &|d| d.astro_score.map(|s| format!("{s:.0}")).unwrap_or_else(|| "---".into())),
                                metric_row("Astro Zone",  &|d| d.astro_label.clone().unwrap_or_else(|| "---".into())),
                            ].spacing(3).into()
                        };

                        column![
                            text("Comparative Analysis").size(theme::TEXT_MD),
                            compare_input_row,
                            chip_row,
                            compare_table,
                        ].spacing(6)
                    },
                    horizontal_rule(1),
                    earnings_section,
                    horizontal_rule(1),
                    price_header,
                    data_rows,
                ].spacing(10).into()
            }
            Tab::Research => {
                // RSS market news section (global, not per-ticker)
                let rss_section = if self.rss_articles.is_empty() {
                    column![
                        text("Market News (RSS)").size(theme::TEXT_MD),
                        text("No RSS articles loaded yet. Run the scraper to fetch headlines from 25+ sources.").size(theme::TEXT_SM),
                    ].spacing(4)
                } else {
                    let rss_items: Vec<Element<Message>> = self.rss_articles.iter().map(|a| {
                        let date = a.published_at.format("%b %d").to_string();
                        let link = a.link.clone();
                        row![
                            text(format!("[{date}]")).size(theme::TEXT_BASE).width(Length::Fixed(52.0)),
                            text(&a.feed_source).size(theme::TEXT_BASE).width(Length::Fixed(90.0)),
                            text(&a.category).size(theme::TEXT_XS).width(Length::Fixed(60.0)),
                            text(&a.headline).size(theme::TEXT_BASE).width(Length::Fill),
                            button(text("Open").size(theme::TEXT_SM)).on_press(Message::OpenUrl(link)),
                        ].spacing(6).align_y(Alignment::Center).into()
                    }).collect();
                    column![
                        text("Market News (RSS — 25 sources)").size(theme::TEXT_MD),
                        scrollable(Column::with_children(rss_items).spacing(4)).height(Length::Fixed(180.0)),
                    ].spacing(4)
                };

                // Research tab: news + filings + RSS + insider trades + holdings
                column![
                    news_filings_row,
                    horizontal_rule(1),
                    rss_section,
                    horizontal_rule(1),
                    insider_section,
                    horizontal_rule(1),
                    holdings_section,
                ].spacing(10).into()
            }
            Tab::Portfolio => {
                // ── Named Watchlists Manager ─────────────────────
                let wl_dropdown: Row<Message> = self.named_watchlists.iter().fold(
                    row![].spacing(4),
                    |r, wl| {
                        let is_active = self.active_watchlist_id == Some(wl.id);
                        let label = if is_active {
                            format!("▶ {}", wl.name)
                        } else {
                            wl.name.clone()
                        };
                        r.push(button(text(label).size(theme::TEXT_SM)).on_press(Message::SelectNamedWatchlist(wl.id)))
                    },
                );

                let wl_create_row = row![
                    text_input("New watchlist name…", &self.new_watchlist_name)
                        .on_input(Message::NewWatchlistNameInput)
                        .on_submit(Message::CreateWatchlist)
                        .width(Length::Fixed(200.0))
                        .size(theme::TEXT_SM),
                    button(text("+").size(theme::TEXT_SM)).on_press(Message::CreateWatchlist),
                ].spacing(6);

                let wl_tickers_section = if self.watchlist_tickers_list.is_empty() {
                    column![text("No tickers in this watchlist.").size(theme::TEXT_SM)].spacing(4)
                } else {
                    let ticker_chips: Vec<Element<Message>> = self.watchlist_tickers_list.iter().map(|t| {
                        row![
                            text(t).size(theme::TEXT_BASE),
                            button(text("✕").size(theme::TEXT_SM)).on_press(Message::WatchlistRemoveTicker(t.clone())),
                        ].spacing(4).into()
                    }).collect();
                    Column::with_children(ticker_chips).spacing(2)
                };

                let wl_add_row = row![
                    text_input("Add ticker…", &self.watchlist_add_ticker)
                        .on_input(Message::WatchlistAddTickerInput)
                        .on_submit(Message::WatchlistAddTicker)
                        .width(Length::Fixed(120.0))
                        .size(theme::TEXT_SM),
                    button(text("Add").size(theme::TEXT_SM)).on_press(Message::WatchlistAddTicker),
                ].spacing(6);

                let wl_actions = row![
                    button(text("Delete Watchlist").size(theme::TEXT_SM)).on_press(Message::DeleteActiveWatchlist),
                    button(text("Export CSV").size(theme::TEXT_SM)).on_press(Message::ExportCsv),
                ].spacing(8);

                let watchlist_mgr = column![
                    text("Watchlists").size(theme::TEXT_MD),
                    wl_dropdown,
                    wl_create_row,
                    horizontal_rule(1),
                    wl_tickers_section,
                    wl_add_row,
                    horizontal_rule(1),
                    wl_actions,
                ].spacing(6);

                // ── Transaction Log ───────────────────────────
                let tx_section: Element<'_, Message> = {
                    let action_label = &self.tx_action;
                    let input_row = row![
                        button(text(action_label).size(theme::TEXT_SM)).on_press(Message::TxToggleAction),
                        text_input("Ticker", &self.tx_ticker_input)
                            .on_input(Message::TxTickerInput)
                            .width(Length::Fixed(80.0))
                            .size(theme::TEXT_SM),
                        text_input("Shares", &self.tx_shares_input)
                            .on_input(Message::TxSharesInput)
                            .width(Length::Fixed(70.0))
                            .size(theme::TEXT_SM),
                        text_input("Price", &self.tx_price_input)
                            .on_input(Message::TxPriceInput)
                            .on_submit(Message::TxSubmit)
                            .width(Length::Fixed(80.0))
                            .size(theme::TEXT_SM),
                        button(text("Add").size(theme::TEXT_SM)).on_press(Message::TxSubmit),
                    ].spacing(6).align_y(Alignment::Center);

                    let tx_rows: Vec<Element<Message>> = self.transactions.iter().take(20).map(|tx| {
                        let color = if tx.action == "BUY" { theme::ZONE_OPTIMAL } else { theme::ZONE_MISALIGNED };
                        let total = tx.shares * tx.price;
                        row![
                            text(&tx.action).size(theme::TEXT_SM).color(color).width(Length::Fixed(40.0)),
                            text(&tx.ticker).size(theme::TEXT_SM).width(Length::Fixed(56.0)),
                            text(format!("{:.1}", tx.shares)).size(theme::TEXT_SM).width(Length::Fixed(56.0)),
                            text(format!("${:.2}", tx.price)).size(theme::TEXT_SM).width(Length::Fixed(72.0)),
                            text(format!("${total:.0}")).size(theme::TEXT_SM).width(Length::Fixed(72.0)),
                            text(tx.trade_date.to_string()).size(theme::TEXT_SM).width(Length::Fixed(80.0)),
                            button(text("✕").size(theme::TEXT_SM)).on_press(Message::TxDelete(tx.id)),
                        ].spacing(4).into()
                    }).collect();

                    column![
                        text("Transaction Log").size(theme::TEXT_MD),
                        input_row,
                        horizontal_rule(1),
                        Column::with_children(tx_rows).spacing(1),
                    ].spacing(6).into()
                };

                // Portfolio tab: portfolio + transactions + watchlists + macro
                column![
                    container(portfolio_section).padding([10, 14]),
                    horizontal_rule(1),
                    container(tx_section).padding([10, 14]),
                    horizontal_rule(1),
                    container(watchlist_mgr).padding([10, 14]),
                    horizontal_rule(1),
                    macro_strip,
                ].spacing(10).into()
            }
            Tab::Settings => {
                // Settings tab
                let theme_label = match self.theme_mode {
                    crate::theme::ThemeMode::Auto => "Auto",
                    crate::theme::ThemeMode::AlwaysLight => "Light",
                    crate::theme::ThemeMode::AlwaysDark => "Dark",
                };

                let theme_row = row![
                    text("Theme:").size(theme::TEXT_BASE),
                    button(text("Auto").size(theme::TEXT_SM)).on_press(Message::SaveSetting("theme_mode".to_string(), "Auto".to_string())),
                    button(text("Light").size(theme::TEXT_SM)).on_press(Message::SaveSetting("theme_mode".to_string(), "Light".to_string())),
                    button(text("Dark").size(theme::TEXT_SM)).on_press(Message::SaveSetting("theme_mode".to_string(), "Dark".to_string())),
                    text(format!("  (current: {theme_label})")).size(theme::TEXT_SM),
                ].spacing(8).align_y(Alignment::Center);

                let refresh_row = row![
                    text("Refresh interval (seconds):").size(theme::TEXT_BASE),
                    text_input("30", &self.settings_refresh_input)
                        .on_input(Message::SettingsRefreshInput)
                        .on_submit(Message::SaveSetting("refresh_interval_secs".to_string(), self.settings_refresh_input.clone()))
                        .width(Length::Fixed(60.0))
                        .size(theme::TEXT_SM),
                    button(text("Save").size(theme::TEXT_SM))
                        .on_press(Message::SaveSetting("refresh_interval_secs".to_string(), self.settings_refresh_input.clone())),
                ].spacing(8).align_y(Alignment::Center);

                let info_section = column![
                    text("Dashboard Info").size(theme::TEXT_MD),
                    horizontal_rule(1),
                    text(format!("Tickers loaded: {}", self.tickers.len())).size(theme::TEXT_SM),
                    text(format!("Universe size: {}", self.universe_total)).size(theme::TEXT_SM),
                    text(format!("Transactions: {}", self.transactions.len())).size(theme::TEXT_SM),
                    text(format!("Named watchlists: {}", self.named_watchlists.len())).size(theme::TEXT_SM),
                    text(format!("Alerts: {} ({} unread)", self.alerts.len(), self.unread_alert_count)).size(theme::TEXT_SM),
                ].spacing(4);

                column![
                    text("Settings").size(theme::TEXT_LG),
                    horizontal_rule(1),
                    theme_row,
                    refresh_row,
                    horizontal_rule(1),
                    info_section,
                ].spacing(10).padding(14).into()
            }
        };

        let content = column![
            header,
            horizontal_rule(1),
            ticker_buttons,
            row![search_bar].spacing(16),
            autocomplete,
            recently_viewed_row,
            text(&self.status).size(theme::TEXT_BASE),
            row![
                button(refresh_label).on_press(Message::RefreshNow),
            ].spacing(8),
            horizontal_rule(1),
            tab_bar,
            horizontal_rule(1),
            tab_content,
        ]
        .spacing(10)
        .padding(20);

        container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
    }
}
