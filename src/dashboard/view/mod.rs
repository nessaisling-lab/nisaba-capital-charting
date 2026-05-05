pub(crate) mod shared;
mod overview;
mod astrology_tab;
mod universe;
mod fundamentals;
mod research;
mod portfolio_tab;
mod paper_trail;
mod settings;

use iced::widget::{button, column, container, mouse_area, row, rule, scrollable, stack, text, text_input, Canvas, Column, Row, Shader, Space};
use iced::{Alignment, Color, Element, Length};

use crate::ornaments::{BookSpine, Corner, PageBorderCorner, PageHeaderOrnament, TabSparkle};

use crate::animation;
use crate::font;
use crate::icons;
use crate::state::{Dashboard, Message};
use crate::tabs::Tab;
use crate::theme;

impl Dashboard {
    pub fn view(&self) -> Element<'_, Message> {
        // ── v11.1: Redesigned navigation ──────────────────────────
        // Row 1: [Search (wider)] [Ticker Name (center)] [Icon actions (right)]
        // Row 2: [Ticker buttons + Recently viewed]
        let search_bar = row![
            text_input("Search any ticker…", &self.ticker_search_input)
                .id(crate::update::SEARCH_INPUT_ID)
                .on_input(Message::TickerSearchInput)
                .on_submit(Message::TickerSearchSubmit)
                .width(Length::Fixed(280.0))
                .size(theme::text_base()),
            button(
                text(icons::SEARCH.to_string()).font(icons::PHOSPHOR).size(theme::text_sm())
            ).on_press(Message::TickerSearchSubmit),
        ]
        .spacing(4)
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

        // Icon-only action buttons (v11.1: no text labels)
        let refresh_btn = button(
            if self.refreshing {
                row![
                    text(icons::ARROW_REPEAT.to_string()).font(icons::PHOSPHOR).size(theme::text_md()),
                    text("\u{2026}").size(theme::text_xs()),
                ].spacing(2).align_y(Alignment::Center)
            } else {
                row![
                    text(icons::ARROW_REPEAT.to_string()).font(icons::PHOSPHOR).size(theme::text_md()),
                ].spacing(0).align_y(Alignment::Center)
            }
        ).on_press(Message::RefreshNow);

        let fetch_btn: Element<Message> = if self.fetching_ticker {
            button(
                row![
                    text(icons::DOWNLOAD.to_string()).font(icons::PHOSPHOR).size(theme::text_md()),
                    text("\u{2026}").size(theme::text_xs()),
                ].spacing(2).align_y(Alignment::Center)
            ).into()
        } else {
            button(
                text(icons::DOWNLOAD.to_string()).font(icons::PHOSPHOR).size(theme::text_md()),
            ).on_press(Message::FetchThisTicker).into()
        };

        let theme_btn = button(
            text(icons::MOON_STARS.to_string()).font(icons::PHOSPHOR).size(theme::text_md()),
        ).on_press(Message::ToggleTheme);

        let action_icons = row![refresh_btn, fetch_btn, theme_btn]
            .spacing(4)
            .align_y(Alignment::Center);

