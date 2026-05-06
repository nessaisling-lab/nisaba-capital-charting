//! Astrology-domain message handlers.
//!
//! Covers: AstroScore, NatalChart, Transits, RetroEvents, Horoscope,
//! AstroAspects, Calendar navigation.

use iced::Task;

use crate::state::{Dashboard, Message};

/// Handle all astrology-related messages. Returns `Some(task)` if handled.
pub(crate) fn handle(state: &mut Dashboard, message: &Message) -> Option<Task<Message>> {
    match message {
        Message::AstroScoreLoaded(Ok(s)) => {
            state.astro_score = s.clone();
            Some(Task::none())
        }
        Message::AstroScoreLoaded(Err(_)) => Some(Task::none()),

        Message::NatalChartLoaded(Ok(p)) => {
            state.natal_positions = p.clone();
            // Wave 9.6.1 — if IPO date already known, scan for upcoming
            // progressed Sun / Moon ingresses now that positions are set.
            if let Some(ipo) = state.natal_ipo_date {
                if !p.is_empty() {
                    emit_progression_ingress_pills(state, ipo);
                }
            }
            // v11.0: Trigger 90-day forecast computation
            if !p.is_empty() {
                let ticker = state.selected_ticker.clone();
                let positions = p.clone();
                Some(Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            crate::update::helpers::compute_forecast(ticker, positions)
                        }).await.unwrap_or_default()
                    },
                    Message::ForecastComputed,
                ))
            } else {
                Some(Task::none())
            }
        }
        Message::NatalChartLoaded(Err(_)) => Some(Task::none()),

        Message::ForecastComputed(days) => {
            state.forecast = days.clone();
            Some(Task::none())
        }

        Message::NatalAnglesLoaded(Ok(a)) => {
            state.natal_angles = a.clone();
            Some(Task::none())
        }
        Message::NatalAnglesLoaded(Err(_)) => Some(Task::none()),

        // Wave 9.5.1 — IPO date load.
        // Wave 9.6.1 — when both IPO + natal positions are present, scan
        // for upcoming progressed Sun / Moon sign ingresses within 12mo
        // and emit Transit pills (deduped via progression_pill_keys).
        Message::IpoDateLoaded(Ok(d)) => {
            state.natal_ipo_date = *d;
            if let (Some(ipo), false) = (*d, state.natal_positions.is_empty()) {
                emit_progression_ingress_pills(state, ipo);
            }
            Some(Task::none())
        }
        Message::IpoDateLoaded(Err(_)) => Some(Task::none()),

        Message::TransitsLoaded(Ok(t)) => {
            state.daily_transits = t.clone();
            Some(Task::none())
        }
        Message::TransitsLoaded(Err(_)) => Some(Task::none()),

        Message::RetroEventsLoaded(Ok(events)) => {
            state.retrograde_events = events.clone();

            // v12.2.5 — emit a Transit pill for each retrograde station
            // event within ±7 days of today, deduped via transit_pill_keys.
            // Click → Astrology tab so user can read the implications.
            let today = chrono::Local::now().date_naive();
            for ev in events.iter() {
                let delta = (ev.fetch_date - today).num_days();
                if delta.abs() > 7 { continue; }
                let key = format!("retro:{}:{}:{}", ev.planet, ev.station, ev.fetch_date);
                if state.transit_pill_keys.insert(key) {
                    let station_label = if ev.station == "Rx" {
                        "stations retrograde"
                    } else {
                        "stations direct"
                    };
                    let when = if delta == 0 { "today".to_string() }
                               else if delta == 1 { "tomorrow".to_string() }
                               else if delta > 0 { format!("in {delta}d") }
                               else { format!("{}d ago", -delta) };
                    let id = state.next_notif_id();
                    let n = crate::notifications::Notification::new(
                        id,
                        crate::notifications::NotificationVariant::Transit,
                        format!("{station_label} {when}"),
                    )
                    .with_emphasis(ev.planet.clone())
                    .with_click(crate::state::Message::TabSelected(
                        crate::tabs::Tab::Astrology,
                    ));
                    state.push_notification(n);
                }
            }
            Some(Task::none())
        }
        Message::RetroEventsLoaded(Err(_)) => Some(Task::none()),

        Message::AstroAspectsLoaded(Ok(v)) => {
            state.astro_aspects = v.as_array().cloned().unwrap_or_default();
            Some(Task::none())
        }
        Message::AstroAspectsLoaded(Err(_)) => Some(Task::none()),

        Message::HoroscopeLoaded(Ok(reading)) => {
            state.horoscope = reading.clone();
            Some(Task::none())
        }
        Message::HoroscopeLoaded(Err(_)) => Some(Task::none()),

        Message::CalendarLoaded(Ok(rows)) => {
            state.calendar_days = rows
                .iter()
                .map(|(date, score, label)| crate::calendar::CalendarDay {
                    date: *date,
                    astro_score: Some(*score),
                    label: label.clone(),
                })
                .collect();
            Some(Task::none())
        }
        Message::CalendarLoaded(Err(_)) => Some(Task::none()),

        // v11.6.D — calendar steps ±3 months per video review request
        // ("It should do at least three months ahead, instead of just one
        // month"). Wrap year boundaries when stepping past December/January.
        Message::CalendarPrevMonth => {
            let mut m = state.calendar_month as i32 - 3;
            let mut y = state.calendar_year;
            while m < 1 { m += 12; y -= 1; }
            state.calendar_month = m as u32;
            state.calendar_year = y;
            Some(state.refresh_calendar())
        }
        Message::CalendarNextMonth => {
            let mut m = state.calendar_month as i32 + 3;
            let mut y = state.calendar_year;
            while m > 12 { m -= 12; y += 1; }
            state.calendar_month = m as u32;
            state.calendar_year = y;
            Some(state.refresh_calendar())
        }

        // Chart layer toggles (v11.1)
        Message::ToggleChartNatal => {
            state.show_natal_planets = !state.show_natal_planets;
            Some(Task::none())
        }
        Message::ToggleChartTransit => {
            state.show_transit_planets = !state.show_transit_planets;
            Some(Task::none())
        }
        Message::ToggleChartAspects => {
            state.show_aspects = !state.show_aspects;
            Some(Task::none())
        }
        Message::ToggleChartRetrogrades => {
            state.show_retrogrades = !state.show_retrogrades;
            Some(Task::none())
        }
        Message::SetChartSize(sz) => {
            state.chart_size = *sz;
            Some(Task::none())
        }
        Message::SetTooltipSize(sz) => {
            state.tooltip_size = *sz;
            Some(Task::none())
        }

        _ => None,
    }
}

