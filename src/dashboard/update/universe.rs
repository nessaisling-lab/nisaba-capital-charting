//! Universe, alerts, and watchlist message handlers.
//!
//! Covers: Universe pagination/filtering/search, Alerts (load, read, dismiss),
//! Named Watchlists (CRUD), Sector Summaries, Export CSV.

use iced::Task;
use std::sync::Arc;

use pursuit_week4_automation::models::LagrangeAlert;

use crate::db::{
    add_to_watchlist, create_watchlist, delete_watchlist, dismiss_alert,
    fetch_watchlist_tickers, mark_alert_read, mark_all_alerts_read,
    remove_from_watchlist,
};
use crate::state::{Dashboard, Message};

use super::helpers::{export_universe_csv, export_watchlist_csv, fire_toast, sort_watchlist};

/// Handle all universe/alert/watchlist messages. Returns `Some(task)` if handled.
pub(crate) fn handle(state: &mut Dashboard, message: Message) -> Option<Task<Message>> {
    match message {
        // ── Universe Explorer ───────────────────────────────────────────
        Message::UniverseLoaded(Ok(rows)) => {
            state.universe_rows = rows;
            Some(Task::none())
        }
        Message::UniverseLoaded(Err(_)) => Some(Task::none()),
        Message::UniverseCountLoaded(Ok(n)) => { state.universe_total = n; Some(Task::none()) }
        Message::UniverseCountLoaded(Err(_)) => Some(Task::none()),
        Message::UniverseSectorsLoaded(Ok(s)) => { state.universe_sectors = s; Some(Task::none()) }
        Message::UniverseSectorsLoaded(Err(_)) => Some(Task::none()),
        Message::SectorSummariesLoaded(Ok(s)) => { state.sector_summaries = s; Some(Task::none()) }
        Message::SectorSummariesLoaded(Err(_)) => Some(Task::none()),

        Message::UniverseFilterZone(zone) => {
            state.universe_filter_zone = zone;
            state.universe_page = 0;
            Some(state.refresh_universe())
        }
        Message::UniverseFilterSector(sector) => {
            state.universe_filter_sector = sector;
            state.universe_page = 0;
            Some(state.refresh_universe())
        }
        Message::UniverseSearchChanged(text) => {
            state.universe_search_text = text;
            state.universe_page = 0;
            Some(state.refresh_universe())
        }
        Message::UniverseSort(col) => {
            if state.universe_sort_col == col {
                state.universe_sort_asc = !state.universe_sort_asc;
            } else {
                state.universe_sort_col = col;
                state.universe_sort_asc = false; // default descending for new column
            }
            state.universe_page = 0;
            Some(state.refresh_universe())
        }
        Message::UniverseNextPage => {
            let max_page = ((state.universe_total as usize).saturating_sub(1)) / 50;
            if state.universe_page < max_page {
                state.universe_page += 1;
                Some(state.refresh_universe())
            } else {
                Some(Task::none())
            }
        }
        Message::UniversePrevPage => {
            if state.universe_page > 0 {
                state.universe_page -= 1;
                Some(state.refresh_universe())
            } else {
                Some(Task::none())
            }
        }

        // ── Alerts ──────────────────────────────────────────────────────
        Message::AlertsLoaded(Ok(alerts)) => {
            let unread_count = alerts.iter().filter(|a| !a.is_read).count();
            state.unread_alert_count = unread_count;

            // v12.1 — emit one Alert pill per *newly seen* unread Lagrange
            // alert (deduped via state.alerted_lagrange_ids). Click → Universe.
            for a in alerts.iter().filter(|a| !a.is_read) {
                if state.alerted_lagrange_ids.insert(a.id) {
                    let id = state.next_notif_id();
                    let n = crate::notifications::Notification::new(
                        id,
                        crate::notifications::NotificationVariant::Alert,
                        format!("→ {}", a.label),
                    )
                    .with_emphasis(a.ticker.clone())
                    .with_click(Message::TabSelected(crate::tabs::Tab::Universe));
                    state.push_notification(n);
                }
            }

            if unread_count > 0 && !state.notifications_fired && state.os_notifications {
                state.notifications_fired = true;
                // v12.2.2 — alert_pill_until removed; per-notification
                // expires_at handles TTL now (universal pill deque).
                let unread: Vec<LagrangeAlert> =
                    alerts.iter().filter(|a| !a.is_read).cloned().collect();
                state.alerts = alerts;
                Some(Task::perform(
                    async move { fire_toast(unread).await },
                    |_| Message::NotifyAlerts,
                ))
            } else {
                state.alerts = alerts;
                Some(Task::none())
            }
        }
        Message::AlertsLoaded(Err(_)) => Some(Task::none()),
        Message::MarkAlertRead(id) => {
            if let Some(a) = state.alerts.iter_mut().find(|a| a.id == id) {
                if !a.is_read {
                    a.is_read = true;
                    state.unread_alert_count = state.unread_alert_count.saturating_sub(1);
                }
            }
            if let Some(pool) = &state.pool {
                let p = Arc::clone(pool);
                Some(Task::perform(
                    async move { mark_alert_read(p, id).await },
                    |_| Message::NotifyAlerts,
                ))
            } else {
                Some(Task::none())
            }
        }
        Message::MarkAllAlertsRead => {
            for a in &mut state.alerts {
                a.is_read = true;
            }
            state.unread_alert_count = 0;
            state.notifications_fired = false; // allow re-notify for future alerts
            if let Some(pool) = &state.pool {
                let p = Arc::clone(pool);
                Some(Task::perform(
                    async move { mark_all_alerts_read(p).await },
                    |_| Message::NotifyAlerts,
                ))
            } else {
                Some(Task::none())
            }
        }
        Message::DismissAlert(id) => {
            if let Some(a) = state.alerts.iter().find(|a| a.id == id) {
                if !a.is_read {
                    state.unread_alert_count = state.unread_alert_count.saturating_sub(1);
                }
            }
            state.alerts.retain(|a| a.id != id);
            if let Some(pool) = &state.pool {
                let p = Arc::clone(pool);
                Some(Task::perform(
                    async move { dismiss_alert(p, id).await },
                    |_| Message::NotifyAlerts,
                ))
            } else {
                Some(Task::none())
            }
        }

        // ── Named Watchlists ────────────────────────────────────────────
        Message::WatchlistLoaded(Ok(mut rows)) => {
            sort_watchlist(&mut rows, state.sort_watchlist_by_score);
            state.watchlist = rows;
            Some(Task::none())
        }
        Message::WatchlistLoaded(Err(_)) => Some(Task::none()),
        Message::NamedWatchlistsLoaded(Ok(wls)) => {
            state.named_watchlists = wls;
            if state.active_watchlist_id.is_none() {
                if let Some(first) = state.named_watchlists.first() {
                    state.active_watchlist_id = Some(first.id);
                    if let Some(pool) = &state.pool {
                        return Some(Task::perform(
                            fetch_watchlist_tickers(Arc::clone(pool), first.id),
                            Message::WatchlistTickersLoaded,
                        ));
                    }
                }
            }
            Some(Task::none())
        }
        Message::NamedWatchlistsLoaded(Err(_)) => Some(Task::none()),
        Message::WatchlistTickersLoaded(Ok(tickers)) => {
            state.watchlist_tickers_list = tickers;
            Some(Task::none())
        }
        Message::WatchlistTickersLoaded(Err(_)) => Some(Task::none()),
        Message::SelectNamedWatchlist(id) => {
            state.active_watchlist_id = Some(id);
            if let Some(pool) = &state.pool {
                Some(Task::perform(
                    fetch_watchlist_tickers(Arc::clone(pool), id),
                    Message::WatchlistTickersLoaded,
                ))
            } else {
                Some(Task::none())
            }
        }
        Message::NewWatchlistNameInput(s) => { state.new_watchlist_name = s; Some(Task::none()) }
        Message::CreateWatchlist => {
            let name = state.new_watchlist_name.trim().to_string();
            if name.is_empty() {
                return Some(Task::none());
            }
            state.new_watchlist_name.clear();
            if let Some(pool) = &state.pool {
                Some(Task::perform(
                    create_watchlist(Arc::clone(pool), name),
                    Message::WatchlistCreated,
                ))
            } else {
                Some(Task::none())
            }
        }
        Message::WatchlistCreated(Ok(wl)) => {
            let new_id = wl.id;
            state.push_toast(format!("Watchlist '{}' created", wl.name));
            state.named_watchlists.push(wl);
            state.active_watchlist_id = Some(new_id);
            state.watchlist_tickers_list.clear();
            Some(Task::none())
        }
        Message::WatchlistCreated(Err(_)) => Some(Task::none()),
        Message::WatchlistAddTickerInput(s) => {
            state.watchlist_add_ticker = s;
            Some(Task::none())
        }
        Message::WatchlistAddTicker => {
            let ticker = state.watchlist_add_ticker.trim().to_uppercase();
            if ticker.is_empty() {
                return Some(Task::none());
            }
            state.watchlist_add_ticker.clear();
            if let (Some(pool), Some(wl_id)) = (&state.pool, state.active_watchlist_id) {
                state.watchlist_tickers_list.push(ticker.clone());
                Some(Task::perform(
                    add_to_watchlist(Arc::clone(pool), wl_id, ticker),
                    Message::WatchlistMutated,
                ))
            } else {
                Some(Task::none())
            }
        }
        Message::WatchlistRemoveTicker(ticker) => {
            state.watchlist_tickers_list.retain(|t| t != &ticker);
            if let (Some(pool), Some(wl_id)) = (&state.pool, state.active_watchlist_id) {
                Some(Task::perform(
                    remove_from_watchlist(Arc::clone(pool), wl_id, ticker),
                    Message::WatchlistMutated,
                ))
            } else {
                Some(Task::none())
            }
        }
        Message::WatchlistMutated(Ok(())) => Some(Task::none()),
        Message::WatchlistMutated(Err(_)) => Some(Task::none()),
        Message::DeleteActiveWatchlist => {
            if let (Some(pool), Some(wl_id)) = (&state.pool, state.active_watchlist_id) {
                state.named_watchlists.retain(|w| w.id != wl_id);
                state.active_watchlist_id = state.named_watchlists.first().map(|w| w.id);
                state.watchlist_tickers_list.clear();
                let pool2 = Arc::clone(pool);
                let mut tasks = vec![Task::perform(
                    delete_watchlist(Arc::clone(pool), wl_id),
                    Message::WatchlistMutated,
                )];
                if let Some(new_id) = state.active_watchlist_id {
                    tasks.push(Task::perform(
                        fetch_watchlist_tickers(pool2, new_id),
                        Message::WatchlistTickersLoaded,
                    ));
                }
                Some(Task::batch(tasks))
            } else {
                Some(Task::none())
            }
        }

        // ── Exports + Sort ──────────────────────────────────────────────
        Message::ExportCsv => {
            let rows = state.watchlist.clone();
            let ticker = state.selected_ticker.clone();
            Some(Task::perform(
                async move { export_watchlist_csv(rows, &ticker).await },
                |_: Result<(), String>| Message::WatchlistMutated(Ok(())),
            ))
        }
        Message::ExportUniverseCsv => {
            let rows = state.universe_rows.clone();
            Some(Task::perform(
                async move { export_universe_csv(rows).await },
                |_: Result<(), String>| Message::WatchlistMutated(Ok(())),
            ))
        }
        Message::ToggleWatchlistSort => {
            state.sort_watchlist_by_score = !state.sort_watchlist_by_score;
            sort_watchlist(&mut state.watchlist, state.sort_watchlist_by_score);
            Some(Task::none())
        }

        _ => None,
    }
}
