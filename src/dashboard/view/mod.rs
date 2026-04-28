mod shared;
mod overview;
mod astrology_tab;
mod universe;
mod fundamentals;
mod research;
mod portfolio_tab;
mod paper_trail;
mod settings;

use iced::widget::{button, column, container, horizontal_rule, mouse_area, row, scrollable, text, text_input, Canvas, Column, Row, Space};
use iced::{Alignment, Color, Element, Length};

use crate::ornaments::{BookSpine, Corner, PageBorderCorner, PageHeaderOrnament};

use crate::animation;
use crate::font;
use crate::icons;
use crate::state::{Dashboard, Message};
use crate::tabs::Tab;
use crate::theme;

impl Dashboard {
    pub fn view(&self) -> Element<'_, Message> {
        // ── Compact navigation row ─────────────────────────────
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

        // ── Ticker buttons ─────────────────────────────────────
        let ticker_buttons: Row<Message> = self.tickers.iter().fold(row![].spacing(6), |r, ticker| {
            let btn = button(text(ticker).size(theme::text_base())).on_press(Message::TickerSelected(ticker.clone()));
            r.push(btn)
        });

        // ── Recently viewed ────────────────────────────────────
        let recently_viewed_row: Element<Message> = if self.recently_viewed.is_empty() {
            row![].into()
        } else {
            let label = text("Recent:").size(theme::text_sm());
            let recent: Vec<_> = self.recently_viewed.iter().rev().take(6).collect();
            let r: Row<Message> = recent.iter().rev().fold(
                row![label].spacing(6),
                |r, t| r.push(
                    button(text(t.as_str()).size(theme::text_xs()))
                        .on_press(Message::TickerSelected((*t).clone()))
                ),
            );
            r.into()
        };

        // ── Header: ticker name + actions ──────────────────────
        let refresh_icon = text(icons::ARROW_REPEAT.to_string())
            .font(icons::PHOSPHOR)
            .size(theme::text_sm());
        let refresh_btn = button(
            if self.refreshing {
                row![refresh_icon, text("…").size(theme::text_sm())].spacing(4).align_y(Alignment::Center)
            } else {
                row![refresh_icon, text("Refresh").size(theme::text_sm())].spacing(4).align_y(Alignment::Center)
            }
        ).on_press(Message::RefreshNow);

