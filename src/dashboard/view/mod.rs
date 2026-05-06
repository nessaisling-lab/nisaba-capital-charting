pub(crate) mod shared;
mod overview;
mod astrology_tab;
mod universe;
mod fundamentals;
mod research;
mod portfolio_tab;
mod paper_trail;
mod settings;
mod encyclopedia;

use iced::widget::{button, column, container, mouse_area, pick_list, row, rule, scrollable, stack, text, text_input, Canvas, Column, Row, Shader, Space};
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
                .width(Length::Fixed(theme::sw(280.0)))
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

        // v11.5.F6 — drop "…" suffix on fetch_btn while fetching; the
        // progress bar below the nav row already conveys activity.
        let fetch_btn: Element<Message> = if self.fetching_ticker {
            button(
                text(icons::DOWNLOAD.to_string()).font(icons::PHOSPHOR).size(theme::text_md()),
            ).into()
        } else {
            button(
                text(icons::DOWNLOAD.to_string()).font(icons::PHOSPHOR).size(theme::text_md()),
            ).on_press(Message::FetchThisTicker).into()
        };

        let theme_btn = button(
            text(icons::MOON_STARS.to_string()).font(icons::PHOSPHOR).size(theme::text_md()),
        ).on_press(Message::ToggleTheme);

        // v11.8.H — Settings moved to corner_top (top edge of page chrome).
        // User couldn't find gear icon among 4 small action icons even with
        // v11.8.A bold + label. Placing in the page edge above the tab
        // strip makes it the highest-contrast button on screen.
        let action_icons = row![refresh_btn, fetch_btn, theme_btn]
            .spacing(6)
            .align_y(Alignment::Center);

        // ── v11.6.A v2 — Hero ticker block ───────────────────────
        // STAR (favorites toggle) + INFO (encyclopedia jump) sit side-by-side
        // before ticker name. Star is filled gold when current ticker is a
        // favorite. Info opens the encyclopedia view for this ticker.
        let p_hdr = theme::palette();
        let is_fav = self.favorites.iter().any(|t| t == &self.selected_ticker);
        let star_color = if is_fav { p_hdr.gold } else { Color { a: 0.4, ..p_hdr.gold } };
        let transparent_btn_style = |_t: &iced::Theme, _s| button::Style {
            background: None,
            text_color: Color::TRANSPARENT,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            snap: false,
        };
        let star_btn = button(
            text(icons::STAR.to_string())
                .font(if is_fav { icons::PHOSPHOR_BOLD } else { icons::PHOSPHOR })
                .size(theme::text_md())
                .color(star_color),
        )
        .on_press(Message::ToggleFavorite(self.selected_ticker.clone()))
        .padding(0)
        .style(transparent_btn_style);
        let info_btn = button(
            text(icons::INFO_CIRCLE.to_string())
                .font(icons::PHOSPHOR)
                .size(theme::text_md())
                .color(p_hdr.gold),
        )
        .on_press(Message::TabSelected(crate::tabs::Tab::Encyclopedia))
        .padding(0)
        .style(transparent_btn_style);
        let hl_stack: Element<Message> = if let Some(last) = self.rows.last() {
            let high: f64 = last.high.to_string().parse().unwrap_or(0.0);
            let low:  f64 = last.low.to_string().parse().unwrap_or(0.0);
            column![
                row![
                    text("H ").size(theme::text_xs()).color(p_hdr.ink_soft),
                    text(format!("${high:.2}")).size(theme::text_xs()).color(p_hdr.ink_soft),
                ],
                row![
                    text("L ").size(theme::text_xs()).color(p_hdr.ink_soft),
                    text(format!("${low:.2}")).size(theme::text_xs()).color(p_hdr.ink_soft),
                ],
            ]
            .spacing(2)
            .into()
        } else {
            Space::new().into()
        };
        // v11.6.A v3 — sandwich: star LEFT, info RIGHT of the price block
        let ticker_block: Element<Message> = if let Some(last) = self.rows.last() {
            let close: f64 = last.close.to_string().parse().unwrap_or(0.0);
            row![
                star_btn,
                text(self.selected_ticker.as_str())
                    .font(font::DISPLAY).size(theme::text_2xl()).color(p_hdr.gold),
                text(format!("${close:.2}"))
                    .font(font::DISPLAY).size(theme::text_lg()),
                hl_stack,
                info_btn,
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
        } else {
            row![
                star_btn,
                text(self.selected_ticker.as_str())
                    .font(font::DISPLAY).size(theme::text_2xl()).color(p_hdr.gold),
                info_btn,
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
        };

        // v11.6.A — Favorites + Recent dropdowns (side-by-side under search)
        let favorites_dd: Element<Message> = if self.favorites.is_empty() {
            container(
                text("\u{2606} no favorites").size(theme::text_xs())
                    .color(theme::palette().ink_soft),
            )
            .padding([5, 10])
            .width(Length::Fill)
            .into()
        } else {
            pick_list(
                self.favorites.clone(),
                None::<String>,
                Message::TickerSelected,
            )
            .placeholder("\u{2605} Favorites")
            .text_size(theme::text_xs())
            .width(Length::Fill)
            .into()
        };

        let recent_dd: Element<Message> = if self.recently_viewed.is_empty() {
            container(
                text("\u{21BA} no recent").size(theme::text_xs())
                    .color(theme::palette().ink_soft),
            )
            .padding([5, 10])
            .width(Length::Fill)
            .into()
        } else {
            pick_list(
                self.recently_viewed.clone(),
                None::<String>,
                Message::TickerSelected,
            )
            .placeholder("\u{21BA} Recent")
            .text_size(theme::text_xs())
            .width(Length::Fill)
            .into()
        };

        // v11.6.A — right column: search+actions row, then fav+recent row.
        let search_action_row = row![
            search_bar,
            action_icons,
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let fav_recent_row = row![
            favorites_dd,
            recent_dd,
        ]
        .spacing(6);

        let right_column = column![
            search_action_row,
            fav_recent_row,
        ]
        .spacing(6)
        .width(Length::Fixed(theme::sw(420.0)));

        // v11.6.A — body row: hero ticker (fills) + right_column (fixed)
        let header_body = row![
            container(ticker_block)
                .padding([theme::SPACE_SM as u16, theme::SPACE_SM as u16])
                .width(Length::Fill),
            right_column,
        ]
        .spacing(theme::SPACE_MD)
        .align_y(Alignment::Center);

        let compact_nav = column![header_body]
            .spacing(theme::SPACE_XS);

        // ── v12.1: push-down banner removed ───────────────────
        // Errors + fetch progress now render as pills in the tab strip
        // via the universal notification system (see build_tab_bar).
        // The page header layout no longer reflows on fetch.

        // ── Tab content dispatch ───────────────────────────────
        let tab_content: Element<Message> = match self.active_tab {
            Tab::Astrology    => self.view_astrology(),
            Tab::Overview     => self.view_overview(),
            Tab::Universe     => self.view_universe(),
            Tab::Fundamentals => self.view_fundamentals(),
            Tab::Research     => self.view_research(),
            Tab::Portfolio    => self.view_portfolio(),
            Tab::PaperTrail   => self.view_paper_trail(),
            Tab::Encyclopedia => self.view_encyclopedia(),
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
        // v11.8.I — corner_top reverted to original spec. Settings gear
        // moved into tab_bar row instead (per video review: "use a gear
        // icon for that button and place it in the top right corner in
        // the same row as the main tabs").
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

        // v11.6.A — Tab strip moves to very top, then ornament, then header
        let page_content = column![
            corner_top,
            tab_bar,
            rule::horizontal(1),
            compact_nav,
            autocomplete,
            Canvas::new(PageHeaderOrnament)
                .width(Length::Fill).height(Length::Fixed(20.0)),
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

        let base_view: Element<'_, Message> = stack![vignette, padded_book]
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        // v11.9 (revised) — loading bar reverted to inline (in fetch
        // banner area below nav row); no overlay needed.
        // v12.2.4 — drawer overlay layered above base_view via stack!.
        // Anchored top-right under the bell icon. Position fixed so it
        // floats over content without affecting layout.
        let main_view: Element<'_, Message> = if self.notifications_drawer_open {
            let drawer = crate::notifications::render_drawer(&self.notification_history);
            let drawer_anchored = container(drawer)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(iced::alignment::Vertical::Top)
                .padding(iced::Padding {
                    top: 60.0, right: 60.0, bottom: 0.0, left: 0.0,
                });
            stack![base_view, drawer_anchored]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            base_view
        };

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
            // v11.9 (revised) — toast as stack overlay (was column[toast,
            // main]) so it never pushes content down. User flagged the
            // top-whitespace bug: toasts in a column flow consume layout
            // space; stack overlay floats them above main_view instead.
            let toast_overlay = container(toast_col)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(iced::alignment::Vertical::Top)
                .padding([12, 18]);

            stack![main_view, toast_overlay]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
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

        // v11.9 — Settings gear in tab row clicks to switch tab (panel,
        // not modal). User: "no overlay or pop, just like before iced 0.14."
        let p_gear = theme::palette();
        let gear_btn = button(
            text(icons::GEAR.to_string())
                .font(icons::PHOSPHOR_BOLD)
                .size(theme::text_md())
                .color(p_gear.gold),
        )
        .on_press(Message::TabSelected(Tab::Settings))
        .padding([4, 8])
        .style(|_t: &iced::Theme, status: button::Status| {
            let p = theme::palette();
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => Color { a: 0.18, ..p.gold },
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: Color::TRANSPARENT,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: false,
            }
        });

        // ── v12.1: Universal pill notification stack ─────────
        // Replaces v11.9 fetching_pill + alert_pill ad-hoc chrome with
        // a single deque-driven pill row. Fetch progress, Lagrange
        // alerts, transit events, errors, success — all render here.
        // The page header layout never reflows on emit.
        let pill_stack = crate::notifications::render_pill_stack(
            &self.notifications,
            self.shader_time,
        );

        // ── v12.2.4: bell icon + drawer toggle ─────────
        // Click bell → open drawer overlay. Badge count = total entries
        // in notification_history (capped at MAX_HISTORY).
        let p_bell = theme::palette();
        let history_count = self.notification_history.len();
        let bell_glyph = if history_count > 0 {
            row![
                text(icons::BELL.to_string())
                    .font(icons::PHOSPHOR_BOLD)
                    .size(theme::text_md())
                    .color(p_bell.gold),
                text(format!("{}", history_count.min(99)))
                    .size(9.0)
                    .color(Color { r: 0.95, g: 0.90, b: 0.80, a: 1.0 }),
            ]
            .spacing(2)
            .align_y(Alignment::Center)
        } else {
            row![text(icons::BELL.to_string())
                .font(icons::PHOSPHOR_BOLD)
                .size(theme::text_md())
                .color(Color { a: 0.55, ..p_bell.gold })]
        };
        let bell_btn = button(bell_glyph)
            .on_press(Message::ToggleNotificationDrawer)
            .padding([4, 8])
            .style(|_t: &iced::Theme, status: button::Status| {
                let p = theme::palette();
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => Color { a: 0.18, ..p.gold },
                    _ => Color::TRANSPARENT,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: Color::TRANSPARENT,
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    snap: false,
                }
            });

        let full_strip = row![
            tab_row,
            Space::new().width(Length::Fill),
            pill_stack,
            Space::new().width(Length::Fixed(6.0)),
            bell_btn,
            Space::new().width(Length::Fixed(4.0)),
            gear_btn,
        ]
        .align_y(Alignment::Center);

        container(full_strip)
            .width(Length::Fill)
            .padding([0, theme::SPACE_XS as u16])
            .into()
    }
}
