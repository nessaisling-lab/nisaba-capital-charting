//! Portfolio, backtest, strategy, and transaction message handlers.

use iced::Task;
use std::sync::Arc;

use crate::db::{fetch_backtest_data, delete_transaction, insert_transaction};
use crate::state::{Dashboard, Message};

/// Handle all portfolio/backtest/strategy/transaction messages.
/// Returns `Some(task)` if handled.
pub(crate) fn handle(state: &mut Dashboard, message: Message) -> Option<Task<Message>> {
    match message {
        // ── Portfolio ───────────────────────────────────────────────────
        Message::PortfolioLoaded(Ok(p)) => { state.portfolio = p; Some(Task::none()) }
        Message::PortfolioLoaded(Err(_)) => Some(Task::none()),
        Message::PortfolioPnlLoaded(Ok(rows)) => { state.portfolio_pnl = rows; Some(Task::none()) }
        Message::PortfolioPnlLoaded(Err(_)) => Some(Task::none()),

        Message::ImportWatchlistToPortfolio => {
            if let Some(pool) = &state.pool {
                let tickers = state.watchlist_tickers_list.clone();
                let pool2 = Arc::clone(pool);
                let pool3 = Arc::clone(pool);
                Some(Task::perform(
                    async move {
                        if let Err(e) = crate::db::import_tickers_to_portfolio(pool2, tickers).await {
                            eprintln!("Portfolio import error: {e}");
                        }
                        crate::db::fetch_portfolio(pool3).await
                    },
                    Message::PortfolioLoaded,
                ))
            } else {
                Some(Task::none())
            }
        }

        // ── Backtest ────────────────────────────────────────────────────
        Message::BacktestBuyInput(s) => { state.backtest_buy_input = s; Some(Task::none()) }
        Message::BacktestSellInput(s) => { state.backtest_sell_input = s; Some(Task::none()) }
        Message::SetBacktestWindowChoice(c) => {
            state.backtest_window_choice = c;
            Some(Task::none())
        }
        Message::ClearBacktest => {
            state.backtest_result = None;
            Some(Task::none())
        }
        Message::RunBacktest => {
            use pursuit_week4_automation::astrology::{ephemeris::Planet, returns::find_returns};
            use crate::state::BacktestWindowChoice;

            let buy = state.backtest_buy_input.parse::<f64>().unwrap_or(65.0);
            let sell = state.backtest_sell_input.parse::<f64>().unwrap_or(35.0);

            // Wave 9.5.6 — Translate the UI choice into a real TimeWindow.
            // For ReturnZone variants, compute the natal chart from the
            // selected ticker's IPO date and find every return inside a
            // 60-year window. If no IPO date is available, fall back to All.
            let time_window = match state.backtest_window_choice {
                BacktestWindowChoice::All => crate::backtest::TimeWindow::All,
                BacktestWindowChoice::Last5y => crate::backtest::TimeWindow::LastYears(5),
                BacktestWindowChoice::SaturnReturnZone | BacktestWindowChoice::JupiterReturnZone => {
                    let planet = if matches!(state.backtest_window_choice, BacktestWindowChoice::SaturnReturnZone) {
                        Planet::Saturn
                    } else {
                        Planet::Jupiter
                    };
                    let zone_days = if matches!(planet, Planet::Saturn) { 365 } else { 180 };
                    if let Some(ipo) = state.natal_ipo_date {
                        let natal = pursuit_week4_automation::astrology::natal::NatalChart::compute(
                            &state.selected_ticker, ipo,
                        );
                        match find_returns(&natal, planet, 60) {
                            Ok(events) => crate::backtest::TimeWindow::ReturnZone {
                                planet,
                                return_dates: events.into_iter().map(|e| e.return_date).collect(),
                                zone_days,
                            },
                            Err(_) => crate::backtest::TimeWindow::All,
                        }
                    } else {
                        crate::backtest::TimeWindow::All
                    }
                }
            };

            state.backtest_config = crate::backtest::BacktestConfig {
                buy_threshold: buy,
                sell_threshold: sell,
                initial_capital: 10_000.0,
                time_window,
            };
            if let Some(pool) = &state.pool {
                Some(Task::perform(
                    fetch_backtest_data(Arc::clone(pool), state.selected_ticker.clone()),
                    Message::BacktestDataLoaded,
                ))
            } else {
                Some(Task::none())
            }
        }
        Message::BacktestDataLoaded(Ok(rows)) => {
            let days: Vec<crate::backtest::BacktestDay> = rows
                .iter()
                .map(|r| crate::backtest::BacktestDay {
                    date: r.date,
                    close: r.close.to_string().parse::<f64>().unwrap_or(0.0),
                    astro_score: r.astro_score,
                })
                .collect();
            let mut result = crate::backtest::run_backtest(
                &state.selected_ticker,
                &days,
                &state.backtest_config,
            );
            // v11.0: Correlate trades with real-world events from loaded news
            for trade in &mut result.trades {
                let mut events: Vec<String> = Vec::new();
                for article in &state.news {
                    let pub_date = article.published_at.date_naive();
                    if pub_date >= trade.buy_date && pub_date <= trade.sell_date {
                        events.push(article.headline.clone());
                    }
                }
                for filing in &state.filings_8k {
                    if filing.filed_date >= trade.buy_date && filing.filed_date <= trade.sell_date {
                        let desc = filing.items.as_deref().unwrap_or("8-K filing");
                        events.push(format!("SEC: {desc}"));
                    }
                }
                // Keep top 3 events per trade
                events.truncate(3);
                trade.events = events;
            }
            state.backtest_result = Some(result);
            Some(Task::none())
        }
        Message::BacktestDataLoaded(Err(_)) => {
            state.backtest_result = None;
            Some(Task::none())
        }

        // ── Strategy Builder ────────────────────────────────────────────
        Message::StrategyAddBuyCond(c) => {
            state.strategy.buy_conditions.push(c);
            Some(Task::none())
        }
        Message::StrategyRemoveBuyCond(i) => {
            if i < state.strategy.buy_conditions.len() {
                state.strategy.buy_conditions.remove(i);
            }
            Some(Task::none())
        }
        Message::StrategyAddSellCond(c) => {
            state.strategy.sell_conditions.push(c);
            Some(Task::none())
        }
        Message::StrategyRemoveSellCond(i) => {
            if i < state.strategy.sell_conditions.len() {
                state.strategy.sell_conditions.remove(i);
            }
            Some(Task::none())
        }
        Message::StrategyToggleBuyLogic => {
            state.strategy.buy_logic = match state.strategy.buy_logic {
                crate::strategy::Logic::And => crate::strategy::Logic::Or,
                crate::strategy::Logic::Or => crate::strategy::Logic::And,
            };
            Some(Task::none())
        }
        Message::StrategyToggleSellLogic => {
            state.strategy.sell_logic = match state.strategy.sell_logic {
                crate::strategy::Logic::And => crate::strategy::Logic::Or,
                crate::strategy::Logic::Or => crate::strategy::Logic::And,
            };
            Some(Task::none())
        }
        Message::RunStrategy => {
            if let Some(pool) = &state.pool {
                Some(Task::perform(
                    fetch_backtest_data(Arc::clone(pool), state.selected_ticker.clone()),
                    Message::StrategyDataLoaded,
                ))
            } else {
                Some(Task::none())
            }
        }
        Message::StrategyDataLoaded(Ok(rows)) => {
            let indicators_map = state.indicators.as_ref();
            let days: Vec<crate::strategy::DaySnapshot> = rows
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    let close = r.close.to_string().parse::<f64>().unwrap_or(0.0);
                    let (rsi, macd, macd_prev, sma50) = if let Some(ind) = indicators_map {
                        let rsi = ind.rsi_vals.get(i).copied().flatten();
                        let macd = ind.macd_line.get(i).copied().flatten();
                        let macd_prev = if i > 0 {
                            ind.macd_line.get(i - 1).copied().flatten()
                        } else {
                            None
                        };
                        let sma50 = ind.sma50.get(i).copied().flatten();
                        (rsi, macd, macd_prev, sma50)
                    } else {
                        (None, None, None, None)
                    };
                    crate::strategy::DaySnapshot {
                        date: r.date,
                        close,
                        astro_score: Some(r.astro_score),
                        rsi,
                        macd,
                        macd_prev,
                        sma50,
                        lagrange_score: None, // not available in backtest context
                    }
                })
                .collect();
            let result = crate::strategy::run_strategy_backtest(
                &state.selected_ticker,
                &days,
                &state.strategy,
                10_000.0,
            );
            state.strategy_result = Some(result);
            Some(Task::none())
        }
        Message::StrategyDataLoaded(Err(_)) => {
            state.strategy_result = None;
            Some(Task::none())
        }

        // ── Transactions ────────────────────────────────────────────────
        Message::TransactionsLoaded(Ok(txs)) => { state.transactions = txs; Some(Task::none()) }
        Message::TransactionsLoaded(Err(_)) => Some(Task::none()),
        Message::TxTickerInput(s) => { state.tx_ticker_input = s; Some(Task::none()) }
        Message::TxSharesInput(s) => { state.tx_shares_input = s; Some(Task::none()) }
        Message::TxPriceInput(s) => { state.tx_price_input = s; Some(Task::none()) }
        Message::TxToggleAction => {
            state.tx_action = if state.tx_action == "BUY" {
                "SELL".to_string()
            } else {
                "BUY".to_string()
            };
            Some(Task::none())
        }
        Message::TxSubmit => {
            let ticker = state.tx_ticker_input.trim().to_uppercase();
            let shares = state.tx_shares_input.parse::<f32>().unwrap_or(0.0);
            let price = state.tx_price_input.parse::<f32>().unwrap_or(0.0);
            if ticker.is_empty() || shares <= 0.0 || price <= 0.0 {
                return Some(Task::none());
            }
            let action = state.tx_action.clone();
            state.tx_ticker_input.clear();
            state.tx_shares_input.clear();
            state.tx_price_input.clear();
            if let Some(pool) = &state.pool {
                Some(Task::perform(
                    insert_transaction(
                        Arc::clone(pool),
                        ticker,
                        action,
                        shares,
                        price,
                        chrono::Local::now().date_naive(),
                        None,
                    ),
                    Message::TxCreated,
                ))
            } else {
                Some(Task::none())
            }
        }
        Message::TxCreated(Ok(tx)) => {
            state.push_toast(format!("Transaction recorded: {}", tx.ticker));
            state.transactions.insert(0, tx);
            Some(Task::none())
        }
        Message::TxCreated(Err(_)) => Some(Task::none()),
        Message::TxDelete(id) => {
            state.transactions.retain(|t| t.id != id);
            if let Some(pool) = &state.pool {
                Some(Task::perform(
                    delete_transaction(Arc::clone(pool), id),
                    Message::TxDeleted,
                ))
            } else {
                Some(Task::none())
            }
        }
        Message::TxDeleted(Ok(())) => Some(Task::none()),
        Message::TxDeleted(Err(_)) => Some(Task::none()),

        _ => None,
    }
}
