//! Ticker data message handlers.
//!
//! Covers: TickerSelected, DataLoaded (prices), InsiderTrades, Filings,
//! Holdings, News, RSS, Polymarket, Earnings, AnalystRating, Sentiment,
//! FearGreed, Macro, ShortInterest, Fundamentals, DCF, Agents, Compare,
//! SectorPeers, LagrangeHistory, search/autocomplete.

use iced::Task;
use std::sync::Arc;

use crate::db::{
    fetch_lagrange_history, fetch_sector_peers, fetch_recently_viewed,
    search_tickers, upsert_recently_viewed,
};
use crate::indicators::Indicators;
use crate::state::{Dashboard, Message};

/// Handle all ticker-data-related messages. Returns `Some(task)` if handled.
pub(crate) fn handle(state: &mut Dashboard, message: Message) -> Option<Task<Message>> {
    match message {
        Message::TickerSelected(ticker) => {
            if ticker == state.selected_ticker {
                return Some(Task::none());
            }
            state.selected_ticker = ticker.clone();
            state.rows = vec![];
            state.indicators = None;
            state.insider_trades = vec![];
            state.filings_8k = vec![];
            state.holdings = vec![];
            state.news = vec![];
            state.analyst_rating = None;
            state.sentiment = None;
            state.astro_score = None;
            state.astro_aspects = vec![];
            state.natal_positions = vec![];
            state.short_interest = None;
            state.fundamentals = None;
            state.agent_analysis = None;
            state.lagrange_history = vec![];
            state.sector_peers = vec![];
            state.status = format!("Loading {ticker}...");
            if let Some(pool) = &state.pool {
                let rv_pool = Arc::clone(pool);
                let rv_ticker = ticker.clone();
                Some(Task::batch([
                    Dashboard::fetch_all(pool, ticker.clone()),
                    Task::perform(
                        fetch_lagrange_history(Arc::clone(pool), ticker.clone()),
                        Message::LagrangeHistoryLoaded,
                    ),
                    Task::perform(
                        fetch_sector_peers(Arc::clone(pool), ticker),
                        Message::SectorPeersLoaded,
                    ),
                    Task::perform(
                        async move {
                            upsert_recently_viewed(Arc::clone(&rv_pool), rv_ticker).await;
                            fetch_recently_viewed(rv_pool).await
                        },
                        Message::RecentlyViewedLoaded,
                    ),
                ]))
            } else {
                Some(Task::none())
            }
        }

        Message::TickerSearchInput(s) => {
            if state.autocomplete_dismissed {
                state.autocomplete_dismissed = false;
                return Some(Task::none());
            }
            state.ticker_search_input = s.clone();
            if s.trim().is_empty() {
                state.autocomplete_suggestions = vec![];
                return Some(Task::none());
            }
            if let Some(pool) = &state.pool {
                Some(Task::perform(
                    search_tickers(Arc::clone(pool), s),
                    |res| Message::AutocompleteResults(res.unwrap_or_default()),
                ))
            } else {
                Some(Task::none())
            }
        }
        Message::AutocompleteResults(suggestions) => {
            if state.autocomplete_dismissed || state.ticker_search_input.is_empty() {
                return Some(Task::none());
            }
            state.autocomplete_suggestions = suggestions;
            Some(Task::none())
        }
        Message::AutocompleteSelected(ticker) => {
            state.ticker_search_input = String::new();
            state.autocomplete_suggestions = vec![];
            state.autocomplete_dismissed = true;
            Some(state.update(Message::TickerSelected(ticker)))
        }
        Message::TickerSearchSubmit => {
            let ticker = state.ticker_search_input.trim().to_uppercase();
            if ticker.is_empty() {
                return Some(Task::none());
            }
            state.ticker_search_input = String::new();
            state.autocomplete_suggestions = vec![];
            state.autocomplete_dismissed = true;
            Some(state.update(Message::TickerSelected(ticker)))
        }

        Message::RecentlyViewedLoaded(Ok(tickers)) => {
            state.recently_viewed = tickers;
            Some(Task::none())
        }
        Message::RecentlyViewedLoaded(Err(_)) => Some(Task::none()),

        Message::DataLoaded(Ok(rows)) => {
            state.refreshing = false;
            state.status = if rows.is_empty() {
                format!(
                    "{} — no data yet (run the scraper first)",
                    state.selected_ticker
                )
            } else {
                format!("Loaded {} rows for {}", rows.len(), state.selected_ticker)
            };
            if rows.len() >= 20 {
                let prices: Vec<f32> = rows
                    .iter()
                    .rev()
                    .map(|r| r.close.to_string().parse::<f32>().unwrap_or(0.0))
                    .collect();
                state.indicators = Some(Indicators::compute(&prices));
            }
            state.rows = rows;
            Some(Task::none())
        }
        Message::DataLoaded(Err(e)) => {
            state.refreshing = false;
            state.status = format!("Query error (showing stale data): {e}");
            Some(Task::none())
        }

        Message::InsiderTradesLoaded(Ok(t)) => { state.insider_trades = t; Some(Task::none()) }
        Message::InsiderTradesLoaded(Err(_)) => Some(Task::none()),
        Message::FilingsLoaded(Ok(f)) => { state.filings_8k = f; Some(Task::none()) }
        Message::FilingsLoaded(Err(_)) => Some(Task::none()),
        Message::HoldingsLoaded(Ok(h)) => { state.holdings = h; Some(Task::none()) }
        Message::HoldingsLoaded(Err(_)) => Some(Task::none()),
        Message::NewsLoaded(Ok(n)) => { state.news = n; Some(Task::none()) }
        Message::NewsLoaded(Err(_)) => Some(Task::none()),
        Message::RssArticlesLoaded(Ok(a)) => { state.rss_articles = a; Some(Task::none()) }
        Message::RssArticlesLoaded(Err(_)) => Some(Task::none()),
        Message::PolymarketLoaded(Ok(m)) => { state.polymarket = m; Some(Task::none()) }
        Message::PolymarketLoaded(Err(_)) => Some(Task::none()),
        Message::GdeltLoaded(Ok(events)) => { state.gdelt_events = events; Some(Task::none()) }
        Message::GdeltLoaded(Err(_)) => Some(Task::none()),
        Message::EarningsLoaded(Ok(e)) => { state.earnings = e; Some(Task::none()) }
        Message::EarningsLoaded(Err(_)) => Some(Task::none()),
        Message::AnalystRatingLoaded(Ok(r)) => { state.analyst_rating = r; Some(Task::none()) }
        Message::AnalystRatingLoaded(Err(_)) => Some(Task::none()),
        Message::SentimentLoaded(Ok(s)) => { state.sentiment = s; Some(Task::none()) }
        Message::SentimentLoaded(Err(_)) => Some(Task::none()),
        Message::FearGreedLoaded(Ok(fg)) => { state.fear_greed = Some(fg); Some(Task::none()) }
        Message::FearGreedLoaded(Err(e)) => { state.fear_greed_err = Some(e); Some(Task::none()) }
        Message::MarketFGLoaded(Ok(fg)) => { state.market_fg = Some(fg); Some(Task::none()) }
        Message::MarketFGLoaded(Err(e)) => { state.market_fg_err = Some(e); Some(Task::none()) }
        Message::MacroDataLoaded(Ok(data)) => { state.macro_data = data; Some(Task::none()) }
        Message::MacroDataLoaded(Err(_)) => Some(Task::none()),
        Message::ShortInterestLoaded(Ok(si)) => { state.short_interest = si; Some(Task::none()) }
        Message::ShortInterestLoaded(Err(_)) => Some(Task::none()),

        Message::FundamentalsLoaded(Ok(f)) => {
            state.fundamentals = f;
            state.compute_dcf_if_ready();
            state.recompute_agent_if_active();
            Some(Task::none())
        }
        Message::FundamentalsLoaded(Err(_)) => Some(Task::none()),

        Message::LagrangeHistoryLoaded(Ok(h)) => { state.lagrange_history = h; Some(Task::none()) }
        Message::LagrangeHistoryLoaded(Err(_)) => Some(Task::none()),

        Message::SectorPeersLoaded(Ok(peers)) => { state.sector_peers = peers; Some(Task::none()) }
        Message::SectorPeersLoaded(Err(_)) => Some(Task::none()),

        // DCF inputs
        Message::DcfGrowthRateInput(s) => { state.dcf_growth_rate = s; Some(Task::none()) }
        Message::DcfGrowthYearsInput(s) => { state.dcf_growth_years = s; Some(Task::none()) }
        Message::DcfTerminalGrowthInput(s) => { state.dcf_terminal_growth = s; Some(Task::none()) }
        Message::DcfDiscountRateInput(s) => { state.dcf_discount_rate = s; Some(Task::none()) }
        Message::DcfCompute => { state.compute_dcf_if_ready(); Some(Task::none()) }

        // Options Greeks inputs
        Message::GreeksSpotInput(s) => { state.greeks_spot = s; Some(Task::none()) }
        Message::GreeksStrikeInput(s) => { state.greeks_strike = s; Some(Task::none()) }
        Message::GreeksExpiryInput(s) => { state.greeks_expiry_days = s; Some(Task::none()) }
        Message::GreeksRateInput(s) => { state.greeks_rate = s; Some(Task::none()) }
        Message::GreeksVolInput(s) => { state.greeks_vol = s; Some(Task::none()) }
        Message::GreeksToggleType => { state.greeks_is_call = !state.greeks_is_call; Some(Task::none()) }
        Message::GreeksMarketPriceInput(s) => { state.greeks_market_price = s; Some(Task::none()) }
        Message::GreeksCompute => { state.compute_greeks(); Some(Task::none()) }
        Message::GreeksSolveIV => { state.solve_implied_vol(); Some(Task::none()) }

        // Agent selection
        Message::AgentSelected(persona) => {
            state.active_agent = Some(persona);
            state.agent_llm_error = None;
            match state.agent_mode {
                crate::agents::AgentMode::Template => {
                    state.recompute_agent_if_active();
                    Some(Task::none())
                }
                crate::agents::AgentMode::Llm => {
                    let api_key = state.settings.get("anthropic_api_key").cloned().unwrap_or_default();
                    if api_key.is_empty() {
                        // Fallback to template if no API key
                        state.push_toast("No API key set — using template analysis");
                        state.recompute_agent_if_active();
                        Some(Task::none())
                    } else {
                        state.agent_loading = true;
                        state.agent_analysis = None;
                        let ctx = state.build_agent_context();
                        Some(Task::perform(
                            crate::agents::analyze_llm(persona, ctx, api_key),
                            Message::LlmAnalysisComplete,
                        ))
                    }
                }
            }
        }

        // Agent mode switch
        Message::SetAgentMode(mode) => {
            state.agent_mode = mode;
            state.agent_analysis = None;
            state.agent_llm_error = None;
            state.agent_loading = false;
            // Re-run analysis for active persona in new mode
            if state.active_agent.is_some() {
                Some(state.update(Message::AgentSelected(state.active_agent.unwrap())))
            } else {
                Some(Task::none())
            }
        }

        // LLM analysis result
        Message::LlmAnalysisComplete(Ok(analysis)) => {
            state.agent_loading = false;
            state.agent_llm_error = None;
            state.agent_analysis = Some(analysis);
            Some(Task::none())
        }
        Message::LlmAnalysisComplete(Err(e)) => {
            state.agent_loading = false;
            state.agent_llm_error = Some(e.clone());
            state.push_toast(format!("LLM error — falling back to template: {}", e.chars().take(80).collect::<String>()));
            // Fallback to template
            state.recompute_agent_if_active();
            Some(Task::none())
        }

        // API key text input buffer
        Message::ApiKeyInput(s) => { state.api_key_input = s; Some(Task::none()) }

        // Comparison
        Message::CompareInput(s) => { state.compare_input = s; Some(Task::none()) }
        Message::CompareAdd => {
            let ticker = state.compare_input.trim().to_uppercase();
            if ticker.is_empty()
                || state.compare_tickers.len() >= 4
                || state.compare_tickers.contains(&ticker)
            {
                return Some(Task::none());
            }
            state.compare_input = String::new();
            state.compare_tickers.push(ticker);
            Some(state.refresh_compare())
        }
        Message::CompareAddDirect(ticker) => {
            let ticker = ticker.to_uppercase();
            if state.compare_tickers.len() >= 4 || state.compare_tickers.contains(&ticker) {
                return Some(Task::none());
            }
            state.compare_tickers.push(ticker);
            Some(state.refresh_compare())
        }
        Message::CompareRemove(ticker) => {
            state.compare_tickers.retain(|t| t != &ticker);
            if state.compare_tickers.is_empty() {
                state.compare_data = vec![];
                Some(Task::none())
            } else {
                Some(state.refresh_compare())
            }
        }
        Message::CompareDataLoaded(Ok(data)) => { state.compare_data = data; Some(Task::none()) }
        Message::CompareDataLoaded(Err(_)) => Some(Task::none()),

        // ── Fetch single ticker via scraper subprocess ─────────
        Message::FetchThisTicker => {
            state.fetching_ticker = true;
            state.fetch_ticker_error = None;
            state.push_toast(format!("Fetching data for {}...", state.selected_ticker));
            let ticker = state.selected_ticker.clone();
            Some(Task::perform(
                async move {
                    // Find the scraper binary adjacent to our own executable
                    let scraper_path = std::env::current_exe()
                        .ok()
                        .and_then(|p| p.parent().map(|d| {
                            let name = if cfg!(windows) { "scraper.exe" } else { "scraper" };
                            d.join(name)
                        }))
                        .unwrap_or_else(|| std::path::PathBuf::from("scraper"));

                    let output = tokio::process::Command::new(&scraper_path)
                        .args(["--ticker", &ticker])
                        .output()
                        .await
                        .map_err(|e| format!("Failed to spawn scraper: {e}"))?;

                    if output.status.success() {
                        Ok(())
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        Err(format!("Scraper failed: {}", stderr.chars().take(200).collect::<String>()))
                    }
                },
                Message::FetchTickerComplete,
            ))
        }
        Message::FetchTickerComplete(result) => {
            state.fetching_ticker = false;
            match result {
                Ok(()) => {
                    state.push_toast(format!("{} data fetched!", state.selected_ticker));
                    // Auto-refresh from DB
                    if let Some(pool) = &state.pool {
                        Some(Task::batch([
                            Dashboard::fetch_all(pool, state.selected_ticker.clone()),
                            Task::perform(
                                crate::db::fetch_lagrange_history(
                                    std::sync::Arc::clone(pool),
                                    state.selected_ticker.clone(),
                                ),
                                Message::LagrangeHistoryLoaded,
                            ),
                        ]))
                    } else {
                        Some(Task::none())
                    }
                }
                Err(e) => {
                    state.fetch_ticker_error = Some(e.clone());
                    state.push_toast(format!("Fetch failed: {e}"));
                    Some(Task::none())
                }
            }
        }

        _ => None,
    }
}
