//! Tab navigation system — Astrology as the flagship first tab.
//!
//! The dashboard uses a 6-tab layout. Astrology is the default and primary tab
//! because it's THE product differentiator. Everything else exists to verify
//! and contextualize the astrological signal.

/// The six dashboard tabs, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// PRIMARY: natal wheel, horoscope reading, transits, moon phase, Top 5/Bottom 5
    Astrology,
    /// Gauges + chart + sparkline + indicators + signal intelligence
    Overview,
    /// Scored universe + alerts + Top 5/Bottom 5 (future: full explorer)
    Universe,
    /// P/E, P/B, EV/EBITDA, DCF (v2.2: agents)
    Fundamentals,
    /// News + 8-K + insider trades + earnings + holdings
    Research,
    /// Portfolio positions + macro strip
    Portfolio,
    /// User settings: theme, refresh, API keys
    Settings,
}

impl Tab {
    /// All tabs in display order.
    pub fn all() -> &'static [Tab] {
        &[
            Tab::Astrology,
            Tab::Overview,
            Tab::Universe,
            Tab::Fundamentals,
            Tab::Research,
            Tab::Portfolio,
            Tab::Settings,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Tab::Astrology    => "Astrology",
            Tab::Overview     => "Overview",
            Tab::Universe     => "Universe",
            Tab::Fundamentals => "Fundamentals",
            Tab::Research     => "Research",
            Tab::Portfolio    => "Portfolio",
            Tab::Settings     => "Settings",
        }
    }

}
