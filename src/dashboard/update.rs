use iced::{Subscription, Task, Theme};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

use crate::db::{
    connect_db, fetch_8k_filings, fetch_all_earnings, fetch_analyst_rating,
    fetch_astro_active_aspects, fetch_astro_score, fetch_daily_transits, fetch_fear_greed,
    fetch_holdings, fetch_insider_trades, fetch_lagrange_history, fetch_macro_indicators,
    fetch_market_fear_greed, fetch_natal_chart, fetch_news, fetch_portfolio, fetch_prices,
    fetch_sentiment, fetch_short_interest, fetch_watchlist_summaries, load_tickers,
};
use crate::indicators::Indicators;
use crate::state::{Dashboard, Message};

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
            Task::perform(fetch_short_interest(Arc::clone(pool), ticker), Message::ShortInterestLoaded),
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
                        Task::perform(fetch_all_earnings(Arc::clone(pool)), Message::EarningsLoaded),
                        Task::perform(fetch_market_fear_greed(Arc::clone(pool)), Message::MarketFGLoaded),
                        Task::perform(fetch_daily_transits(Arc::clone(pool)), Message::TransitsLoaded),
                        Task::perform(fetch_macro_indicators(Arc::clone(pool)), Message::MacroDataLoaded),
                        Task::perform(fetch_watchlist_summaries(Arc::clone(pool)), Message::WatchlistLoaded),
                        Task::perform(fetch_lagrange_history(Arc::clone(pool), self.selected_ticker.clone()), Message::LagrangeHistoryLoaded),
                        Task::perform(fetch_portfolio(Arc::clone(pool)), Message::PortfolioLoaded),
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
                self.lagrange_history = vec![];
                self.status = format!("Loading {ticker}...");
                if let Some(pool) = &self.pool {
                    Task::batch([
                        Self::fetch_all(pool, ticker.clone()),
                        Task::perform(fetch_lagrange_history(Arc::clone(pool), ticker), Message::LagrangeHistoryLoaded),
                    ])
                } else {
                    Task::none()
                }
            }
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
            Message::MacroDataLoaded(Ok(data))    => { self.macro_data = data;     Task::none() }
            Message::MacroDataLoaded(Err(_))       => Task::none(),
            Message::ShortInterestLoaded(Ok(si))  => { self.short_interest = si;   Task::none() }
            Message::ShortInterestLoaded(Err(_))   => Task::none(),
            Message::WatchlistLoaded(Ok(mut rows)) => {
                rows.sort_by(|a, b| b.quick_score().partial_cmp(&a.quick_score()).unwrap_or(std::cmp::Ordering::Equal));
                self.watchlist = rows;
                Task::none()
            }
            Message::WatchlistLoaded(Err(_)) => Task::none(),
            Message::LagrangeHistoryLoaded(Ok(h))  => { self.lagrange_history = h; Task::none() }
            Message::LagrangeHistoryLoaded(Err(_))  => Task::none(),
            Message::PortfolioLoaded(Ok(p))         => { self.portfolio = p;        Task::none() }
            Message::PortfolioLoaded(Err(_))         => Task::none(),
            Message::CopyText(s)  => iced::clipboard::write(s),
            Message::OpenUrl(url) => {
                let _ = open::that_detached(&url);
                Task::none()
            }
            Message::ToggleTheme => {
                self.theme = if self.theme == Theme::Dark { Theme::Light } else { Theme::Dark };
                Task::none()
            }
            Message::RefreshNow => {
                if let Some(pool) = &self.pool {
                    self.refreshing = true;
                    self.status = "Refreshing...".to_string();
                    Self::fetch_all(pool, self.selected_ticker.clone())
                } else {
                    Task::none()
                }
            }
            Message::Tick => {
                if let Some(pool) = &self.pool {
                    Self::fetch_all(pool, self.selected_ticker.clone())
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(30)).map(|_| Message::Tick)
    }
}
