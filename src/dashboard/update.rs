use chrono::Datelike;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::widget::text_input;
use iced::{Subscription, Task, Theme};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use notify_rust;

use pursuit_week4_automation::models::LagrangeAlert;

use crate::db::{
    add_to_watchlist, connect_db, create_watchlist, delete_watchlist, fetch_8k_filings,
    fetch_backtest_data, fetch_portfolio_pnl, fetch_transactions,
    insert_transaction, delete_transaction, fetch_astro_calendar, fetch_settings, upsert_setting, WatchlistRow,
    fetch_alerts, fetch_analyst_rating, fetch_astro_active_aspects, fetch_horoscope,
    fetch_astro_score, fetch_available_sectors, fetch_compare_data, fetch_daily_transits,
    fetch_fear_greed, fetch_fundamentals, fetch_holdings, fetch_insider_trades, fetch_ticker_earnings,
    fetch_lagrange_history, fetch_macro_indicators, fetch_market_fear_greed, fetch_natal_chart,
    fetch_named_watchlists, fetch_news, fetch_portfolio, fetch_prices, fetch_recently_viewed,
    fetch_sector_summaries, fetch_sentiment, fetch_short_interest, fetch_universe_count,
    fetch_universe_page, fetch_watchlist_summaries, fetch_watchlist_tickers, load_tickers,
    mark_alert_read, remove_from_watchlist, search_tickers, upsert_recently_viewed,
};
use crate::indicators::Indicators;
use crate::state::{Dashboard, Message};
use crate::tabs::Tab;

/// Stable ID for the ticker search text_input, used for programmatic focus.
pub const SEARCH_INPUT_ID: &str = "ticker-search";

