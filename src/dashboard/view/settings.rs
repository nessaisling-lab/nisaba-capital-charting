use iced::widget::{button, column, pick_list, row, rule, slider, text, text_input};
use iced::{Alignment, Element, Length};

use crate::font;
use crate::icons;
use crate::state::{ChartSize, Dashboard, Message, TooltipSize};
use crate::theme;
use super::shared::{card, eyebrow, section_heading, section_rule};

impl Dashboard {
    pub(crate) fn view_settings(&self) -> Element<'_, Message> {
        let theme_label = self.theme_mode.label();

        // ── Appearance card ─────────────────────────────────
        let theme_row = row![
            text("Theme:").font(font::BODY_BOLD).size(theme::text_sm()),
            button(text("Auto").size(theme::text_sm())).on_press(Message::SaveSetting("theme_mode".to_string(), "Auto".to_string())),
            button(text("Parchment").size(theme::text_sm())).on_press(Message::SaveSetting("theme_mode".to_string(), "Parchment".to_string())),
            button(text("Leather").size(theme::text_sm())).on_press(Message::SaveSetting("theme_mode".to_string(), "Leather".to_string())),
            text(format!("  (current: {theme_label})")).size(theme::text_sm()),
        ].spacing(8).align_y(Alignment::Center);

        let font_row = row![
            text("Text Size:").font(font::BODY_BOLD).size(theme::text_sm()),
            button(text("Compact").size(theme::text_sm())).on_press(Message::SaveSetting("font_scale".to_string(), "Compact".to_string())),
            button(text("Default").size(theme::text_sm())).on_press(Message::SaveSetting("font_scale".to_string(), "Default".to_string())),
            button(text("Large").size(theme::text_sm())).on_press(Message::SaveSetting("font_scale".to_string(), "Large".to_string())),
            button(text("XL").size(theme::text_sm())).on_press(Message::SaveSetting("font_scale".to_string(), "XL".to_string())),
            text(format!("  (current: {})", self.font_scale_label)).size(theme::text_sm()),
        ].spacing(8).align_y(Alignment::Center);

        // ── Circadian slider ────────────────────────────────
        let effective_hour = self.circadian_override.unwrap_or_else(crate::theme::current_hour);
        let hour_name = match effective_hour {
            5..=7   => "Dawn",
            8..=11  => "Morning",
            12..=13 => "Midday",
            14..=16 => "Afternoon",
            17..=19 => "Dusk",
            20..=22 => "Evening",
            _       => "Night",
        };
        let slider_label = if self.circadian_override.is_some() {
            format!("Circadian: {:02}:00 — {} (override)", effective_hour, hour_name)
        } else {
            format!("Circadian: {:02}:00 — {} (auto)", effective_hour, hour_name)
        };
        let circadian_row = column![
            row![
                text(slider_label).size(theme::text_sm()),
                iced::widget::Space::new().width(Length::Fill),
                button(text("Reset to clock").size(theme::text_xs()))
                    .on_press(Message::CircadianSliderReset),
            ].spacing(8).align_y(Alignment::Center),
            slider(0..=23, effective_hour as u16, |v| Message::CircadianSliderChanged(v as u32))
                .width(Length::Fill),
        ].spacing(4);

        // ── v11.3 — chart + tooltip size pickers ────────────
        let chart_size_row = row![
            text("Natal wheel size:").font(font::BODY_BOLD).size(theme::text_sm()),
            pick_list(
                ChartSize::all().to_vec(),
                Some(self.chart_size),
                Message::SetChartSize,
            ).text_size(theme::text_sm()),
        ].spacing(8).align_y(Alignment::Center);

        let tooltip_size_row = row![
            text("Candle tooltip size:").font(font::BODY_BOLD).size(theme::text_sm()),
            pick_list(
                TooltipSize::all().to_vec(),
                Some(self.tooltip_size),
                Message::SetTooltipSize,
            ).text_size(theme::text_sm()),
        ].spacing(8).align_y(Alignment::Center);

