use iced::widget::canvas::Canvas;
use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, Column, Row};
use iced::{Alignment, Element, Length, Theme};

use crate::astrology::{build_transits_section, build_wheel_legend, NatalWheel};
use crate::charts::{LagrangeSparkline, PriceChart};
use crate::gauges::FearGreedGauge;
use crate::helpers::{describe_8k_items, format_market_value, format_market_value_i64, format_shares};
use crate::indicators::{compute_lagrange_score, compute_ticker_score, Indicators};
use crate::signals::generate_signal_bullets;
use crate::state::{Dashboard, Message};

impl Dashboard {
    pub fn view(&self) -> Element<'_, Message> {
        // Ticker selector buttons
        let ticker_buttons: Row<Message> = self.tickers.iter().fold(row![].spacing(6), |r, ticker| {
            let btn = button(text(ticker).size(13)).on_press(Message::TickerSelected(ticker.clone()));
            r.push(btn)
        });

        // Close prices in chronological order for chart
        let chart_data: Vec<f32> = self.rows.iter().rev()
            .map(|r| r.close.to_string().parse::<f32>().unwrap_or(0.0))
            .collect();

        let (sma20, sma50, bb_upper, bb_lower) = match &self.indicators {
            Some(ind) => (ind.sma20.clone(), ind.sma50.clone(), ind.bb_upper.clone(), ind.bb_lower.clone()),
            None => (vec![], vec![], vec![], vec![]),
        };

        let chart = Canvas::new(PriceChart {
            data: chart_data,
            ticker: self.selected_ticker.clone(),
            sma20,
            sma50,
            bb_upper,
            bb_lower,
            rows_chrono: self.rows.iter().rev().cloned().collect(),
        })
        .width(Length::Fill)
        .height(Length::Fixed(220.0));

        // Lagrange Score sparkline strip
        let sparkline_strip = column![
            text("Lagrange Score — 90-day history").size(11),
            Canvas::new(LagrangeSparkline { history: self.lagrange_history.clone() })
                .width(Length::Fill)
                .height(Length::Fixed(60.0)),
        ].spacing(2);

