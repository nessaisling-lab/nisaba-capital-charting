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
use iced::widget::operation;
use iced::{Subscription, Task, Theme};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

use crate::tabs::Tab;

use crate::db::{
    connect_db, fetch_alerts, fetch_available_sectors, fetch_daily_transits,
    fetch_lagrange_history, fetch_macro_indicators, fetch_market_fear_greed,
    fetch_gdelt, fetch_named_watchlists, fetch_polymarket, fetch_portfolio, fetch_portfolio_pnl,
    fetch_recently_viewed, fetch_retrograde_events, fetch_rss_articles,
    fetch_sector_summaries, fetch_settings, fetch_ticker_earnings,
    fetch_transactions, fetch_universe_count, fetch_universe_page,
    fetch_watchlist_summaries, fetch_astro_calendar, fetch_fear_greed,
    load_tickers, upsert_setting,
    paper::{fetch_paper_account, fetch_paper_positions, fetch_paper_trades, fetch_paper_daily_values},
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
            Task::perform(fetch_natal_angles(Arc::clone(pool), ticker.clone()), Message::NatalAnglesLoaded),
            Task::perform(fetch_astro_active_aspects(Arc::clone(pool), ticker.clone()), Message::AstroAspectsLoaded),
            Task::perform(fetch_horoscope(Arc::clone(pool), ticker.clone()), Message::HoroscopeLoaded),
            Task::perform(fetch_short_interest(Arc::clone(pool), ticker.clone()), Message::ShortInterestLoaded),
            Task::perform(fetch_rss_tone(Arc::clone(pool), ticker.clone()), Message::RssToneLoaded),
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
                        Task::perform(fetch_universe_page(Arc::clone(pool), None, None, None, 0, 50, crate::state::UniverseSortCol::default().sql_expr(), false), Message::UniverseLoaded),
                        Task::perform(fetch_universe_count(Arc::clone(pool), None, None, None), Message::UniverseCountLoaded),
                        Task::perform(fetch_available_sectors(Arc::clone(pool)), Message::UniverseSectorsLoaded),
                        Task::perform(fetch_sector_summaries(Arc::clone(pool)), Message::SectorSummariesLoaded),
                        Task::perform(fetch_named_watchlists(Arc::clone(pool)), Message::NamedWatchlistsLoaded),
                        Task::perform(fetch_portfolio_pnl(Arc::clone(pool)), Message::PortfolioPnlLoaded),
                        Task::perform(fetch_transactions(Arc::clone(pool)), Message::TransactionsLoaded),
                        Task::perform(fetch_settings(Arc::clone(pool)), Message::SettingsLoaded),
                        Task::perform(fetch_rss_articles(Arc::clone(pool)), Message::RssArticlesLoaded),
                        Task::perform(fetch_polymarket(Arc::clone(pool)), Message::PolymarketLoaded),
                        Task::perform(fetch_gdelt(Arc::clone(pool)), Message::GdeltLoaded),
                        // Paper Trail data
                        Task::perform(fetch_paper_account(Arc::clone(pool)), Message::PaperAccountLoaded),
                        Task::perform(fetch_paper_positions(Arc::clone(pool)), Message::PaperPositionsLoaded),
                        Task::perform(fetch_paper_trades(Arc::clone(pool)), Message::PaperTradesLoaded),
                        Task::perform(fetch_paper_daily_values(Arc::clone(pool)), Message::PaperValuesLoaded),
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
                let old_idx = self.active_tab.index();
                let new_idx = tab.index();
                if old_idx != new_idx {
                    self.tab_indicator_from = old_idx;
                    self.tab_indicator_to = new_idx;
                    self.tab_indicator_progress = 0.0;
                    // Page transition: fade from old tab content
                    self.page_transition_from = Some(self.active_tab);
                    self.page_transition_progress = 0.0;
                    self.animating = true;
                }
                self.active_tab = tab;
                Task::none()
            }
            Message::TabHoverEnter(tab) => {
                self.hovered_tab = Some(tab);
                self.animating = true;
                Task::none()
            }
            Message::TabHoverExit(tab) => {
                if self.hovered_tab == Some(tab) {
                    self.hovered_tab = None;
                    self.animating = true;
                }
                Task::none()
            }
            Message::ToggleTheme => {
                self.theme_mode = self.theme_mode.next();
                let hour = self.circadian_override.unwrap_or_else(crate::theme::current_hour);
                self.theme = crate::theme::iced_theme(self.theme_mode, hour);
                Task::none()
            }
            Message::CircadianSliderChanged(hour) => {
                self.circadian_override = Some(hour);
                self.theme = crate::theme::iced_theme(self.theme_mode, hour);
                Task::none()
            }
            Message::CircadianSliderReset => {
                self.circadian_override = None;
                self.theme = crate::theme::iced_theme(self.theme_mode, crate::theme::current_hour());
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
            Message::CopyText(s) => {
                self.push_toast("Copied to clipboard");
                iced::clipboard::write(s)
            }
            Message::OpenUrl(url) => {
                let _ = open::that_detached(&url);
                Task::none()
            }
            Message::FocusSearch => operation::focus(SEARCH_INPUT_ID),
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
                // ── Animation advancement (16ms ticks) ─────────
                if self.animating {
                    let dt = crate::animation::TICK_DELTA;
                    let mut still_animating = false;

                    // Gauge sweep
                    if self.gauge_anim_progress < 1.0 {
                        self.gauge_anim_progress = (self.gauge_anim_progress
                            + dt / crate::animation::GAUGE_SWEEP_DURATION)
                            .min(1.0);
                        still_animating |= self.gauge_anim_progress < 1.0;
                    }
                    // Score count-up
                    if self.score_count_progress < 1.0 {
                        self.score_count_progress = (self.score_count_progress
                            + dt / crate::animation::COUNT_UP_DURATION)
                            .min(1.0);
                        still_animating |= self.score_count_progress < 1.0;
                    }
                    // Candlestick chart draw-in (v9.0) — 500ms staggered entrance
                    if self.chart_draw_progress < 1.0 {
                        self.chart_draw_progress = (self.chart_draw_progress
                            + dt / 0.5)  // 500ms total duration
                            .min(1.0);
                        still_animating |= self.chart_draw_progress < 1.0;
                    }
                    // Tab indicator slide
                    if self.tab_indicator_progress < 1.0 {
                        self.tab_indicator_progress = (self.tab_indicator_progress
                            + dt / crate::animation::TAB_SLIDE_DURATION)
                            .min(1.0);
                        still_animating |= self.tab_indicator_progress < 1.0;
                    }
                    // Per-tab hover expand/collapse (v7.3 Grimoire)
                    for idx in 0..8 {
                        let tab = Tab::all()[idx];
                        let target = if self.hovered_tab == Some(tab) { 1.0_f32 } else { 0.0 };
                        if (self.tab_hover_progress[idx] - target).abs() > 0.001 {
                            let speed = if target > self.tab_hover_progress[idx] {
                                crate::animation::TAB_HOVER_EXPAND_DURATION
                            } else {
                                crate::animation::TAB_HOVER_COLLAPSE_DURATION
                            };
                            let delta = dt / speed;
                            if target > self.tab_hover_progress[idx] {
                                self.tab_hover_progress[idx] = (self.tab_hover_progress[idx] + delta).min(1.0);
                            } else {
                                self.tab_hover_progress[idx] = (self.tab_hover_progress[idx] - delta).max(0.0);
                            }
                            still_animating = true;
                        }
                    }
                    // Page transition crossfade (v7.3)
                    if self.page_transition_progress < 1.0 {
                        self.page_transition_progress = (self.page_transition_progress
                            + dt / crate::animation::PAGE_TRANSITION_DURATION)
                            .min(1.0);
                        still_animating |= self.page_transition_progress < 1.0;
                        if self.page_transition_progress >= 1.0 {
                            self.page_transition_from = None;
                        }
                    }

                    // Advance shader time for dust mote animation (v7.4)
                    self.shader_time += dt;

                    // Astrology tab needs continuous 60fps for shader animations
                    // (planet pulse, transit drift, aspect shimmer, orbital trails)
                    still_animating |= self.active_tab == crate::tabs::Tab::Astrology;

                    self.animating = still_animating;
                    // During animation, skip expensive data fetches
                    return Task::none();
                }

                // ── Normal 30s tick — data refresh ─────────────
                self.expire_toasts();
                // Refresh palette on tick (skip if user has slider override)
                if self.circadian_override.is_none() {
                    self.theme = crate::theme::iced_theme(self.theme_mode, crate::theme::current_hour());
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

            Message::WindowResized(_id, size) => {
                self.viewport_width = size.width;
                crate::theme::set_viewport_width(size.width);
                Task::none()
            }

            // ── Settings ────────────────────────────────────────────────
            Message::SettingsLoaded(Ok(pairs)) => {
                for (k, v) in pairs {
                    if k == "theme_mode" {
                        self.theme_mode = match v.as_str() {
                            "Parchment" | "Light" => crate::theme::ThemeMode::Parchment,
                            "Leather" | "Dark" => crate::theme::ThemeMode::Leather,
                            _ => crate::theme::ThemeMode::Auto,
                        };
                        self.theme = crate::theme::iced_theme(self.theme_mode, crate::theme::current_hour());
                    }
                    if k == "refresh_interval_secs" {
                        self.settings_refresh_input = v.clone();
                    }
                    if k == "agent_mode" {
                        self.agent_mode = match v.as_str() {
                            "LLM" => crate::agents::AgentMode::Llm,
                            _ => crate::agents::AgentMode::Template,
                        };
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
                        "Parchment" | "Light" => crate::theme::ThemeMode::Parchment,
                        "Leather" | "Dark" => crate::theme::ThemeMode::Leather,
                        _ => crate::theme::ThemeMode::Auto,
                    };
                    self.theme = crate::theme::iced_theme(self.theme_mode, crate::theme::current_hour());
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
            Message::SettingSaved(Ok(())) => {
                self.push_toast("Setting saved");
                Task::none()
            }
            Message::SettingSaved(Err(_)) => Task::none(),

            // ── Paper Trail ────────────────────────────────────────────
            Message::PaperAccountLoaded(Ok(acc)) => {
                self.paper_account = acc;
                Task::none()
            }
            Message::PaperAccountLoaded(Err(_)) => Task::none(),
            Message::PaperPositionsLoaded(Ok(pos)) => {
                self.paper_positions = pos;
                Task::none()
            }
            Message::PaperPositionsLoaded(Err(_)) => Task::none(),
            Message::PaperTradesLoaded(Ok(trades)) => {
                self.paper_trades = trades;
                Task::none()
            }
            Message::PaperTradesLoaded(Err(_)) => Task::none(),
            Message::PaperValuesLoaded(Ok((paper, spy))) => {
                self.paper_daily_values = paper;
                self.paper_spy_values = spy;
                Task::none()
            }
            Message::PaperValuesLoaded(Err(_)) => Task::none(),

            // Catch-all: message was already handled by a domain module
            // or is unknown. This shouldn't happen in practice.
            _ => Task::none(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let tick_rate = if self.animating {
            Duration::from_millis(16) // 60fps during animation
        } else {
            Duration::from_secs(30)   // idle polling
        };
        Subscription::batch([
            iced::time::every(tick_rate).map(|_| Message::Tick),
            iced::keyboard::listen().filter_map(|event| {
                if let iced::keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                    handle_key_press(key, modifiers)
                } else {
                    None
                }
            }),
            iced::window::resize_events().map(|(id, size)| Message::WindowResized(id, size)),
        ])
    }
}