/// Wave 9.6.1 — Scan upcoming progressed Sun/Moon sign ingresses within
/// 12 months of today and emit one Transit pill per new ingress signature.
/// Deduped via `state.progression_pill_keys`. Click → Astrology tab.
///
/// Progressed Sun ingresses are rare (~once every 30 years) but mark
/// major character shifts. Progressed Moon ingresses every ~2.3 years
/// set the emotional tone.
fn emit_progression_ingress_pills(
    state: &mut Dashboard,
    ipo: chrono::NaiveDate,
) {
    use pursuit_week4_automation::astrology::natal::NatalChart;
    use pursuit_week4_automation::astrology::progressions::upcoming_sign_ingresses;

    let ticker = state.selected_ticker.clone();
    // Build a NatalChart from the loaded positions. We only do this if
    // positions are actually available (caller checks).
    let natal = NatalChart::compute(&ticker, ipo);
    let today = chrono::Local::now().date_naive();

    let ingresses = match upcoming_sign_ingresses(&natal, today, 1) {
        Ok(v) => v,
        Err(_) => return,
    };

    for ev in ingresses.iter() {
        // Only surface ingresses within the next 90 days — that's the
        // window where the user can act on the signal.
        if ev.days_offset < 0 || ev.days_offset > 90 { continue; }

        let key = format!("{}:{:?}:{}:{}",
            ticker, ev.planet, ev.from_sign, ev.ingress_date,
        );
        if !state.progression_pill_keys.insert(key) { continue; }

        let id = state.next_notif_id();
        let n = crate::notifications::Notification::new(
            id,
            crate::notifications::NotificationVariant::Transit,
            format!(
                "Prog. {} ingressing {} in {}d",
                ev.planet.name(),
                ev.to_sign,
                ev.days_offset,
            ),
        )
        .with_emphasis(ticker.clone())
        .with_click(crate::state::Message::TabSelected(
            crate::tabs::Tab::Astrology,
        ));
        state.push_notification(n);
    }
}
