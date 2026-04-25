mod shared;
mod overview;
mod astrology_tab;
mod universe;
mod fundamentals;
mod research;
mod portfolio_tab;
mod settings;

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input, Column, Row};
use iced::{Alignment, Color, Element, Length};

use crate::icons;
use crate::state::{Dashboard, Message};
use crate::tabs::Tab;
use crate::theme;

impl Dashboard {
    pub fn view(&self) -> Element<'_, Message> {
        // ── Ticker selector buttons (pinned watchlist) ──────
        let ticker_buttons: Row<Message> = self.tickers.iter().fold(row![].spacing(6), |r, ticker| {
            let btn = button(text(ticker).size(theme::text_base())).on_press(Message::TickerSelected(ticker.clone()));
            r.push(btn)
        });

        // ── Search bar ──────────────────────────────────────
        let search_bar = row![
            text_input("Search any ticker…", &self.ticker_search_input)
                .id(crate::update::SEARCH_INPUT_ID)
                .on_input(Message::TickerSearchInput)
                .on_submit(Message::TickerSearchSubmit)
                .width(Length::Fixed(200.0))
                .size(theme::text_base()),
            button(text("Go").size(theme::text_base()))
                .on_press(Message::TickerSearchSubmit),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        // ── Autocomplete dropdown ───────────────────────────
        let autocomplete: Element<Message> = if self.autocomplete_suggestions.is_empty() {
            row![].into()
        } else {
            let items: Vec<Element<Message>> = self.autocomplete_suggestions.iter()
                .map(|(ticker, name)| {
                    button(
                        text(format!("{ticker}  —  {name}")).size(theme::text_base())
                    )
                    .on_press(Message::AutocompleteSelected(ticker.clone()))
                    .width(Length::Fixed(340.0))
                    .into()
                })
                .collect();
            iced::widget::column(items).spacing(2).into()
        };

        // ── Recently viewed ─────────────────────────────────
        let recently_viewed_row: Row<Message> = if self.recently_viewed.is_empty() {
            row![text("Recently viewed: —").size(theme::text_base())].spacing(4)
        } else {
            let label = text("Recently:").size(theme::text_base());
            self.recently_viewed.iter().fold(
                row![label].spacing(6),
                |r, t| r.push(
                    button(text(t).size(theme::text_base()))
                        .on_press(Message::TickerSelected(t.clone()))
                ),
            )
        };

        // ── Header with tab subtitle + theme toggle ─────────
        let tab_subtitle = match self.active_tab {
            Tab::Astrology => "Astrology & Timing",
            Tab::Overview => "Daily Price Data",
            Tab::Universe => "Universe Explorer",
            Tab::Fundamentals => "Fundamentals & Agents",
            Tab::Research => "Research & Filings",
            Tab::Portfolio => "Portfolio & Positions",
            Tab::Settings => "Settings",
        };
        let theme_label = format!("Theme: {}", self.theme_mode.label());
        let header = row![
            text(format!("{} — {}", self.selected_ticker, tab_subtitle)).size(theme::text_2xl()),
            iced::widget::Space::with_width(Length::Fill),
            button(text(theme_label).size(theme::text_sm())).on_press(Message::ToggleTheme),
        ].align_y(Alignment::Center);

        // ── Status + refresh ────────────────────────────────
        let refresh_icon = text(icons::ARROW_REPEAT.to_string())
            .font(icons::BOOTSTRAP)
            .size(theme::text_sm());
        let refresh_label = if self.refreshing {
            row![refresh_icon, text("Refreshing...").size(theme::text_sm())].spacing(4).align_y(Alignment::Center)
        } else {
            row![refresh_icon, text("Refresh").size(theme::text_sm())].spacing(4).align_y(Alignment::Center)
        };

        // ── Tab bar (icon + label, active underline) ────────
        let tab_bar: Row<Message> = Tab::all().iter().fold(row![].spacing(2), |r, &tab| {
            let is_active = tab == self.active_tab;
            let icon_text = text(tab.icon().to_string())
                .font(icons::BOOTSTRAP)
                .size(theme::text_base());
            let label_text = text(tab.label()).size(theme::text_sm());
            let tab_content = row![icon_text, label_text]
                .spacing(5)
                .align_y(Alignment::Center);
            let tab_el: Element<Message> = if is_active {
                // Active tab: underline via a bottom border
                let inner = column![
                    tab_content,
                    container(row![])
                        .width(Length::Fill)
                        .height(Length::Fixed(2.0))
                        .style(container::bordered_box),
                ].spacing(2);
                button(inner)
                    .on_press(Message::TabSelected(tab))
                    .padding([6, 12])
                    .into()
            } else {
                button(tab_content)
                    .on_press(Message::TabSelected(tab))
                    .padding([6, 12])
                    .into()
            };
            r.push(tab_el)
        });

        // ── Tab content dispatch ────────────────────────────
        let tab_content: Element<Message> = match self.active_tab {
            Tab::Astrology    => self.view_astrology(),
            Tab::Overview     => self.view_overview(),
            Tab::Universe     => self.view_universe(),
            Tab::Fundamentals => self.view_fundamentals(),
            Tab::Research     => self.view_research(),
            Tab::Portfolio    => self.view_portfolio(),
            Tab::Settings     => self.view_settings(),
        };

        // ── Final assembly ──────────────────────────────────
        let content = column![
            header,
            horizontal_rule(1),
            ticker_buttons,
            row![search_bar].spacing(16),
            autocomplete,
            recently_viewed_row,
            text(&self.status).size(theme::text_base()),
            {
                let fetch_btn: Element<Message> = if self.fetching_ticker {
                    button(
                        row![
                            text(icons::DOWNLOAD.to_string()).font(icons::BOOTSTRAP).size(theme::text_sm()),
                            text("Fetching...").size(theme::text_sm()),
                        ].spacing(4).align_y(Alignment::Center)
                    ).into()
                } else {
                    button(
                        row![
                            text(icons::DOWNLOAD.to_string()).font(icons::BOOTSTRAP).size(theme::text_sm()),
                            text(format!("Fetch {}", self.selected_ticker)).size(theme::text_sm()),
                        ].spacing(4).align_y(Alignment::Center)
                    ).on_press(Message::FetchThisTicker).into()
                };
                row![
                    button(refresh_label).on_press(Message::RefreshNow),
                    fetch_btn,
                ].spacing(8)
            },
            horizontal_rule(1),
            tab_bar,
            horizontal_rule(1),
            tab_content,
        ]
        .spacing(10)
        .padding(20);

        // ── Toast overlay ───────────────────────────────────
        let main_view: Element<'_, Message> = container(scrollable(content))
            .width(Length::Fill).height(Length::Fill).into();

        if self.toasts.is_empty() {
            main_view
        } else {
            let toast_col: Column<Message> = self.toasts.iter().fold(
                column![].spacing(4).width(Length::Shrink),
                |col, (msg, _)| {
                    col.push(
                        container(
                            text(msg.clone()).size(theme::text_sm()).color(Color::WHITE),
                        )
                        .padding([6, 14])
                        .style(|_theme: &iced::Theme| container::Style {
                            background: Some(iced::Background::Color(
                                Color::from_rgba(0.1, 0.1, 0.15, 0.92),
                            )),
                            border: iced::Border {
                                radius: 6.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    )
                },
            );
            let toast_overlay = container(toast_col)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .padding([10, 20]);

            column![toast_overlay, main_view].into()
        }
    }
}