        // Ticker + price + day high/low (v11.3 — pulled from last PriceRow)
        let p_hdr = theme::palette();
        let ticker_block: Element<Message> = if let Some(last) = self.rows.last() {
            let close: f64 = last.close.to_string().parse().unwrap_or(0.0);
            let high:  f64 = last.high.to_string().parse().unwrap_or(0.0);
            let low:   f64 = last.low.to_string().parse().unwrap_or(0.0);
            row![
                text(self.selected_ticker.as_str())
                    .font(font::DISPLAY).size(theme::text_2xl()).color(p_hdr.gold),
                text(format!("${close:.2}"))
                    .font(font::DISPLAY).size(theme::text_lg()),
                text(format!("H ${high:.2}"))
                    .size(theme::text_xs()).color(p_hdr.ink_soft),
                text(format!("L ${low:.2}"))
                    .size(theme::text_xs()).color(p_hdr.ink_soft),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
        } else {
            text(self.selected_ticker.as_str())
                .font(font::DISPLAY).size(theme::text_2xl()).color(p_hdr.gold).into()
        };

        // Row 1: search left, ticker+price center, actions right
        let nav_row_1 = row![
            search_bar,
            Space::new().width(Length::Fill),
            ticker_block,
            Space::new().width(Length::Fill),
            action_icons,
        ]
        .spacing(theme::SPACE_SM)
        .align_y(Alignment::Center);

        // Row 2: ticker DB buttons + recently viewed
        let ticker_buttons: Row<Message> = self.tickers.iter().fold(row![].spacing(4), |r, ticker| {
            r.push(
                button(text(ticker).size(theme::text_xs()))
                    .on_press(Message::TickerSelected(ticker.clone()))
            )
        });

        let recently_viewed_row: Element<Message> = if self.recently_viewed.is_empty() {
            row![].into()
        } else {
            let recent: Vec<_> = self.recently_viewed.iter().rev().take(6).collect();
            let r: Row<Message> = recent.iter().rev().fold(
                row![text("Recent:").size(theme::text_xs()).color(theme::palette().ink_soft)].spacing(4),
                |r, t| r.push(
                    button(text(t.as_str()).size(theme::text_xs()))
                        .on_press(Message::TickerSelected((*t).clone()))
                ),
            );
            r.into()
        };

        let nav_row_2 = row![ticker_buttons, Space::new().width(Length::Fixed(16.0)), recently_viewed_row]
            .spacing(4)
            .align_y(Alignment::Center);

        let compact_nav = column![nav_row_1, nav_row_2]
            .spacing(theme::SPACE_XS);

        // ── Fetch error banner (v7.5) ──────────────────────────
        let fetch_error_banner: Element<Message> = if let Some(err) = &self.fetch_ticker_error {
            let _p_err = theme::palette();
            container(
                text(format!("\u{26A0} {err}")).size(theme::text_sm())
                    .color(Color { r: 0.95, g: 0.6, b: 0.2, a: 1.0 }),
            )
            .padding([4, 8])
            .width(Length::Fill)
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(iced::Background::Color(
                        Color { r: 0.3, g: 0.15, b: 0.0, a: 0.25 },
                    )),
                    border: iced::Border {
                        color: Color { r: 0.6, g: 0.3, b: 0.0, a: 0.4 },
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into()
        } else if self.fetching_ticker {
            // v11.3: Determinate progress bar — time-based fill capped at 0.85
            // until the FetchTickerComplete message arrives. Expected duration ~30s.
            let p_load = theme::palette();
            let progress: f32 = self.fetch_start_time
                .map(|start| {
                    let elapsed = start.elapsed().as_secs_f32();
                    (elapsed / 30.0).min(0.85)
                })
                .unwrap_or(0.0);
            // Outer track + inner fill via a row with FillPortion
            let fill_pct = (progress * 100.0).round() as u16;
            let rest_pct = 100u16.saturating_sub(fill_pct);
            let fill_bar = container(Space::new())
                .width(Length::FillPortion(fill_pct.max(1)))
                .height(Length::Fixed(3.0))
                .style(move |_theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(p_load.gold)),
                    ..Default::default()
                });
            let track_rest = container(Space::new())
                .width(Length::FillPortion(rest_pct.max(1)))
                .height(Length::Fixed(3.0))
                .style(move |_theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(
                        Color { a: 0.15, ..p_load.gold },
                    )),
                    ..Default::default()
                });
            row![fill_bar, track_rest].width(Length::Fill).into()
        } else {
            Space::new().height(Length::Fixed(0.0)).into()
        };

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

        // ── Page transition: layered stagger (v9.0) ────────────
        // Background + gold glow settle in first ~100ms (3× speed),
        // then remaining animations complete over full 300ms duration.
        // Visual effect: background snaps in fast, content follows.
        let progress = self.page_transition_progress;
        let page_alpha = if progress < 1.0 {
            animation::ease_out_cubic((progress * 3.0).min(1.0))
        } else {
            1.0
        };

        // ── Book page content area with ornaments ──────────────
        let corner_top = row![
            Canvas::new(PageBorderCorner { corner: Corner::TopLeft })
                .width(Length::Fixed(20.0)).height(Length::Fixed(20.0)),
            Space::new().width(Length::Fill),
            Canvas::new(PageBorderCorner { corner: Corner::TopRight })
                .width(Length::Fixed(20.0)).height(Length::Fixed(20.0)),
        ];
        let corner_bottom = row![
            Canvas::new(PageBorderCorner { corner: Corner::BottomLeft })
                .width(Length::Fixed(20.0)).height(Length::Fixed(20.0)),
            Space::new().width(Length::Fill),
            Canvas::new(PageBorderCorner { corner: Corner::BottomRight })
                .width(Length::Fixed(20.0)).height(Length::Fixed(20.0)),
        ];

        // ── Horizontal tab bar (v7.4.1) ──────────────────────
        let tab_bar = self.build_tab_bar();

        let page_content = column![
            corner_top,
            compact_nav,
            fetch_error_banner,
            autocomplete,
            Canvas::new(PageHeaderOrnament)
                .width(Length::Fill).height(Length::Fixed(28.0)),
            tab_bar,
            rule::horizontal(1),
            tab_content,
            corner_bottom,
        ]
        .spacing(theme::SPACE_XS)
        .padding(iced::Padding { top: 0.0, right: 20.0, bottom: 0.0, left: 0.0 }); // v9.3: right gutter for scrollbar (was 10)

        // Themed scrollbar — gold scroller on translucent rail (v7.5)
        let styled_scroll = scrollable(page_content)
            .style(shared::gold_scrollbar_style);

        let book_page: Element<Message> = container(styled_scroll)
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

        // ── Final assembly: spine + page on GPU vignette ──
        let book_layout = row![spine, book_page];

        // GPU atmospheric background (v7.4)
        let p_view = theme::palette();
        let bg = theme::grimoire_outer_bg();
        let vignette = Shader::new(crate::shaders::VignetteProgram {
            time: self.shader_time,
            page_alpha: if self.page_transition_progress < 1.0 {
                animation::ease_out_cubic(self.page_transition_progress)
            } else {
                1.0
            },
            bg_color: [bg.r, bg.g, bg.b, bg.a],
            gold_color: [p_view.gold.r, p_view.gold.g, p_view.gold.b, p_view.gold.a],
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let padded_book = container(book_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(theme::SPACE_SM as u16);

        let main_view: Element<'_, Message> = stack![vignette, padded_book]
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

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

    /// Build the horizontal tab bar — grimoire chapter headers (v7.4.1).
    fn build_tab_bar(&self) -> Element<'_, Message> {
        let tab_row: Row<Message> = Tab::all().iter().enumerate().fold(
            row![].spacing(2).align_y(Alignment::Center),
            |r, (idx, &tab)| {
                let is_active = tab == self.active_tab;
                let progress = self.tab_hover_progress[idx];
                let eased = animation::ease_out_cubic(progress);

                // Icon — bold + gold glow when active (v9.2)
                let icon_font = if is_active { icons::PHOSPHOR_BOLD } else { icons::PHOSPHOR };
                let p = theme::palette();
                let icon_color = if is_active { p.gold } else { p.ink };
                let icon_el = text(tab.icon().to_string())
                    .font(icon_font)
                    .size(theme::text_md())
                    .color(icon_color);

                // Active tab: gold icon + label + subtle sparkle shimmer.
                // Hover: label + sparkle fade in.
                let tab_content: Element<Message> = if is_active {
                    let label = text(tab.label())
                        .font(font::DISPLAY_BOLD)
                        .size(16.0)  // slightly larger than text_sm (14) for active emphasis
                        .color(p.gold);
                    // Persistent subtle shimmer on active tab (2 particles, low alpha)
                    let active_sparkle = Canvas::new(TabSparkle {
                        alpha: 0.4,  // subtle persistent glow
                        seed: idx as u32 + 100,  // offset seed from hover sparkle
                    })
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(14.0));
                    row![icon_el, label, active_sparkle]
                        .spacing(5)
                        .align_y(Alignment::Center)
                        .into()
                } else if progress > 0.05 {
                    let label_alpha = eased.min(1.0);
                    let sparkle_alpha = (eased * 1.5 - 0.3).clamp(0.0, 1.0);
                    let label = text(tab.label())
                        .font(font::DISPLAY)
                        .size(theme::text_sm())
                        .color(Color { a: label_alpha, ..p.ink });
                    let sparkle_canvas = Canvas::new(TabSparkle {
                        alpha: sparkle_alpha,
                        seed: idx as u32,
                    })
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(16.0));
                    row![icon_el, label, sparkle_canvas]
                        .spacing(4)
                        .align_y(Alignment::Center)
                        .into()
                } else {
                    icon_el.into()
                };

                // v11.1: Bookmark-tab shape — gold border (no bg fill), rounded top, square bottom
                let (tab_bg, border_width, border_color) = if is_active {
                    (Color::TRANSPARENT, 2.0, p.gold)
                } else if progress > 0.1 {
                    (Color::TRANSPARENT, 1.0, Color { a: eased * 0.3, ..p.gold })
                } else {
                    (Color::TRANSPARENT, 0.0, Color::TRANSPARENT)
                };

                let styled_tab: Element<Message> = container(tab_content)
                    .padding([6, 14])
                    .center_y(Length::Shrink)
                    .style(move |_theme: &iced::Theme| {
                        container::Style {
                            background: Some(iced::Background::Color(tab_bg)),
                            border: iced::Border {
                                color: border_color,
                                width: border_width,
                                radius: iced::border::Radius {
                                    top_left: 5.0, top_right: 5.0,
                                    bottom_right: 0.0, bottom_left: 0.0,
                                }, // bookmark: rounded top, flat bottom
                            },
                            ..Default::default()
                        }
                    })
                    .into();

                // Wrap in button for click + mouse_area for hover
                // Transparent button style so inner container styling shows through
                let clickable = button(styled_tab)
                    .on_press(Message::TabSelected(tab))
                    .padding(0)
                    .style(|_theme: &iced::Theme, _status| {
                        iced::widget::button::Style {
                            background: None,
                            border: iced::Border::default(),
                            text_color: Color::TRANSPARENT,
                            ..Default::default()
                        }
                    });

                let hoverable: Element<Message> = mouse_area(clickable)
                    .on_enter(Message::TabHoverEnter(tab))
                    .on_exit(Message::TabHoverExit(tab))
                    .into();

                r.push(hoverable)
            },
        );

        // Wrap in container — full width under header ornament
        container(tab_row)
            .width(Length::Fill)
            .padding([0, theme::SPACE_XS as u16])
            .into()
    }
}