impl Dashboard {
    pub fn new() -> (Self, Task<Message>) {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:dev@localhost:5432/financial_dashboard".to_string()
        });
        (
            Dashboard { status: "Connecting to database...".to_string(), ..Default::default() },
            Task::batch([
                Task::perform(connect_db(database_url), Message::PoolReady),
                Task::perform(fetch_fear_greed(), Message::FearGreedLoaded),
            ]),
        )
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn fetch_all(pool: &Arc<PgPool>, ticker: String) -> Task<Message> {
        Task::batch([
            Task::perform(fetch_prices(Arc::clone(pool), ticker.clone()), Message::DataLoaded),
            Task::perform(fetch_insider_trades(Arc::clone(pool), ticker.clone()), Message::InsiderTradesLoaded),
            Task::perform(fetch_8k_filings(Arc::clone(pool), ticker.clone()), Message::FilingsLoaded),
            Task::perform(fetch_holdings(Arc::clone(pool), ticker.clone()), Message::HoldingsLoaded),
            Task::perform(fetch_news(Arc::clone(pool), ticker.clone()), Message::NewsLoaded),
            Task::perform(fetch_analyst_rating(Arc::clone(pool), ticker.clone()), Message::AnalystRatingLoaded),
            Task::perform(fetch_sentiment(Arc::clone(pool), ticker.clone()), Message::SentimentLoaded),
            Task::perform(fetch_astro_score(Arc::clone(pool), ticker.clone()), Message::AstroScoreLoaded),
            Task::perform(fetch_natal_chart(Arc::clone(pool), ticker.clone()), Message::NatalChartLoaded),
            Task::perform(fetch_astro_active_aspects(Arc::clone(pool), ticker.clone()), Message::AstroAspectsLoaded),
            Task::perform(fetch_horoscope(Arc::clone(pool), ticker.clone()), Message::HoroscopeLoaded),
            Task::perform(fetch_short_interest(Arc::clone(pool), ticker.clone()), Message::ShortInterestLoaded),
            Task::perform(fetch_fundamentals(Arc::clone(pool), ticker.clone()), Message::FundamentalsLoaded),
            Task::perform(fetch_ticker_earnings(Arc::clone(pool), ticker), Message::EarningsLoaded),
        ])
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PoolReady(Ok(pool)) => {
                self.status = "Loading tickers...".to_string();
                let p = Arc::clone(&pool);
                self.pool = Some(pool);
                Task::perform(load_tickers(p), Message::TickersLoaded)
            }
            Message::PoolReady(Err(e)) => {
                self.status = format!("DB connection failed: {e}");
                Task::none()
            }
            Message::TickersLoaded(Ok(tickers)) => {
                if !tickers.is_empty() && !tickers.contains(&self.selected_ticker) {
                    self.selected_ticker = tickers[0].clone();
                }
                self.tickers = tickers;
                if let Some(pool) = &self.pool {
                    Task::batch([
                        Self::fetch_all(pool, self.selected_ticker.clone()),
                        Task::perform(fetch_ticker_earnings(Arc::clone(pool), self.selected_ticker.clone()), Message::EarningsLoaded),
                        Task::perform(fetch_market_fear_greed(Arc::clone(pool)), Message::MarketFGLoaded),
                        Task::perform(fetch_daily_transits(Arc::clone(pool)), Message::TransitsLoaded),
                        Task::perform(fetch_macro_indicators(Arc::clone(pool)), Message::MacroDataLoaded),
                        Task::perform(fetch_watchlist_summaries(Arc::clone(pool)), Message::WatchlistLoaded),
                        Task::perform(fetch_lagrange_history(Arc::clone(pool), self.selected_ticker.clone()), Message::LagrangeHistoryLoaded),
                        Task::perform(fetch_portfolio(Arc::clone(pool)), Message::PortfolioLoaded),
                        Task::perform(fetch_recently_viewed(Arc::clone(pool)), Message::RecentlyViewedLoaded),
                        Task::perform(fetch_alerts(Arc::clone(pool)), Message::AlertsLoaded),
                        Task::perform(fetch_universe_page(Arc::clone(pool), None, None, 0, 50), Message::UniverseLoaded),
                        Task::perform(fetch_universe_count(Arc::clone(pool), None, None), Message::UniverseCountLoaded),
                        Task::perform(fetch_available_sectors(Arc::clone(pool)), Message::UniverseSectorsLoaded),
                        Task::perform(fetch_sector_summaries(Arc::clone(pool)), Message::SectorSummariesLoaded),
                        Task::perform(fetch_named_watchlists(Arc::clone(pool)), Message::NamedWatchlistsLoaded),
                        Task::perform(fetch_portfolio_pnl(Arc::clone(pool)), Message::PortfolioPnlLoaded),
                        Task::perform(fetch_transactions(Arc::clone(pool)), Message::TransactionsLoaded),
                        Task::perform(fetch_settings(Arc::clone(pool)), Message::SettingsLoaded),
                        {
                            let now = chrono::Local::now().date_naive();
                            let start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
                            let end = if now.month() == 12 {
                                chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap().pred_opt().unwrap()
                            } else {
                                chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1).unwrap().pred_opt().unwrap()
                            };
                            Task::perform(
                                fetch_astro_calendar(Arc::clone(pool), self.selected_ticker.clone(), start, end),
                                Message::CalendarLoaded,
                            )
                        },
                    ])
                } else {
                    Task::none()
                }
            }
            Message::TickersLoaded(Err(e)) => {
                self.status = format!("Failed to load tickers: {e}");
                Task::none()
            }
            Message::TickerSelected(ticker) => {
                if ticker == self.selected_ticker { return Task::none(); }
                self.selected_ticker = ticker.clone();
                self.rows = vec![];
                self.indicators = None;
                self.insider_trades = vec![];
                self.filings_8k = vec![];
                self.holdings = vec![];
                self.news = vec![];
                self.analyst_rating = None;
                self.sentiment = None;
                self.astro_score = None;
                self.astro_aspects = vec![];
                self.natal_positions = vec![];
                self.short_interest = None;
                self.fundamentals = None;
                self.agent_analysis = None;
                self.lagrange_history = vec![];
                self.status = format!("Loading {ticker}...");
                if let Some(pool) = &self.pool {
                    let rv_pool    = Arc::clone(pool);
                    let rv_ticker  = ticker.clone();
                    Task::batch([
                        Self::fetch_all(pool, ticker.clone()),
                        Task::perform(fetch_lagrange_history(Arc::clone(pool), ticker), Message::LagrangeHistoryLoaded),
                        Task::perform(
                            async move {
                                upsert_recently_viewed(Arc::clone(&rv_pool), rv_ticker).await;
                                fetch_recently_viewed(rv_pool).await
                            },
                            Message::RecentlyViewedLoaded,
                        ),
                    ])
                } else {
                    Task::none()
                }
            }
            Message::TickerSearchInput(s) => {
                // Guard: if we just dismissed (via selection or submit), ignore
                // the on_input event that fires when the text_input is cleared.
                if self.autocomplete_dismissed {
                    self.autocomplete_dismissed = false;
                    return Task::none();
                }
                self.ticker_search_input = s.clone();
                if s.trim().is_empty() {
                    self.autocomplete_suggestions = vec![];
                    return Task::none();
                }
                if let Some(pool) = &self.pool {
                    Task::perform(
                        search_tickers(Arc::clone(pool), s),
                        |res| Message::AutocompleteResults(res.unwrap_or_default()),
                    )
                } else {
                    Task::none()
                }
            }
            Message::AutocompleteResults(suggestions) => {
                // Don't repopulate if the dropdown was dismissed since the query was sent
                if self.autocomplete_dismissed || self.ticker_search_input.is_empty() {
                    return Task::none();
                }
                self.autocomplete_suggestions = suggestions;
                Task::none()
            }
            Message::AutocompleteSelected(ticker) => {
                self.ticker_search_input = String::new();
                self.autocomplete_suggestions = vec![];
                self.autocomplete_dismissed = true;
                self.update(Message::TickerSelected(ticker))
            }
            Message::TickerSearchSubmit => {
                let ticker = self.ticker_search_input.trim().to_uppercase();
                if ticker.is_empty() { return Task::none(); }
                self.ticker_search_input = String::new();
                self.autocomplete_suggestions = vec![];
                self.autocomplete_dismissed = true;
                self.update(Message::TickerSelected(ticker))
            }
            Message::RecentlyViewedLoaded(Ok(tickers)) => {
                self.recently_viewed = tickers;
                Task::none()
            }
            Message::RecentlyViewedLoaded(Err(_)) => Task::none(),
            Message::DataLoaded(Ok(rows)) => {
                self.refreshing = false;
                self.status = if rows.is_empty() {
                    format!("{} — no data yet (run the scraper first)", self.selected_ticker)
                } else {
                    format!("Loaded {} rows for {}", rows.len(), self.selected_ticker)
                };
                if rows.len() >= 20 {
                    let prices: Vec<f32> = rows.iter().rev()
                        .map(|r| r.close.to_string().parse::<f32>().unwrap_or(0.0))
                        .collect();
                    self.indicators = Some(Indicators::compute(&prices));
                }
                self.rows = rows;
                Task::none()
            }
            Message::DataLoaded(Err(e)) => {
                self.refreshing = false;
                self.status = format!("Query error (showing stale data): {e}");
                Task::none()
            }
            Message::InsiderTradesLoaded(Ok(t))   => { self.insider_trades = t;    Task::none() }
            Message::InsiderTradesLoaded(Err(_))   => Task::none(),
            Message::FilingsLoaded(Ok(f))           => { self.filings_8k = f;       Task::none() }
            Message::FilingsLoaded(Err(_))           => Task::none(),
            Message::HoldingsLoaded(Ok(h))          => { self.holdings = h;         Task::none() }
            Message::HoldingsLoaded(Err(_))          => Task::none(),
            Message::NewsLoaded(Ok(n))              => { self.news = n;             Task::none() }
            Message::NewsLoaded(Err(_))              => Task::none(),
            Message::EarningsLoaded(Ok(e))          => { self.earnings = e;         Task::none() }
            Message::EarningsLoaded(Err(_))          => Task::none(),
            Message::AnalystRatingLoaded(Ok(r))     => { self.analyst_rating = r;   Task::none() }
            Message::AnalystRatingLoaded(Err(_))     => Task::none(),
            Message::SentimentLoaded(Ok(s))         => { self.sentiment = s;        Task::none() }
            Message::SentimentLoaded(Err(_))         => Task::none(),
            Message::FearGreedLoaded(Ok(fg))        => { self.fear_greed = Some(fg);      Task::none() }
            Message::FearGreedLoaded(Err(e))         => { self.fear_greed_err = Some(e);  Task::none() }
            Message::MarketFGLoaded(Ok(fg))         => { self.market_fg = Some(fg);       Task::none() }
            Message::MarketFGLoaded(Err(e))          => { self.market_fg_err = Some(e);   Task::none() }
            Message::AstroScoreLoaded(Ok(s))        => { self.astro_score = s;      Task::none() }
            Message::AstroScoreLoaded(Err(_))        => Task::none(),
            Message::NatalChartLoaded(Ok(p))        => { self.natal_positions = p;  Task::none() }
            Message::NatalChartLoaded(Err(_))        => Task::none(),
            Message::TransitsLoaded(Ok(t))          => { self.daily_transits = t;   Task::none() }
            Message::TransitsLoaded(Err(_))          => Task::none(),
            Message::AstroAspectsLoaded(Ok(v)) => {
                self.astro_aspects = v.as_array().cloned().unwrap_or_default();
                Task::none()
            }
            Message::AstroAspectsLoaded(Err(_)) => Task::none(),
            Message::HoroscopeLoaded(Ok(reading)) => { self.horoscope = reading; Task::none() }
            Message::HoroscopeLoaded(Err(_))      => Task::none(),
            Message::MacroDataLoaded(Ok(data))    => { self.macro_data = data;     Task::none() }
            Message::MacroDataLoaded(Err(_))       => Task::none(),
            Message::ShortInterestLoaded(Ok(si))  => { self.short_interest = si;   Task::none() }
            Message::ShortInterestLoaded(Err(_))   => Task::none(),
            Message::FundamentalsLoaded(Ok(f))     => {
                self.fundamentals = f;
                // Auto-compute DCF if we have fundamentals data
                self.compute_dcf_if_ready();
                // Re-run active agent analysis with new fundamentals
                self.recompute_agent_if_active();
                Task::none()
            }
            Message::FundamentalsLoaded(Err(_))     => Task::none(),
            Message::DcfGrowthRateInput(s)    => { self.dcf_growth_rate = s;    Task::none() }
            Message::DcfGrowthYearsInput(s)   => { self.dcf_growth_years = s;   Task::none() }
            Message::DcfTerminalGrowthInput(s) => { self.dcf_terminal_growth = s; Task::none() }
            Message::DcfDiscountRateInput(s)  => { self.dcf_discount_rate = s;  Task::none() }
            Message::DcfCompute => {
                self.compute_dcf_if_ready();
                Task::none()
            }
            Message::AgentSelected(persona) => {
                self.active_agent = Some(persona);
                self.recompute_agent_if_active();
                Task::none()
            }
            Message::CompareInput(s) => { self.compare_input = s; Task::none() }
            Message::CompareAdd => {
                let ticker = self.compare_input.trim().to_uppercase();
                if ticker.is_empty() || self.compare_tickers.len() >= 4
                    || self.compare_tickers.contains(&ticker)
                {
                    return Task::none();
                }
                self.compare_input = String::new();
                self.compare_tickers.push(ticker);
                self.refresh_compare()
            }
            Message::CompareRemove(ticker) => {
                self.compare_tickers.retain(|t| t != &ticker);
                if self.compare_tickers.is_empty() {
                    self.compare_data = vec![];
                    Task::none()
                } else {
                    self.refresh_compare()
                }
            }
            Message::CompareDataLoaded(Ok(data)) => {
                self.compare_data = data;
                Task::none()
            }
            Message::CompareDataLoaded(Err(_)) => Task::none(),
            Message::UniverseLoaded(Ok(rows)) => {
                self.universe_rows = rows;
                Task::none()
            }
            Message::UniverseLoaded(Err(_)) => Task::none(),
            Message::UniverseCountLoaded(Ok(n)) => { self.universe_total = n; Task::none() }
            Message::UniverseCountLoaded(Err(_)) => Task::none(),
            Message::UniverseSectorsLoaded(Ok(s)) => { self.universe_sectors = s; Task::none() }
            Message::UniverseSectorsLoaded(Err(_)) => Task::none(),
            Message::SectorSummariesLoaded(Ok(s)) => { self.sector_summaries = s; Task::none() }
            Message::SectorSummariesLoaded(Err(_)) => Task::none(),
            Message::UniverseFilterZone(zone) => {
                self.universe_filter_zone = zone;
                self.universe_page = 0;
                self.refresh_universe()
            }
            Message::UniverseFilterSector(sector) => {
                self.universe_filter_sector = sector;
                self.universe_page = 0;
                self.refresh_universe()
            }
            Message::UniverseNextPage => {
                let max_page = ((self.universe_total as usize).saturating_sub(1)) / 50;
                if self.universe_page < max_page {
                    self.universe_page += 1;
                    self.refresh_universe()
                } else {
                    Task::none()
                }
            }
            Message::UniversePrevPage => {
                if self.universe_page > 0 {
                    self.universe_page -= 1;
                    self.refresh_universe()
                } else {
                    Task::none()
                }
            }
            Message::WatchlistLoaded(Ok(mut rows)) => {
                sort_watchlist(&mut rows, self.sort_watchlist_by_score);
                self.watchlist = rows;
                Task::none()
            }
            Message::WatchlistLoaded(Err(_)) => Task::none(),
            // ── Named Watchlists ──────────────────────────────────
            Message::NamedWatchlistsLoaded(Ok(wls)) => {
                self.named_watchlists = wls;
                // Auto-select first watchlist if none selected
                if self.active_watchlist_id.is_none() {
                    if let Some(first) = self.named_watchlists.first() {
                        self.active_watchlist_id = Some(first.id);
                        if let Some(pool) = &self.pool {
                            return Task::perform(
                                fetch_watchlist_tickers(Arc::clone(pool), first.id),
                                Message::WatchlistTickersLoaded,
                            );
                        }
                    }
                }
                Task::none()
            }
            Message::NamedWatchlistsLoaded(Err(_)) => Task::none(),
            Message::WatchlistTickersLoaded(Ok(tickers)) => {
                self.watchlist_tickers_list = tickers;
                Task::none()
            }
            Message::WatchlistTickersLoaded(Err(_)) => Task::none(),
            Message::SelectNamedWatchlist(id) => {
                self.active_watchlist_id = Some(id);
                if let Some(pool) = &self.pool {
                    Task::perform(
                        fetch_watchlist_tickers(Arc::clone(pool), id),
                        Message::WatchlistTickersLoaded,
                    )
                } else {
                    Task::none()
                }
            }
            Message::NewWatchlistNameInput(s) => {
                self.new_watchlist_name = s;
                Task::none()
            }
            Message::CreateWatchlist => {
                let name = self.new_watchlist_name.trim().to_string();
                if name.is_empty() { return Task::none(); }
                self.new_watchlist_name.clear();
                if let Some(pool) = &self.pool {
                    Task::perform(
                        create_watchlist(Arc::clone(pool), name),
                        Message::WatchlistCreated,
                    )
                } else {
                    Task::none()
                }
            }
            Message::WatchlistCreated(Ok(wl)) => {
                let new_id = wl.id;
                self.named_watchlists.push(wl);
                self.active_watchlist_id = Some(new_id);
                self.watchlist_tickers_list.clear();
                Task::none()
            }
            Message::WatchlistCreated(Err(_)) => Task::none(),
            Message::WatchlistAddTickerInput(s) => {
                self.watchlist_add_ticker = s;
                Task::none()
            }
            Message::WatchlistAddTicker => {
                let ticker = self.watchlist_add_ticker.trim().to_uppercase();
                if ticker.is_empty() { return Task::none(); }
                self.watchlist_add_ticker.clear();
                if let (Some(pool), Some(wl_id)) = (&self.pool, self.active_watchlist_id) {
                    self.watchlist_tickers_list.push(ticker.clone());
                    Task::perform(
                        add_to_watchlist(Arc::clone(pool), wl_id, ticker),
                        Message::WatchlistMutated,
                    )
                } else {
                    Task::none()
                }
            }
            Message::WatchlistRemoveTicker(ticker) => {
                self.watchlist_tickers_list.retain(|t| t != &ticker);
                if let (Some(pool), Some(wl_id)) = (&self.pool, self.active_watchlist_id) {
                    Task::perform(
                        remove_from_watchlist(Arc::clone(pool), wl_id, ticker),
                        Message::WatchlistMutated,
                    )
                } else {
                    Task::none()
                }
            }
            Message::WatchlistMutated(Ok(())) => Task::none(),
            Message::WatchlistMutated(Err(_)) => Task::none(),
            Message::DeleteActiveWatchlist => {
                if let (Some(pool), Some(wl_id)) = (&self.pool, self.active_watchlist_id) {
                    self.named_watchlists.retain(|w| w.id != wl_id);
                    self.active_watchlist_id = self.named_watchlists.first().map(|w| w.id);
                    self.watchlist_tickers_list.clear();
                    let pool2 = Arc::clone(pool);
                    let tasks = vec![
                        Task::perform(delete_watchlist(Arc::clone(pool), wl_id), Message::WatchlistMutated),
                    ];
                    // Reload tickers for newly selected watchlist
                    if let Some(new_id) = self.active_watchlist_id {
                        return Task::batch(
                            tasks.into_iter().chain(std::iter::once(
                                Task::perform(fetch_watchlist_tickers(pool2, new_id), Message::WatchlistTickersLoaded),
                            ))
                        );
                    }
                    Task::batch(tasks)
                } else {
                    Task::none()
                }
            }
            Message::ExportCsv => {
                // Export current watchlist data to CSV via native file dialog
                let rows = self.watchlist.clone();
                let ticker = self.selected_ticker.clone();
                Task::perform(async move {
                    export_watchlist_csv(rows, &ticker).await
                }, |_: Result<(), String>| Message::WatchlistMutated(Ok(())))
            }
            Message::ToggleWatchlistSort => {
                self.sort_watchlist_by_score = !self.sort_watchlist_by_score;
                sort_watchlist(&mut self.watchlist, self.sort_watchlist_by_score);
                Task::none()
            }
            Message::LagrangeHistoryLoaded(Ok(h))  => { self.lagrange_history = h; Task::none() }
            Message::LagrangeHistoryLoaded(Err(_))  => Task::none(),
            Message::PortfolioLoaded(Ok(p))         => { self.portfolio = p;        Task::none() }
            Message::PortfolioLoaded(Err(_))         => Task::none(),
            Message::CopyText(s)  => iced::clipboard::write(s),
            Message::OpenUrl(url) => {
                let _ = open::that_detached(&url);
                Task::none()
            }
            Message::TabSelected(tab) => {
                self.active_tab = tab;
                Task::none()
            }
            Message::ToggleTheme => {
                self.theme_mode = self.theme_mode.next();
                self.theme = crate::theme::iced_theme(self.theme_mode);
                Task::none()
            }
            Message::RefreshNow => {
                if let Some(pool) = &self.pool {
                    self.refreshing = true;
                    self.status = "Refreshing...".to_string();
                    Task::batch([
                        Self::fetch_all(pool, self.selected_ticker.clone()),
                        Task::perform(fetch_lagrange_history(Arc::clone(pool), self.selected_ticker.clone()), Message::LagrangeHistoryLoaded),
                        Task::perform(fetch_market_fear_greed(Arc::clone(pool)), Message::MarketFGLoaded),
                        Task::perform(fetch_watchlist_summaries(Arc::clone(pool)), Message::WatchlistLoaded),
                        Task::perform(fetch_macro_indicators(Arc::clone(pool)), Message::MacroDataLoaded),
                        Task::perform(fetch_alerts(Arc::clone(pool)), Message::AlertsLoaded),
                    ])
                } else {
                    Task::none()
                }
            }
            Message::Tick => {
                // Update theme for Auto mode (picks up time-of-day changes)
                if self.theme_mode == crate::theme::ThemeMode::Auto {
                    self.theme = crate::theme::iced_theme(self.theme_mode);
                }
                if let Some(pool) = &self.pool {
                    Task::batch([
                        Self::fetch_all(pool, self.selected_ticker.clone()),
                        Task::perform(fetch_alerts(Arc::clone(pool)), Message::AlertsLoaded),
                    ])
                } else {
                    Task::none()
                }
            }
            Message::AlertsLoaded(Ok(alerts)) => {
                let unread_count = alerts.iter().filter(|a| !a.is_read).count();
                self.unread_alert_count = unread_count;
                if unread_count > 0 && !self.notifications_fired {
                    self.notifications_fired = true;
                    let unread: Vec<LagrangeAlert> = alerts.iter().filter(|a| !a.is_read).cloned().collect();
                    self.alerts = alerts;
                    Task::perform(
                        async move { fire_toast(unread).await },
                        |_| Message::NotifyAlerts,
                    )
                } else {
                    self.alerts = alerts;
                    Task::none()
                }
            }
            Message::AlertsLoaded(Err(_)) => Task::none(),
            Message::MarkAlertRead(id) => {
                // Optimistic in-memory flip — no waiting for DB round-trip
                if let Some(a) = self.alerts.iter_mut().find(|a| a.id == id) {
                    if !a.is_read {
                        a.is_read = true;
                        self.unread_alert_count = self.unread_alert_count.saturating_sub(1);
                    }
                }
                if let Some(pool) = &self.pool {
                    let p = Arc::clone(pool);
                    Task::perform(async move { mark_alert_read(p, id).await }, |_| Message::NotifyAlerts)
                } else {
                    Task::none()
                }
            }
            // ── Backtest ─���────────────────────────────────────
            Message::SetTimeframe(tf) => {
                self.chart_timeframe = tf;
                Task::none()
            }
            // ── Strategy Builder ─────────────────────────────
            Message::StrategyAddBuyCond(c) => {
                self.strategy.buy_conditions.push(c);
                Task::none()
            }
            Message::StrategyRemoveBuyCond(i) => {
                if i < self.strategy.buy_conditions.len() {
                    self.strategy.buy_conditions.remove(i);
                }
                Task::none()
            }
            Message::StrategyAddSellCond(c) => {
                self.strategy.sell_conditions.push(c);
                Task::none()
            }
            Message::StrategyRemoveSellCond(i) => {
                if i < self.strategy.sell_conditions.len() {
                    self.strategy.sell_conditions.remove(i);
                }
                Task::none()
            }
            Message::StrategyToggleBuyLogic => {
                self.strategy.buy_logic = match self.strategy.buy_logic {
                    crate::strategy::Logic::And => crate::strategy::Logic::Or,
                    crate::strategy::Logic::Or => crate::strategy::Logic::And,
                };
                Task::none()
            }
            Message::StrategyToggleSellLogic => {
                self.strategy.sell_logic = match self.strategy.sell_logic {
                    crate::strategy::Logic::And => crate::strategy::Logic::Or,
                    crate::strategy::Logic::Or => crate::strategy::Logic::And,
                };
                Task::none()
            }
            Message::RunStrategy => {
                if let Some(pool) = &self.pool {
                    Task::perform(
                        fetch_backtest_data(Arc::clone(pool), self.selected_ticker.clone()),
                        Message::StrategyDataLoaded,
                    )
                } else {
                    Task::none()
                }
            }
            Message::StrategyDataLoaded(Ok(rows)) => {
                // Build DaySnapshots with indicators
                let indicators_map = self.indicators.as_ref();
                let days: Vec<crate::strategy::DaySnapshot> = rows.iter().enumerate().map(|(i, r)| {
                    let close = r.close.to_string().parse::<f64>().unwrap_or(0.0);
                    // Try to find matching indicator data (indicators are in chronological order)
                    let (rsi, macd, macd_prev, sma50) = if let Some(ind) = indicators_map {
                        // Indicators match the full rows set, but backtest data may have different dates.
                        // Use simple index mapping if lengths match, otherwise None.
                        let rsi = ind.rsi_vals.get(i).copied().flatten();
                        let macd = ind.macd_line.get(i).copied().flatten();
                        let macd_prev = if i > 0 { ind.macd_line.get(i - 1).copied().flatten() } else { None };
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
                    }
                }).collect();
                let result = crate::strategy::run_strategy_backtest(
                    &self.selected_ticker,
                    &days,
                    &self.strategy,
                    10_000.0,
                );
                self.strategy_result = Some(result);
                Task::none()
            }
            Message::StrategyDataLoaded(Err(_)) => {
                self.strategy_result = None;
                Task::none()
            }
            Message::BacktestBuyInput(s) => {
                self.backtest_buy_input = s;
                Task::none()
            }
            Message::BacktestSellInput(s) => {
                self.backtest_sell_input = s;
                Task::none()
            }
            Message::RunBacktest => {
                // Parse thresholds from text inputs
                let buy = self.backtest_buy_input.parse::<f64>().unwrap_or(65.0);
                let sell = self.backtest_sell_input.parse::<f64>().unwrap_or(35.0);
                self.backtest_config = crate::backtest::BacktestConfig {
                    buy_threshold: buy,
                    sell_threshold: sell,
                    initial_capital: 10_000.0,
                };
                if let Some(pool) = &self.pool {
                    Task::perform(
                        fetch_backtest_data(Arc::clone(pool), self.selected_ticker.clone()),
                        Message::BacktestDataLoaded,
                    )
                } else {
                    Task::none()
                }
            }
            Message::BacktestDataLoaded(Ok(rows)) => {
                let days: Vec<crate::backtest::BacktestDay> = rows.iter().map(|r| {
                    crate::backtest::BacktestDay {
                        date: r.date,
                        close: r.close.to_string().parse::<f64>().unwrap_or(0.0),
                        astro_score: r.astro_score,
                    }
                }).collect();
                let result = crate::backtest::run_backtest(
                    &self.selected_ticker,
                    &days,
                    &self.backtest_config,
                );
                self.backtest_result = Some(result);
                Task::none()
            }
            Message::BacktestDataLoaded(Err(_)) => {
                self.backtest_result = None;
                Task::none()
            }
            // ── Portfolio P&L ───────���────────────────────────────
            Message::PortfolioPnlLoaded(Ok(rows)) => {
                self.portfolio_pnl = rows;
                Task::none()
            }
            Message::PortfolioPnlLoaded(Err(_)) => Task::none(),
            // ── Astro Calendar ──────────────────────────────
            Message::CalendarLoaded(Ok(rows)) => {
                self.calendar_days = rows.into_iter().map(|(date, score, label)| {
                    crate::calendar::CalendarDay { date, astro_score: Some(score), label }
                }).collect();
                Task::none()
            }
            Message::CalendarLoaded(Err(_)) => Task::none(),
            Message::CalendarPrevMonth => {
                if self.calendar_month == 1 {
                    self.calendar_month = 12;
                    self.calendar_year -= 1;
                } else {
                    self.calendar_month -= 1;
                }
                self.refresh_calendar()
            }
            Message::CalendarNextMonth => {
                if self.calendar_month == 12 {
                    self.calendar_month = 1;
                    self.calendar_year += 1;
                } else {
                    self.calendar_month += 1;
                }
                self.refresh_calendar()
            }
            // ── Transaction Log ──────────────────────────────
            // ── Settings ─────────────────────────────────────
            Message::SettingsLoaded(Ok(pairs)) => {
                for (k, v) in pairs {
                    // Apply settings to state
                    if k == "theme_mode" {
                        self.theme_mode = match v.as_str() {
                            "Light" => crate::theme::ThemeMode::AlwaysLight,
                            "Dark" => crate::theme::ThemeMode::AlwaysDark,
                            _ => crate::theme::ThemeMode::Auto,
                        };
                        self.theme = crate::theme::iced_theme(self.theme_mode);
                    }
                    if k == "refresh_interval_secs" {
                        self.settings_refresh_input = v.clone();
                    }
                    self.settings.insert(k, v);
                }
                Task::none()
            }
            Message::SettingsLoaded(Err(_)) => Task::none(),
            Message::SettingsRefreshInput(s) => { self.settings_refresh_input = s; Task::none() }
            Message::SaveSetting(key, value) => {
                self.settings.insert(key.clone(), value.clone());
                // Apply theme_mode immediately
                if key == "theme_mode" {
                    self.theme_mode = match value.as_str() {
                        "Light" => crate::theme::ThemeMode::AlwaysLight,
                        "Dark" => crate::theme::ThemeMode::AlwaysDark,
                        _ => crate::theme::ThemeMode::Auto,
                    };
                    self.theme = crate::theme::iced_theme(self.theme_mode);
                }
                if let Some(pool) = &self.pool {
                    Task::perform(
                        upsert_setting(Arc::clone(pool), key, value),
                        Message::SettingSaved,
                    )
                } else {
                    Task::none()
                }
            }
            Message::SettingSaved(Ok(())) => Task::none(),
            Message::SettingSaved(Err(_)) => Task::none(),
            Message::TransactionsLoaded(Ok(txs)) => { self.transactions = txs; Task::none() }
            Message::TransactionsLoaded(Err(_)) => Task::none(),
            Message::TxTickerInput(s) => { self.tx_ticker_input = s; Task::none() }
            Message::TxSharesInput(s) => { self.tx_shares_input = s; Task::none() }
            Message::TxPriceInput(s) => { self.tx_price_input = s; Task::none() }
            Message::TxToggleAction => {
                self.tx_action = if self.tx_action == "BUY" { "SELL".to_string() } else { "BUY".to_string() };
                Task::none()
            }
            Message::TxSubmit => {
                let ticker = self.tx_ticker_input.trim().to_uppercase();
                let shares = self.tx_shares_input.parse::<f32>().unwrap_or(0.0);
                let price = self.tx_price_input.parse::<f32>().unwrap_or(0.0);
                if ticker.is_empty() || shares <= 0.0 || price <= 0.0 { return Task::none(); }
                let action = self.tx_action.clone();
                self.tx_ticker_input.clear();
                self.tx_shares_input.clear();
                self.tx_price_input.clear();
                if let Some(pool) = &self.pool {
                    Task::perform(
                        insert_transaction(
                            Arc::clone(pool), ticker, action, shares, price,
                            chrono::Local::now().date_naive(), None,
                        ),
                        Message::TxCreated,
                    )
                } else {
                    Task::none()
                }
            }
            Message::TxCreated(Ok(tx)) => {
                self.transactions.insert(0, tx); // prepend (most recent first)
                Task::none()
            }
            Message::TxCreated(Err(_)) => Task::none(),
            Message::TxDelete(id) => {
                self.transactions.retain(|t| t.id != id);
                if let Some(pool) = &self.pool {
                    Task::perform(delete_transaction(Arc::clone(pool), id), Message::TxDeleted)
                } else {
                    Task::none()
                }
            }
            Message::TxDeleted(Ok(())) => Task::none(),
            Message::TxDeleted(Err(_)) => Task::none(),
            Message::FocusSearch => text_input::focus(SEARCH_INPUT_ID),
            Message::EscapePressed => {
                self.ticker_search_input = String::new();
                self.autocomplete_suggestions = vec![];
                self.autocomplete_dismissed = true;
                Task::none()
            }
            Message::NotifyAlerts => Task::none(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_secs(30)).map(|_| Message::Tick),
            iced::keyboard::on_key_press(handle_key_press),
        ])
    }

    /// Refresh the Universe Explorer with current filters and page.
    fn refresh_universe(&self) -> Task<Message> {
        if let Some(pool) = &self.pool {
            Task::batch([
                Task::perform(
                    fetch_universe_page(
                        Arc::clone(pool),
                        self.universe_filter_zone.clone(),
                        self.universe_filter_sector.clone(),
                        self.universe_page,
                        50,
                    ),
                    Message::UniverseLoaded,
                ),
                Task::perform(
                    fetch_universe_count(
                        Arc::clone(pool),
                        self.universe_filter_zone.clone(),
                        self.universe_filter_sector.clone(),
                    ),
                    Message::UniverseCountLoaded,
                ),
            ])
        } else {
            Task::none()
        }
    }

    /// Fetch comparison data for the current compare_tickers list.
    fn refresh_compare(&self) -> Task<Message> {
        if let Some(pool) = &self.pool {
            let tickers = self.compare_tickers.clone();
            Task::perform(
                fetch_compare_data(Arc::clone(pool), tickers),
                Message::CompareDataLoaded,
            )
        } else {
            Task::none()
        }
    }

    fn refresh_calendar(&self) -> Task<Message> {
        if let Some(pool) = &self.pool {
            let start = chrono::NaiveDate::from_ymd_opt(self.calendar_year, self.calendar_month, 1).unwrap();
            let end = if self.calendar_month == 12 {
                chrono::NaiveDate::from_ymd_opt(self.calendar_year + 1, 1, 1).unwrap().pred_opt().unwrap()
            } else {
                chrono::NaiveDate::from_ymd_opt(self.calendar_year, self.calendar_month + 1, 1).unwrap().pred_opt().unwrap()
            };
            Task::perform(
                fetch_astro_calendar(Arc::clone(pool), self.selected_ticker.clone(), start, end),
                Message::CalendarLoaded,
            )
        } else {
            Task::none()
        }
    }

}

