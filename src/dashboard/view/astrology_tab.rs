use iced::widget::canvas::Canvas;
use iced::widget::{button, column, container, horizontal_rule, row, text, text_input, Column, Shader};
use iced::{Alignment, Element, Length};

use crate::astrology::{build_transits_section, build_wheel_legend};
use crate::calendar::AstroCalendar;
use crate::shaders::NatalWheel3DProgram;
use crate::font;
use crate::state::{Dashboard, Message};
use super::shared::{eyebrow, section_rule};
use crate::strategy::Condition;
use crate::theme;

impl Dashboard {
    pub(crate) fn view_astrology(&self) -> Element<'_, Message> {
        // ── Natal wheel + transits + horoscope ──────────────
        let moon_phase = self
            .astro_score
            .as_ref()
            .and_then(|s| s.moon_phase.as_deref());
        let moon_deg = self.astro_score.as_ref().and_then(|s| s.moon_phase_deg);
        let mercury_rx = self
            .astro_score
            .as_ref()
            .and_then(|s| s.mercury_rx)
            .unwrap_or(false);

        let astrology_section: Element<Message> = if self.natal_positions.is_empty() {
            column![
                text(format!("{} Astrology", self.selected_ticker)).font(font::DISPLAY).size(theme::text_lg()),
                horizontal_rule(1),
                text(format!("No birth chart yet for {}.", self.selected_ticker)).size(theme::text_base()),
                text("The scraper enriches ~50 tickers per day via SEC EDGAR.").size(theme::text_sm()),
                text("Once an IPO date is found, the natal chart is computed automatically.")
                    .size(theme::text_sm()),
            ]
            .spacing(6)
            .into()
        } else {
            let p = theme::palette();
            // Active zodiac sign from Sun's transit longitude (first transit = Sun)
            let active_sign = self.daily_transits.first()
                .map(|sun| (sun.longitude as f32 / 30.0).floor().clamp(0.0, 11.0))
                .unwrap_or(0.0);

            let natal_wheel = Shader::new(NatalWheel3DProgram {
                time: self.shader_time,
                natal_positions: self.natal_positions.clone(),
                transit_positions: self.daily_transits.clone(),
                bg_color: [p.bg.r, p.bg.g, p.bg.b, p.bg.a],
                gold_color: [
                    theme::NATAL_GOLD.r, theme::NATAL_GOLD.g,
                    theme::NATAL_GOLD.b, theme::NATAL_GOLD.a,
                ],
                transit_color: [
                    theme::TRANSIT_BLUE.r, theme::TRANSIT_BLUE.g,
                    theme::TRANSIT_BLUE.b, theme::TRANSIT_BLUE.a,
                ],
                retro_color: [
                    theme::RETROGRADE_RED.r, theme::RETROGRADE_RED.g,
                    theme::RETROGRADE_RED.b, theme::RETROGRADE_RED.a,
                ],
                active_sign,
            })
            .width(Length::Fixed(400.0))
            .height(Length::Fixed(400.0));

            // v11.0: Sun/Moon/Rising "Big Three" summary
            let big_three_row: Element<Message> = {
                let sun_sign = self.natal_positions.iter()
                    .find(|p| p.planet == "Sun")
                    .map(|p| p.sign.as_str())
                    .unwrap_or("—");
                let moon_sign = self.natal_positions.iter()
                    .find(|p| p.planet == "Moon")
                    .map(|p| p.sign.as_str())
                    .unwrap_or("—");
                let rising_sign = self.natal_angles.as_ref()
                    .map(|a| {
                        let (sign, _) = pursuit_week4_automation::astrology::ephemeris::longitude_to_sign(a.ascendant);
                        sign
                    })
                    .unwrap_or("—");

                let gold = theme::palette().gold;
                row![
                    text(format!("☉ Sun: {sun_sign}")).font(font::INTER).size(theme::text_base()).color(gold),
                    text("  ·  ").size(theme::text_base()),
                    text(format!("☽ Moon: {moon_sign}")).font(font::INTER).size(theme::text_base()).color(gold),
                    text("  ·  ").size(theme::text_base()),
                    text(format!("↑ Rising: {rising_sign}")).font(font::INTER).size(theme::text_base()).color(gold),
                ]
                .spacing(0)
                .align_y(Alignment::Center)
                .into()
            };

            let wheel_col = column![
                text(format!("{} Birth Chart", self.selected_ticker)).font(font::DISPLAY).size(theme::text_lg()),
                big_three_row,
                natal_wheel,
                build_wheel_legend(),
            ]
            .spacing(4)
            .align_x(Alignment::Center);

            let transits_col =
                column![build_transits_section(
                    &self.astro_aspects,
                    moon_phase,
                    moon_deg,
                    mercury_rx
                ),]
                .width(Length::Fill);

            // Horoscope reading
            let horoscope_section: Element<Message> = if let Some(ref h) = self.horoscope {
                let key_transit_items: Vec<Element<Message>> = h
                    .key_transits
                    .iter()
                    .map(|t| {
                        row![
                            text(&t.transit_desc)
                                .size(theme::text_sm())
                                .width(Length::Fixed(180.0)),
                            text(&t.strength)
                                .size(theme::text_xs())
                                .width(Length::Fixed(120.0)),
                            text(&t.financial_implication)
                                .size(theme::text_xs())
                                .width(Length::Fill),
                        ]
                        .spacing(8)
                        .into()
                    })
                    .collect();

                let mercury_line: Element<Message> = if let Some(ref warn) = h.mercury_warning {
                    text(format!("Mercury: {warn}"))
                        .size(theme::text_sm())
                        .color(theme::ZONE_UNFAVORABLE)
                        .into()
                } else {
                    text("Mercury: Direct — clear communications")
                        .size(theme::text_sm())
                        .into()
                };

                column![
                    horizontal_rule(1),
                    text("Horoscope Reading").font(font::DISPLAY).size(theme::text_lg()),
                    text(&h.overall_outlook).size(theme::text_base()),
                    row![
                        text(format!("Theme: {}", h.dominant_theme)).size(theme::text_sm()),
                        text(format!("Confidence: {:.0}/100", h.confidence)).size(theme::text_sm()),
                    ]
                    .spacing(20),
                    text("Key Transits:").size(theme::text_sm()),
                    Column::with_children(key_transit_items).spacing(2),
                    row![
                        text(&h.moon_guidance).size(theme::text_sm()),
                        mercury_line,
                    ]
                    .spacing(20),
                    text(format!("Timing: {}", h.timing_window)).size(theme::text_sm()),
                ]
                .spacing(6)
                .into()
            } else {
                text("Horoscope reading not yet generated for today. Run the scraper to compute.")
                    .size(theme::text_sm())
                    .into()
            };

            column![
                row![wheel_col, transits_col]
                    .spacing(20)
                    .align_y(Alignment::Start),
                horoscope_section,
            ]
            .spacing(12)
            .into()
        };

