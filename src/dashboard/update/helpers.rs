use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::Task;
use std::sync::Arc;

use pursuit_week4_automation::astrology::ephemeris::{Planet, PlanetSnapshot, longitude_to_sign};
use pursuit_week4_automation::astrology::natal::{NatalChart, compute_transit_score};
use pursuit_week4_automation::models::{FundamentalMetric, LagrangeAlert, NatalPosition, PriceRow};

use crate::db::{
    fetch_astro_calendar, fetch_compare_data, fetch_universe_count, fetch_universe_page,
    UniverseRow, WatchlistRow,
};
use crate::state::{Dashboard, Message};
use crate::tabs::Tab;

impl Dashboard {
    /// Refresh the Universe Explorer with current filters, search, and page.
    pub(crate) fn refresh_universe(&self) -> Task<Message> {
        if let Some(pool) = &self.pool {
            let search = if self.universe_search_text.is_empty() {
                None
            } else {
                Some(self.universe_search_text.clone())
            };
            Task::batch([
                Task::perform(
                    fetch_universe_page(
                        Arc::clone(pool),
                        self.universe_filter_zone.clone(),
                        self.universe_filter_sector.clone(),
                        search.clone(),
                        self.universe_page,
                        50,
                        self.universe_sort_col.sql_expr(),
                        self.universe_sort_asc,
                    ),
                    Message::UniverseLoaded,
                ),
                Task::perform(
                    fetch_universe_count(
                        Arc::clone(pool),
                        self.universe_filter_zone.clone(),
                        self.universe_filter_sector.clone(),
                        search,
                    ),
                    Message::UniverseCountLoaded,
                ),
            ])
        } else {
            Task::none()
        }
    }

