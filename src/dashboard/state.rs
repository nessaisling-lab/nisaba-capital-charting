use chrono::Datelike;
use iced::Theme;
use crate::theme::ThemeMode;
use pursuit_week4_automation::models::{
    AnalystRating, AstroScore, DailyTransit, EarningsDate, FilingRow, FundamentalMetric,
    HoldingRow, InsiderTradeRow, LagrangeAlert, LagrangeHistory, MacroIndicator, NatalPosition,
    NewsArticle, PaperTrade, PortfolioPosition, PriceRow, SentimentScore, ShortInterest,
};
use crate::agents::{AgentAnalysis, AgentMode, AgentPersona};
use crate::backtest::{BacktestConfig, BacktestResult};
use crate::strategy::Strategy;
use crate::db::{CompareRow, NamedWatchlist, PortfolioPnlRow, RetroEvent, SectorSummary, TransactionRow, UniverseRow, WatchlistRow};
use crate::tabs::Tab;
use sqlx::PgPool;
use std::sync::Arc;

use crate::indicators::Indicators;

/// One day in the 90-day astro forecast timeline (v11.0).
#[derive(Debug, Clone)]
pub struct ForecastDay {
    pub date:       chrono::NaiveDate,
    pub score:      f32,
    pub key_aspect: Option<String>,
}

/// Candlestick hover tooltip size preference (v11.3 — user setting).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TooltipSize {
    Small,
    Default,
    Large,
}

impl TooltipSize {
    pub fn all() -> &'static [TooltipSize] {
        &[Self::Small, Self::Default, Self::Large]
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Small   => "Small",
            Self::Default => "Default",
            Self::Large   => "Large",
        }
    }
    /// (font_px, box_width_px, box_height_px)
    /// v11.5.F1+F2 — widened to fit "Opening / High / Low / Closing / Volume"
    /// word labels + "shares" suffix on the volume row.
    pub fn dims(self) -> (f32, f32, f32) {
        match self {
            Self::Small   => (9.0,  148.0, 88.0),
            Self::Default => (10.0, 168.0, 100.0),
            Self::Large   => (13.0, 220.0, 130.0),
        }
    }
}

impl std::fmt::Display for TooltipSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Natal wheel size preference (v11.3 — user setting).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartSize {
    Compact,
    Default,
    Large,
}

impl ChartSize {
    pub fn all() -> &'static [ChartSize] {
        &[Self::Compact, Self::Default, Self::Large]
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Default => "Default",
            Self::Large   => "Large",
        }
    }
    pub fn pixels(self) -> f32 {
        match self {
            Self::Compact => 320.0,
            Self::Default => 400.0,
            Self::Large   => 520.0,
        }
    }
}

impl std::fmt::Display for ChartSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartTimeframe {
    OneMonth,
    ThreeMonths,
    SixMonths,
    OneYear,
    All,
}

impl ChartTimeframe {
    pub fn all() -> &'static [ChartTimeframe] {
        &[Self::OneMonth, Self::ThreeMonths, Self::SixMonths, Self::OneYear, Self::All]
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::OneMonth => "1M",
            Self::ThreeMonths => "3M",
            Self::SixMonths => "6M",
            Self::OneYear => "1Y",
            Self::All => "ALL",
        }
    }
    pub fn max_bars(self) -> usize {
        match self {
            Self::OneMonth => 22,
            Self::ThreeMonths => 66,
            Self::SixMonths => 132,
            Self::OneYear => 252,
            Self::All => usize::MAX,
        }
    }
}

impl Default for ChartTimeframe {
    fn default() -> Self { Self::SixMonths }
}

/// Column the Universe table is sorted by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniverseSortCol {
    Ticker,
    Astro,
    Score,
    Fin,
    Macro,
    Short,
}

impl UniverseSortCol {
    /// SQL ORDER BY expression for this column.
    pub fn sql_expr(self) -> &'static str {
        match self {
            Self::Ticker => "a.ticker",
            Self::Astro  => "a.astro_score",
            Self::Score  => "score",
            Self::Fin    => "lh.fin_score",
            Self::Macro  => "lh.macro_score",
            Self::Short  => "lh.short_score",
        }
    }
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            Self::Ticker => "Ticker",
            Self::Astro  => "Astro",
            Self::Score  => "Score",
            Self::Fin    => "Fin",
            Self::Macro  => "Macro",
            Self::Short  => "Short",
        }
    }
}

