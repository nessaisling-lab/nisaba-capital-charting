use iced::widget::{button, column, horizontal_rule, row, text, text_input, Column, Row};
use iced::{Alignment, Element, Length};

use crate::helpers;
use crate::state::{Dashboard, Message};
use crate::theme;

impl Dashboard {
    pub(crate) fn view_portfolio(&self) -> Element<'_, Message> {
        // ── Portfolio P&L Section ────────────────────────────
        let portfolio_section = self.build_portfolio_section();

        // ── Named Watchlists Manager ─────────────────────────
        let wl_dropdown: Row<Message> = self.named_watchlists.iter().fold(
            row![].spacing(4),
            |r, wl| {
                let is_active = self.active_watchlist_id == Some(wl.id);
                let label = if is_active {
                    format!("▶ {}", wl.name)
                } else {
                    wl.name.clone()
                };
                r.push(button(text(label).size(theme::text_sm())).on_press(Message::SelectNamedWatchlist(wl.id)))
            },
        );

        let wl_create_row = row![
            text_input("New watchlist name…", &self.new_watchlist_name)
                .on_input(Message::NewWatchlistNameInput)
                .on_submit(Message::CreateWatchlist)
                .width(Length::Fixed(200.0))
                .size(theme::text_sm()),
            button(text("+").size(theme::text_sm())).on_press(Message::CreateWatchlist),
        ].spacing(6);

        let wl_tickers_section = if self.watchlist_tickers_list.is_empty() {
            column![text("No tickers in this watchlist.").size(theme::text_sm())].spacing(4)
        } else {
            let ticker_chips: Vec<Element<Message>> = self.watchlist_tickers_list.iter().map(|t| {
                row![
                    text(t).size(theme::text_base()),
                    button(text("✕").size(theme::text_sm())).on_press(Message::WatchlistRemoveTicker(t.clone())),
                ].spacing(4).into()
            }).collect();
            Column::with_children(ticker_chips).spacing(2)
        };

        let wl_add_row = row![
            text_input("Add ticker…", &self.watchlist_add_ticker)
                .on_input(Message::WatchlistAddTickerInput)
                .on_submit(Message::WatchlistAddTicker)
                .width(Length::Fixed(120.0))
                .size(theme::text_sm()),
            button(text("Add").size(theme::text_sm())).on_press(Message::WatchlistAddTicker),
        ].spacing(6);

        let wl_actions = row![
            button(text("Delete Watchlist").size(theme::text_sm())).on_press(Message::DeleteActiveWatchlist),
            button(text("Export CSV").size(theme::text_sm())).on_press(Message::ExportCsv),
            button(text("Import to Portfolio").size(theme::text_sm())).on_press(Message::ImportWatchlistToPortfolio),
        ].spacing(8);

        let watchlist_mgr = column![
            text("Watchlists").size(theme::text_md()),
            wl_dropdown,
            wl_create_row,
            horizontal_rule(1),
            wl_tickers_section,
            wl_add_row,
            horizontal_rule(1),
            wl_actions,
        ].spacing(6);

        // ── Transaction Log ───────────────────────────────
        let tx_section: Element<'_, Message> = {
            let action_label = &self.tx_action;
            let input_row = row![
                button(text(action_label).size(theme::text_sm())).on_press(Message::TxToggleAction),
                text_input("Ticker", &self.tx_ticker_input)
                    .on_input(Message::TxTickerInput)
                    .width(Length::Fixed(80.0))
                    .size(theme::text_sm()),
                text_input("Shares", &self.tx_shares_input)
                    .on_input(Message::TxSharesInput)
                    .width(Length::Fixed(70.0))
                    .size(theme::text_sm()),
                text_input("Price", &self.tx_price_input)
                    .on_input(Message::TxPriceInput)
                    .on_submit(Message::TxSubmit)
                    .width(Length::Fixed(80.0))
                    .size(theme::text_sm()),
                button(text("Add").size(theme::text_sm())).on_press(Message::TxSubmit),
            ].spacing(6).align_y(Alignment::Center);

            let tx_rows: Vec<Element<Message>> = self.transactions.iter().take(20).map(|tx| {
                let color = if tx.action == "BUY" { theme::ZONE_OPTIMAL } else { theme::ZONE_MISALIGNED };
                let total = tx.shares * tx.price;
                row![
                    text(&tx.action).size(theme::text_sm()).color(color).width(Length::Fixed(40.0)),
                    text(&tx.ticker).size(theme::text_sm()).width(Length::Fixed(56.0)),
                    text(format!("{:.1}", tx.shares)).size(theme::text_sm()).width(Length::Fixed(56.0)),
                    text(helpers::format_price(tx.price as f64)).size(theme::text_sm()).width(Length::Fixed(72.0)),
                    text(format!("${}", helpers::format_compact(total as f64))).size(theme::text_sm()).width(Length::Fixed(72.0)),
                    text(tx.trade_date.to_string()).size(theme::text_sm()).width(Length::Fixed(80.0)),
                    button(text("✕").size(theme::text_sm())).on_press(Message::TxDelete(tx.id)),
                ].spacing(4).into()
            }).collect();

