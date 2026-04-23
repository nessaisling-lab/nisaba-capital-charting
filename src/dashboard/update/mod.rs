mod helpers;

use chrono::Datelike;
use iced::widget::text_input;
use iced::{Subscription, Task, Theme};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

use pursuit_week4_automation::models::LagrangeAlert;

use crate::db::{
    add_to_watchlist, connect_db, create_watchlist, delete_watchlist, fetch_8k_filings,
    fetch_backtest_data, fetch_portfolio_pnl, fetch_transactions,
    insert_transaction, delete_transaction, fetch_astro_calendar, fetch_settings, upsert_setting,
    dismiss_alert, fetch_alerts, fetch_analyst_rating, fetch_astro_active_aspects, fetch_horoscope,
    mark_all_alerts_read,
    fetch_astro_score, fetch_available_sectors, fetch_daily_transits, fetch_retrograde_events,
    fetch_fear_greed, fetch_fundamentals, fetch_holdings, fetch_insider_trades, fetch_ticker_earnings,
    fetch_lagrange_history, fetch_macro_indicators, fetch_market_fear_greed, fetch_natal_chart,
    fetch_named_watchlists, fetch_news, fetch_polymarket, fetch_portfolio, fetch_prices, fetch_recently_viewed, fetch_rss_articles,
    fetch_sector_summaries, fetch_sentiment, fetch_short_interest, fetch_universe_count,
    fetch_universe_page, fetch_watchlist_summaries, fetch_watchlist_tickers, load_tickers,
    mark_alert_read, remove_from_watchlist, search_tickers, upsert_recently_viewed,
    fetch_sector_peers,
};
use crate::indicators::Indicators;
use crate::state::{Dashboard, Message};

