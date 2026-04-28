use iced::widget::{canvas::Canvas, column, horizontal_rule, row, text};
use iced::{Alignment, Color, Element, Length};

use crate::charts::EquityCurve;
use crate::font;
use crate::helpers;
use crate::icons;
use crate::state::{Dashboard, Message};
use crate::theme;
use crate::view::shared::{card, eyebrow, section_heading, section_rule};

impl Dashboard {
    pub(crate) fn view_paper_trail(&self) -> Element<'_, Message> {
        let mut content = column![].spacing(theme::SPACE_SM);

        content = content.push(eyebrow("PAPER TRADING"));
        content = content.push(self.build_paper_account_card());
        content = content.push(section_rule());
        content = content.push(eyebrow("OPEN POSITIONS"));
        content = content.push(self.build_paper_positions_card());
        content = content.push(section_rule());
        content = content.push(eyebrow("PERFORMANCE"));
        content = content.push(self.build_paper_stats_card());
        content = content.push(section_rule());
        content = content.push(eyebrow("EQUITY CURVE"));
        content = content.push(self.build_equity_curve_card());
        content = content.push(section_rule());
        content = content.push(eyebrow("TRADE LOG"));
        content = content.push(self.build_paper_trades_card());

        content.into()
    }

    fn build_paper_account_card(&self) -> Element<'_, Message> {
        match &self.paper_account {
            Some(acc) => {
                let initial: f64 = acc.initial_capital.to_string().parse().unwrap_or(0.0);
                let cash: f64 = acc.cash_balance.to_string().parse().unwrap_or(0.0);
                let portfolio_val: f64 = acc.portfolio_value
                    .as_ref()
                    .and_then(|v| v.to_string().parse().ok())
                    .unwrap_or(0.0);
                let total_value = cash + portfolio_val;
                let total_return_pct = if initial > 0.0 {
                    (total_value - initial) / initial * 100.0
                } else {
                    0.0
                };
                let return_color = if total_return_pct >= 0.0 {
                    Color::from_rgb(0.2, 0.8, 0.4)
                } else {
                    Color::from_rgb(0.9, 0.3, 0.3)
                };

                let sim_date_str = acc.last_sim_date
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "No simulations yet".to_string());

                let trades_count = acc.total_trades.unwrap_or(0);

                card(column![
                    section_heading(icons::WALLET, "Paper Trading Account"),
                    horizontal_rule(1),
                    row![
                        column![
                            text("Initial Capital").size(theme::text_sm()),
                            text(helpers::format_price(initial)).font(font::INTER).size(theme::text_md()),
                        ].spacing(2),
                        column![
                            text("Cash Balance").size(theme::text_sm()),
                            text(helpers::format_price(cash)).font(font::INTER).size(theme::text_md()),
                        ].spacing(2),
                        column![
                            text("Portfolio Value").size(theme::text_sm()),
                            text(helpers::format_price(portfolio_val)).font(font::INTER).size(theme::text_md()),
                        ].spacing(2),
                        column![
                            text("Total Value").size(theme::text_sm()),
                            text(helpers::format_price(total_value)).font(font::INTER).size(theme::text_md()),
                        ].spacing(2),
                        column![
                            text("Total Return").size(theme::text_sm()),
                            text(format!("{total_return_pct:+.2}%"))
                                .font(font::INTER)
                                .size(theme::text_md())
                                .color(return_color),
                        ].spacing(2),
                    ].spacing(30),
                    row![
                        text(format!("Last simulation: {sim_date_str}")).size(theme::text_sm()),
                        text(format!("Total trades: {trades_count}")).size(theme::text_sm()),
                    ].spacing(20),
                ].spacing(8))
            }
            None => {
                card(column![
                    section_heading(icons::WALLET, "Paper Trading Account"),
                    horizontal_rule(1),
                    text("No paper trading account found. Run the scraper to initialize.").size(theme::text_base()),
                    text("The paper engine runs automatically as Phase 5 of the daily pipeline.").size(theme::text_sm()),
                ].spacing(8))
            }
        }
    }

    fn build_paper_positions_card(&self) -> Element<'_, Message> {
        let mut col = column![
            section_heading(icons::BRIEFCASE, "Open Positions"),
            horizontal_rule(1),
        ].spacing(6);

        if self.paper_positions.is_empty() {
            col = col.push(text("No open positions. The paper engine will buy when Lagrange scores exceed 65.").size(theme::text_sm()));
        } else {
            // Header row
            let header = row![
                text("Ticker").size(theme::text_sm()).width(Length::Fixed(80.0)),
                text("Shares").size(theme::text_sm()).width(Length::Fixed(80.0)),
                text("Entry $").size(theme::text_sm()).width(Length::Fixed(90.0)),
                text("Current $").size(theme::text_sm()).width(Length::Fixed(90.0)),
                text("P&L %").size(theme::text_sm()).width(Length::Fixed(80.0)),
                text("Entry Score").size(theme::text_sm()).width(Length::Fixed(90.0)),
                text("Current Score").size(theme::text_sm()).width(Length::Fixed(100.0)),
                text("Entry Date").size(theme::text_sm()).width(Length::Fixed(100.0)),
            ].spacing(8).align_y(Alignment::Center);
            col = col.push(header);
            col = col.push(horizontal_rule(1));

            for pos in &self.paper_positions {
                let shares: f64 = pos.shares.to_string().parse().unwrap_or(0.0);
                let entry: f64 = pos.entry_price.to_string().parse().unwrap_or(0.0);
                let current: f64 = pos.last_close
                    .as_ref()
                    .and_then(|v| v.to_string().parse().ok())
                    .unwrap_or(entry);
                let pnl_pct = if entry > 0.0 { (current - entry) / entry * 100.0 } else { 0.0 };
                let pnl_color = if pnl_pct >= 0.0 {
                    Color::from_rgb(0.2, 0.8, 0.4)
                } else {
                    Color::from_rgb(0.9, 0.3, 0.3)
                };

                let entry_score_str = pos.entry_score
                    .map(|s| format!("{s:.1}"))
                    .unwrap_or_else(|| "-".to_string());
                let current_score_str = pos.last_score
                    .map(|s| format!("{s:.1}"))
                    .unwrap_or_else(|| "-".to_string());

                let r = row![
                    text(&pos.ticker).size(theme::text_base()).width(Length::Fixed(80.0)),
                    text(format!("{shares:.2}")).font(font::INTER).size(theme::text_base()).width(Length::Fixed(80.0)),
                    text(format!("${entry:.2}")).font(font::INTER).size(theme::text_base()).width(Length::Fixed(90.0)),
                    text(format!("${current:.2}")).font(font::INTER).size(theme::text_base()).width(Length::Fixed(90.0)),
                    text(format!("{pnl_pct:+.2}%")).font(font::INTER).size(theme::text_base()).width(Length::Fixed(80.0)).color(pnl_color),
                    text(entry_score_str).font(font::INTER).size(theme::text_base()).width(Length::Fixed(90.0)),
                    text(current_score_str).font(font::INTER).size(theme::text_base()).width(Length::Fixed(100.0)),
                    text(pos.entry_date.to_string()).font(font::INTER).size(theme::text_base()).width(Length::Fixed(100.0)),
                ].spacing(8).align_y(Alignment::Center);
                col = col.push(r);
            }
        }

        card(col)
    }

    fn build_paper_stats_card(&self) -> Element<'_, Message> {
        use pursuit_week4_automation::stats;

        let sharpe = stats::sharpe_ratio(&self.paper_daily_values);
        let max_dd = stats::max_drawdown_pct(&self.paper_daily_values);

        // Portfolio total return %
        let portfolio_return_pct = if self.paper_daily_values.len() >= 2 {
            let first = self.paper_daily_values[0];
            let last = *self.paper_daily_values.last().unwrap();
            if first > 0.0 { (last - first) / first * 100.0 } else { 0.0 }
        } else {
            0.0
        };

        // SPY benchmark stats
        let spy_sharpe = stats::sharpe_ratio(&self.paper_spy_values);
        let spy_max_dd = stats::max_drawdown_pct(&self.paper_spy_values);
        let spy_return_pct = if self.paper_spy_values.len() >= 2 {
            let first = self.paper_spy_values[0];
            let last = *self.paper_spy_values.last().unwrap();
            if first > 0.0 { (last - first) / first * 100.0 } else { 0.0 }
        } else {
            0.0
        };
        let alpha = portfolio_return_pct - spy_return_pct;

        // Compute win rate and avg holding from trade log
        let closed_returns: Vec<f64> = self.paper_trades.iter()
            .filter(|t| t.action == "SELL")
            .filter_map(|sell| {
                // Find matching BUY for same ticker
                self.paper_trades.iter()
                    .filter(|t| t.action == "BUY" && t.ticker == sell.ticker && t.trade_date <= sell.trade_date)
                    .last()
                    .map(|buy| {
                        let buy_price: f64 = buy.price.to_string().parse().unwrap_or(1.0);
                        let sell_price: f64 = sell.price.to_string().parse().unwrap_or(1.0);
                        (sell_price - buy_price) / buy_price * 100.0
                    })
            })
            .collect();

        let win_rate = stats::win_rate_pct(&closed_returns);

        let holding_pairs: Vec<(chrono::NaiveDate, chrono::NaiveDate)> = self.paper_trades.iter()
            .filter(|t| t.action == "SELL")
            .filter_map(|sell| {
                self.paper_trades.iter()
                    .filter(|t| t.action == "BUY" && t.ticker == sell.ticker && t.trade_date <= sell.trade_date)
                    .last()
                    .map(|buy| (buy.trade_date, sell.trade_date))
            })
            .collect();

        let avg_hold = stats::avg_holding_days(&holding_pairs);

        let has_data = !self.paper_daily_values.is_empty() || !closed_returns.is_empty();
        let has_spy = self.paper_spy_values.len() >= 2;

        let return_color = |v: f64| if v >= 0.0 {
            Color::from_rgb(0.2, 0.8, 0.4)
        } else {
            Color::from_rgb(0.9, 0.3, 0.3)
        };

        let stats_body: Element<'_, Message> = if has_data {
            let mut stats_col = column![
                // Portfolio stats row
                text("Portfolio").size(theme::text_sm()),
                row![
                    column![
                        text("Return").size(theme::text_sm()),
                        text(format!("{portfolio_return_pct:+.2}%"))
                            .font(font::INTER)
                            .size(theme::text_md())
                            .color(return_color(portfolio_return_pct)),
                    ].spacing(2),
                    column![
                        text("Sharpe Ratio").size(theme::text_sm()),
                        text(format!("{sharpe:.3}")).font(font::INTER).size(theme::text_md()),
                    ].spacing(2),
                    column![
                        text("Max Drawdown").size(theme::text_sm()),
                        text(format!("{max_dd:.2}%")).font(font::INTER).size(theme::text_md()),
                    ].spacing(2),
                    column![
                        text("Win Rate").size(theme::text_sm()),
                        text(format!("{win_rate:.1}%")).font(font::INTER).size(theme::text_md()),
                    ].spacing(2),
                    column![
                        text("Avg Hold (days)").size(theme::text_sm()),
                        text(format!("{avg_hold:.1}")).font(font::INTER).size(theme::text_md()),
                    ].spacing(2),
                    column![
                        text("Closed Trades").size(theme::text_sm()),
                        text(format!("{}", closed_returns.len())).font(font::INTER).size(theme::text_md()),
                    ].spacing(2),
                ].spacing(30),
            ].spacing(4);

            if has_spy {
                stats_col = stats_col.push(horizontal_rule(1));
                stats_col = stats_col.push(
                    text("vs. SPY Benchmark").size(theme::text_sm())
                );
                stats_col = stats_col.push(
                    row![
                        column![
                            text("SPY Return").size(theme::text_sm()),
                            text(format!("{spy_return_pct:+.2}%"))
                                .font(font::INTER)
                                .size(theme::text_md())
                                .color(return_color(spy_return_pct)),
                        ].spacing(2),
                        column![
                            text("SPY Sharpe").size(theme::text_sm()),
                            text(format!("{spy_sharpe:.3}")).font(font::INTER).size(theme::text_md()),
                        ].spacing(2),
                        column![
                            text("SPY Max DD").size(theme::text_sm()),
                            text(format!("{spy_max_dd:.2}%")).font(font::INTER).size(theme::text_md()),
                        ].spacing(2),
                        column![
                            text("Alpha").size(theme::text_sm()),
                            text(format!("{alpha:+.2}%"))
                                .font(font::INTER)
                                .size(theme::text_md())
                                .color(return_color(alpha)),
                        ].spacing(2),
                    ].spacing(30),
                );
            }

            stats_col.into()
        } else {
            text("No simulation data yet. Statistics will appear after the paper engine runs.").size(theme::text_sm()).into()
        };

        card(column![
            section_heading(icons::ACTIVITY, "Performance Statistics"),
            horizontal_rule(1),
            stats_body,
        ].spacing(8))
    }

    fn build_equity_curve_card(&self) -> Element<'_, Message> {
        card(column![
            section_heading(icons::GRAPH_UP, "Equity Curve"),
            horizontal_rule(1),
            Canvas::new(EquityCurve {
                portfolio_values: self.paper_daily_values.clone(),
                spy_values: self.paper_spy_values.clone(),
            })
            .width(Length::Fill)
            .height(Length::Fixed(220.0)),
        ].spacing(6))
    }

    fn build_paper_trades_card(&self) -> Element<'_, Message> {
        let mut col = column![
            section_heading(icons::CLOCK, "Trade Log"),
            horizontal_rule(1),
        ].spacing(6);

        if self.paper_trades.is_empty() {
            col = col.push(text("No trades recorded yet.").size(theme::text_sm()));
        } else {
            // Header
            let header = row![
                text("Date").size(theme::text_sm()).width(Length::Fixed(100.0)),
                text("Action").size(theme::text_sm()).width(Length::Fixed(60.0)),
                text("Ticker").size(theme::text_sm()).width(Length::Fixed(80.0)),
                text("Shares").size(theme::text_sm()).width(Length::Fixed(80.0)),
                text("Price").size(theme::text_sm()).width(Length::Fixed(90.0)),
                text("Score").size(theme::text_sm()).width(Length::Fixed(70.0)),
            ].spacing(8).align_y(Alignment::Center);
            col = col.push(header);
            col = col.push(horizontal_rule(1));

            // Show most recent 50 trades
            for trade in self.paper_trades.iter().take(50) {
                let shares: f64 = trade.shares.to_string().parse().unwrap_or(0.0);
                let price: f64 = trade.price.to_string().parse().unwrap_or(0.0);
                let action_color = if trade.action == "BUY" {
                    Color::from_rgb(0.2, 0.8, 0.4)
                } else {
                    Color::from_rgb(0.9, 0.3, 0.3)
                };
                let score_str = trade.score
                    .map(|s| format!("{s:.1}"))
                    .unwrap_or_else(|| "-".to_string());

                let r = row![
                    text(trade.trade_date.to_string()).font(font::INTER).size(theme::text_base()).width(Length::Fixed(100.0)),
                    text(&trade.action).size(theme::text_base()).width(Length::Fixed(60.0)).color(action_color),
                    text(&trade.ticker).size(theme::text_base()).width(Length::Fixed(80.0)),
                    text(format!("{shares:.2}")).font(font::INTER).size(theme::text_base()).width(Length::Fixed(80.0)),
                    text(format!("${price:.2}")).font(font::INTER).size(theme::text_base()).width(Length::Fixed(90.0)),
                    text(score_str).font(font::INTER).size(theme::text_base()).width(Length::Fixed(70.0)),
                ].spacing(8).align_y(Alignment::Center);
                col = col.push(r);
            }

            if self.paper_trades.len() > 50 {
                col = col.push(
                    text(format!("... and {} more trades", self.paper_trades.len() - 50))
                        .size(theme::text_sm())
                );
            }
        }

        card(col)
    }
}