/// Export watchlist rows to a CSV file via a native save-file dialog.
async fn export_watchlist_csv(rows: Vec<WatchlistRow>, _ticker: &str) -> Result<(), String> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Export Watchlist CSV")
        .add_filter("CSV", &["csv"])
        .set_file_name("watchlist.csv")
        .save_file()
        .await;
    let Some(handle) = handle else { return Ok(()); }; // user cancelled
    let path = handle.path();
    let mut wtr = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
    wtr.write_record(["Ticker", "Astro Score", "Astro Label", "Sentiment", "Sentiment Label", "Short %"])
        .map_err(|e| e.to_string())?;
    for r in &rows {
        wtr.write_record(&[
            &r.ticker,
            &r.astro_score.map(|v| format!("{v:.1}")).unwrap_or_default(),
            r.astro_label.as_deref().unwrap_or(""),
            &r.sentiment_score.as_ref().map(|v| v.to_string()).unwrap_or_default(),
            r.sentiment_label.as_deref().unwrap_or(""),
            &r.short_pct.as_ref().map(|v| v.to_string()).unwrap_or_default(),
        ]).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())?;
    Ok(())
}

impl Dashboard {
    /// Build an `AgentContext` from current state and run the active persona's analysis.
    pub fn recompute_agent_if_active(&mut self) {
        let Some(persona) = self.active_agent else { return; };
        let price = self.rows.first()
            .map(|r| r.close.to_string().parse::<f64>().unwrap_or(0.0));

        let ctx = crate::agents::AgentContext {
            ticker: self.selected_ticker.clone(),
            fundamentals: self.fundamentals.clone(),
            astro_score: self.astro_score.as_ref().and_then(|s| s.astro_score.map(|v| v as f64)),
            astro_label: self.astro_score.as_ref().and_then(|s| s.astro_label.clone()),
            dominant_theme: None, // v2.0.3 horoscope engine (future)
            concordance: None,    // v2.0.5 concordance (future)
            lagrange_score: self.indicators.as_ref().map(|ind| {
                let (score, _, _) = crate::indicators::compute_lagrange_score(
                    ind, &self.rows, &self.sentiment,
                    &self.astro_score, &self.macro_data, &self.short_interest,
                );
                score
            }),
            lagrange_label: self.indicators.as_ref().map(|ind| {
                let (_, label, _) = crate::indicators::compute_lagrange_score(
                    ind, &self.rows, &self.sentiment,
                    &self.astro_score, &self.macro_data, &self.short_interest,
                );
                label
            }),
            current_price: price,
            mercury_rx: self.astro_score.as_ref().and_then(|s| s.mercury_rx).unwrap_or(false),
            moon_phase: self.astro_score.as_ref().and_then(|s| s.moon_phase.clone()),
        };
        self.agent_analysis = Some(crate::agents::analyze(persona, &ctx));
    }