pub(crate) use helpers::{handle_key_press, sort_watchlist, export_watchlist_csv, export_universe_csv, fire_toast};

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
                        Task::perform(fetch_universe_page(Arc::clone(pool), None, None, None, 0, 50), Message::UniverseLoaded),
                        Task::perform(fetch_universe_count(Arc::clone(pool), None, None, None), Message::UniverseCountLoaded),
                        Task::perform(fetch_available_sectors(Arc::clone(pool)), Message::UniverseSectorsLoaded),
                        Task::perform(fetch_sector_summaries(Arc::clone(pool)), Message::SectorSummariesLoaded),
                        Task::perform(fetch_named_watchlists(Arc::clone(pool)), Message::NamedWatchlistsLoaded),
                        Task::perform(fetch_portfolio_pnl(Arc::clone(pool)), Message::PortfolioPnlLoaded),
                        Task::perform(fetch_transactions(Arc::clone(pool)), Message::TransactionsLoaded),
                        Task::perform(fetch_settings(Arc::clone(pool)), Message::SettingsLoaded),
                        Task::perform(fetch_rss_articles(Arc::clone(pool)), Message::RssArticlesLoaded),
                        Task::perform(fetch_polymarket(Arc::clone(pool)), Message::PolymarketLoaded),
                        {
                            // Fetch retrograde station events for the last year (chart overlay)
                            let retro_start = chrono::Local::now().date_naive() - chrono::Duration::days(365);
                            let retro_end   = chrono::Local::now().date_naive();
                            Task::perform(
                                fetch_retrograde_events(Arc::clone(pool), retro_start, retro_end),
                                Message::RetroEventsLoaded,
                            )
                        },
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
                self.sector_peers = vec![];
                self.status = format!("Loading {ticker}...");
                if let Some(pool) = &self.pool {
                    let rv_pool    = Arc::clone(pool);
                    let rv_ticker  = ticker.clone();
                    Task::batch([
                        Self::fetch_all(pool, ticker.clone()),
                        Task::perform(fetch_lagrange_history(Arc::clone(pool), ticker.clone()), Message::LagrangeHistoryLoaded),
                        Task::perform(fetch_sector_peers(Arc::clone(pool), ticker), Message::SectorPeersLoaded),
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
            Message::RssArticlesLoaded(Ok(a))       => { self.rss_articles = a;     Task::none() }
            Message::RssArticlesLoaded(Err(_))       => Task::none(),
            Message::PolymarketLoaded(Ok(m))        => { self.polymarket = m;       Task::none() }
            Message::PolymarketLoaded(Err(_))        => Task::none(),
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
            Message::RetroEventsLoaded(Ok(events))   => { self.retrograde_events = events; Task::none() }
            Message::RetroEventsLoaded(Err(_))        => Task::none(),
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
                self.compute_dcf_if_ready();
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
            Message::CompareAddDirect(ticker) => {
                let ticker = ticker.to_uppercase();
                if self.compare_tickers.len() >= 4 || self.compare_tickers.contains(&ticker) {
                    return Task::none();
                }
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
            Message::SectorPeersLoaded(Ok(peers)) => { self.sector_peers = peers; Task::none() }
            Message::SectorPeersLoaded(Err(_)) => Task::none(),
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
            Message::UniverseSearchChanged(text) => {
                self.universe_search_text = text;
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
            Message::NamedWatchlistsLoaded(Ok(wls)) => {
                self.named_watchlists = wls;
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
                let rows = self.watchlist.clone();
                let ticker = self.selected_ticker.clone();
                Task::perform(async move {
                    export_watchlist_csv(rows, &ticker).await
                }, |_: Result<(), String>| Message::WatchlistMutated(Ok(())))
            }
            Message::ExportUniverseCsv => {
                let rows = self.universe_rows.clone();
                Task::perform(async move {
                    export_universe_csv(rows).await
                }, |_: Result<(), String>| Message::WatchlistMutated(Ok(())))
            }
            Message::ToggleWatchlistSort => {
                self.sort_watchlist_by_score = !self.sort_watchlist_by_score;
                sort_watchlist(&mut self.watchlist, self.sort_watchlist_by_score);
                Task::none()
            }
            Message::TogglePriceTable => {
                self.show_price_table = !self.show_price_table;
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
            Message::MarkAllAlertsRead => {
                for a in &mut self.alerts { a.is_read = true; }
                self.unread_alert_count = 0;
                if let Some(pool) = &self.pool {
                    let p = Arc::clone(pool);
                    Task::perform(async move { mark_all_alerts_read(p).await }, |_| Message::NotifyAlerts)
                } else {
                    Task::none()
                }
            }
            Message::DismissAlert(id) => {
                if let Some(a) = self.alerts.iter().find(|a| a.id == id) {
                    if !a.is_read {
                        self.unread_alert_count = self.unread_alert_count.saturating_sub(1);
                    }
                }
                self.alerts.retain(|a| a.id != id);
                if let Some(pool) = &self.pool {
                    let p = Arc::clone(pool);
                    Task::perform(async move { dismiss_alert(p, id).await }, |_| Message::NotifyAlerts)
                } else {
                    Task::none()
                }
            }
            Message::SetTimeframe(tf) => {
                self.chart_timeframe = tf;
                Task::none()
            }
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
                let indicators_map = self.indicators.as_ref();
                let days: Vec<crate::strategy::DaySnapshot> = rows.iter().enumerate().map(|(i, r)| {
                    let close = r.close.to_string().parse::<f64>().unwrap_or(0.0);
                    let (rsi, macd, macd_prev, sma50) = if let Some(ind) = indicators_map {
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
            Message::PortfolioPnlLoaded(Ok(rows)) => {
                self.portfolio_pnl = rows;
                Task::none()
            }
            Message::PortfolioPnlLoaded(Err(_)) => Task::none(),
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
            Message::SettingsLoaded(Ok(pairs)) => {
                for (k, v) in pairs {
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
                    if k == "font_scale" {
                        let (scale, label) = match v.as_str() {
                            "Compact" => (0.85, "Compact"),
                            "Large"   => (1.15, "Large"),
                            "XL"      => (1.35, "XL"),
                            _         => (1.0,  "Default"),
                        };
                        crate::theme::set_font_scale(scale);
                        self.font_scale_label = label.to_string();
                    }
                    self.settings.insert(k, v);
                }
                Task::none()
            }
            Message::SettingsLoaded(Err(_)) => Task::none(),
            Message::SettingsRefreshInput(s) => { self.settings_refresh_input = s; Task::none() }
            Message::SaveSetting(key, value) => {
                self.settings.insert(key.clone(), value.clone());
                if key == "theme_mode" {
                    self.theme_mode = match value.as_str() {
                        "Light" => crate::theme::ThemeMode::AlwaysLight,
                        "Dark" => crate::theme::ThemeMode::AlwaysDark,
                        _ => crate::theme::ThemeMode::Auto,
                    };
                    self.theme = crate::theme::iced_theme(self.theme_mode);
                }
                if key == "font_scale" {
                    let (scale, label) = match value.as_str() {
                        "Compact" => (0.85, "Compact"),
                        "Large"   => (1.15, "Large"),
                        "XL"      => (1.35, "XL"),
                        _         => (1.0,  "Default"),
                    };
                    crate::theme::set_font_scale(scale);
                    self.font_scale_label = label.to_string();
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
                self.transactions.insert(0, tx);
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
            Message::ImportWatchlistToPortfolio => {
                if let Some(pool) = &self.pool {
                    let tickers = self.watchlist_tickers_list.clone();
                    let pool2 = Arc::clone(pool);
                    let pool3 = Arc::clone(pool);
                    Task::perform(async move {
                        let _ = crate::db::import_tickers_to_portfolio(pool2, tickers).await;
                        crate::db::fetch_portfolio(pool3).await
                    }, Message::PortfolioLoaded)
                } else {
                    Task::none()
                }
            }
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
}
