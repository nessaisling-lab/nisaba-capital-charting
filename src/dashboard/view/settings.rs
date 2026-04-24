use iced::widget::{button, column, horizontal_rule, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::font;
use crate::icons;
use crate::state::{Dashboard, Message};
use crate::theme;
use super::shared::{card, section_heading};

impl Dashboard {
    pub(crate) fn view_settings(&self) -> Element<'_, Message> {
        let theme_label = self.theme_mode.label();

        // ── Appearance card ─────────────────────────────────
        let theme_row = row![
            text("Theme:").font(font::INTER_BOLD).size(theme::text_sm()),
            button(text("Auto").size(theme::text_sm())).on_press(Message::SaveSetting("theme_mode".to_string(), "Auto".to_string())),
            button(text("Latte").size(theme::text_sm())).on_press(Message::SaveSetting("theme_mode".to_string(), "Light".to_string())),
            button(text("Mocha").size(theme::text_sm())).on_press(Message::SaveSetting("theme_mode".to_string(), "Dark".to_string())),
            button(text("Tokyo").size(theme::text_sm())).on_press(Message::SaveSetting("theme_mode".to_string(), "TokyoNight".to_string())),
            text(format!("  (current: {theme_label})")).size(theme::text_sm()),
        ].spacing(8).align_y(Alignment::Center);

        let font_row = row![
            text("Text Size:").font(font::INTER_BOLD).size(theme::text_sm()),
            button(text("Compact").size(theme::text_sm())).on_press(Message::SaveSetting("font_scale".to_string(), "Compact".to_string())),
            button(text("Default").size(theme::text_sm())).on_press(Message::SaveSetting("font_scale".to_string(), "Default".to_string())),
            button(text("Large").size(theme::text_sm())).on_press(Message::SaveSetting("font_scale".to_string(), "Large".to_string())),
            button(text("XL").size(theme::text_sm())).on_press(Message::SaveSetting("font_scale".to_string(), "XL".to_string())),
            text(format!("  (current: {})", self.font_scale_label)).size(theme::text_sm()),
        ].spacing(8).align_y(Alignment::Center);

        let appearance_card = card(column![
            section_heading(icons::GEAR, "Appearance"),
            horizontal_rule(1),
            theme_row,
            font_row,
        ].spacing(6));

        // ── Data card ───────────────────────────────────────
        let refresh_row = row![
            text("Refresh interval (seconds):").size(theme::text_sm()),
            text_input("30", &self.settings_refresh_input)
                .on_input(Message::SettingsRefreshInput)
                .on_submit(Message::SaveSetting("refresh_interval_secs".to_string(), self.settings_refresh_input.clone()))
                .width(Length::Fixed(60.0))
                .size(theme::text_sm()),
            button(text("Save").size(theme::text_sm()))
                .on_press(Message::SaveSetting("refresh_interval_secs".to_string(), self.settings_refresh_input.clone())),
        ].spacing(8).align_y(Alignment::Center);

        let data_card = card(column![
            section_heading(icons::ARROW_REPEAT, "Data & Refresh"),
            horizontal_rule(1),
            refresh_row,
        ].spacing(6));

        // ── Alerts card ─────────────────────────────────────
        let alerts_card = card(column![
            section_heading(icons::BELL, "Alert Thresholds"),
            horizontal_rule(1),
            text("Alerts fire when a ticker transitions into an extreme Lagrange zone:").size(theme::text_sm()),
            text("  Optimal (score >= 70) or Misaligned (score < 30)").size(theme::text_sm()),
            text(format!("Active alerts: {}  |  Unread: {}", self.alerts.len(), self.unread_alert_count)).size(theme::text_sm()),
        ].spacing(4));

        // ── Info card ───────────────────────────────────────
        let info_card = card(column![
            section_heading(icons::INFO_CIRCLE, "Dashboard Info"),
            horizontal_rule(1),
            text(format!("Tickers loaded: {}", self.tickers.len())).size(theme::text_sm()),
            text(format!("Universe size: {}", self.universe_total)).size(theme::text_sm()),
            text(format!("Transactions: {}", self.transactions.len())).size(theme::text_sm()),
            text(format!("Named watchlists: {}", self.named_watchlists.len())).size(theme::text_sm()),
            text(format!("Alerts: {} ({} unread)", self.alerts.len(), self.unread_alert_count)).size(theme::text_sm()),
        ].spacing(4));

        column![
            appearance_card,
            data_card,
            alerts_card,
            info_card,
        ].spacing(10).padding(8).into()
    }
}
