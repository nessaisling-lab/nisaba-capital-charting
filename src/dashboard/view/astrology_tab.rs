use iced::widget::canvas::Canvas;
use iced::widget::{button, column, container, horizontal_rule, row, text, text_input, Column};
use iced::{Alignment, Element, Length};

use crate::astrology::{build_transits_section, build_wheel_legend, NatalWheel};
use crate::calendar::AstroCalendar;
use crate::state::{Dashboard, Message};
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
                text(format!("{} Astrology", self.selected_ticker)).size(theme::text_lg()),
                horizontal_rule(1),
                text("No birth chart yet for this ticker.").size(theme::text_base()),
                text("The scraper enriches ~50 tickers per day via SEC EDGAR.").size(theme::text_sm()),
                text("Once an IPO date is found, the natal chart is computed automatically.")
                    .size(theme::text_sm()),
            ]
            .spacing(6)
            .into()
        } else {
            let natal_wheel = Canvas::new(NatalWheel {
                natal: self.natal_positions.clone(),
                transits: self.daily_transits.clone(),
            })
            .width(Length::Fixed(300.0))
            .height(Length::Fixed(300.0));

            let wheel_col = column![
                text(format!("{} Birth Chart", self.selected_ticker)).size(theme::text_lg()),
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
                    text("Horoscope Reading").size(theme::text_lg()),
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
                if let Some(ref msg) = bt.insufficient_data {
                    return column![
                        text("Astro Backtest").size(theme::text_md()),
                        config_row,
                        horizontal_rule(1),
                        text(msg.as_str())
                            .size(theme::text_base())
                            .color(theme::ZONE_UNFAVORABLE),
                        text("Run the scraper to fetch more astro + price history.")
                            .size(theme::text_sm()),
                    ]
                    .spacing(6)
                    .into();
                }
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
                    .map(|t| {
                        let color = if t.return_pct > 0.0 {
                            theme::ZONE_OPTIMAL
                        } else {
                            theme::ZONE_MISALIGNED
                        };
                        text(format!(
                            "  {} @ ${:.2}  ->  {} @ ${:.2}  ({:+.1}%)",
                            t.buy_date, t.buy_price, t.sell_date, t.sell_price, t.return_pct
                        ))
                        .size(theme::text_sm())
                        .color(color)
                        .into()
                    })
                    .collect();

                column![
                    text(format!(
                        "Backtest: {} ({} days)",
                        bt.ticker, bt.days_tested
                    ))
                    .size(theme::text_md()),
                    config_row,
                    horizontal_rule(1),
                    metrics,
                    horizontal_rule(1),
                    text("Recent Trades (last 10)").size(theme::text_sm()),
                    Column::with_children(trade_rows).spacing(1),
                ]
                .spacing(6)
                .into()
            } else {
                column![
                    text("Astro Backtest").size(theme::text_md()),
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
                text("Strategy Builder").size(theme::text_md()),
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
                text("Astro Calendar").size(theme::text_md()),
                button(text("Next >").size(theme::text_sm())).on_press(Message::CalendarNextMonth),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            let legend = row![
                text("■")
                    .size(theme::text_sm())
                    .color(iced::Color::from_rgb(0.3, 0.8, 0.4)),
                text("Favorable (>50)").size(theme::text_xs()),
                text("■")
                    .size(theme::text_sm())
                    .color(iced::Color::from_rgb(0.5, 0.7, 0.2)),
                text("Neutral (~50)").size(theme::text_xs()),
                text("■")
                    .size(theme::text_sm())
                    .color(iced::Color::from_rgb(0.8, 0.3, 0.3)),
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

        // ── Final assembly ──────────────────────────────────
        column![
            astrology_section,
            horizontal_rule(1),
            container(calendar_section).padding([10, 14]),
            horizontal_rule(1),
            container(backtest_section).padding([10, 14]),
            horizontal_rule(1),
            container(strategy_section).padding([10, 14]),
        ]
        .spacing(10)
        .into()
    }
}
