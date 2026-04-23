use iced::widget::{button, column, horizontal_rule, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::state::{Dashboard, Message};
use crate::theme;

impl Dashboard {
    pub(crate) fn view_settings(&self) -> Element<'_, Message> {
        let theme_label = match self.theme_mode {
            crate::theme::ThemeMode::Auto => "Auto",
            crate::theme::ThemeMode::AlwaysLight => "Light",
            crate::theme::ThemeMode::AlwaysDark => "Dark",
        };

        let theme_row = row![
            text("Theme:").size(theme::text_base()),
            button(text("Auto").size(theme::text_sm())).on_press(Message::SaveSetting("theme_mode".to_string(), "Auto".to_string())),
            button(text("Light").size(theme::text_sm())).on_press(Message::SaveSetting("theme_mode".to_string(), "Light".to_string())),
            button(text("Dark").size(theme::text_sm())).on_press(Message::SaveSetting("theme_mode".to_string(), "Dark".to_string())),
            text(format!("  (current: {theme_label})")).size(theme::text_sm()),
        ].spacing(8).align_y(Alignment::Center);

        let refresh_row = row![
            text("Refresh interval (seconds):").size(theme::text_base()),
            text_input("30", &self.settings_refresh_input)
                .on_input(Message::SettingsRefreshInput)
                .on_submit(Message::SaveSetting("refresh_interval_secs".to_string(), self.settings_refresh_input.clone()))
                .width(Length::Fixed(60.0))
                .size(theme::text_sm()),
            button(text("Save").size(theme::text_sm()))
                .on_press(Message::SaveSetting("refresh_interval_secs".to_string(), self.settings_refresh_input.clone())),
        ].spacing(8).align_y(Alignment::Center);

        let info_section = column![
            text("Dashboard Info").size(theme::text_md()),
            horizontal_rule(1),
            text(format!("Tickers loaded: {}", self.tickers.len())).size(theme::text_sm()),
            text(format!("Universe size: {}", self.universe_total)).size(theme::text_sm()),
            text(format!("Transactions: {}", self.transactions.len())).size(theme::text_sm()),
            text(format!("Named watchlists: {}", self.named_watchlists.len())).size(theme::text_sm()),
            text(format!("Alerts: {} ({} unread)", self.alerts.len(), self.unread_alert_count)).size(theme::text_sm()),
        ].spacing(4);

        let font_row = row![
            text("Text Size:").size(theme::text_base()),
            button(text("Compact").size(theme::text_sm())).on_press(Message::SaveSetting("font_scale".to_string(), "Compact".to_string())),
            button(text("Default").size(theme::text_sm())).on_press(Message::SaveSetting("font_scale".to_string(), "Default".to_string())),
            button(text("Large").size(theme::text_sm())).on_press(Message::SaveSetting("font_scale".to_string(), "Large".to_string())),
            button(text("XL").size(theme::text_sm())).on_press(Message::SaveSetting("font_scale".to_string(), "XL".to_string())),
            text(format!("  (current: {})", self.font_scale_label)).size(theme::text_sm()),
        ].spacing(8).align_y(Alignment::Center);

        let alert_section = column![
            text("Alert Thresholds").size(theme::text_md()),
            horizontal_rule(1),
            text("Alerts fire when a ticker transitions into an extreme Lagrange zone:").size(theme::text_sm()),
            text("  Optimal (score >= 70) or Misaligned (score < 30)").size(theme::text_sm()),
            text(format!("Active alerts: {}  |  Unread: {}", self.alerts.len(), self.unread_alert_count)).size(theme::text_sm()),
        ].spacing(4);

        column![
            text("Settings").size(theme::text_lg()),
            horizontal_rule(1),
            theme_row,
            font_row,
            refresh_row,
            horizontal_rule(1),
            alert_section,
            horizontal_rule(1),
            info_section,
        ].spacing(10).padding(14).into()
    }
}