        // ── Backtest Section ────────────────────────────────
        let backtest_section: Element<'_, Message> = {
            let config_row = row![
                text("Buy when astro >").size(theme::text_sm()),
                text_input("65", &self.backtest_buy_input)
                    .on_input(Message::BacktestBuyInput)
                    .width(Length::Fixed(50.0))
                    .size(theme::text_sm()),
                text("Sell when astro <").size(theme::text_sm()),
                text_input("35", &self.backtest_sell_input)
                    .on_input(Message::BacktestSellInput)
                    .width(Length::Fixed(50.0))
                    .size(theme::text_sm()),
                button(text("Run Backtest").size(theme::text_sm())).on_press(Message::RunBacktest),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            if let Some(ref bt) = self.backtest_result {
                let clear_btn = button(text("Clear Results").size(theme::text_sm()))
                    .on_press(Message::ClearBacktest);

                if let Some(ref msg) = bt.insufficient_data {
                    column![
                        text("Astro Backtest").font(font::DISPLAY).size(theme::text_md()),
                        config_row,
                        clear_btn,
                        horizontal_rule(1),
                        text(msg.as_str())
                            .size(theme::text_base())
                            .color(theme::ZONE_UNFAVORABLE),
                        text("Run the scraper to fetch more astro + price history.")
                            .size(theme::text_sm()),
                    ]
                    .spacing(6)
                    .into()
                } else {
                    let strat_color = if bt.total_return_pct > bt.buy_hold_return_pct {
                        theme::ZONE_OPTIMAL
                    } else {
                        theme::ZONE_MISALIGNED
                    };
                    let acc_color = if bt.signal_accuracy_pct >= 55.0 {
                        theme::ZONE_OPTIMAL
                    } else if bt.signal_accuracy_pct >= 45.0 {
                        theme::ZONE_NEUTRAL
                    } else {
                        theme::ZONE_MISALIGNED
                    };

                    let metrics = column![
                        row![
                            text(format!("Strategy: {:.1}%", bt.total_return_pct))
                                .size(theme::text_base())
                                .color(strat_color),
                            text("  vs  ").size(theme::text_sm()),
                            text(format!("Buy & Hold: {:.1}%", bt.buy_hold_return_pct))
                                .size(theme::text_base()),
                        ]
                        .spacing(4),
                        row![
                            text(format!("Trades: {}", bt.num_trades)).size(theme::text_sm()),
                            text(format!("Win Rate: {:.0}%", bt.win_rate_pct)).size(theme::text_sm()),
                            text(format!("Max DD: {:.1}%", bt.max_drawdown_pct)).size(theme::text_sm()),
                            text(format!("Final: ${:.0}", bt.final_capital)).size(theme::text_sm()),
                        ]
                        .spacing(12),
                        text(format!(
                            "Astro Signal Accuracy (30d): {:.1}%",
                            bt.signal_accuracy_pct
                        ))
                        .size(theme::text_base())
                        .color(acc_color),
                    ]
                    .spacing(4);

                    let trade_rows: Vec<Element<Message>> = bt
                        .trades
                        .iter()
                        .rev()
                        .take(10)
                        .flat_map(|t| {
                            let color = if t.return_pct > 0.0 {
                                theme::ZONE_OPTIMAL
                            } else {
                                theme::ZONE_MISALIGNED
                            };
                            let mut items: Vec<Element<Message>> = vec![
                                text(format!(
                                    "  {} @ ${:.2}  ->  {} @ ${:.2}  ({:+.1}%)",
                                    t.buy_date, t.buy_price, t.sell_date, t.sell_price, t.return_pct
                                ))
                                .size(theme::text_sm())
                                .color(color)
                                .into()
                            ];
                            // v11.0: Show correlated real-world events
                            for ev in &t.events {
                                items.push(
                                    text(format!("    \u{25AB} {ev}"))
                                        .size(theme::text_xs())
                                        .into()
                                );
                            }
                            items
                        })
                        .collect();

                    column![
                        text(format!(
                            "Backtest: {} ({} days)",
                            bt.ticker, bt.days_tested
                        ))
                        .size(theme::text_md()),
                        row![config_row, clear_btn].spacing(8).align_y(Alignment::Center),
                        horizontal_rule(1),
                        metrics,
                        horizontal_rule(1),
                        text("Recent Trades (last 10)").size(theme::text_sm()),
                        Column::with_children(trade_rows).spacing(1),
                    ]
                    .spacing(6)
                    .into()
                }
            } else {
                column![
                    text("Astro Backtest").font(font::DISPLAY).size(theme::text_md()),
                    config_row,
                    text("Press 'Run Backtest' to test the astro signal for this ticker.")
                        .size(theme::text_sm()),
                ]
                .spacing(6)
                .into()
            }
        };