    /// Fetch comparison data for the current compare_tickers list.
    pub(crate) fn refresh_compare(&self) -> Task<Message> {
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

    /// v11.6.D — fetch a 3-month window starting at calendar_year/month.
    /// Stepping in 3-month increments (`CalendarPrev/NextMonth`) means each
    /// page shows the next/previous quarter as one block.
    pub(crate) fn refresh_calendar(&self) -> Task<Message> {
        if let Some(pool) = &self.pool {
            let now = chrono::Local::now().date_naive();
            let start = chrono::NaiveDate::from_ymd_opt(self.calendar_year, self.calendar_month, 1)
                .unwrap_or(now);
            // End = last day of (calendar_month + 2). Wrap year if needed.
            let mut end_year = self.calendar_year;
            let mut end_month = self.calendar_month + 3;
            while end_month > 12 { end_month -= 12; end_year += 1; }
            let end = chrono::NaiveDate::from_ymd_opt(end_year, end_month, 1)
                .and_then(|d| d.pred_opt())
                .unwrap_or(now);
            Task::perform(
                fetch_astro_calendar(Arc::clone(pool), self.selected_ticker.clone(), start, end),
                Message::CalendarLoaded,
            )
        } else {
            Task::none()
        }
    }

    /// Build an `AgentContext` from current dashboard state.
    pub(crate) fn build_agent_context(&self) -> crate::agents::AgentContext {
        let price = self.rows.first()
            .map(|r| r.close.to_string().parse::<f64>().unwrap_or(0.0));

        crate::agents::AgentContext {
            ticker: self.selected_ticker.clone(),
            fundamentals: self.fundamentals.clone(),
            astro_score: self.astro_score.as_ref().and_then(|s| s.astro_score.map(|v| v as f64)),
            astro_label: self.astro_score.as_ref().and_then(|s| s.astro_label.clone()),
            dominant_theme: None,
            concordance: None,
            lagrange_score: self.indicators.as_ref().map(|ind| {
                let (score, _, _) = crate::indicators::compute_lagrange_score(
                    ind, &self.rows, &self.sentiment,
                    &self.astro_score, &self.macro_data, &self.short_interest,
                    &self.rss_tone,
                );
                score
            }),
            lagrange_label: self.indicators.as_ref().map(|ind| {
                let (_, label, _) = crate::indicators::compute_lagrange_score(
                    ind, &self.rows, &self.sentiment,
                    &self.astro_score, &self.macro_data, &self.short_interest,
                    &self.rss_tone,
                );
                label
            }),
            current_price: price,
            mercury_rx: self.astro_score.as_ref().and_then(|s| s.mercury_rx).unwrap_or(false),
            moon_phase: self.astro_score.as_ref().and_then(|s| s.moon_phase.clone()),
            // v10.0 "The Signal" — richer context
            sector: None,   // TODO: populate from company_metadata when loaded
            industry: None,
            recent_headlines: self.news.iter().take(3)
                .map(|n| n.headline.clone())
                .collect(),
            rss_tone_score: self.rss_tone.as_ref()
                .and_then(|r| r.tone_score.as_ref())
                .and_then(|v| v.to_string().parse::<f64>().ok()),
            rss_tone_label: self.rss_tone.as_ref()
                .and_then(|r| r.tone_label.clone()),
        }
    }

    /// Build an `AgentContext` from current state and run the active persona's template analysis.
    pub(crate) fn recompute_agent_if_active(&mut self) {
        let Some(persona) = self.active_agent else { return; };
        let ctx = self.build_agent_context();
        self.agent_analysis = Some(crate::agents::analyze(persona, &ctx));
    }

    /// Compute DCF if we have the required data (FCF + shares + price).
    pub(crate) fn compute_dcf_if_ready(&mut self) {
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

    /// Compute Black-Scholes option price and Greeks from user inputs.
    pub(crate) fn compute_greeks(&mut self) {
        let spot = if self.greeks_spot.is_empty() {
            // Auto-fill from current price if available
            self.rows.first()
                .map(|r| r.close.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0)
        } else {
            self.greeks_spot.parse::<f64>().unwrap_or(0.0)
        };
        let strike = self.greeks_strike.parse::<f64>().unwrap_or(0.0);
        let days = self.greeks_expiry_days.parse::<f64>().unwrap_or(30.0);
        let rate = self.greeks_rate.parse::<f64>().unwrap_or(4.5) / 100.0;
        let vol = self.greeks_vol.parse::<f64>().unwrap_or(25.0) / 100.0;
        let option_type = if self.greeks_is_call {
            crate::greeks::OptionType::Call
        } else {
            crate::greeks::OptionType::Put
        };

        let inputs = crate::greeks::BsInputs {
            spot, strike,
            time_years: days / 365.0,
            risk_free_rate: rate,
            volatility: vol,
            option_type,
        };
        self.greeks_result = crate::greeks::compute_greeks(&inputs);
    }

    /// Solve for implied volatility given a market price.
    pub(crate) fn solve_implied_vol(&mut self) {
        let spot = if self.greeks_spot.is_empty() {
            self.rows.first()
                .map(|r| r.close.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0)
        } else {
            self.greeks_spot.parse::<f64>().unwrap_or(0.0)
        };
        let strike = self.greeks_strike.parse::<f64>().unwrap_or(0.0);
        let days = self.greeks_expiry_days.parse::<f64>().unwrap_or(30.0);
        let rate = self.greeks_rate.parse::<f64>().unwrap_or(4.5) / 100.0;
        let market_price = self.greeks_market_price.parse::<f64>().unwrap_or(0.0);
        let option_type = if self.greeks_is_call {
            crate::greeks::OptionType::Call
        } else {
            crate::greeks::OptionType::Put
        };

        self.greeks_iv = crate::greeks::implied_volatility(
            spot, strike, days / 365.0, rate, market_price, option_type,
        );

        // Also recompute Greeks with the solved IV
        if let Some(iv) = self.greeks_iv {
            self.greeks_vol = format!("{:.1}", iv * 100.0);
            self.compute_greeks();
        }
    }

    /// v11.0: Auto-fill calculator inputs from loaded data.
    /// Only overwrites fields that still hold their original defaults.
    pub(crate) fn suggest_calculator_defaults(&mut self) {
        // --- DCF: growth rate from fundamentals ---
        if self.dcf_growth_rate == "10" {
            if let Some(ref f) = self.fundamentals {
                let growth = estimate_growth_rate(f);
                self.dcf_growth_rate = format!("{growth:.1}");
            }
        }

        // --- Greeks: strike near current price, vol from historical data ---
        let current_price = self.rows.first()
            .and_then(|r| r.close.to_string().parse::<f64>().ok());

        if let Some(price) = current_price {
            // Strike: nearest $5 increment for prices > $20, nearest $1 otherwise
            if self.greeks_strike == "100" || self.greeks_strike.is_empty() {
                let strike = if price > 20.0 {
                    (price / 5.0).round() * 5.0
                } else {
                    price.round()
                };
                self.greeks_strike = format!("{strike:.0}");
            }
        }

        // Volatility: 30-day historical vol from price data
        if self.greeks_vol == "25" {
            if let Some(vol) = historical_vol_30d(&self.rows) {
                self.greeks_vol = format!("{vol:.1}");
            }
        }
    }

    /// Push an in-app toast notification (auto-expires after 4 seconds).
    pub(crate) fn push_toast(&mut self, msg: impl Into<String>) {
        let expiry = std::time::Instant::now() + std::time::Duration::from_secs(4);
        self.toasts.push((msg.into(), expiry));
        // Cap at 5 visible toasts
        if self.toasts.len() > 5 {
            self.toasts.remove(0);
        }
    }

    /// Remove expired toasts. Called on Tick.
    pub(crate) fn expire_toasts(&mut self) {
        let now = std::time::Instant::now();
        self.toasts.retain(|(_, expiry)| *expiry > now);
    }

    /// v12.1 — Push a notification onto the active deque and append a
    /// copy to history. Caller responsible for variant choice + TTL.
    pub(crate) fn push_notification(
        &mut self,
        notif: crate::notifications::Notification,
    ) {
        // Same-id replace (idempotent re-push, e.g. fetch progress
        // ticking).  Otherwise append.
        if let Some(pos) = self.notifications.iter().position(|n| n.id == notif.id) {
            self.notifications[pos] = notif.clone();
        } else {
            self.notifications.push_back(notif.clone());
        }
        // Cap history.
        self.notification_history.push(notif);
        if self.notification_history.len() > crate::notifications::MAX_HISTORY {
            let drop = self.notification_history.len() - crate::notifications::MAX_HISTORY;
            self.notification_history.drain(0..drop);
        }
    }

    /// Allocate the next monotonic notification id.
    pub(crate) fn next_notif_id(&mut self) -> u64 {
        let id = self.next_notification_id;
        self.next_notification_id = self.next_notification_id.wrapping_add(1).max(1);
        id
    }

    /// Remove a notification by id (click-to-dismiss / external trigger).
    pub(crate) fn dismiss_notification(&mut self, id: u64) {
        self.notifications.retain(|n| n.id != id);
    }

    /// Expire pass — runs on each Tick. Drops notifications whose TTL
    /// has elapsed. Sticky pills (expires_at = None) are untouched.
    pub(crate) fn expire_notifications(&mut self) {
        let now = std::time::Instant::now();
        self.notifications.retain(|n| !n.is_expired(now));
    }

    /// v12.1 — Convenience emitters for the common pill types.
    pub(crate) fn notify_error(&mut self, text: impl Into<String>) {
        let id = self.next_notif_id();
        let n = crate::notifications::Notification::new(
            id,
            crate::notifications::NotificationVariant::Error,
            text,
        );
        self.push_notification(n);
    }

    pub(crate) fn notify_success(&mut self, text: impl Into<String>) {
        let id = self.next_notif_id();
        let n = crate::notifications::Notification::new(
            id,
            crate::notifications::NotificationVariant::Success,
            text,
        );
        self.push_notification(n);
    }

    #[allow(dead_code)]
    pub(crate) fn notify_info(&mut self, text: impl Into<String>) {
        let id = self.next_notif_id();
        let n = crate::notifications::Notification::new(
            id,
            crate::notifications::NotificationVariant::Info,
            text,
        );
        self.push_notification(n);
    }
}

// ---------------------------------------------------------------------------
// v11.0 calculator default helpers (free functions)
// ---------------------------------------------------------------------------

/// Estimate an appropriate FCF growth rate from fundamental metrics.
/// Uses PEG-implied earnings growth if available, otherwise size-based heuristic.
fn estimate_growth_rate(f: &FundamentalMetric) -> f64 {
    // PEG = P/E ÷ earnings_growth → earnings_growth = P/E ÷ PEG
    if let (Some(pe), Some(peg)) = (f.pe_ratio, f.peg_ratio) {
        if peg > 0.1 && pe > 0.0 {
            let implied_growth = pe / peg;
            // Clamp to reasonable range (3% - 30%)
            return implied_growth.clamp(3.0, 30.0);
        }
    }
    // Fallback: size-based heuristic (larger companies grow slower)
    match f.revenue {
        Some(rev) if rev > 50_000_000_000 => 8.0,   // mega-cap: ~8%
        Some(rev) if rev > 10_000_000_000 => 12.0,  // large-cap: ~12%
        Some(rev) if rev > 1_000_000_000  => 15.0,  // mid-cap: ~15%
        _ => 10.0,                                    // default
    }
}

/// Compute 30-day annualized historical volatility from price rows.
/// Returns percentage (e.g., 25.0 for 25%).
fn historical_vol_30d(rows: &[PriceRow]) -> Option<f64> {
    if rows.len() < 31 { return None; }
    // rows are newest-first; take 31 most recent, reverse to oldest-first
    let prices: Vec<f64> = rows.iter().take(31).rev()
        .filter_map(|r| r.close.to_string().parse::<f64>().ok())
        .collect();
    if prices.len() < 31 { return None; }
    let returns: Vec<f64> = prices.windows(2)
        .map(|w| (w[1] / w[0]).ln())
        .collect();
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>() / (returns.len() - 1) as f64;
    Some(variance.sqrt() * (252.0_f64).sqrt() * 100.0)
}

/// Convert dashboard NatalPosition (DB model) to lib PlanetSnapshot (astrology engine).
fn natal_positions_to_chart(ticker: &str, positions: &[NatalPosition]) -> Option<NatalChart> {
    if positions.is_empty() { return None; }
    let snapshots: Vec<PlanetSnapshot> = positions.iter()
        .filter_map(|p| {
            let planet = Planet::from_name(&p.planet)?;
            let (sign, degree) = longitude_to_sign(p.longitude);
            Some(PlanetSnapshot {
                planet,
                longitude: p.longitude,
                sign,
                degree,
                retrograde: p.retrograde,
            })
        })
        .collect();
    if snapshots.is_empty() { return None; }
    // Use today as ipo_date placeholder (not used in transit scoring)
    Some(NatalChart {
        ticker: ticker.to_string(),
        ipo_date: chrono::Utc::now().date_naive(),
        positions: snapshots,
        // v11.4 (Wave 6.B3) — ascendant computed in dashboard from natal_angles
        // table when available; pass None here since helpers.rs builds chart
        // from snapshots only (used for backtests, not transit scoring).
        ascendant: None,
    })
}

/// Compute 90-day astro forecast for a ticker. Blocking (uses Swiss Ephemeris).
pub fn compute_forecast(
    ticker: String,
    positions: Vec<NatalPosition>,
) -> Vec<crate::state::ForecastDay> {
    let Some(natal) = natal_positions_to_chart(&ticker, &positions) else {
        return vec![];
    };
    let today = chrono::Utc::now().date_naive();

    (1..=90)
        .map(|day_offset| {
            let date = today + chrono::Duration::days(day_offset);
            let ts = compute_transit_score(&natal, date);

            // v12.2.2 — `label` field removed from ForecastDay (was set
            // here, never read). Score band labeling is reconstructed at
            // render time when needed.

            // Pick strongest aspect as key event
            let key_aspect = ts.active_aspects.first().map(|a| {
                let dir = if a.applying { "applying" } else { "separating" };
                format!("{} {} {} ({dir})",
                    a.transit_planet.name(),
                    a.aspect.name(),
                    a.natal_planet.name(),
                )
            });

            crate::state::ForecastDay { date, score: ts.astro_score, key_aspect }
        })
        .collect()
}

/// Sort watchlist rows by score or alphabetically.
pub(crate) fn sort_watchlist(rows: &mut Vec<WatchlistRow>, by_score: bool) {
    if by_score {
        rows.sort_by(|a, b| b.quick_score().partial_cmp(&a.quick_score()).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        rows.sort_by(|a, b| a.ticker.cmp(&b.ticker));
    }
}

/// Export watchlist rows to a CSV file via a native save-file dialog.
pub(crate) async fn export_watchlist_csv(rows: Vec<WatchlistRow>, _ticker: &str) -> Result<(), String> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Export Watchlist CSV")
        .add_filter("CSV", &["csv"])
        .set_file_name("watchlist.csv")
        .save_file()
        .await;
    let Some(handle) = handle else { return Ok(()); };
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

/// Export universe rows to a CSV file via a native save-file dialog.
pub(crate) async fn export_universe_csv(rows: Vec<UniverseRow>) -> Result<(), String> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Export Universe CSV")
        .add_filter("CSV", &["csv"])
        .set_file_name("universe.csv")
        .save_file()
        .await;
    let Some(handle) = handle else { return Ok(()); };
    let path = handle.path();
    let mut wtr = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
    wtr.write_record(["Ticker", "Company", "Sector", "Score", "Zone", "Astro", "Fin", "Macro", "Short", "Concordance"])
        .map_err(|e| e.to_string())?;
    for r in &rows {
        let f = |v: Option<f32>| v.map(|x| format!("{x:.1}")).unwrap_or_default();
        wtr.write_record(&[
            &r.ticker,
            r.company_name.as_deref().unwrap_or(""),
            r.sector.as_deref().unwrap_or(""),
            &format!("{:.1}", r.score),
            &r.label,
            &f(r.astro_score),
            &f(r.fin_score),
            &f(r.macro_score),
            &f(r.short_score),
            r.concordance.as_deref().unwrap_or(""),
        ]).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())?;
    Ok(())
}

const APP_USER_MODEL_ID: &str = "PursuitAstro.Dashboard";
const APP_DISPLAY_NAME: &str = "Pursuit Astro";

/// v12.0.A — Real fix for Windows toast HRESULT 0x80070005.
/// Three pieces required:
///   1. AppUserModelID registry entry (DisplayName + IconUri)
///   2. Start Menu .lnk shortcut WITH IPropertyStore PKEY_AppUserModel_ID
///      property bound (this was the missing piece in v11.8.I — PowerShell's
///      WScript.Shell creates a basic .lnk but does NOT bind the AUMID
///      property; only IShellLinkW + IPropertyStore can)
///   3. notify-rust call passes AUMID via Notification::app_id()
///
/// We do (2) directly via windows-rs: IShellLinkW.SetPath +
/// IPropertyStore.SetValue(PKEY_AppUserModel_ID) + IPersistFile.Save.
#[cfg(windows)]
pub(crate) fn register_app_user_model_id() {
    let key = format!(r"HKCU\Software\Classes\AppUserModelId\{APP_USER_MODEL_ID}");
    let icon_uri = r"%windir%\System32\imageres.dll,-5302";
    let r1 = std::process::Command::new("reg")
        .args(["add", &key, "/v", "DisplayName", "/t", "REG_SZ", "/d", APP_DISPLAY_NAME, "/f"])
        .output();
    let r2 = std::process::Command::new("reg")
        .args(["add", &key, "/v", "IconUri", "/t", "REG_EXPAND_SZ", "/d", icon_uri, "/f"])
        .output();
    let reg_ok = matches!(&r1, Ok(o) if o.status.success())
        && matches!(&r2, Ok(o) if o.status.success());
    if !reg_ok {
        eprintln!("[notifications] AUMID registry write failed");
        return;
    }
    eprintln!("[notifications] AUMID registry registered: {APP_USER_MODEL_ID}");

    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[notifications] cannot get current_exe path: {e}");
            return;
        }
    };
    // %APPDATA%\Microsoft\Windows\Start Menu\Programs\Pursuit Astro.lnk
    let appdata = match std::env::var("APPDATA") {
        Ok(p) => p,
        Err(_) => { eprintln!("[notifications] %APPDATA% not set"); return; }
    };
    let lnk_path = std::path::PathBuf::from(appdata)
        .join("Microsoft").join("Windows").join("Start Menu")
        .join("Programs").join("Pursuit Astro.lnk");

