use iced::widget::canvas::Canvas;
use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input, Column, Row};
use iced::{Alignment, Element, Length};

use crate::font;
use crate::heatmap::SectorHeatMap;
use crate::state::{Dashboard, Message, UniverseSortCol};
use super::shared::{eyebrow, gold_scrollbar_style, section_rule};
use crate::theme;

impl Dashboard {
    pub(crate) fn view_universe(&self) -> Element<'_, Message> {
        let page_size = 50usize;
        let total = self.universe_total;
        let max_page = if total == 0 {
            0
        } else {
            ((total as usize).saturating_sub(1)) / page_size
        };
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
            row![text("Zone:").size(theme::text_sm())]
                .spacing(4)
                .align_y(Alignment::Center),
            |r, (label, val)| {
                let is_active = self.universe_filter_zone == val;
                let display = if is_active {
                    format!("[{label}]")
                } else {
                    label.to_string()
                };
                r.push(
                    button(text(display).size(theme::text_sm()))
                        .on_press(Message::UniverseFilterZone(val)),
                )
            },
        );

        // Sector filter
        let sector_bar: Row<Message> = {
            let mut r = row![text("Sector:").size(theme::text_sm())]
                .spacing(4)
                .align_y(Alignment::Center);
            let is_all = self.universe_filter_sector.is_none();
            let all_label = if is_all {
                "[All]".to_string()
            } else {
                "All".to_string()
            };
            r = r.push(
                button(text(all_label).size(theme::text_xs()))
                    .on_press(Message::UniverseFilterSector(None)),
            );
            for sector in &self.universe_sectors {
                let is_active =
                    self.universe_filter_sector.as_deref() == Some(sector.as_str());
                let label = if is_active {
                    format!("[{sector}]")
                } else {
                    sector.clone()
                };
                let val = Some(sector.clone());
                r = r.push(
                    button(text(label).size(theme::text_xs()))
                        .on_press(Message::UniverseFilterSector(val)),
                );
            }
            r
        };

        // Pagination
        let pagination = row![
            button(text("◀ Prev").size(theme::text_sm())).on_press(Message::UniversePrevPage),
            text(page_label).size(theme::text_sm()),
            button(text("Next ▶").size(theme::text_sm())).on_press(Message::UniverseNextPage),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        // Universe table
        let universe_table: Element<'_, Message> = if self.universe_rows.is_empty() {
            text("No scored tickers yet. Run the scraper to compute astro scores.")
                .size(theme::text_base())
                .into()
        } else {
            // Sortable column header helper
            let sort_hdr = |label: &str, col: UniverseSortCol, width: Length| -> Element<'_, Message> {
                let indicator = if self.universe_sort_col == col {
                    if self.universe_sort_asc { " ▲" } else { " ▼" }
                } else {
                    ""
                };
                button(text(format!("{label}{indicator}")).size(theme::text_sm()))
                    .on_press(Message::UniverseSort(col))
                    .width(width)
                    .into()
            };
            let hdr = row![
                text("#").size(theme::text_sm()).width(Length::Fixed(30.0)),
                sort_hdr("Ticker", UniverseSortCol::Ticker, Length::Fixed(64.0)),
                text("Company")
                    .size(theme::text_sm())
                    .width(Length::FillPortion(3)),
                text("Sector")
                    .size(theme::text_sm())
                    .width(Length::FillPortion(2)),
                sort_hdr("Astro", UniverseSortCol::Astro, Length::Fixed(56.0)),
                sort_hdr("Score", UniverseSortCol::Score, Length::Fixed(56.0)),
                text("Zone")
                    .size(theme::text_sm())
                    .width(Length::Fixed(90.0)),
                sort_hdr("Fin", UniverseSortCol::Fin, Length::Fixed(44.0)),
                sort_hdr("Macro", UniverseSortCol::Macro, Length::Fixed(44.0)),
                sort_hdr("Short", UniverseSortCol::Short, Length::Fixed(44.0)),
                text("Conc")
                    .size(theme::text_sm())
                    .width(Length::Fixed(90.0)),
            ]
            .spacing(6);

            let offset = self.universe_page * page_size;
            let rows: Vec<Element<Message>> = self
                .universe_rows
                .iter()
                .enumerate()
                .map(|(i, u)| {
                    let zone_color = match u.label.as_str() {
                        "Optimal" => theme::ZONE_OPTIMAL,
                        "Favorable" => theme::ZONE_FAVORABLE,
                        "Neutral" => theme::ZONE_NEUTRAL,
                        "Unfavorable" => theme::ZONE_UNFAVORABLE,
                        _ => theme::ZONE_MISALIGNED,
                    };
                    let astro_str = u
                        .astro_score
                        .map(|s| format!("{s:.0}"))
                        .unwrap_or_else(|| "---".into());
                    let fin_str = u
                        .fin_score
                        .map(|s| format!("{s:.0}"))
                        .unwrap_or_else(|| "---".into());
                    let macro_str = u
                        .macro_score
                        .map(|s| format!("{s:.0}"))
                        .unwrap_or_else(|| "---".into());
                    let short_str = u
                        .short_score
                        .map(|s| format!("{s:.0}"))
                        .unwrap_or_else(|| "---".into());
                    let conc = u.concordance.as_deref().unwrap_or("---");
                    let company = u.company_name.as_deref().unwrap_or("—");
                    let sector = u.sector.as_deref().unwrap_or("—");

                    let ticker_btn = button(text(u.ticker.clone()).size(theme::text_sm()))
                        .on_press(Message::TickerSelected(u.ticker.clone()));

                    row![
                        text(format!("{}", offset + i + 1))
                            .size(theme::text_sm())
                            .width(Length::Fixed(30.0)),
                        ticker_btn,
                        text(company.to_string())
                            .size(theme::text_xs())
                            .width(Length::FillPortion(3)),
                        text(sector.to_string())
                            .size(theme::text_xs())
                            .width(Length::FillPortion(2)),
                        text(astro_str)
                            .font(font::INTER)
                            .size(theme::text_sm())
                            .width(Length::Fixed(56.0)),
                        text(format!("{:.0}", u.score))
                            .font(font::INTER)
                            .size(theme::text_sm())
                            .width(Length::Fixed(56.0)),
                        text(u.label.clone())
                            .size(theme::text_sm())
                            .color(zone_color)
                            .width(Length::Fixed(90.0)),
                        text(fin_str)
                            .font(font::INTER)
                            .size(theme::text_sm())
                            .width(Length::Fixed(44.0)),
                        text(macro_str)
                            .font(font::INTER)
                            .size(theme::text_sm())
                            .width(Length::Fixed(44.0)),
                        text(short_str)
                            .font(font::INTER)
                            .size(theme::text_sm())
                            .width(Length::Fixed(44.0)),
                        text(conc.to_string())
                            .size(theme::text_xs())
                            .width(Length::Fixed(90.0)),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into()
                })
                .collect();

            column![
                hdr,
                horizontal_rule(1),
                scrollable(Column::with_children(rows).spacing(3))
                    .height(Length::Fixed(400.0))
                    .style(gold_scrollbar_style),
            ]
            .spacing(4)
            .into()
        };

        // Sector heat map
        let sector_heatmap = Canvas::new(SectorHeatMap {
            sectors: self.sector_summaries.clone(),
        })
        .width(Length::Fill)
        .height(Length::Fixed(70.0));

        // Alerts panel
        let alerts_section = self.build_alerts_section();

        // Search input + export button
        let search_bar = row![
            text("Search:").size(theme::text_sm()),
            text_input("Ticker or company name...", &self.universe_search_text)
                .on_input(Message::UniverseSearchChanged)
                .size(theme::text_sm())
                .width(Length::Fixed(240.0)),
            button(text("Export CSV").size(theme::text_sm()))
                .on_press(Message::ExportUniverseCsv),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        column![
            eyebrow("UNIVERSE EXPLORER"),
            text(format!("Universe Explorer — {} scored tickers", total)).font(font::DISPLAY).size(theme::text_lg()),
            section_rule(),
            eyebrow("SECTOR MAP"),
            text("Sector Heat Map (by avg astro score)").size(theme::text_sm()),
            sector_heatmap,
            section_rule(),
            search_bar,
            scrollable(sector_bar)
                .direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::default()
                ))
                .style(gold_scrollbar_style),
            zone_bar,
            pagination,
            universe_table,
            section_rule(),
            eyebrow("LAGRANGE ALERTS"),
            container(alerts_section).padding([10, 14]),
        ]
        .spacing(theme::SPACE_SM)
        .into()
    }

    /// Build the alerts panel (used by Universe tab).
    fn build_alerts_section(&self) -> Column<'_, Message> {
        let unread = self.unread_alert_count;
        let heading = if unread > 0 {
            format!("Lagrange Alerts  [{unread} new]")
        } else {
            "Lagrange Alerts".to_string()
        };
        if self.alerts.is_empty() {
            column![
                text(heading).font(font::DISPLAY).size(theme::text_lg()),
                text("No alerts yet — fires when a ticker enters Optimal or Misaligned zone")
                    .size(theme::text_base()),
            ]
            .spacing(4)
        } else {
            // Action bar: Mark All Read (only if unread exist)
            let mut action_bar = row![].spacing(8);
            if unread > 0 {
                action_bar = action_bar.push(
                    button(text("Mark All Read").size(theme::text_sm()))
                        .on_press(Message::MarkAllAlertsRead),
                );
            }

            let hdr = row![
                text("Date")
                    .size(theme::text_base())
                    .width(Length::Fixed(90.0)),
                text("Ticker")
                    .size(theme::text_base())
                    .width(Length::Fixed(64.0)),
                text("Score")
                    .size(theme::text_base())
                    .width(Length::Fixed(56.0)),
                text("Zone")
                    .size(theme::text_base())
                    .width(Length::Fixed(110.0)),
                text("Was")
                    .size(theme::text_base())
                    .width(Length::Fill),
                text("").size(theme::text_base()).width(Length::Fixed(130.0)),
            ]
            .spacing(8);
            let alert_rows: Vec<Element<Message>> = self
                .alerts
                .iter()
                .map(|a| {
                    let zone_color = if a.label == "Optimal" {
                        theme::ZONE_OPTIMAL
                    } else {
                        theme::ZONE_MISALIGNED
                    };
                    let prev = a.prev_label.as_deref().unwrap_or("—");
                    let actions: Element<Message> = if a.is_read {
                        row![
                            text("✓").size(theme::text_sm()),
                            button(text("✕").size(theme::text_sm()))
                                .on_press(Message::DismissAlert(a.id)),
                        ]
                        .spacing(6)
                        .width(Length::Fixed(130.0))
                        .into()
                    } else {
                        row![
                            button(text("Ack").size(theme::text_sm()))
                                .on_press(Message::MarkAlertRead(a.id)),
                            button(text("✕").size(theme::text_sm()))
                                .on_press(Message::DismissAlert(a.id)),
                        ]
                        .spacing(6)
                        .width(Length::Fixed(130.0))
                        .into()
                    };
                    row![
                        text(a.alert_date.to_string())
                            .font(font::INTER)
                            .size(theme::text_base())
                            .width(Length::Fixed(90.0)),
                        text(&a.ticker)
                            .size(theme::text_base())
                            .width(Length::Fixed(64.0)),
                        text(format!("{:.1}", a.score))
                            .font(font::INTER)
                            .size(theme::text_base())
                            .width(Length::Fixed(56.0)),
                        text(&a.label)
                            .size(theme::text_base())
                            .color(zone_color)
                            .width(Length::Fixed(110.0)),
                        text(prev.to_string())
                            .size(theme::text_base())
                            .width(Length::Fill),
                        actions,
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .into()
                })
                .collect();
            column![
                row![
                    text(heading).font(font::DISPLAY).size(theme::text_lg()),
                    action_bar,
                ].spacing(12).align_y(Alignment::Center),
                horizontal_rule(1),
                hdr,
                horizontal_rule(1),
                scrollable(Column::with_children(alert_rows).spacing(4))
                    .height(Length::Fixed(160.0))
                    .style(gold_scrollbar_style),
            ]
            .spacing(4)
        }
    }
}
