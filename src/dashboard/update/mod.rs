//! Update dispatcher — routes messages to domain-specific handlers.
//!
//! Domain modules:
//! - `astro`: astrology scores, natal charts, transits, horoscope, calendar
//! - `data`: ticker selection, price/financial data loads, search, agents, DCF
//! - `universe`: universe explorer, alerts, named watchlists, exports
//! - `portfolio`: portfolio, backtest, strategy builder, transactions

mod astro;
mod data;
mod helpers;
mod portfolio;
mod universe;

use chrono::Datelike;
use iced::widget::text_input;
use iced::{Subscription, Task, Theme};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

use crate::db::{
    connect_db, fetch_alerts, fetch_available_sectors, fetch_daily_transits,
    fetch_lagrange_history, fetch_macro_indicators, fetch_market_fear_greed,
    fetch_named_watchlists, fetch_polymarket, fetch_portfolio, fetch_portfolio_pnl,
    fetch_recently_viewed, fetch_retrograde_events, fetch_rss_articles,
    fetch_sector_summaries, fetch_settings, fetch_ticker_earnings,
    fetch_transactions, fetch_universe_count, fetch_universe_page,
    fetch_watchlist_summaries, fetch_astro_calendar, fetch_fear_greed,
    load_tickers, upsert_setting,
};
use crate::state::{Dashboard, Message};

pub(crate) use helpers::handle_key_press;

/// Stable ID for the ticker search text_input, used for programmatic focus.
pub const SEARCH_INPUT_ID: &str = "ticker-search";

impl Dashboard {
    pub fn new() -> (Self, Task<Message>) {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:dev@localhost:5432/financial_dashboard".to_string()
        });
        (
            Dashboard {
                status: "Connecting to database...".to_string(),
                ..Default::default()
            },
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
        use crate::db::*;
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
        // Try domain handlers first (astro borrows, others consume via clone)
        if let Some(task) = astro::handle(self, &message) {
            return task;
        }

        // Clone message for the consuming handlers
        let msg_clone = message.clone();
        if let Some(task) = data::handle(self, message) {
            return task;
        }
        let msg_clone2 = msg_clone.clone();
        if let Some(task) = universe::handle(self, msg_clone) {
            return task;
        }
        let msg_clone3 = msg_clone2.clone();
        if let Some(task) = portfolio::handle(self, msg_clone2) {
            return task;
        }

        // Lifecycle + UI messages handled here in mod.rs
        match msg_clone3 {
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
                            let retro_start = chrono::Local::now().date_naive() - chrono::Duration::days(365);
                            let retro_end = chrono::Local::now().date_naive();
                            Task::perform(
                                fetch_retrograde_events(Arc::clone(pool), retro_start, retro_end),
                                Message::RetroEventsLoaded,
                            )
                        },
                        {
                            let now = chrono::Local::now().date_naive();
                            let start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
                                .unwrap_or(now);
                            let end = if now.month() == 12 {
                                chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1)
                                    .and_then(|d| d.pred_opt())
                                    .unwrap_or(now)
                            } else {
                                chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
                                    .and_then(|d| d.pred_opt())
                                    .unwrap_or(now)
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

            // ── UI / lifecycle ──────────────────────────────────────────
            Message::TabSelected(tab) => {
                self.active_tab = tab;
                Task::none()
            }
            Message::ToggleTheme => {
                self.theme_mode = self.theme_mode.next();
                self.theme = crate::theme::iced_theme(self.theme_mode);
                Task::none()
            }
            Message::TogglePriceTable => {
                self.show_price_table = !self.show_price_table;
                Task::none()
            }
            Message::SetTimeframe(tf) => {
                self.chart_timeframe = tf;
                Task::none()
            }
            Message::CopyText(s) => iced::clipboard::write(s),
            Message::OpenUrl(url) => {
                let _ = open::that_detached(&url);
                Task::none()
            }
            Message::FocusSearch => text_input::focus(SEARCH_INPUT_ID),
            Message::EscapePressed => {
                self.ticker_search_input = String::new();
                self.autocomplete_suggestions = vec![];
                self.autocomplete_dismissed = true;
                Task::none()
            }
            Message::NotifyAlerts => Task::none(),

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

            // ── Settings ────────────────────────────────────────────────
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
                            "Large" => (1.15, "Large"),
                            "XL" => (1.35, "XL"),
                            _ => (1.0, "Default"),
                        };
                        crate::theme::set_font_scale(scale);
                        self.font_scale_label = label.to_string();
                    }
                    self.settings.insert(k, v);
                }
                Task::none()
            }
            Message::SettingsLoaded(Err(_)) => Task::none(),
            Message::SettingsRefreshInput(s) => {
                self.settings_refresh_input = s;
                Task::none()
            }
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
                        "Large" => (1.15, "Large"),
                        "XL" => (1.35, "XL"),
                        _ => (1.0, "Default"),
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

            // Catch-all: message was already handled by a domain module
            // or is unknown. This shouldn't happen in practice.
            _ => Task::none(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_secs(30)).map(|_| Message::Tick),
            iced::keyboard::on_key_press(handle_key_press),
        ])
    }
}