    match unsafe { create_aumid_shortcut(&exe_path, &lnk_path, APP_USER_MODEL_ID) } {
        Ok(()) => {
            eprintln!("[notifications] AUMID-bound shortcut created: {}", lnk_path.display());
            eprintln!("[notifications] Toast notifications should resolve on next show().");
            eprintln!("[notifications] If first attempt still fails, sign out + back in once so explorer indexes the AUMID binding.");
        }
        Err(e) => eprintln!("[notifications] AUMID shortcut creation failed: {e:?}"),
    }
}

/// v12.0.A — Create a .lnk file and bind the AppUserModelID property
/// via IPropertyStore.SetValue(PKEY_AppUserModel_ID). This is what
/// notify-rust needs Windows to find when it calls Toast::show with
/// our AUMID — without the property binding, Windows rejects the
/// toast with HRESULT 0x80070005.
#[cfg(windows)]
unsafe fn create_aumid_shortcut(
    target: &std::path::Path,
    lnk: &std::path::Path,
    aumid: &str,
) -> windows::core::Result<()> {
    use windows::core::*;
    use windows::Win32::System::Com::*;
    use windows::Win32::System::Com::StructuredStorage::*;
    use windows::Win32::UI::Shell::*;
    use windows::Win32::UI::Shell::PropertiesSystem::*;
    use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;

    // Initialize COM apartment for this thread (idempotent — Iced may
    // have already done it, but second call returns S_FALSE harmlessly)
    let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

    let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

    // Set target path
    let target_wide: Vec<u16> = target
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    shell_link.SetPath(PCWSTR(target_wide.as_ptr()))?;

    // Set description (shows on hover in Start Menu)
    let desc_wide: Vec<u16> = APP_DISPLAY_NAME
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    shell_link.SetDescription(PCWSTR(desc_wide.as_ptr()))?;

    // Bind AUMID property — this is what was missing in v11.8.I.
    // PKEY_AppUserModel_ID = {9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3}, pid=5
    let property_store: IPropertyStore = shell_link.cast()?;
    let aumid_wide: Vec<u16> = aumid
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let propvar = InitPropVariantFromStringAsVector(PCWSTR(aumid_wide.as_ptr()));
    if let Ok(pv) = propvar {
        property_store.SetValue(&PKEY_AppUserModel_ID, &pv)?;
        property_store.Commit()?;
    }

    // Save .lnk
    // Ensure parent dir exists
    if let Some(parent) = lnk.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let lnk_wide: Vec<u16> = lnk
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let persist_file: IPersistFile = shell_link.cast()?;
    persist_file.Save(PCWSTR(lnk_wide.as_ptr()), true)?;

    Ok(())
}

