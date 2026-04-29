use iced::widget::{button, column, row, scrollable, text, Column};
use iced::{Alignment, Element, Length};

use crate::font;
use crate::helpers::{describe_8k_items, format_market_value, format_shares};
use super::shared::{eyebrow, gold_scrollbar_style, section_rule};
use crate::state::{Dashboard, Message};
use crate::theme;

impl Dashboard {
    pub(crate) fn view_research(&self) -> Element<'_, Message> {
        // ── 8-K Filings ─────────────────────────────────────
        let filings_section = if self.filings_8k.is_empty() {
            column![
                text("Recent 8-K Filings").font(font::DISPLAY).size(theme::text_md()),
                text(format!("No recent filings loaded for {} yet.", self.selected_ticker)).size(theme::text_base()),
                text("Material events (earnings, M&A, leadership changes) appear here.")
                    .size(theme::text_sm()),
            ]
            .spacing(4)
        } else {
            let filing_rows: Vec<Element<Message>> = self
                .filings_8k
                .iter()
                .map(|f| {
                    let desc = f
                        .items
                        .as_deref()
                        .map(describe_8k_items)
                        .unwrap_or_else(|| "—".into());
                    let url = f.edgar_url.clone();
                    row![
                        text(f.filed_date.to_string())
                            .size(theme::text_base())
                            .width(Length::Fixed(90.0)),
                        text(desc).size(theme::text_base()).width(Length::Fill),
                        button(text("Copy").size(theme::text_sm()))
                            .on_press(Message::CopyText(url.clone())),
                        button(text("Open").size(theme::text_sm()))
                            .on_press(Message::OpenUrl(url)),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .into()
                })
                .collect();
            column![
                text("Recent 8-K Filings").font(font::DISPLAY).size(theme::text_md()),
                scrollable(Column::with_children(filing_rows).spacing(4))
                    .height(Length::Fixed(100.0))
                    .style(gold_scrollbar_style),
            ]
            .spacing(4)
        };

        // ── News Headlines ──────────────────────────────────
        let news_section = if self.news.is_empty() {
            column![
                text("Recent News").font(font::DISPLAY).size(theme::text_md()),
                text(format!("No headlines loaded for {} yet.", self.selected_ticker)).size(theme::text_base()),
                text("News articles are fetched from Finnhub during scraper runs.")
                    .size(theme::text_sm()),
            ]
            .spacing(4)
        } else {
            let news_items: Vec<Element<Message>> = self
                .news
                .iter()
                .map(|n| {
                    let source = n.source.as_deref().unwrap_or("—");
                    let date = n.published_at.format("%b %d").to_string();
                    let url = n.url.clone();
                    let copy_text = format!("{} — {}", n.headline, n.url);
                    row![
                        text(format!("[{date}]"))
                            .size(theme::text_base())
                            .width(Length::Fixed(52.0)),
                        text(source.to_string())
                            .size(theme::text_base())
                            .width(Length::Fixed(72.0)),
                        text(&n.headline)
                            .size(theme::text_base())
                            .width(Length::Fill),
                        button(text("Copy").size(theme::text_sm()))
                            .on_press(Message::CopyText(copy_text)),
                        button(text("Open").size(theme::text_sm()))
                            .on_press(Message::OpenUrl(url)),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into()
                })
                .collect();
            // Sentiment badge from AlphaVantage sentiment_scores
            let sentiment_badge: Element<'_, Message> = if let Some(ref s) = self.sentiment {
                let (badge_color, badge_text) = match s.sentiment_label.as_deref() {
                    Some("Bullish")         => (theme::ZONE_OPTIMAL,     "Bullish"),
                    Some("Somewhat-Bullish") => (theme::ZONE_FAVORABLE,  "Somewhat Bullish"),
                    Some("Neutral")         => (theme::ZONE_NEUTRAL,     "Neutral"),
                    Some("Somewhat-Bearish") => (theme::ZONE_UNFAVORABLE,"Somewhat Bearish"),
                    Some("Bearish")         => (theme::ZONE_MISALIGNED,  "Bearish"),
                    _ => (theme::ZONE_NEUTRAL, "Unknown"),
                };
                let score_str = s.sentiment_score
                    .as_ref()
                    .map(|v| format!("{v}"))
                    .unwrap_or_default();
                text(format!(" [{badge_text} {score_str}]"))
                    .size(theme::text_sm())
                    .color(badge_color)
                    .into()
            } else {
                text("").into()
            };

            let news_header = row![
                text("Recent News").font(font::DISPLAY).size(theme::text_md()),
                sentiment_badge,
            ].spacing(6).align_y(Alignment::Center);

            column![
                news_header,
                scrollable(Column::with_children(news_items).spacing(4))
                    .height(Length::Fixed(120.0))
                    .style(gold_scrollbar_style),
            ]
            .spacing(4)
        };

        // ── Insider Trades ──────────────────────────────────
        let insider_section = if self.insider_trades.is_empty() {
            column![
                text("Recent Insider Trades").font(font::DISPLAY).size(theme::text_md()),
                text(format!("No Form 4 insider transactions loaded for {} yet.", self.selected_ticker)).size(theme::text_base()),
                text("Insider buys and sells are fetched from SEC EDGAR.").size(theme::text_sm()),
            ]
            .spacing(4)
        } else {
            let hdr = row![
                text("Date").width(Length::FillPortion(2)),
                text("Insider").width(Length::FillPortion(4)),
                text("Title").width(Length::FillPortion(3)),
                text("Type").width(Length::FillPortion(1)),
                text("Shares").width(Length::FillPortion(2)),
                text("Price").width(Length::FillPortion(2)),
            ]
            .spacing(8);
            let trade_rows: Vec<Element<Message>> = self
                .insider_trades
                .iter()
                .map(|t| {
                    row![
                        text(t.transaction_date.to_string()).width(Length::FillPortion(2)),
                        text(&t.insider_name).width(Length::FillPortion(4)),
                        text(t.insider_title.as_deref().unwrap_or("—")).width(Length::FillPortion(3)),
                        text(if t.transaction_type == "A" { "Buy" } else { "Sell" })
                            .width(Length::FillPortion(1)),
                        text(format!("{:.0}", t.shares)).font(font::INTER).width(Length::FillPortion(2)),
                        text(format!("${:.2}", t.price_per_share)).font(font::INTER).width(Length::FillPortion(2)),
                    ]
                    .spacing(8)
                    .into()
                })
                .collect();
            column![
                text("Recent Insider Trades").font(font::DISPLAY).size(theme::text_md()),
                hdr,
                scrollable(Column::with_children(trade_rows).spacing(4))
                    .height(Length::Fixed(130.0))
                    .style(gold_scrollbar_style),
            ]
            .spacing(4)
        };

        // ── Institutional Holdings ──────────────────────────
        let holdings_section = if self.holdings.is_empty() {
            column![
                text("Top Institutional Holders").font(font::DISPLAY).size(theme::text_md()),
                text(format!("No institutional holdings loaded for {} yet.", self.selected_ticker)).size(theme::text_base()),
                text("The scraper fetches 13F filings from SEC EDGAR.").size(theme::text_sm()),
            ]
            .spacing(4)
        } else {
            let hdr = row![
                text("Institution").width(Length::FillPortion(4)),
                text("Shares Held").width(Length::FillPortion(3)),
                text("Market Value").width(Length::FillPortion(3)),
                text("Period").width(Length::FillPortion(2)),
            ]
            .spacing(8);
            let holding_rows: Vec<Element<Message>> = self
                .holdings
                .iter()
                .map(|h| {
                    row![
                        text(&h.institution_name).width(Length::FillPortion(4)),
                        text(format_shares(h.shares_held)).font(font::INTER).width(Length::FillPortion(3)),
                        text(format_market_value(&h.market_value)).font(font::INTER).width(Length::FillPortion(3)),
                        text(h.report_period.to_string()).width(Length::FillPortion(2)),
                    ]
                    .spacing(8)
                    .into()
                })
                .collect();
            column![
                text("Top Institutional Holders").font(font::DISPLAY).size(theme::text_md()),
                hdr,
                scrollable(Column::with_children(holding_rows).spacing(4))
                    .height(Length::Fixed(120.0))
                    .style(gold_scrollbar_style),
            ]
            .spacing(4)
        };

        // ── RSS Market News ─────────────────────────────────
        let rss_section = if self.rss_articles.is_empty() {
            column![
                text("Market News (RSS)").font(font::DISPLAY).size(theme::text_md()),
                text("No RSS articles loaded yet. Run the scraper to fetch headlines from 25+ sources.")
                    .size(theme::text_sm()),
            ]
            .spacing(4)
        } else {
            // Sort: articles mentioning the selected ticker first, then by date
            let ticker_upper = self.selected_ticker.to_uppercase();
            let mut sorted_articles: Vec<&pursuit_week4_automation::models::RssArticle> =
                self.rss_articles.iter().collect();
            sorted_articles.sort_by(|a, b| {
                let a_relevant = a.headline.to_uppercase().contains(&ticker_upper)
                    || a.summary.as_deref().unwrap_or("").to_uppercase().contains(&ticker_upper);
                let b_relevant = b.headline.to_uppercase().contains(&ticker_upper)
                    || b.summary.as_deref().unwrap_or("").to_uppercase().contains(&ticker_upper);
                b_relevant.cmp(&a_relevant).then(b.published_at.cmp(&a.published_at))
            });

            let rss_items: Vec<Element<Message>> = sorted_articles
                .iter()
                .map(|a| {
                    let date = a.published_at.format("%b %d").to_string();
                    let link = a.link.clone();
                    let is_relevant = a.headline.to_uppercase().contains(&ticker_upper)
                        || a.summary.as_deref().unwrap_or("").to_uppercase().contains(&ticker_upper);
                    let mut headline_text = text(&a.headline)
                        .size(theme::text_base())
                        .width(Length::Fill);
                    if is_relevant {
                        headline_text = headline_text.color(theme::ZONE_OPTIMAL);
                    }
                    row![
                        text(format!("[{date}]"))
                            .size(theme::text_base())
                            .width(Length::Fixed(52.0)),
                        text(&a.feed_source)
                            .size(theme::text_base())
                            .width(Length::Fixed(90.0)),
                        text(&a.category)
                            .size(theme::text_xs())
                            .width(Length::Fixed(60.0)),
                        headline_text,
                        button(text("Open").size(theme::text_sm()))
                            .on_press(Message::OpenUrl(link)),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into()
                })
                .collect();
            column![
                text("Market News (RSS — 25 sources)").font(font::DISPLAY).size(theme::text_md()),
                scrollable(Column::with_children(rss_items).spacing(4))
                    .height(Length::Fixed(180.0))
                    .style(gold_scrollbar_style),
            ]
            .spacing(4)
        };

        // ── GDELT Geopolitical Events ───────────────────────
        let gdelt_section = if self.gdelt_events.is_empty() {
            column![
                text("Geopolitical Events (GDELT)").font(font::DISPLAY).size(theme::text_md()),
                text("No geopolitical events loaded yet. Run the scraper to fetch from GDELT.")
                    .size(theme::text_sm()),
            ]
            .spacing(4)
        } else {
            let gdelt_items: Vec<Element<Message>> = self
                .gdelt_events
                .iter()
                .map(|ev| {
                    let date = ev.published_at.format("%b %d %H:%M").to_string();
                    let country = ev.source_country.as_deref().unwrap_or("—");
                    let domain = ev.domain.as_deref().unwrap_or("—");
                    let url = ev.url.clone();

                    // Tone coloring: negative = red, neutral = gray, positive = green
                    let tone_str = ev.tone.map(|t| format!("{t:+.1}")).unwrap_or_else(|| "—".into());
                    let tone_color = match ev.tone {
                        Some(t) if t > 2.0 => theme::ZONE_OPTIMAL,
                        Some(t) if t > 0.0 => theme::ZONE_FAVORABLE,
                        Some(t) if t > -2.0 => theme::ZONE_NEUTRAL,
                        Some(_) => theme::ZONE_MISALIGNED,
                        None => theme::ZONE_NEUTRAL,
                    };

                    row![
                        text(format!("[{date}]"))
                            .size(theme::text_base())
                            .width(Length::Fixed(100.0)),
                        text(country)
                            .size(theme::text_xs())
                            .width(Length::Fixed(36.0)),
                        text(tone_str)
                            .size(theme::text_xs())
                            .color(tone_color)
                            .width(Length::Fixed(42.0)),
                        text(&ev.title)
                            .size(theme::text_base())
                            .width(Length::Fill),
                        text(domain)
                            .size(theme::text_xs())
                            .width(Length::Fixed(90.0)),
                        button(text("Open").size(theme::text_sm()))
                            .on_press(Message::OpenUrl(url)),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into()
                })
                .collect();
            column![
                text("Geopolitical Events (GDELT)").font(font::DISPLAY).size(theme::text_md()),
                scrollable(Column::with_children(gdelt_items).spacing(4))
                    .height(Length::Fixed(160.0))
                    .style(gold_scrollbar_style),
            ]
            .spacing(4)
        };

        // ── News + Filings side by side ─────────────────────
        let news_filings_row = row![
            column![filings_section].width(Length::FillPortion(1)),
            column![news_section].width(Length::FillPortion(1)),
        ]
        .spacing(20);

        column![
            eyebrow("FILINGS & NEWS"),
            news_filings_row,
            section_rule(),
            eyebrow("MARKET NEWS"),
            rss_section,
            section_rule(),
            eyebrow("GEOPOLITICS"),
            gdelt_section,
            section_rule(),
            eyebrow("INSIDER ACTIVITY"),
            insider_section,
            section_rule(),
            eyebrow("INSTITUTIONAL HOLDERS"),
            holdings_section,
        ]
        .spacing(theme::SPACE_SM)
        .into()
    }
}
