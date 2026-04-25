use iced::widget::{canvas::Canvas, column, container, horizontal_rule, row, text, Column};
use iced::{Alignment, Element, Length};

use crate::font;
use crate::gauges::FearGreedGauge;
use crate::icons;
use crate::state::{Dashboard, Message};
use crate::theme;

// ---------------------------------------------------------------------------
// Reusable layout components
// ---------------------------------------------------------------------------

/// Wrap content in a card panel (rounded background, padding, full width).
pub fn card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(12)
        .width(Length::Fill)
        .style(container::rounded_box)
        .into()
}

/// Section heading with icon + bold title.
pub fn section_heading<'a>(icon_char: char, title: &str) -> Element<'a, Message> {
    row![
        text(icon_char.to_string()).font(icons::BOOTSTRAP).size(theme::text_md()),
        text(title.to_string()).font(font::INTER_BOLD).size(theme::text_md()),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

/// Card with a titled header: icon + title + horizontal rule + content.
#[allow(dead_code)]
pub fn titled_card<'a>(
    icon_char: char,
    title: &str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    card(column![
        section_heading(icon_char, title),
        horizontal_rule(1),
        content.into(),
    ].spacing(6))
}

/// Gauge helper: renders a title + FearGreedGauge canvas, or a fallback label.
pub fn make_gauge<'a>(
    title: String,
    data: Option<(f32, String)>,
    fallback: String,
) -> Element<'a, Message> {
    match data {
        Some((score, label)) => column![
            text(title).size(theme::text_sm()),
            Canvas::new(FearGreedGauge { score, label })
                .width(Length::Fixed(148.0))
                .height(Length::Fixed(82.0)),
        ]
        .align_x(Alignment::Center)
        .spacing(2)
        .into(),
        None => column![
            text(title).size(theme::text_sm()),
            text(fallback).size(theme::text_sm()),
        ]
        .align_x(Alignment::Center)
        .spacing(4)
        .into(),
    }
}

impl Dashboard {
    /// Build the macro indicator strip (US FRED + international DBnomics).
    /// Used by Overview and Portfolio tabs.
    pub(crate) fn build_macro_strip(&self) -> Column<'_, Message> {
        let macro_data = &self.macro_data;
        let macro_fmt = |label: &str, id: &str, suffix: &str, prefix: &str| -> String {
            let val = macro_data
                .iter()
                .find(|m| m.series_id == id)
                .and_then(|m| m.value.as_ref())
                .and_then(|v| v.to_string().parse::<f64>().ok());
            match val {
                Some(v) => format!("{label}: {prefix}{v:.2}{suffix}"),
                None => format!("{label}: —"),
            }
        };
        let has_value = |id: &str| -> bool {
            macro_data.iter()
                .find(|m| m.series_id == id)
                .and_then(|m| m.value.as_ref())
                .is_some()
        };
        let macro_strip_us = row![
            text(macro_fmt("Fed Funds", "FEDFUNDS", "%", "")).size(theme::text_base()),
            text(macro_fmt("CPI YoY", "CPIAUCSL_YOY", "%", "")).size(theme::text_base()),
            text(macro_fmt("Unemploy", "UNRATE", "%", "")).size(theme::text_base()),
            text(macro_fmt("10Y", "GS10", "%", "")).size(theme::text_base()),
            text(macro_fmt("2Y", "GS2", "%", "")).size(theme::text_base()),
            text(macro_fmt("Spread", "T10Y2Y", "%", "")).size(theme::text_base()),
            text(macro_fmt("VIX", "VIXCLS", "", "")).size(theme::text_base()),
            text(macro_fmt("WTI Oil", "DCOILWTICO", "", "$")).size(theme::text_base()),
        ]
        .spacing(20);

        // Only show international row if at least one DBnomics series has data
        let intl_ids = [
            "DBNOMICS:ECB/FM/M.U2.EUR.RT.MM.EURIBOR3MD_.HSTA",
            "DBNOMICS:BIS/WS_CBPOL/M.CN",
            "DBNOMICS:Eurostat/prc_hicp_manr/M.RCH_A.CP00.EA",
            "DBNOMICS:OECD/MEI_CLI/LOLITOAA.USA.M",
            "DBNOMICS:BIS/WS_TC/Q.US.P.A.M.770.A",
        ];
        let has_any_intl = intl_ids.iter().any(|id| has_value(id));

        if has_any_intl {
            let macro_strip_intl = row![
                text(macro_fmt("Euribor 3M", intl_ids[0], "%", "")).size(theme::text_base()),
                text(macro_fmt("PBoC", intl_ids[1], "%", "")).size(theme::text_base()),
                text(macro_fmt("EU CPI", intl_ids[2], "%", "")).size(theme::text_base()),
                text(macro_fmt("OECD CLI", intl_ids[3], "", "")).size(theme::text_base()),
                text(macro_fmt("Credit/GDP", intl_ids[4], "%", "")).size(theme::text_base()),
            ]
            .spacing(20);
            column![macro_strip_us, macro_strip_intl].spacing(4)
        } else {
            column![macro_strip_us]
        }
    }
}
