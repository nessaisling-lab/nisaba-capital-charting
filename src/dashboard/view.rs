use iced::widget::canvas::Canvas;
use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input, Column, Row};
use iced::{Alignment, Element, Length, Theme};

use crate::astrology::{build_transits_section, build_wheel_legend, NatalWheel};
use crate::charts::{LagrangeSparkline, PriceChart};
use crate::gauges::FearGreedGauge;
use crate::helpers::{describe_8k_items, format_market_value, format_market_value_i64, format_shares};
use crate::indicators::{compute_lagrange_score, compute_ticker_score, Indicators};
use crate::signals::generate_signal_bullets;
use crate::state::{Dashboard, Message};
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

        // Close prices in chronological order for chart
        let chart_data: Vec<f32> = self.rows.iter().rev()
            .map(|r| r.close.to_string().parse::<f32>().unwrap_or(0.0))
            .collect();

        let (sma20, sma50, bb_upper, bb_lower) = match &self.indicators {
            Some(ind) => (ind.sma20.clone(), ind.sma50.clone(), ind.bb_upper.clone(), ind.bb_lower.clone()),
            None => (vec![], vec![], vec![], vec![]),
        };

        let chart: Element<Message> = if self.rows.is_empty() {
            container(
                text(format!(
                    "{} — no price data yet.  Run the scraper to fetch OHLCV history.",
                    self.selected_ticker
                ))
                .size(theme::TEXT_BASE),
            )
            .center_x(Length::Fill)
            .padding([40, 20])
            .into()
        } else {
            Canvas::new(PriceChart {
                data: chart_data,
                ticker: self.selected_ticker.clone(),
                sma20,
                sma50,
                bb_upper,
                bb_lower,
                rows_chrono: self.rows.iter().rev().cloned().collect(),
            })
            .width(Length::Fill)
            .height(Length::Fixed(220.0))
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
                text("Earnings Calendar").size(theme::TEXT_MD),
                text("No earnings data — run the scraper to fetch from Finnhub").size(theme::TEXT_BASE),
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
                scrollable(Column::with_children(items).spacing(4)).height(Length::Fixed(110.0)),
            ].spacing(4)
        };

        // Institutional holdings (13F)
        let holdings_section = if self.holdings.is_empty() {
            column![
                text("Top Institutional Holders").size(theme::TEXT_MD),
                text("No 13F data — run the scraper to fetch from EDGAR").size(theme::TEXT_BASE),
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
                scrollable(Column::with_children(holding_rows).spacing(4)).height(Length::Fixed(100.0)),
            ].spacing(4)
        };

        // 8-K filings
        let filings_section = if self.filings_8k.is_empty() {
            column![
                text("Recent 8-K Filings").size(theme::TEXT_MD),
                text("No 8-K data — run the scraper to fetch from EDGAR").size(theme::TEXT_BASE),
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
                text("No news — run the scraper to fetch from Finnhub").size(theme::TEXT_BASE),
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
                text("No insider trade data — run the scraper to fetch from EDGAR").size(theme::TEXT_BASE),
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
        let theme_label   = if self.theme == Theme::Dark { "Light Mode" } else { "Dark Mode" };

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

        let header = row![
            text(format!("{} — Daily Price Data", self.selected_ticker)).size(theme::TEXT_2XL),
            iced::widget::Space::with_width(Length::Fill),
            button(theme_label).on_press(Message::ToggleTheme),
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
                    .height(Length::Fixed(200.0)),
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
        let macro_strip = row![
            text(format!("Fed Funds: {}%",   macro_find("FEDFUNDS"))).size(theme::TEXT_BASE),
            text(format!("CPI YoY: {}%",       macro_find("CPIAUCSL_YOY"))).size(theme::TEXT_BASE),
            text(format!("Unemploy: {}%",     macro_find("UNRATE"))).size(theme::TEXT_BASE),
            text(format!("10Y: {}%",          macro_find("GS10"))).size(theme::TEXT_BASE),
            text(format!("2Y: {}%",           macro_find("GS2"))).size(theme::TEXT_BASE),
            text(format!("Spread: {}%",       macro_find("T10Y2Y"))).size(theme::TEXT_BASE),
            text(format!("VIX: {}",           macro_find("VIXCLS"))).size(theme::TEXT_BASE),
            text(format!("WTI Oil: ${}",      macro_find("DCOILWTICO"))).size(theme::TEXT_BASE),
        ].spacing(20);

        // Astrology section: natal wheel + transits table side by side
        let moon_phase  = self.astro_score.as_ref().and_then(|s| s.moon_phase.as_deref());
        let moon_deg    = self.astro_score.as_ref().and_then(|s| s.moon_phase_deg);
        let mercury_rx  = self.astro_score.as_ref().and_then(|s| s.mercury_rx).unwrap_or(false);

        let astrology_section: Element<Message> = if self.natal_positions.is_empty() {
            column![
                text(format!("{} Astrology", self.selected_ticker)).size(theme::TEXT_LG),
                horizontal_rule(1),
                text("No birth chart yet for this ticker.").size(theme::TEXT_BASE),
                text("The scraper enriches ~50 new tickers per day via SEC EDGAR.").size(theme::TEXT_BASE),
                text("Run the scraper again tomorrow — once an IPO date is found the natal chart is computed automatically.").size(theme::TEXT_BASE),
            ]
            .spacing(6)
            .into()
        } else {
            let natal_wheel = Canvas::new(NatalWheel {
                natal:    self.natal_positions.clone(),
                transits: self.daily_transits.clone(),
            })
            .width(Length::Fixed(240.0))
            .height(Length::Fixed(240.0));

            let wheel_col = column![
                text(format!("{} Birth Chart", self.selected_ticker)).size(theme::TEXT_LG),
                natal_wheel,
                build_wheel_legend(),
            ].spacing(4).align_x(Alignment::Center);

            let transits_col = column![
                build_transits_section(&self.astro_aspects, moon_phase, moon_deg, mercury_rx),
            ].width(Length::Fill);

            row![wheel_col, transits_col]
                .spacing(20)
                .align_y(Alignment::Start)
                .into()
        };

        // News and 8-K side by side
        let news_filings_row = row![
            column![filings_section].width(Length::FillPortion(1)),
            column![news_section].width(Length::FillPortion(1)),
        ].spacing(20);

        // Portfolio panel
        let portfolio_section = if self.portfolio.is_empty() {
            column![
                text("Portfolio").size(theme::TEXT_MD),
                text("No positions — add rows to portfolio_positions table via portfolio_seed.sql").size(theme::TEXT_BASE),
            ].spacing(4)
        } else {
            let hdr = row![
                text("Ticker").size(theme::TEXT_BASE).width(Length::Fixed(64.0)),
                text("Shares").size(theme::TEXT_BASE).width(Length::Fixed(72.0)),
                text("Avg Cost").size(theme::TEXT_BASE).width(Length::Fixed(88.0)),
                text("Cost Basis").size(theme::TEXT_BASE).width(Length::Fill),
            ].spacing(8);

            let pos_rows: Vec<Element<Message>> = self.portfolio.iter().map(|p| {
                let cost_basis = p.shares * p.avg_cost;
                let notes = p.notes.as_deref().unwrap_or("");
                let label = if notes.is_empty() {
                    format!("${cost_basis:.0}")
                } else {
                    format!("${cost_basis:.0}  —  {notes}")
                };
                row![
                    text(&p.ticker).size(theme::TEXT_BASE).width(Length::Fixed(64.0)),
                    text(format!("{:.2}", p.shares)).size(theme::TEXT_BASE).width(Length::Fixed(72.0)),
                    text(format!("${:.2}", p.avg_cost)).size(theme::TEXT_BASE).width(Length::Fixed(88.0)),
                    text(label).size(theme::TEXT_BASE).width(Length::Fill),
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
                        .height(Length::Fixed(140.0)),
                ].spacing(4)
            }
        };

        let content = column![
            header,
            horizontal_rule(1),
            gauges_row,
            horizontal_rule(1),
            ticker_buttons,
            row![search_bar].spacing(16),
            autocomplete,
            recently_viewed_row,
            text(&self.status).size(theme::TEXT_BASE),
            button(refresh_label).on_press(Message::RefreshNow),
            chart,
            sparkline_strip,
            indicator_row,
            macro_strip,
            horizontal_rule(1),
            container(signal_section).padding([10, 14]),
            horizontal_rule(1),
            container(watchlist_section).padding([10, 14]),
            horizontal_rule(1),
            container(alerts_section).padding([10, 14]),
            horizontal_rule(1),
            astrology_section,
            horizontal_rule(1),
            earnings_section,
            horizontal_rule(1),
            holdings_section,
            horizontal_rule(1),
            container(portfolio_section).padding([10, 14]),
            horizontal_rule(1),
            news_filings_row,
            horizontal_rule(1),
            insider_section,
            horizontal_rule(1),
            price_header,
            data_rows,
        ]
        .spacing(10)
        .padding(20);

        container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
    }
}