    /// Compute DCF if we have the required data (FCF + shares + price).
    pub fn compute_dcf_if_ready(&mut self) {
        let fcf = self.fundamentals.as_ref().and_then(|f| f.fcf);
        let shares = self.fundamentals.as_ref().and_then(|f| f.shares_outstanding);
        let price = self.rows.first()
            .map(|r| r.close.to_string().parse::<f64>().unwrap_or(0.0));

        if let (Some(fcf), Some(shares), Some(price)) = (fcf, shares, price) {
            if fcf <= 0 || shares <= 0 || price <= 0.0 {
                self.dcf_result = None;
                return;
            }
            let growth_rate = self.dcf_growth_rate.parse::<f64>().unwrap_or(10.0) / 100.0;
            let growth_years = self.dcf_growth_years.parse::<u32>().unwrap_or(5);
            let terminal_growth = self.dcf_terminal_growth.parse::<f64>().unwrap_or(2.5) / 100.0;
            let discount_rate = self.dcf_discount_rate.parse::<f64>().unwrap_or(10.0) / 100.0;

            let inputs = crate::dcf::DcfInputs {
                fcf: fcf as f64,
                growth_rate,
                growth_years,
                terminal_growth,
                discount_rate,
                shares_outstanding: shares as f64,
                current_price: price,
            };
            self.dcf_result = Some(crate::dcf::compute_dcf(&inputs));
        } else {
            self.dcf_result = None;
        }
    }
}

