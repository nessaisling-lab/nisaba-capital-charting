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
mod notifications;
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
    // v11.8.D — Register Windows AppUserModelID before any UI loads so
    // toast notifications work without HRESULT 0x80070005 access denied.
    update::register_app_user_model_id();
    iced::application(Dashboard::new, Dashboard::update, Dashboard::view)
        .title("Nisaba Terminal")
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
