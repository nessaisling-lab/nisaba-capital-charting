mod astrology;
mod charts;
mod db;
mod gauges;
mod helpers;
mod indicators;
mod signals;
mod state;
mod theme;
mod update;
mod view;

use state::Dashboard;

pub fn main() -> iced::Result {
    iced::application("Financial Dashboard", Dashboard::update, Dashboard::view)
        .subscription(Dashboard::subscription)
        .theme(Dashboard::theme)
        .run_with(Dashboard::new)
}