fn sort_watchlist(rows: &mut Vec<crate::db::WatchlistRow>, by_score: bool) {
    if by_score {
        rows.sort_by(|a, b| b.quick_score().partial_cmp(&a.quick_score()).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        rows.sort_by(|a, b| a.ticker.cmp(&b.ticker));
    }
}

async fn fire_toast(alerts: Vec<LagrangeAlert>) {
    let entries: Vec<String> = alerts.iter().take(3)
        .map(|a| format!("{} → {}", a.ticker, a.label))
        .collect();
    let mut body = entries.join(", ");
    if alerts.len() > 3 {
        body = format!("{} (+{} more)", body, alerts.len() - 3);
    }
    let summary = format!("Lagrange: {} alert{}", alerts.len(), if alerts.len() == 1 { "" } else { "s" });
    notify_rust::Notification::new()
        .summary(&summary)
        .body(&body)
        .show()
        .ok();
}

/// Global keyboard shortcut handler.
///
/// Ctrl+1..6  switch tabs
/// Ctrl+T     focus the ticker search box
/// Ctrl+R     refresh all data
/// Escape     clear search input and autocomplete
fn handle_key_press(key: Key, modifiers: Modifiers) -> Option<Message> {
    if modifiers.control() {
        match &key {
            Key::Character(c) => match c.as_str() {
                "1" => Some(Message::TabSelected(Tab::Astrology)),
                "2" => Some(Message::TabSelected(Tab::Overview)),
                "3" => Some(Message::TabSelected(Tab::Universe)),
                "4" => Some(Message::TabSelected(Tab::Fundamentals)),
                "5" => Some(Message::TabSelected(Tab::Research)),
                "6" => Some(Message::TabSelected(Tab::Portfolio)),
                "7" => Some(Message::TabSelected(Tab::Settings)),
                "t" | "T" => Some(Message::FocusSearch),
                "r" | "R" => Some(Message::RefreshNow),
                _ => None,
            },
            _ => None,
        }
    } else {
        match key {
            Key::Named(Named::Escape) => Some(Message::EscapePressed),
            _ => None,
        }
    }
}