        // Indicator summary row
        let indicator_row = match &self.indicators {
            None => row![text("Indicators: loading...").size(11)].spacing(20),
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
                    text(rsi_str).size(11),
                    text(macd_str).size(11),
                    text(sma_str).size(11),
                    text(analyst_str).size(11),
                    text(sentiment_str).size(11),
                    text(short_str).size(11),
                ]
                .spacing(24)
            }
        };

        // Earnings calendar
        let earnings_section = if self.earnings.is_empty() {
            column![
                text("Earnings Calendar").size(14),
                text("No earnings data — run the scraper to fetch from Finnhub").size(12),
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
                text("Earnings Calendar").size(14),
                hdr,
                scrollable(Column::with_children(items).spacing(3)).height(Length::Fixed(110.0)),
            ].spacing(4)
        };

        // Institutional holdings (13F)
        let holdings_section = if self.holdings.is_empty() {
            column![
                text("Top Institutional Holders").size(14),
                text("No 13F data — run the scraper to fetch from EDGAR").size(12),
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
                text("Top Institutional Holders").size(14),
                hdr,
                scrollable(Column::with_children(holding_rows).spacing(3)).height(Length::Fixed(100.0)),
            ].spacing(4)
        };

        // 8-K filings
        let filings_section = if self.filings_8k.is_empty() {
            column![
                text("Recent 8-K Filings").size(14),
                text("No 8-K data — run the scraper to fetch from EDGAR").size(12),
            ].spacing(4)
        } else {
            let filing_rows: Vec<Element<Message>> = self.filings_8k.iter().map(|f| {
                let desc = f.items.as_deref().map(describe_8k_items).unwrap_or_else(|| "—".into());
                let url = f.edgar_url.clone();
                row![
                    text(f.filed_date.to_string()).size(11).width(Length::Fixed(90.0)),
                    text(desc).size(11).width(Length::Fill),
                    button(text("Copy").size(10)).on_press(Message::CopyText(url.clone())),
                    button(text("Open").size(10)).on_press(Message::OpenUrl(url)),
                ].spacing(8).align_y(Alignment::Center).into()
            }).collect();
            column![
                text("Recent 8-K Filings").size(14),
                scrollable(Column::with_children(filing_rows).spacing(3)).height(Length::Fixed(100.0)),
            ].spacing(4)
        };

        // News headlines
        let news_section = if self.news.is_empty() {
            column![
                text("Recent News").size(14),
                text("No news — run the scraper to fetch from Finnhub").size(12),
            ].spacing(4)
        } else {
            let news_items: Vec<Element<Message>> = self.news.iter().map(|n| {
                let source = n.source.as_deref().unwrap_or("—");
                let date = n.published_at.format("%b %d").to_string();
                let url = n.url.clone();
                let copy_text = format!("{} — {}", n.headline, n.url);
                row![
                    text(format!("[{date}]")).size(11).width(Length::Fixed(52.0)),
                    text(source.to_string()).size(11).width(Length::Fixed(72.0)),
                    text(&n.headline).size(11).width(Length::Fill),
                    button(text("Copy").size(10)).on_press(Message::CopyText(copy_text)),
                    button(text("Open").size(10)).on_press(Message::OpenUrl(url)),
                ].spacing(6).align_y(Alignment::Center).into()
            }).collect();
            column![
                text("Recent News").size(14),
                scrollable(Column::with_children(news_items).spacing(4)).height(Length::Fixed(120.0)),
            ].spacing(4)
        };

        // Insider trades
        let insider_section = if self.insider_trades.is_empty() {
            column![
                text("Recent Insider Trades").size(14),
                text("No insider trade data — run the scraper to fetch from EDGAR").size(12),
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
                text("Recent Insider Trades").size(14),
                hdr,
                scrollable(Column::with_children(trade_rows).spacing(3)).height(Length::Fixed(130.0)),
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
            column![text(&self.status).size(15)]
        } else {
            let price_rows: Vec<Element<Message>> = self.rows.iter().map(|r| {
                row![
                    text(r.date.to_string()).width(Length::FillPortion(2)),
                    text(format!("{:.2}", r.open)).width(Length::FillPortion(2)),
                    text(format!("{:.2}", r.high)).width(Length::FillPortion(2)),
                    text(format!("{:.2}", r.low)).width(Length::FillPortion(2)),
                    text(format!("{:.2}", r.close)).width(Length::FillPortion(2)),
                    text(r.volume.to_string()).width(Length::FillPortion(3)),
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
                    text(title).size(10),
                    Canvas::new(FearGreedGauge { score, label })
                        .width(Length::Fixed(148.0))
                        .height(Length::Fixed(82.0)),
                ].align_x(Alignment::Center).spacing(2).into(),
                None => column![
                    text(title).size(10),
                    text(fallback).size(10),
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
                "loading...".to_string(),
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
                "loading...".to_string(),
            ),
        };

        let gauges_row = scrollable(
            row![crypto_gauge, equities_gauge, ticker_gauge, astro_gauge, lagrange_gauge]
                .spacing(16)
        )
        .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default()));

        let header = row![
            text(format!("{} — Daily Price Data", self.selected_ticker)).size(24),
            iced::widget::Space::with_width(Length::Fill),
            button(theme_label).on_press(Message::ToggleTheme),
        ].align_y(Alignment::Center);

        // Signal Intelligence panel
        let signal_section = match &self.indicators {
            None => column![
                text("Signal Intelligence").size(14),
                text("Loading indicators...").size(11),
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
                    .map(|b| text(format!("  • {b}")).size(11).into())
                    .collect();
                let verdict = text(format!(
                    "  Lagrange Score: {lagrange:.0}/100 — {lagrange_label}"
                )).size(12);
                column![
                    text(format!("Signal Intelligence: {}", self.selected_ticker)).size(14),
                    horizontal_rule(1),
                    Column::with_children(bullet_items).spacing(4),
                    horizontal_rule(1),
                    verdict,
                ].spacing(6)
            }
        };

        // Watchlist Ranking panel
        let watchlist_section = if self.watchlist.is_empty() {
            column![
                text("Watchlist Ranking").size(14),
                text("Loading...").size(11),
            ].spacing(4)
        } else {
            let hdr = row![
                text("#").size(11).width(Length::Fixed(24.0)),
                text("Ticker").size(11).width(Length::Fixed(64.0)),
                text("Score").size(11).width(Length::Fixed(90.0)),
                text("Astro").size(11).width(Length::Fixed(100.0)),
                text("Sentiment").size(11).width(Length::Fixed(120.0)),
                text("Short%").size(11).width(Length::Fill),
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
                let score_zone = match score as u32 {
                    0..=24  => "■ Mis",
                    25..=44 => "■ Unf",
                    45..=55 => "■ Neu",
                    56..=75 => "■ Fav",
                    _       => "■ Opt",
                };
                let ticker_btn = button(text(w.ticker.clone()).size(11))
                    .on_press(Message::TickerSelected(w.ticker.clone()));
                row![
                    text(format!("{}", i + 1)).size(11).width(Length::Fixed(24.0)),
                    ticker_btn,
                    text(format!("{score:.0} {score_zone}")).size(11).width(Length::Fixed(90.0)),
                    text(astro_str).size(11).width(Length::Fixed(100.0)),
                    text(sent_str).size(11).width(Length::Fixed(120.0)),
                    text(short_str).size(11).width(Length::Fill),
                ].spacing(8).align_y(Alignment::Center).into()
            }).collect();
            column![
                text("Watchlist Ranking").size(14),
                horizontal_rule(1),
                hdr,
                horizontal_rule(1),
                Column::with_children(rank_rows).spacing(5),
            ].spacing(5)
        };

        // Macro strip — latest value for key FRED series
        let macro_find = |id: &str| -> String {
            self.macro_data.iter()
                .find(|m| m.series_id == id)
                .and_then(|m| m.value.as_ref().map(|v| v.to_string()))
                .unwrap_or_else(|| "—".into())
        };
        let macro_strip = row![
            text(format!("Fed Funds: {}%",   macro_find("FEDFUNDS"))).size(11),
            text(format!("CPI YoY: {}%",       macro_find("CPIAUCSL_YOY"))).size(11),
            text(format!("Unemploy: {}%",     macro_find("UNRATE"))).size(11),
            text(format!("10Y: {}%",          macro_find("GS10"))).size(11),
            text(format!("2Y: {}%",           macro_find("GS2"))).size(11),
            text(format!("Spread: {}%",       macro_find("T10Y2Y"))).size(11),
            text(format!("VIX: {}",           macro_find("VIXCLS"))).size(11),
            text(format!("WTI Oil: ${}",      macro_find("DCOILWTICO"))).size(11),
        ].spacing(20);

        // Astrology section: natal wheel + transits table side by side
        let moon_phase  = self.astro_score.as_ref().and_then(|s| s.moon_phase.as_deref());
        let moon_deg    = self.astro_score.as_ref().and_then(|s| s.moon_phase_deg);
        let mercury_rx  = self.astro_score.as_ref().and_then(|s| s.mercury_rx).unwrap_or(false);

        let natal_wheel = Canvas::new(NatalWheel {
            natal:    self.natal_positions.clone(),
            transits: self.daily_transits.clone(),
        })
        .width(Length::Fixed(240.0))
        .height(Length::Fixed(240.0));

        let wheel_col = column![
            text(format!("{} Birth Chart", self.selected_ticker)).size(14),
            natal_wheel,
            build_wheel_legend(),
        ].spacing(4).align_x(Alignment::Center);

        let transits_col = column![
            build_transits_section(&self.astro_aspects, moon_phase, moon_deg, mercury_rx),
        ].width(Length::Fill);

        let astrology_section = row![
            wheel_col,
            transits_col,
        ].spacing(20).align_y(Alignment::Start);

        // News and 8-K side by side
        let news_filings_row = row![
            column![filings_section].width(Length::FillPortion(1)),
            column![news_section].width(Length::FillPortion(1)),
        ].spacing(20);

        // Portfolio panel
        let portfolio_section = if self.portfolio.is_empty() {
            column![
                text("Portfolio").size(14),
                text("No positions — add rows to portfolio_positions table via portfolio_seed.sql").size(11),
            ].spacing(4)
        } else {
            let hdr = row![
                text("Ticker").size(11).width(Length::Fixed(64.0)),
                text("Shares").size(11).width(Length::Fixed(72.0)),
                text("Avg Cost").size(11).width(Length::Fixed(88.0)),
                text("Cost Basis").size(11).width(Length::Fill),
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
                    text(&p.ticker).size(11).width(Length::Fixed(64.0)),
                    text(format!("{:.2}", p.shares)).size(11).width(Length::Fixed(72.0)),
                    text(format!("${:.2}", p.avg_cost)).size(11).width(Length::Fixed(88.0)),
                    text(label).size(11).width(Length::Fill),
                ].spacing(8).into()
            }).collect();

            let total_basis: f32 = self.portfolio.iter().map(|p| p.shares * p.avg_cost).sum();
            column![
                text("Portfolio").size(14),
                horizontal_rule(1),
                hdr,
                Column::with_children(pos_rows).spacing(2),
                horizontal_rule(1),
                text(format!("Total cost basis: ${total_basis:.0}")).size(11),
            ].spacing(4)
        };

        let content = column![
            header,
            horizontal_rule(1),
            gauges_row,
            horizontal_rule(1),
            ticker_buttons,
            text(&self.status).size(11),
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
