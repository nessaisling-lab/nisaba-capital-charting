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
            Some(Task::none())
        }
        Message::NatalChartLoaded(Err(_)) => Some(Task::none()),

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

        Message::CalendarPrevMonth => {
            if state.calendar_month == 1 {
                state.calendar_month = 12;
                state.calendar_year -= 1;
            } else {
                state.calendar_month -= 1;
            }
            Some(state.refresh_calendar())
        }
        Message::CalendarNextMonth => {
            if state.calendar_month == 12 {
                state.calendar_month = 1;
                state.calendar_year += 1;
            } else {
                state.calendar_month += 1;
            }
            Some(state.refresh_calendar())
        }

        _ => None,
    }
}