            column![
                text("Transaction Log").size(theme::text_md()),
                input_row,
                horizontal_rule(1),
                Column::with_children(tx_rows).spacing(1),
            ].spacing(6).into()
        };

        // ── Macro Strip ─────────────────────────────────────
        let macro_strip = self.build_macro_strip();

        // ── Final assembly ──────────────────────────────────
        column![
            iced::widget::container(portfolio_section).padding([10, 14]),
            horizontal_rule(1),
            iced::widget::container(tx_section).padding([10, 14]),
            horizontal_rule(1),
            iced::widget::container(watchlist_mgr).padding([10, 14]),
            horizontal_rule(1),
            macro_strip,
        ].spacing(10).into()
    }

    /// Build the portfolio P&L section (used by Portfolio tab).
    fn build_portfolio_section(&self) -> Column<'_, Message> {
        if !self.portfolio_pnl.is_empty() {
            let hdr = row![
                text("Ticker").size(theme::text_sm()).width(Length::Fixed(60.0)),
                text("Shares").size(theme::text_sm()).width(Length::Fixed(60.0)),
                text("Avg Cost").size(theme::text_sm()).width(Length::Fixed(72.0)),
                text("Last").size(theme::text_sm()).width(Length::Fixed(72.0)),
                text("P&L").size(theme::text_sm()).width(Length::Fixed(88.0)),
                text("P&L %").size(theme::text_sm()).width(Length::Fixed(60.0)),
                text("Astro").size(theme::text_sm()).width(Length::Fill),
            ].spacing(6);

            let mut total_cost = 0.0_f64;
            let mut total_value = 0.0_f64;

            let pos_rows: Vec<Element<Message>> = self.portfolio_pnl.iter().map(|p| {
                let cost_basis = p.shares as f64 * p.avg_cost as f64;
                let last_price = p.last_close.as_ref()
                    .and_then(|v| v.to_string().parse::<f64>().ok())
                    .unwrap_or(0.0);
                let mkt_value = p.shares as f64 * last_price;
                let pnl = mkt_value - cost_basis;
                let pnl_pct = if cost_basis > 0.0 { pnl / cost_basis * 100.0 } else { 0.0 };

                total_cost += cost_basis;
                total_value += mkt_value;

                let pnl_color = if pnl > 0.0 { theme::ZONE_OPTIMAL } else if pnl < 0.0 { theme::ZONE_MISALIGNED } else { theme::ZONE_NEUTRAL };
                let astro_label = match (&p.astro_score, &p.astro_label) {
                    (Some(s), Some(l)) => format!("{s:.0} {l}"),
                    _ => "---".to_string(),
                };

                row![
                    text(&p.ticker).size(theme::text_sm()).width(Length::Fixed(60.0)),
                    text(format!("{:.1}", p.shares)).size(theme::text_sm()).width(Length::Fixed(60.0)),
                    text(helpers::format_price(p.avg_cost as f64)).size(theme::text_sm()).width(Length::Fixed(72.0)),
                    text(if last_price > 0.0 { helpers::format_price(last_price) } else { "---".to_string() }).size(theme::text_sm()).width(Length::Fixed(72.0)),
                    text(format!("{:+.0}", pnl)).size(theme::text_sm()).color(pnl_color).width(Length::Fixed(88.0)),
                    text(helpers::format_pct(pnl_pct)).size(theme::text_sm()).color(pnl_color).width(Length::Fixed(60.0)),
                    text(astro_label).size(theme::text_sm()).width(Length::Fill),
                ].spacing(6).into()
            }).collect();

            let total_pnl = total_value - total_cost;
            let total_pnl_pct = if total_cost > 0.0 { total_pnl / total_cost * 100.0 } else { 0.0 };
            let total_color = if total_pnl > 0.0 { theme::ZONE_OPTIMAL } else if total_pnl < 0.0 { theme::ZONE_MISALIGNED } else { theme::ZONE_NEUTRAL };

            column![
                text("Portfolio").size(theme::text_md()),
                horizontal_rule(1),
                hdr,
                Column::with_children(pos_rows).spacing(2),
                horizontal_rule(1),
                row![
                    text(format!("Cost: ${}", helpers::format_compact(total_cost))).size(theme::text_sm()),
                    text(format!("Value: ${}", helpers::format_compact(total_value))).size(theme::text_sm()),
                    text(format!("P&L: {:+.0} ({})", total_pnl, helpers::format_pct(total_pnl_pct))).size(theme::text_base()).color(total_color),
                ].spacing(16),
            ].spacing(4)
        } else if self.portfolio.is_empty() {
            column![
                text("Portfolio").size(theme::text_md()),
                text("No positions tracked yet.").size(theme::text_base()),
                text("Add rows to portfolio_positions via portfolio_seed.sql.").size(theme::text_sm()),
            ].spacing(4)
        } else {
            let hdr = row![
                text("Ticker").size(theme::text_base()).width(Length::Fixed(64.0)),
                text("Shares").size(theme::text_base()).width(Length::Fixed(72.0)),
                text("Avg Cost").size(theme::text_base()).width(Length::Fixed(88.0)),
                text("Cost Basis").size(theme::text_base()).width(Length::Fill),
            ].spacing(8);

            let pos_rows: Vec<Element<Message>> = self.portfolio.iter().map(|p| {
                let cost_basis = p.shares * p.avg_cost;
                row![
                    text(&p.ticker).size(theme::text_base()).width(Length::Fixed(64.0)),
                    text(format!("{:.2}", p.shares)).size(theme::text_base()).width(Length::Fixed(72.0)),
                    text(helpers::format_price(p.avg_cost as f64)).size(theme::text_base()).width(Length::Fixed(88.0)),
                    text(format!("${}", helpers::format_compact(cost_basis as f64))).size(theme::text_base()).width(Length::Fill),
                ].spacing(8).into()
            }).collect();

            let total_basis: f32 = self.portfolio.iter().map(|p| p.shares * p.avg_cost).sum();
            column![
                text("Portfolio").size(theme::text_md()),
                horizontal_rule(1),
                hdr,
                Column::with_children(pos_rows).spacing(2),
                horizontal_rule(1),
                text(format!("Total cost basis: ${total_basis:.0}")).size(theme::text_base()),
            ].spacing(4)
        }
    }
}
