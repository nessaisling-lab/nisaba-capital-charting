mod agents;
mod animation;
mod astrology;
mod backtest;
mod calendar;
mod charts;
mod dcf;
mod db;
mod greeks;
mod error;
mod font;
mod gauges;
mod heatmap;
mod helpers;
mod icons;
mod indicators;
mod ornaments;
mod patterns;
mod shaders;
mod signals;
mod state;
mod strategy;
mod tabs;
mod theme;
mod update;
mod view;

use state::Dashboard;

pub fn main() -> iced::Result {
    iced::application(Dashboard::new, Dashboard::update, Dashboard::view)
        .title("Financial Dashboard")
        .subscription(Dashboard::subscription)
        .theme(Dashboard::theme)
        .default_font(font::BODY)
        .font(font::FRAUNCES_BYTES)
        .font(font::SOURCE_SERIF_BYTES)
        .font(font::INTER_REGULAR_BYTES)
        .font(font::INTER_SEMIBOLD_BYTES)
        .font(font::MONO_REGULAR_BYTES)
        .font(icons::PHOSPHOR_BYTES)
        .font(icons::PHOSPHOR_BOLD_BYTES)
        .run()
}
