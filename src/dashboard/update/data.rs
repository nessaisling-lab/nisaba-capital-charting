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
            // v9.0: reset chart draw-in animation on ticker switch
            state.chart_draw_progress = 0.0;
            state.animating = true;
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

        Message::FavoritesLoaded(Ok(tickers)) => {
            state.favorites = tickers;
            Some(Task::none())
        }
        Message::FavoritesLoaded(Err(_)) => Some(Task::none()),

        Message::ToggleFavorite(ticker) => {
            if let Some(pool) = &state.pool {
                Some(Task::perform(
                    crate::db::toggle_favorite(Arc::clone(pool), ticker),
                    Message::FavoritesLoaded,
                ))
            } else {
                Some(Task::none())
            }
        }

        Message::WikiSummaryLoaded(Ok(maybe)) => {
            state.wiki_thumbnail_bytes = None;
            let task = match &maybe {
                Some(s) => match s.thumbnail_url.as_deref() {
                    Some(url) if !url.is_empty() => Task::perform(
                        crate::db::fetch_wiki_thumbnail(url.to_string()),
                        Message::WikiThumbnailLoaded,
                    ),
                    _ => Task::none(),
                },
                None => Task::none(),
            };
            state.wiki_summary = maybe;
            Some(task)
        }
        Message::WikiSummaryLoaded(Err(_)) => {
            state.wiki_summary = None;
            state.wiki_thumbnail_bytes = None;
            Some(Task::none())
        }
        Message::WikiThumbnailLoaded(Ok(bytes)) => {
            state.wiki_thumbnail_bytes = Some(bytes);
            Some(Task::none())
        }
        Message::WikiThumbnailLoaded(Err(_)) => {
            state.wiki_thumbnail_bytes = None;
            Some(Task::none())
        }

        Message::DataLoaded(Ok(rows)) => {
            state.refreshing = false;
            // v11.6.K — invalidate price-chart cache so the static layers
            // re-render with fresh OHLCV / SMA / BB data on next view tick.
            state.price_chart_cache.clear();
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
            state.suggest_calculator_defaults();
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
        Message::FearGreedLoaded(Ok(fg)) => {
            // Trigger gauge sweep animation
            state.gauge_anim_from = state.fear_greed.as_ref().map(|f| f.0).unwrap_or(0.0);
            state.gauge_anim_to = fg.0;
            state.gauge_anim_progress = 0.0;
            state.animating = true;
            state.fear_greed = Some(fg);
            Some(Task::none())
        }
        Message::FearGreedLoaded(Err(e)) => { state.fear_greed_err = Some(e); Some(Task::none()) }
        Message::MarketFGLoaded(Ok(fg)) => { state.market_fg = Some(fg); Some(Task::none()) }
        Message::MarketFGLoaded(Err(e)) => { state.market_fg_err = Some(e); Some(Task::none()) }
        Message::MacroDataLoaded(Ok(data)) => { state.macro_data = data; Some(Task::none()) }
        Message::MacroDataLoaded(Err(_)) => Some(Task::none()),
        Message::ShortInterestLoaded(Ok(si)) => { state.short_interest = si; Some(Task::none()) }
        Message::ShortInterestLoaded(Err(_)) => Some(Task::none()),
        Message::RssToneLoaded(Ok(tone)) => { state.rss_tone = tone; Some(Task::none()) }
        Message::RssToneLoaded(Err(_)) => Some(Task::none()),

        Message::FundamentalsLoaded(Ok(f)) => {
            state.fundamentals = f;
            state.suggest_calculator_defaults();
            state.compute_dcf_if_ready();
            state.recompute_agent_if_active();
            Some(Task::none())
        }
        Message::FundamentalsLoaded(Err(_)) => Some(Task::none()),

        Message::LagrangeHistoryLoaded(Ok(h)) => {
            // v11.5.F7 — pre-fill backtest threshold inputs from this
            // ticker's actual astro distribution. Buy = µ + 0.7σ (top
            // ~25% of the ticker's days), Sell = µ − 0.7σ (bottom ~25%).
            // Falls back to 65/35 when the sample is too thin.
            let astros: Vec<f32> = h.iter().filter_map(|r| r.astro_score).collect();
            if astros.len() >= 14 {
                let n = astros.len() as f32;
                let mean = astros.iter().sum::<f32>() / n;
                let var = astros.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / n;
                let sd = var.sqrt();
                let buy  = (mean + 0.7 * sd).clamp(50.0, 90.0).round() as i32;
                let sell = (mean - 0.7 * sd).clamp(10.0, 50.0).round() as i32;
                state.backtest_buy_input  = buy.to_string();
                state.backtest_sell_input = sell.to_string();
            }
            state.lagrange_history = h;
            Some(Task::none())
        }
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
            if let Some(persona) = state.active_agent {
                Some(state.update(Message::AgentSelected(persona)))
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
            let ticker = state.selected_ticker.clone();

            // Pre-check: locate scraper binary before spawning
            let scraper_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| {
                    let name = if cfg!(windows) { "scraper.exe" } else { "scraper" };
                    d.join(name)
                }))
                .unwrap_or_else(|| std::path::PathBuf::from("scraper"));

            if !scraper_path.exists() {
                let msg = format!(
                    "Scraper not found at {}",
                    scraper_path.display()
                );
                state.notify_error(msg);
                return Some(Task::none());
            }

            state.fetching_ticker = true;
            state.fetch_start_time = Some(std::time::Instant::now());

            // v12.1 — emit a sticky sparkly fetch pill. Replaced on
            // FetchTickerComplete (success → success pill, error → error
            // pill). Sticky = no TTL so it persists for the full fetch.
            let id = state.next_notif_id();
            // Track id so we can dismiss it on completion.
            state.fetch_notification_id = Some(id);
            let n = crate::notifications::Notification::new(
                id,
                crate::notifications::NotificationVariant::Sparkly,
                format!("Fetching {ticker}…"),
            )
            .with_emphasis(ticker.clone())
            .sticky();
            state.push_notification(n);
            // v11.9 (revised) — no push_toast here. Chrome fetching pill
            // (built in build_tab_bar) is the visible indicator while
            // fetching_ticker = true. Toast was causing layout shift +
            // expired before fetch completed.
            // v11.6.J — 90-second hard timeout. Without this, an unresponsive
            // child process leaves the UI stuck on "Fetching..." forever.
            // User feedback: "When I fetched, this ended up being stuck."
            Some(Task::perform(
                async move {
                    let fetch_future = async {
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
                    };
                    match tokio::time::timeout(std::time::Duration::from_secs(90), fetch_future).await {
                        Ok(result) => result,
                        Err(_) => Err("Fetch timed out after 90s. Scraper may be hung — try again.".to_string()),
                    }
                },
                Message::FetchTickerComplete,
            ))
        }
        Message::FetchTickerComplete(result) => {
            state.fetching_ticker = false;
            state.fetch_start_time = None;

            // v12.1 — drop the sticky sparkly fetch pill, replace with
            // success or error pill (TTL'd).
            if let Some(fetch_id) = state.fetch_notification_id.take() {
                state.dismiss_notification(fetch_id);
            }

            match result {
                Ok(()) => {
                    state.notify_success(format!("Fetched {}", state.selected_ticker));
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
                    state.notify_error(format!("Fetch failed: {e}"));
                    Some(Task::none())
                }
            }
        }

        _ => None,
    }
}