#[cfg(not(windows))]
pub(crate) fn register_app_user_model_id() {
    // No-op on non-Windows — XDG (Linux) and macOS don't need this.
}

/// Fire desktop notification toast for new alerts. v11.7.C — diagnostic
/// logging. v11.8.D — explicit AppUserModelID so Windows Action Center
/// accepts the toast (HRESULT 0x80070005 fix).
pub(crate) async fn fire_toast(alerts: Vec<LagrangeAlert>) {
    eprintln!("[fire_toast] invoked with {} alert(s)", alerts.len());
    if alerts.is_empty() {
        eprintln!("[fire_toast] empty alerts — bailing");
        return;
    }
    let any_optimal = alerts.iter().any(|a| a.label.eq_ignore_ascii_case("Optimal"));
    let lead = &alerts[0];
    let summary = if alerts.len() == 1 {
        format!("{} → {}", lead.ticker, lead.label)
    } else {
        format!("{} → {} (+{} more)", lead.ticker, lead.label, alerts.len() - 1)
    };
    let entries: Vec<String> = alerts.iter().take(4)
        .map(|a| format!("{}: {}", a.ticker, a.label))
        .collect();
    let mut body = entries.join("\n");
    if alerts.len() > 4 {
        body = format!("{}\n…and {} more", body, alerts.len() - 4);
    }
    let mut n = notify_rust::Notification::new();
    n.summary(&summary).body(&body).appname(APP_DISPLAY_NAME);
    #[cfg(windows)]
    n.app_id(APP_USER_MODEL_ID);
    if any_optimal {
        n.urgency(notify_rust::Urgency::Critical);
    }
    match n.show() {
        Ok(_) => eprintln!("[fire_toast] OS notification shown — summary: {summary}"),
        Err(e) => eprintln!("[fire_toast] OS notification FAILED: {e}"),
    }
}

