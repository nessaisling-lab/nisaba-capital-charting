mod shared;
mod overview;
mod astrology_tab;
mod universe;
mod fundamentals;
mod research;
mod portfolio_tab;
mod paper_trail;
mod settings;

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input, Column, Row};
use iced::{Alignment, Color, Element, Length};

use crate::font;
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
            let recent: Vec<_> = self.recently_viewed.iter().rev().take(6).collect();
            recent.iter().rev().fold(
                row![label].spacing(6),
                |r, t| r.push(
                    button(text(t.as_str()).size(theme::text_sm()))
                        .on_press(Message::TickerSelected((*t).clone()))
                ),
            )
        };

        // ── Header row: ticker + actions (right-aligned) ────
        let theme_label = format!("Theme: {}", self.theme_mode.label());
        let refresh_icon = text(icons::ARROW_REPEAT.to_string())
            .font(icons::PHOSPHOR)
            .size(theme::text_sm());
        let refresh_btn = button(
            if self.refreshing {
                row![refresh_icon, text("Refreshing...").size(theme::text_sm())].spacing(4).align_y(Alignment::Center)
            } else {
                row![refresh_icon, text("Refresh").size(theme::text_sm())].spacing(4).align_y(Alignment::Center)
            }
        ).on_press(Message::RefreshNow);
        let fetch_btn: Element<Message> = if self.fetching_ticker {
            button(
                row![
                    text(icons::DOWNLOAD.to_string()).font(icons::PHOSPHOR).size(theme::text_sm()),
                    text("Fetching...").size(theme::text_sm()),
                ].spacing(4).align_y(Alignment::Center)
            ).into()
        } else {
            button(
                row![
                    text(icons::DOWNLOAD.to_string()).font(icons::PHOSPHOR).size(theme::text_sm()),
                    text(format!("Fetch {}", self.selected_ticker)).size(theme::text_sm()),
                ].spacing(4).align_y(Alignment::Center)
            ).on_press(Message::FetchThisTicker).into()
        };
        let header_row = row![
            text(self.selected_ticker.as_str()).font(font::DISPLAY).size(theme::text_2xl()),
            iced::widget::Space::with_width(Length::Fill),
            refresh_btn,
            fetch_btn,
            button(text(theme_label).size(theme::text_sm())).on_press(Message::ToggleTheme),
        ]
        .spacing(theme::SPACE_SM)
        .align_y(Alignment::Center);

        // ── Tab bar (icon + label, animated underline) ──────
        let tab_anim_progress = self.tab_indicator_progress;
        let tab_anim_from = self.tab_indicator_from;
        let tab_anim_to = self.tab_indicator_to;
        let tab_bar: Row<Message> = Tab::all().iter().enumerate().fold(row![].spacing(2), |r, (idx, &tab)| {
            let is_active = tab == self.active_tab;
            let icon_text = text(tab.icon().to_string())
                .font(icons::PHOSPHOR)
                .size(theme::text_base());
            let label_text = text(tab.label()).size(theme::text_sm());
            let tab_content = row![icon_text, label_text]
                .spacing(5)
                .align_y(Alignment::Center);

            // Compute underline opacity with animation
            let underline_alpha = if tab_anim_progress < 1.0 {
                let eased = crate::animation::ease_in_out_quad(tab_anim_progress);
                if idx == tab_anim_to {
                    eased // fade in
                } else if idx == tab_anim_from {
                    1.0 - eased // fade out
                } else {
                    0.0
                }
            } else if is_active {
                1.0
            } else {
                0.0
            };

            let tab_el: Element<Message> = if underline_alpha > 0.01 {
                let alpha = underline_alpha;
                let inner = column![
                    tab_content,
                    container(row![])
                        .width(Length::Fill)
                        .height(Length::Fixed(2.0))
                        .style(move |_theme: &iced::Theme| container::Style {
                            background: Some(iced::Background::Color(
                                Color { a: alpha, ..theme::palette().gold }
                            )),
                            ..Default::default()
                        }),
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
            Tab::PaperTrail   => self.view_paper_trail(),
            Tab::Settings     => self.view_settings(),
        };

        // ── Nav strip: search + tickers + recently viewed ───
        let nav_strip = row![
            search_bar,
            ticker_buttons,
            iced::widget::Space::with_width(Length::Fill),
            recently_viewed_row,
        ]
        .spacing(theme::SPACE_SM)
        .align_y(Alignment::Center);

        // ── Final assembly ──────────────────────────────────
        let content = column![
            header_row,
            horizontal_rule(1),
            nav_strip,
            autocomplete,
            horizontal_rule(1),
            tab_bar,
            horizontal_rule(1),
            tab_content,
        ]
        .spacing(theme::SPACE_SM)
        .padding(theme::SPACE_LG);

        // ── Toast overlay ───────────────────────────────────
        let main_view: Element<'_, Message> = container(scrollable(shared::max_container(content)))
            .width(Length::Fill).height(Length::Fill).into();

        if self.toasts.is_empty() {
            main_view
        } else {
            let now = std::time::Instant::now();
            let toast_col: Column<Message> = self.toasts.iter().fold(
                column![].spacing(4).width(Length::Shrink),
                |col, (msg, expiry)| {
                    // Fade out over last 500ms of toast lifetime
                    let remaining = expiry.saturating_duration_since(now).as_secs_f32();
                    let fade_alpha = if remaining < 0.5 {
                        (remaining / 0.5).max(0.0)
                    } else {
                        1.0
                    };
                    let base_alpha = 0.94 * fade_alpha;
                    col.push(
                        container(
                            text(msg.clone()).size(theme::text_sm()).color(
                                Color { a: fade_alpha, ..theme::palette().ink }
                            ),
                        )
                        .padding([6, 14])
                        .style(move |_theme: &iced::Theme| {
                            let p = theme::palette();
                            container::Style {
                                background: Some(iced::Background::Color(
                                    Color { a: base_alpha, ..p.surface },
                                )),
                                border: iced::Border {
                                    color: Color { a: base_alpha, ..p.rule },
                                    width: 1.0,
                                    radius: 6.0.into(),
                                },
                                ..Default::default()
                            }
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