        let fetch_btn: Element<Message> = if self.fetching_ticker {
            button(
                row![
                    text(icons::DOWNLOAD.to_string()).font(icons::PHOSPHOR).size(theme::text_sm()),
                    text("…").size(theme::text_sm()),
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

        let theme_label = format!("{}", self.theme_mode.label());
        let compact_nav = column![
            row![
                text(self.selected_ticker.as_str()).font(font::DISPLAY).size(theme::text_2xl()),
                iced::widget::Space::with_width(Length::Fill),
                search_bar,
                ticker_buttons,
                iced::widget::Space::with_width(Length::Fill),
                refresh_btn,
                fetch_btn,
                button(text(theme_label).size(theme::text_xs())).on_press(Message::ToggleTheme),
            ]
            .spacing(theme::SPACE_SM)
            .align_y(Alignment::Center),
            recently_viewed_row,
        ]
        .spacing(theme::SPACE_XS);

        // ── Tab content dispatch ───────────────────────────────
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

        // ── Page transition alpha ──────────────────────────────
        let page_alpha = if self.page_transition_progress < 1.0 {
            animation::ease_out_cubic(self.page_transition_progress)
        } else {
            1.0
        };

        // ── Book page content area with ornaments ──────────────
        let corner_top = row![
            Canvas::new(PageBorderCorner { corner: Corner::TopLeft })
                .width(Length::Fixed(20.0)).height(Length::Fixed(20.0)),
            Space::with_width(Length::Fill),
            Canvas::new(PageBorderCorner { corner: Corner::TopRight })
                .width(Length::Fixed(20.0)).height(Length::Fixed(20.0)),
        ];
        let corner_bottom = row![
            Canvas::new(PageBorderCorner { corner: Corner::BottomLeft })
                .width(Length::Fixed(20.0)).height(Length::Fixed(20.0)),
            Space::with_width(Length::Fill),
            Canvas::new(PageBorderCorner { corner: Corner::BottomRight })
                .width(Length::Fixed(20.0)).height(Length::Fixed(20.0)),
        ];

        let page_content = column![
            corner_top,
            Canvas::new(PageHeaderOrnament)
                .width(Length::Fill).height(Length::Fixed(28.0)),
            compact_nav,
            autocomplete,
            horizontal_rule(1),
            tab_content,
            corner_bottom,
        ]
        .spacing(theme::SPACE_XS);

        let book_page: Element<Message> = container(scrollable(page_content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(theme::SPACE_MD as u16)
            .style(move |_theme: &iced::Theme| {
                let p = theme::palette();
                container::Style {
                    background: Some(iced::Background::Color(
                        Color { a: 0.4 + 0.6 * page_alpha, ..p.bg }
                    )),
                    border: iced::Border {
                        color: p.rule,
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into();

        // ── Book spine + page ──────────────────────────────────
        let spine = Canvas::new(BookSpine)
            .width(Length::Fixed(20.0))
            .height(Length::Fill);

        // ── Right-side grimoire tabs ───────────────────────────
        let grimoire_tabs = self.build_grimoire_tabs();

        // ── Final assembly: spine + page + tabs on dark frame ──
        let book_layout = row![spine, book_page, grimoire_tabs];

        let main_view: Element<'_, Message> = shared::outer_frame(book_layout);

        // ── Toast overlay ──────────────────────────────────────
        if self.toasts.is_empty() {
            main_view
        } else {
            let now = std::time::Instant::now();
            let toast_col: Column<Message> = self.toasts.iter().fold(
                column![].spacing(4).width(Length::Shrink),
                |col, (msg, expiry)| {
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

    /// Build the right-side grimoire tab dividers — the book's physical tabs.
    fn build_grimoire_tabs(&self) -> Element<'_, Message> {
        let tabs_col: Column<Message> = Tab::all().iter().enumerate().fold(
            column![].spacing(2),
            |col, (idx, &tab)| {
                let is_active = tab == self.active_tab;
                let progress = self.tab_hover_progress[idx];
                let eased = animation::ease_out_back(progress);

                // Dynamic width: 48px collapsed → 168px expanded
                let tab_width = animation::lerp(48.0, 168.0, eased);

                // Icon
                let icon_font = if is_active { icons::PHOSPHOR_BOLD } else { icons::PHOSPHOR };
                let icon_size = if is_active { theme::text_lg() } else { theme::text_md() };
                let icon_el = text(tab.icon().to_string())
                    .font(icon_font)
                    .size(icon_size);

                // Build content: icon only or icon + label
                let tab_content: Element<Message> = if progress > 0.05 {
                    let label_alpha = progress.min(1.0);
                    let p = theme::palette();
                    let label = text(tab.label())
                        .font(font::DISPLAY)
                        .size(theme::text_sm())
                        .color(Color { a: label_alpha, ..p.ink });
                    row![icon_el, label]
                        .spacing(8)
                        .align_y(Alignment::Center)
                        .into()
                } else {
                    container(icon_el)
                        .center_x(Length::Fill)
                        .into()
                };

                // Tab styling
                let p = theme::palette();
                let tab_bg = if is_active {
                    p.bg  // matches page — visual continuity
                } else {
                    Color {
                        r: p.surface.r * 0.92,
                        g: p.surface.g * 0.92,
                        b: p.surface.b * 0.92,
                        a: p.surface.a,
                    }
                };
                let gold_accent = is_active || progress > 0.1;
                let gold_width = if gold_accent { 3.0 } else { 0.0 };

                // Stagger offset — lower tabs protrude more (book divider cascade)
                let stagger = (idx as f32) * 3.0;

                let styled_tab: Element<Message> = container(tab_content)
                    .width(Length::Fixed(tab_width))
                    .height(Length::Fixed(44.0))
                    .padding([8, 10])
                    .center_y(Length::Fill)
                    .style(move |_theme: &iced::Theme| {
                        container::Style {
                            background: Some(iced::Background::Color(tab_bg)),
                            border: iced::Border {
                                color: if gold_accent { p.gold } else { p.rule },
                                width: gold_width,
                                radius: 4.0.into(), // rounded corners
                            },
                            ..Default::default()
                        }
                    })
                    .into();

                // Wrap in button for click + mouse_area for hover
                let clickable = button(styled_tab)
                    .on_press(Message::TabSelected(tab))
                    .padding(0);

                let hoverable: Element<Message> = mouse_area(
                    container(clickable)
                        .padding(iced::Padding { top: 0.0, right: 0.0, bottom: 0.0, left: stagger })
                )
                .on_enter(Message::TabHoverEnter(tab))
                .on_exit(Message::TabHoverExit(tab))
                .into();

                col.push(hoverable)
            },
        );

        // Wrap tab column in dark background strip
        container(
            column![
                iced::widget::Space::with_height(Length::Fixed(theme::SPACE_XL)),
                tabs_col,
                iced::widget::Space::with_height(Length::Fill),
            ]
        )
        .width(Length::Shrink)
        .height(Length::Fill)
        .padding([0, theme::SPACE_XS as u16])
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: Some(iced::Background::Color(
                    Color { a: 0.5, ..theme::grimoire_outer_bg() }
                )),
                ..Default::default()
            }
        })
        .into()
    }
}
