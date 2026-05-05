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

        Message::TransitsLoaded(Ok(t)) => {
            state.daily_transits = t.clone();
            Some(Task::none())
        }
        Message::TransitsLoaded(Err(_)) => Some(Task::none()),

        Message::RetroEventsLoaded(Ok(events)) => {
            state.retrograde_events = events.clone();
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