/// v11.7.C — manual test notification fired from Settings card. Lets
/// the user verify their OS path independent of the alerts pipeline.
pub(crate) async fn fire_test_notification() {
    eprintln!("[fire_test_notification] invoked");
    let mut n = notify_rust::Notification::new();
    n.summary("Pursuit Astro — Test")
        .body("If you see this, OS notifications are wired correctly.\nLagrange alerts will appear here when they trigger.")
        .appname(APP_DISPLAY_NAME);
    #[cfg(windows)]
    n.app_id(APP_USER_MODEL_ID);
    match n.show() {
        Ok(_) => eprintln!("[fire_test_notification] shown OK"),
        Err(e) => eprintln!("[fire_test_notification] FAILED: {e}"),
    }
}

/// Global keyboard shortcut handler.
///
/// Ctrl+1..8  switch tabs
/// Ctrl+T     focus the ticker search box
/// Ctrl+R     refresh all data
/// Escape     clear search input and autocomplete
pub(crate) fn handle_key_press(key: Key, modifiers: Modifiers) -> Option<Message> {
    if modifiers.control() {
        match &key {
            Key::Character(c) => match c.as_str() {
                "1" => Some(Message::TabSelected(Tab::Astrology)),
                "2" => Some(Message::TabSelected(Tab::Overview)),
                "3" => Some(Message::TabSelected(Tab::Universe)),
                "4" => Some(Message::TabSelected(Tab::Fundamentals)),
                "5" => Some(Message::TabSelected(Tab::Research)),
                "6" => Some(Message::TabSelected(Tab::Portfolio)),
                "7" => Some(Message::TabSelected(Tab::PaperTrail)),
                "8" => Some(Message::TabSelected(Tab::Settings)),
                "9" | "," => Some(Message::TabSelected(Tab::Encyclopedia)),
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