        let appearance_card = card(column![
            section_heading(icons::GEAR, "Appearance"),
            rule::horizontal(1),
            theme_row,
            font_row,
            chart_size_row,
            tooltip_size_row,
            circadian_row,
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
            rule::horizontal(1),
            refresh_row,
        ].spacing(6));

        // ── API Keys card ───────────────────────────────────
        let current_key = self.settings.get("anthropic_api_key").cloned().unwrap_or_default();
        let key_display = if current_key.len() > 8 {
            format!("{}...{}", &current_key[..4], &current_key[current_key.len()-4..])
        } else if !current_key.is_empty() {
            "****".to_string()
        } else {
            "Not set".to_string()
        };
        let api_keys_card = card(column![
            section_heading(icons::KEY, "API Keys"),
            rule::horizontal(1),
            row![
                text("Anthropic API Key:").size(theme::text_sm()),
                text_input("sk-ant-...", &self.api_key_input)
                    .on_input(Message::ApiKeyInput)
                    .on_submit(Message::SaveSetting("anthropic_api_key".to_string(), self.api_key_input.clone()))
                    .width(Length::Fixed(280.0))
                    .size(theme::text_sm()),
                button(text("Save").size(theme::text_sm()))
                    .on_press(Message::SaveSetting("anthropic_api_key".to_string(), self.api_key_input.clone())),
            ].spacing(8).align_y(Alignment::Center),
            text(format!("Current: {key_display}")).size(theme::text_xs()),
            text("Used for LLM-backed agent analysis (Fundamentals tab). Model: claude-sonnet.").size(theme::text_xs()),
        ].spacing(4));

        // ── Alerts card ─────────────────────────────────────
        // v11.5.D3 — OS notification toggle
        let os_toggle_label = if self.os_notifications { "On" } else { "Off" };
        let os_toggle_btn = button(
            text(format!("OS Notifications: {os_toggle_label}")).size(theme::text_sm()),
        )
        .on_press(Message::ToggleOsNotifications(!self.os_notifications));
        let alerts_card = card(column![
            section_heading(icons::BELL, "Alert Thresholds"),
            rule::horizontal(1),
            text("Alerts fire when a ticker transitions into an extreme Lagrange zone:").size(theme::text_sm()),
            text("  Optimal (score >= 70) or Misaligned (score < 30)").size(theme::text_sm()),
            text(format!("Active alerts: {}  |  Unread: {}", self.alerts.len(), self.unread_alert_count)).size(theme::text_sm()),
            os_toggle_btn,
            text("Toggle desktop toast notifications when new alerts fire. Optimal-zone alerts use Critical urgency.").size(theme::text_xs()),
        ].spacing(4));

        // ── Info card ───────────────────────────────────────
        let info_card = card(column![
            section_heading(icons::INFO_CIRCLE, "Dashboard Info"),
            rule::horizontal(1),
            text(format!("Tickers loaded: {}", self.tickers.len())).size(theme::text_sm()),
            text(format!("Universe size: {}", self.universe_total)).size(theme::text_sm()),
            text(format!("Transactions: {}", self.transactions.len())).size(theme::text_sm()),
            text(format!("Named watchlists: {}", self.named_watchlists.len())).size(theme::text_sm()),
            text(format!("Alerts: {} ({} unread)", self.alerts.len(), self.unread_alert_count)).size(theme::text_sm()),
        ].spacing(4));

        column![
            eyebrow("APPEARANCE"),
            appearance_card,
            section_rule(),
            eyebrow("DATA & REFRESH"),
            data_card,
            section_rule(),
            eyebrow("API KEYS"),
            api_keys_card,
            section_rule(),
            eyebrow("ALERTS"),
            alerts_card,
            section_rule(),
            eyebrow("DASHBOARD INFO"),
            info_card,
        ].spacing(theme::SPACE_SM).padding(8).into()
    }
}