impl Default for UniverseSortCol {
    fn default() -> Self { Self::Astro }
}

pub struct Dashboard {
    pub pool:              Option<Arc<PgPool>>,
    pub tickers:           Vec<String>,
    pub selected_ticker:   String,
    pub rows:              Vec<PriceRow>,
    pub indicators:        Option<Indicators>,
    pub insider_trades:    Vec<InsiderTradeRow>,
    pub filings_8k:        Vec<FilingRow>,
    pub holdings:          Vec<HoldingRow>,
    pub news:              Vec<NewsArticle>,
    pub rss_articles:      Vec<pursuit_week4_automation::models::RssArticle>,
    pub polymarket:        Vec<pursuit_week4_automation::models::PolymarketMarket>,
    pub gdelt_events:      Vec<pursuit_week4_automation::models::GdeltEvent>,
    pub earnings:          Vec<EarningsDate>,
    pub analyst_rating:    Option<AnalystRating>,
    pub sentiment:         Option<SentimentScore>,
    pub fear_greed:        Option<(f32, String)>,
    pub fear_greed_err:    Option<String>,
    pub market_fg:         Option<(f32, String)>,
    pub market_fg_err:     Option<String>,
    pub astro_score:       Option<AstroScore>,
    pub astro_aspects:     Vec<serde_json::Value>, // decoded from active_aspects JSONB
    pub natal_positions:   Vec<NatalPosition>,
    pub natal_angles:      Option<pursuit_week4_automation::models::NatalAngles>,
    pub daily_transits:    Vec<DailyTransit>,
    pub retrograde_events: Vec<RetroEvent>,
    pub horoscope:         Option<pursuit_week4_automation::astrology::interpretation::HoroscopeReading>,
    pub macro_data:        Vec<MacroIndicator>,
    pub short_interest:    Option<ShortInterest>,
    pub rss_tone:          Option<pursuit_week4_automation::models::RssToneScore>,
    pub fundamentals:      Option<FundamentalMetric>,
    // DCF calculator inputs (user-editable strings for text_input widgets)
    pub dcf_growth_rate:    String,
    pub dcf_growth_years:   String,
    pub dcf_terminal_growth: String,
    pub dcf_discount_rate:  String,
    pub dcf_result:         Option<crate::dcf::DcfResult>,
    // Options Greeks calculator inputs
    pub greeks_spot:        String,
    pub greeks_strike:      String,
    pub greeks_expiry_days: String,
    pub greeks_rate:        String,
    pub greeks_vol:         String,
    pub greeks_is_call:     bool,
    pub greeks_market_price: String, // for IV solve
    pub greeks_result:      Option<crate::greeks::BsResult>,
    pub greeks_iv:          Option<f64>,
    pub watchlist:         Vec<WatchlistRow>,
    pub lagrange_history:  Vec<LagrangeHistory>,
    pub portfolio:         Vec<PortfolioPosition>,
    pub alerts:              Vec<LagrangeAlert>,
    pub unread_alert_count:  usize,
    pub notifications_fired: bool,
    pub ticker_search_input:      String,
    pub autocomplete_suggestions: Vec<(String, String)>,  // (ticker, company_name)
    pub autocomplete_dismissed:   bool,                    // guard against async race on selection
    pub active_agent:             Option<AgentPersona>,
    pub agent_analysis:           Option<AgentAnalysis>,
    pub compare_tickers:          Vec<String>,       // up to 4 tickers for comparison
    pub compare_input:            String,             // text input for adding compare ticker
    pub compare_data:             Vec<CompareRow>,
    pub sector_peers:             Vec<String>,
    pub sort_watchlist_by_score:  bool,
    pub recently_viewed:          Vec<String>,
    pub favorites:                Vec<String>,
    pub active_tab:               Tab,
    pub status:                   String,
    pub refreshing:               bool,
    // In-app toast notifications
    pub toasts:                   Vec<(String, std::time::Instant)>,
    pub theme:                    Theme,
    pub theme_mode:               ThemeMode,
    pub circadian_override:       Option<u32>,  // None = auto clock, Some(0..23) = slider
    // Universe Explorer
    pub universe_rows:            Vec<UniverseRow>,
    pub universe_total:           i64,
    pub universe_page:            usize,
    pub universe_filter_zone:     Option<String>,
    pub universe_filter_sector:   Option<String>,
    pub universe_search_text:     String,
    pub universe_sort_col:        UniverseSortCol,
    pub universe_sort_asc:        bool,
    pub universe_sectors:         Vec<String>,
    pub sector_summaries:         Vec<SectorSummary>,
    // Named Watchlists
    pub named_watchlists:         Vec<NamedWatchlist>,
    pub active_watchlist_id:      Option<i32>,
    pub watchlist_tickers_list:   Vec<String>,   // tickers in active watchlist
    pub new_watchlist_name:       String,
    pub watchlist_add_ticker:     String,
    // Chart timeframe
    pub show_price_table:         bool,
    pub chart_timeframe:          ChartTimeframe,
    // Backtesting
    pub backtest_config:          BacktestConfig,
    pub backtest_result:          Option<BacktestResult>,
    pub backtest_buy_input:       String,
    pub backtest_sell_input:      String,
    // Astro Forecast (v11.0)
    pub forecast:                 Vec<ForecastDay>,
    // Portfolio P&L
    pub portfolio_pnl:            Vec<PortfolioPnlRow>,
    // Strategy Builder
    pub strategy:                 Strategy,
    pub strategy_result:          Option<BacktestResult>,
    // Transaction Log
    pub transactions:             Vec<TransactionRow>,
    pub tx_ticker_input:          String,
    pub tx_shares_input:          String,
    pub tx_price_input:           String,
    pub tx_action:                String, // "BUY" or "SELL"
    // Settings
    pub settings:                 std::collections::HashMap<String, String>,
    pub settings_refresh_input:   String,
    pub font_scale_label:         String,  // "Compact" / "Default" / "Large" / "XL"
    // Astro Calendar
    pub calendar_days:            Vec<crate::calendar::CalendarDay>,
    pub calendar_year:            i32,
    pub calendar_month:           u32,
    // Fetch ticker
    pub fetching_ticker:          bool,
    /// Wall-clock instant the current fetch began (v11.3) — drives progress bar fill.
    pub fetch_start_time:         Option<std::time::Instant>,
    // v12.2.2 — fetch_ticker_error removed; errors now flow through pill system.
    // LLM agent mode
    pub agent_mode:               AgentMode,
    pub agent_loading:            bool,
    pub agent_llm_error:          Option<String>,
    pub api_key_input:            String,
    // Paper Trail (v6.0)
    pub paper_account:            Option<crate::db::paper::PaperAccountSummary>,
    pub paper_positions:          Vec<crate::db::paper::PaperPositionRow>,
    pub paper_trades:             Vec<PaperTrade>,
    pub paper_daily_values:       Vec<f64>,  // for Sharpe ratio + chart
    pub paper_spy_values:         Vec<f64>,  // SPY benchmark series
    // Animation state (v7.2)
    pub animating:                bool,      // true when any animation active (16ms tick)
    pub gauge_anim_progress:      f32,       // 0.0→1.0 needle sweep
    pub gauge_anim_from:          f32,       // score needle sweeps from
    pub gauge_anim_to:            f32,       // score needle sweeps to
    pub score_count_progress:     f32,       // 0.0→1.0 number count-up
    #[allow(dead_code)]
    pub score_count_target:       f32,       // target Lagrange score for count-up (future)
    pub tab_indicator_progress:   f32,       // 0.0→1.0 tab underline slide
    pub tab_indicator_from:       usize,     // tab index sliding from
    pub tab_indicator_to:         usize,     // tab index sliding to
    // Viewport (v7.2)
    pub viewport_width:           f32,       // current window width in pixels
    // Grimoire UI (v7.3)
    pub hovered_tab:              Option<Tab>,  // which tab mouse is over
    pub tab_hover_progress:       [f32; 8],     // per-tab expand animation 0.0→1.0
    pub page_transition_progress: f32,          // crossfade on tab switch 0.0→1.0
    pub page_transition_from:     Option<Tab>,  // tab fading away from
    // Shader effects (v7.4)
    pub shader_time:              f32,            // cumulative time for GPU vignette
    // Chart animation (v9.0)
    pub chart_draw_progress:      f32,            // 0.0→1.0 candlestick draw-in animation
    // Chart layer visibility toggles (v11.1)
    pub show_natal_planets:       bool,
    pub show_transit_planets:     bool,
    pub show_aspects:             bool,
    pub show_retrogrades:         bool,
    // Natal wheel size (v11.3)
    pub chart_size:               ChartSize,
    // Candlestick hover tooltip size (v11.3)
    pub tooltip_size:             TooltipSize,
    pub os_notifications:         bool,
    pub natal_zoom:               f32,
    pub price_chart_cache:        std::sync::Arc<iced::widget::canvas::Cache>,
    // v12.2.2 — alert_pill_until removed. Replaced by per-notification
    // expires_at in the v12.1 universal pill deque.
    pub wiki_summary:             Option<crate::db::WikiSummary>,
    pub wiki_thumbnail_bytes:     Option<Vec<u8>>,
    /// v12.1 — Universal pill notification deque. Active pills render
    /// between right spacer and gear in tab strip. Replaces the v11.9
    /// fetching_pill / alert_pill ad-hoc chrome and the inline
    /// fetch_error_banner that used to push the page header down.
    pub notifications:            std::collections::VecDeque<crate::notifications::Notification>,
    /// v12.1 — Append-only history (capped at MAX_HISTORY) for future
    /// notification drawer / audit log.
    pub notification_history:     Vec<crate::notifications::Notification>,
    /// Monotonic counter for notification ids.
    pub next_notification_id:     u64,
    /// Tracks which Lagrange alert IDs already produced a pill so the
    /// AlertsLoaded handler doesn't re-emit on every refresh.
    pub alerted_lagrange_ids:     std::collections::HashSet<i32>,
    /// v12.2.5 — dedupe set for transit pills. Key format:
    /// `{planet}:{station}:{date}` so re-loading the same retrograde
    /// event window doesn't re-emit pills the user already saw.
    pub transit_pill_keys:        std::collections::HashSet<String>,
    /// v12.1 — id of the sticky sparkly fetch pill so it can be
    /// dismissed on FetchTickerComplete.
    pub fetch_notification_id:    Option<u64>,
    /// v12.2.4 — drawer open/closed (bell-click reveals notification_history).
    pub notifications_drawer_open: bool,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self {
            pool:            None,
            tickers:         vec![],
            selected_ticker: "AAPL".to_string(),
            rows:            vec![],
            indicators:      None,
            insider_trades:  vec![],
            filings_8k:      vec![],
            holdings:        vec![],
            news:            vec![],
            rss_articles:    vec![],
            polymarket:      vec![],
            gdelt_events:    vec![],
            earnings:        vec![],
            analyst_rating:  None,
            sentiment:       None,
            fear_greed:      None,
            fear_greed_err:  None,
            market_fg:       None,
            market_fg_err:   None,
            astro_score:     None,
            astro_aspects:   vec![],
            natal_positions: vec![],
            natal_angles:    None,
            daily_transits:  vec![],
            retrograde_events: vec![],
            horoscope:         None,
            macro_data:        vec![],
            short_interest:    None,
            rss_tone:          None,
            fundamentals:      None,
            dcf_growth_rate:    "10".to_string(),
            dcf_growth_years:   "5".to_string(),
            dcf_terminal_growth: "2.5".to_string(),
            dcf_discount_rate:  "10".to_string(),
            dcf_result:         None,
            greeks_spot:        String::new(),
            greeks_strike:      String::new(),
            greeks_expiry_days: "30".to_string(),
            greeks_rate:        "4.5".to_string(),
            greeks_vol:         "25".to_string(),
            greeks_is_call:     true,
            greeks_market_price: String::new(),
            greeks_result:      None,
            greeks_iv:          None,
            watchlist:         vec![],
            lagrange_history:  vec![],
            portfolio:         vec![],
            alerts:              vec![],
            unread_alert_count:  0,
            notifications_fired: false,
            ticker_search_input:      String::new(),
            autocomplete_suggestions: vec![],
            autocomplete_dismissed:   false,
            active_agent:             None,
            agent_analysis:           None,
            compare_tickers:          vec![],
            compare_input:            String::new(),
            compare_data:             vec![],
            sector_peers:             vec![],
            sort_watchlist_by_score:  true,
            recently_viewed:          vec![],
            favorites:                vec![],
            active_tab:               Tab::Astrology,
            status:                   String::new(),
            refreshing:               false,
            toasts:                   vec![],
            theme:                    crate::theme::iced_theme(ThemeMode::default(), crate::theme::current_hour()),
            theme_mode:               ThemeMode::default(),
            circadian_override:       None,
            universe_rows:            vec![],
            universe_total:           0,
            universe_page:            0,
            universe_filter_zone:     None,
            universe_filter_sector:   None,
            universe_search_text:     String::new(),
            universe_sort_col:        UniverseSortCol::default(),
            universe_sort_asc:        false, // descending by default (highest first)
            universe_sectors:         vec![],
            sector_summaries:         vec![],
            named_watchlists:         vec![],
            active_watchlist_id:      None,
            watchlist_tickers_list:   vec![],
            new_watchlist_name:       String::new(),
            watchlist_add_ticker:     String::new(),
            show_price_table:         false,
            chart_timeframe:          ChartTimeframe::default(),
            backtest_config:          BacktestConfig::default(),
            backtest_result:          None,
            backtest_buy_input:       "65".to_string(),
            backtest_sell_input:      "35".to_string(),
            forecast:                 vec![],
            portfolio_pnl:            vec![],
            strategy:                 Strategy::default(),
            strategy_result:          None,
            transactions:             vec![],
            tx_ticker_input:          String::new(),
            tx_shares_input:          String::new(),
            tx_price_input:           String::new(),
            tx_action:                "BUY".to_string(),
            settings:                 std::collections::HashMap::new(),
            settings_refresh_input:   "30".to_string(),
            font_scale_label:         "Default".to_string(),
            calendar_days:            vec![],
            calendar_year:            chrono::Local::now().year(),
            calendar_month:           chrono::Local::now().month(),
            fetching_ticker:          false,
            fetch_start_time:         None,
            agent_mode:               AgentMode::Template,
            agent_loading:            false,
            agent_llm_error:          None,
            api_key_input:            String::new(),
            paper_account:            None,
            paper_positions:          vec![],
            paper_trades:             vec![],
            paper_daily_values:       vec![],
            paper_spy_values:         vec![],
            // Animation
            animating:                false,
            gauge_anim_progress:      1.0,  // start fully settled
            gauge_anim_from:          0.0,
            gauge_anim_to:            0.0,
            score_count_progress:     1.0,
            score_count_target:       0.0,
            tab_indicator_progress:   1.0,
            tab_indicator_from:       0,
            tab_indicator_to:         0,
            viewport_width:           1280.0,
            // Grimoire
            hovered_tab:              None,
            tab_hover_progress:       [0.0; 8],
            page_transition_progress: 1.0,
            page_transition_from:     None,
            // Shader
            shader_time:              42.0,  // non-zero seed for initial dust mote positions
            // Chart animation (v9.0)
            chart_draw_progress:      1.0,   // starts complete (animates on ticker switch)
            // Chart layer toggles (v11.1)
            show_natal_planets:       true,
            show_transit_planets:     true,
            show_aspects:             true,
            show_retrogrades:         true,
            chart_size:               ChartSize::Default,
            tooltip_size:             TooltipSize::Default,
            os_notifications:         true,
            natal_zoom:               1.0,
            price_chart_cache:        std::sync::Arc::new(iced::widget::canvas::Cache::default()),
            wiki_summary:             None,
            wiki_thumbnail_bytes:     None,
            notifications:            std::collections::VecDeque::new(),
            notification_history:     Vec::new(),
            next_notification_id:     1,
            alerted_lagrange_ids:     std::collections::HashSet::new(),
            transit_pill_keys:        std::collections::HashSet::new(),
            fetch_notification_id:    None,
            notifications_drawer_open: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    PoolReady(Result<Arc<PgPool>, String>),
    TickersLoaded(Result<Vec<String>, String>),
    DataLoaded(Result<Vec<PriceRow>, String>),
    InsiderTradesLoaded(Result<Vec<InsiderTradeRow>, String>),
    FilingsLoaded(Result<Vec<FilingRow>, String>),
    HoldingsLoaded(Result<Vec<HoldingRow>, String>),
    NewsLoaded(Result<Vec<NewsArticle>, String>),
    RssArticlesLoaded(Result<Vec<pursuit_week4_automation::models::RssArticle>, String>),
    PolymarketLoaded(Result<Vec<pursuit_week4_automation::models::PolymarketMarket>, String>),
    GdeltLoaded(Result<Vec<pursuit_week4_automation::models::GdeltEvent>, String>),
    EarningsLoaded(Result<Vec<EarningsDate>, String>),
    AnalystRatingLoaded(Result<Option<AnalystRating>, String>),
    SentimentLoaded(Result<Option<SentimentScore>, String>),
    FearGreedLoaded(Result<(f32, String), String>),
    MarketFGLoaded(Result<(f32, String), String>),
    AstroScoreLoaded(Result<Option<AstroScore>, String>),
    NatalChartLoaded(Result<Vec<NatalPosition>, String>),
    NatalAnglesLoaded(Result<Option<pursuit_week4_automation::models::NatalAngles>, String>),
    TransitsLoaded(Result<Vec<DailyTransit>, String>),
    RetroEventsLoaded(Result<Vec<RetroEvent>, String>),
    AstroAspectsLoaded(Result<serde_json::Value, String>),
    HoroscopeLoaded(Result<Option<pursuit_week4_automation::astrology::interpretation::HoroscopeReading>, String>),
    MacroDataLoaded(Result<Vec<MacroIndicator>, String>),
    ShortInterestLoaded(Result<Option<ShortInterest>, String>),
    RssToneLoaded(Result<Option<pursuit_week4_automation::models::RssToneScore>, String>),
    FundamentalsLoaded(Result<Option<FundamentalMetric>, String>),
    DcfGrowthRateInput(String),
    DcfGrowthYearsInput(String),
    DcfTerminalGrowthInput(String),
    DcfDiscountRateInput(String),
    DcfCompute,
    // Options Greeks
    GreeksSpotInput(String),
    GreeksStrikeInput(String),
    GreeksExpiryInput(String),
    GreeksRateInput(String),
    GreeksVolInput(String),
    GreeksToggleType,
    GreeksMarketPriceInput(String),
    GreeksCompute,
    GreeksSolveIV,
    WatchlistLoaded(Result<Vec<WatchlistRow>, String>),
    LagrangeHistoryLoaded(Result<Vec<LagrangeHistory>, String>),
    PortfolioLoaded(Result<Vec<PortfolioPosition>, String>),
    CopyText(String),
    OpenUrl(String),
    TickerSelected(String),
    AlertsLoaded(Result<Vec<LagrangeAlert>, String>),
    MarkAlertRead(i32),
    MarkAllAlertsRead,
    DismissAlert(i32),
    NotifyAlerts,
    TickerSearchInput(String),
    TickerSearchSubmit,
    AutocompleteResults(Vec<(String, String)>),
    AutocompleteSelected(String),
    RecentlyViewedLoaded(Result<Vec<String>, String>),
    FavoritesLoaded(Result<Vec<String>, String>),
    ToggleFavorite(String),
    // v11.9 — Settings reverted to own-tab. Modal Messages retired.
    ToggleOsNotifications(bool),
    TestNotification,
    NotificationTestComplete,
    NatalWheelZoom(f32),
    NatalWheelZoomReset,
    WikiSummaryLoaded(Result<Option<crate::db::WikiSummary>, String>),
    WikiThumbnailLoaded(Result<Vec<u8>, String>),
    TabSelected(Tab),
    TabHoverEnter(Tab),
    TabHoverExit(Tab),
    ToggleTheme,
    AgentSelected(AgentPersona),
    CompareInput(String),
    CompareAdd,
    CompareAddDirect(String),
    CompareRemove(String),
    CompareDataLoaded(Result<Vec<CompareRow>, String>),
    SectorPeersLoaded(Result<Vec<String>, String>),
    UniverseLoaded(Result<Vec<UniverseRow>, String>),
    UniverseCountLoaded(Result<i64, String>),
    UniverseSectorsLoaded(Result<Vec<String>, String>),
    SectorSummariesLoaded(Result<Vec<SectorSummary>, String>),
    UniverseFilterZone(Option<String>),
    UniverseFilterSector(Option<String>),
    UniverseSearchChanged(String),
    UniverseSort(UniverseSortCol),
    UniverseNextPage,
    UniversePrevPage,
    // Named Watchlists
    NamedWatchlistsLoaded(Result<Vec<NamedWatchlist>, String>),
    WatchlistTickersLoaded(Result<Vec<String>, String>),
    SelectNamedWatchlist(i32),
    NewWatchlistNameInput(String),
    CreateWatchlist,
    WatchlistCreated(Result<NamedWatchlist, String>),
    WatchlistAddTickerInput(String),
    WatchlistAddTicker,
    WatchlistRemoveTicker(String),
    WatchlistMutated(Result<(), String>),
    DeleteActiveWatchlist,
    ExportCsv,
    ExportUniverseCsv,
    ToggleWatchlistSort,
    TogglePriceTable,
    // Chart timeframe
    SetTimeframe(ChartTimeframe),
    // Backtest
    BacktestBuyInput(String),
    BacktestSellInput(String),
    RunBacktest,
    ClearBacktest,
    ForecastComputed(Vec<ForecastDay>),
    BacktestDataLoaded(Result<Vec<crate::db::BacktestDayRow>, String>),
    // Portfolio P&L
    PortfolioPnlLoaded(Result<Vec<PortfolioPnlRow>, String>),
    // Strategy Builder
    StrategyAddBuyCond(crate::strategy::Condition),
    StrategyRemoveBuyCond(usize),
    StrategyAddSellCond(crate::strategy::Condition),
    StrategyRemoveSellCond(usize),
    StrategyToggleBuyLogic,
    StrategyToggleSellLogic,
    RunStrategy,
    StrategyDataLoaded(Result<Vec<crate::db::BacktestDayRow>, String>),
    // Astro Calendar
    CalendarLoaded(Result<Vec<(chrono::NaiveDate, f64, Option<String>)>, String>),
    CalendarPrevMonth,
    CalendarNextMonth,
    // Settings
    SettingsLoaded(Result<Vec<(String, String)>, String>),
    SaveSetting(String, String),
    SettingSaved(Result<(), String>),
    SettingsRefreshInput(String),
    // Transaction Log
    TransactionsLoaded(Result<Vec<TransactionRow>, String>),
    TxTickerInput(String),
    TxSharesInput(String),
    TxPriceInput(String),
    TxToggleAction,
    TxSubmit,
    TxCreated(Result<TransactionRow, String>),
    TxDelete(i32),
    TxDeleted(Result<(), String>),
    ImportWatchlistToPortfolio,
    FocusSearch,
    EscapePressed,
    RefreshNow,
    Tick,
    WindowResized(iced::window::Id, iced::Size),
    // Fetch single ticker via scraper subprocess
    FetchThisTicker,
    FetchTickerComplete(Result<(), String>),
    // Circadian slider
    CircadianSliderChanged(u32),
    CircadianSliderReset,
    // LLM agent mode
    SetAgentMode(crate::agents::AgentMode),
    LlmAnalysisComplete(Result<AgentAnalysis, String>),
    ApiKeyInput(String),
    // Paper Trail
    PaperAccountLoaded(Result<Option<crate::db::paper::PaperAccountSummary>, String>),
    PaperPositionsLoaded(Result<Vec<crate::db::paper::PaperPositionRow>, String>),
    PaperTradesLoaded(Result<Vec<PaperTrade>, String>),
    PaperValuesLoaded(Result<(Vec<f64>, Vec<f64>), String>),
    // Chart layer toggles (v11.1)
    ToggleChartNatal,
    ToggleChartTransit,
    ToggleChartAspects,
    ToggleChartRetrogrades,
    // Chart size (v11.3)
    SetChartSize(ChartSize),
    // Candlestick tooltip size (v11.3)
    SetTooltipSize(TooltipSize),
    // v12.1 — Universal pill notification system
    #[allow(dead_code)] // wired in v12.2.4 drawer "clear one" action
    DismissNotification(u64),
    #[allow(dead_code)]
    NotificationsTick, // expire pass driven by Tick
    /// v12.2.3 — emitted when user clicks a pill. Handler dismisses the
    /// pill, then dispatches the pill's stored `on_click` message (if any)
    /// so a single click both routes (e.g. → Universe) and clears chrome.
    NotificationClicked(u64),
    /// v12.2.4 — bell icon click → toggle drawer.
    ToggleNotificationDrawer,
    /// v12.2.4 — drawer "Clear All" button → wipe history + active deque.
    ClearAllNotifications,
}