        // ── Strategy Builder ────────────────────────────────
        let strategy_section: Element<'_, Message> = {
            let buy_logic_btn = button(
                text(self.strategy.buy_logic.label()).size(theme::text_sm()),
            )
            .on_press(Message::StrategyToggleBuyLogic);

            let buy_conds: Vec<Element<Message>> = self
                .strategy
                .buy_conditions
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    row![
                        text(c.label()).size(theme::text_sm()),
                        button(text("✕").size(theme::text_sm()))
                            .on_press(Message::StrategyRemoveBuyCond(i)),
                    ]
                    .spacing(4)
                    .into()
                })
                .collect();

            let sell_logic_btn = button(
                text(self.strategy.sell_logic.label()).size(theme::text_sm()),
            )
            .on_press(Message::StrategyToggleSellLogic);

            let sell_conds: Vec<Element<Message>> = self
                .strategy
                .sell_conditions
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    row![
                        text(c.label()).size(theme::text_sm()),
                        button(text("✕").size(theme::text_sm()))
                            .on_press(Message::StrategyRemoveSellCond(i)),
                    ]
                    .spacing(4)
                    .into()
                })
                .collect();

            let buy_add = row![
                button(text("+Astro>70").size(theme::text_sm()))
                    .on_press(Message::StrategyAddBuyCond(Condition::AstroAbove(70.0))),
                button(text("+RSI<70").size(theme::text_sm()))
                    .on_press(Message::StrategyAddBuyCond(Condition::RsiBelow(70.0))),
                button(text("+Astro>80").size(theme::text_sm()))
                    .on_press(Message::StrategyAddBuyCond(Condition::AstroAbove(80.0))),
                button(text("+P>SMA50").size(theme::text_sm()))
                    .on_press(Message::StrategyAddBuyCond(Condition::PriceAboveSma50)),
            ]
            .spacing(4);

            let sell_add = row![
                button(text("+Astro<30").size(theme::text_sm()))
                    .on_press(Message::StrategyAddSellCond(Condition::AstroBelow(30.0))),
                button(text("+RSI>80").size(theme::text_sm()))
                    .on_press(Message::StrategyAddSellCond(Condition::RsiAbove(80.0))),
                button(text("+Astro<20").size(theme::text_sm()))
                    .on_press(Message::StrategyAddSellCond(Condition::AstroBelow(20.0))),
                button(text("+P<SMA50").size(theme::text_sm()))
                    .on_press(Message::StrategyAddSellCond(Condition::PriceBelowSma50)),
            ]
            .spacing(4);

            let mut strat_col = column![
                text("Strategy Builder").font(font::DISPLAY).size(theme::text_md()),
                row![text("BUY when").size(theme::text_sm()), buy_logic_btn]
                    .spacing(6)
                    .align_y(Alignment::Center),
                Column::with_children(buy_conds).spacing(2),
                buy_add,
                row![text("SELL when").size(theme::text_sm()), sell_logic_btn]
                    .spacing(6)
                    .align_y(Alignment::Center),
                Column::with_children(sell_conds).spacing(2),
                sell_add,
                button(text("Run Strategy Backtest").size(theme::text_sm()))
                    .on_press(Message::RunStrategy),
            ]
            .spacing(6);

            if let Some(ref sr) = self.strategy_result {
                strat_col = strat_col.push(horizontal_rule(1));
                if let Some(ref msg) = sr.insufficient_data {
                    strat_col = strat_col.push(
                        text(msg.as_str())
                            .size(theme::text_base())
                            .color(theme::ZONE_UNFAVORABLE),
                    );
                } else {
                    let color = if sr.total_return_pct > sr.buy_hold_return_pct {
                        theme::ZONE_OPTIMAL
                    } else {
                        theme::ZONE_MISALIGNED
                    };
                    strat_col = strat_col.push(
                        row![
                            text(format!("Strategy: {:.1}%", sr.total_return_pct))
                                .size(theme::text_base())
                                .color(color),
                            text(format!("vs B&H: {:.1}%", sr.buy_hold_return_pct))
                                .size(theme::text_base()),
                            text(format!("Trades: {}", sr.num_trades)).size(theme::text_sm()),
                            text(format!("Win: {:.0}%", sr.win_rate_pct)).size(theme::text_sm()),
                        ]
                        .spacing(12),
                    );
                }
            }

            strat_col.into()
        };

        // ── Astro Calendar ──────────────────────────────────
        let calendar_section: Element<'_, Message> = {
            let nav = row![
                button(text("< Prev").size(theme::text_sm())).on_press(Message::CalendarPrevMonth),
                text("Astro Calendar").font(font::DISPLAY).size(theme::text_md()),
                button(text("Next >").size(theme::text_sm())).on_press(Message::CalendarNextMonth),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            let legend = row![
                text("■")
                    .size(theme::text_sm())
                    .color(theme::ZONE_OPTIMAL),
                text("Favorable (>50)").size(theme::text_xs()),
                text("■")
                    .size(theme::text_sm())
                    .color(theme::ZONE_NEUTRAL),
                text("Neutral (~50)").size(theme::text_xs()),
                text("■")
                    .size(theme::text_sm())
                    .color(theme::ZONE_MISALIGNED),
                text("Unfavorable (<50)").size(theme::text_xs()),
            ]
            .spacing(6)
            .align_y(Alignment::Center);

            column![
                nav,
                Canvas::new(AstroCalendar {
                    year: self.calendar_year,
                    month: self.calendar_month,
                    days: self.calendar_days.clone(),
                })
                .width(Length::Fill)
                .height(Length::Fixed(180.0)),
                legend,
            ]
            .spacing(6)
            .into()
        };

        // ── Forecast Timeline (v11.0) ─────────────────────
        let forecast_section: Element<'_, Message> = if self.forecast.is_empty() {
            text("Computing forecast...").size(theme::text_sm()).into()
        } else {
            let mut windows: Vec<String> = Vec::new();
            let mut i = 0;
            while i < self.forecast.len() && windows.len() < 6 {
                let day = &self.forecast[i];
                if day.score > 60.0 || day.score < 40.0 {
                    let zone = if day.score > 60.0 { "Favorable" } else { "Unfavorable" };
                    let start = day.date;
                    let mut end = start;
                    while i + 1 < self.forecast.len() {
                        let next = &self.forecast[i + 1];
                        if (day.score > 60.0 && next.score > 60.0) || (day.score < 40.0 && next.score < 40.0) {
                            end = next.date;
                            i += 1;
                        } else { break; }
                    }
                    let hint = day.key_aspect.as_deref().unwrap_or("");
                    if start == end {
                        windows.push(format!("{zone}: {start}  {hint}"));
                    } else {
                        windows.push(format!("{zone}: {start} \u{2192} {end}  {hint}"));
                    }
                }
                i += 1;
            }
            let window_items: Vec<Element<Message>> = windows.iter().map(|w| {
                let color = if w.starts_with("Favorable") { theme::ZONE_OPTIMAL } else { theme::ZONE_MISALIGNED };
                text(format!("  \u{2022} {w}")).size(theme::text_sm()).color(color).into()
            }).collect();
            let key_aspects: Vec<Element<Message>> = self.forecast.iter()
                .take(30)
                .filter_map(|d| d.key_aspect.as_ref().map(|a| (d.date, a)))
                .take(5)
                .map(|(date, aspect)| text(format!("  {date}: {aspect}")).size(theme::text_xs()).into())
                .collect();
            let mut col = column![
                text(format!("90-Day Forecast: {}", self.selected_ticker))
                    .font(font::DISPLAY).size(theme::text_md()),
                horizontal_rule(1),
            ].spacing(4);
            if window_items.is_empty() {
                col = col.push(text("  No strong signals in next 90 days.").size(theme::text_sm()));
            } else {
                col = col.push(Column::with_children(window_items).spacing(2));
            }
            if !key_aspects.is_empty() {
                col = col.push(text("Key Aspects (30d):").size(theme::text_sm()));
                col = col.push(Column::with_children(key_aspects).spacing(1));
            }
            col.into()
        };

        // ── Final assembly ─────────────────────────────────
        column![
            eyebrow("NATAL CHART"),
            astrology_section,
            section_rule(),
            eyebrow("FORECAST"),
            container(forecast_section).padding([10, 14]),
            section_rule(),
            eyebrow("ASTRO CALENDAR"),
            container(calendar_section).padding([10, 14]),
            section_rule(),
            eyebrow("BACKTEST"),
            container(backtest_section).padding([10, 14]),
            section_rule(),
            container(strategy_section).padding([10, 14]),
        ]
        .spacing(theme::SPACE_SM)
        .into()
    }
}
