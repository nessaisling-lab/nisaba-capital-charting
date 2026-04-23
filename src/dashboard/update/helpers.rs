use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::Task;
use std::sync::Arc;

use pursuit_week4_automation::models::LagrangeAlert;

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

    pub(crate) fn refresh_calendar(&self) -> Task<Message> {
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

    /// Build an `AgentContext` from current state and run the active persona's analysis.
    pub(crate) fn recompute_agent_if_active(&mut self) {
        let Some(persona) = self.active_agent else { return; };
        let price = self.rows.first()
            .map(|r| r.close.to_string().parse::<f64>().unwrap_or(0.0));

        let ctx = crate::agents::AgentContext {
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

/// Fire desktop notification toast for new alerts.
pub(crate) async fn fire_toast(alerts: Vec<LagrangeAlert>) {
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
/// Ctrl+1..7  switch tabs
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
