mod shared;
mod overview;
mod astrology_tab;
mod universe;
mod fundamentals;
mod research;
mod portfolio_tab;
mod settings;

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input, Row};
use iced::{Alignment, Element, Length};

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
        let refresh_label = if self.refreshing { "Refreshing..." } else { "Refresh Now" };

        // ── Tab bar ─────────────────────────────────────────
        let tab_bar: Row<Message> = Tab::all().iter().fold(row![].spacing(4), |r, &tab| {
            let label = if tab == self.active_tab {
                format!("[{}]", tab.label())
            } else {
                tab.label().to_string()
            };
            let btn: Element<Message> = button(text(label).size(theme::text_base()))
                .on_press(Message::TabSelected(tab))
                .into();
            r.push(btn)
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
