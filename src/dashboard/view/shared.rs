use iced::widget::{canvas::Canvas, column, container, horizontal_rule, row, scrollable, text, Column, Space};
use iced::{Alignment, Color, Element, Length};

use crate::font;
use crate::gauges::FearGreedGauge;
use crate::icons;
use crate::state::{Dashboard, Message};
use crate::theme;

// ---------------------------------------------------------------------------
// Reusable layout components
// ---------------------------------------------------------------------------

/// Wrap content in a card panel — warm surface with subtle rule border.
pub fn card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(theme::SPACE_MD as u16)
        .width(Length::Fill)
        .style(|_theme: &iced::Theme| {
            let p = theme::palette();
            container::Style {
                background: Some(iced::Background::Color(p.surface)),
                border: iced::Border {
                    color: p.rule,
                    width: 1.0,
                    radius: theme::RADIUS_CARD.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Section heading with icon + Fraunces display title.
pub fn section_heading<'a>(icon_char: char, title: &str) -> Element<'a, Message> {
    row![
        text(icon_char.to_string()).font(icons::PHOSPHOR).size(theme::text_md()),
        text(title.to_string()).font(font::DISPLAY).size(theme::text_md()),
    ]
    .spacing(8)
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

// ---------------------------------------------------------------------------
// v7.1 Spatial primitives — BH redesign spatial language
// ---------------------------------------------------------------------------

/// Wrap content in a max-width centered container (BH: 1240px).
#[allow(dead_code)]
pub fn max_container<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .max_width(theme::MAX_WIDTH)
        .center_x(Length::Fill)
        .into()
}

/// Eyebrow label — uppercase, small, accent-colored section categorizer.
/// BH redesign: italic uppercase 0.78rem with letter-spacing. Iced has no
/// letter-spacing, so we use uppercase + bold weight + gold color.
pub fn eyebrow<'a>(label: &str) -> Element<'a, Message> {
    text(label.to_uppercase())
        .font(font::INTER_BOLD)
        .size(theme::text_xs())
        .color(theme::palette().gold)
        .into()
}

/// Section divider — horizontal rule with vertical breathing room.
/// Use between major content sections (not inside cards).
pub fn section_rule<'a>() -> Element<'a, Message> {
    column![
        Space::with_height(Length::Fixed(theme::SPACE_SM)),
        horizontal_rule(1),
        Space::with_height(Length::Fixed(theme::SPACE_SM)),
    ]
    .into()
}

// ---------------------------------------------------------------------------
// v7.6 Gold scrollbar style — reusable for sub-scrollables
// ---------------------------------------------------------------------------

/// Gold-themed scrollbar style matching the main page scrollbar.
/// Apply via `.style(gold_scrollbar_style)` on any `scrollable()`.
pub fn gold_scrollbar_style(_theme: &iced::Theme, _status: scrollable::Status) -> scrollable::Style {
    let p = theme::palette();
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: Some(iced::Background::Color(Color { a: 0.08, ..p.surface })),
            border: iced::Border::default(),
            scroller: scrollable::Scroller {
                color: Color { a: 0.35, ..p.gold },
                border: iced::Border { radius: 3.0.into(), ..Default::default() },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: None,
            border: iced::Border::default(),
            scroller: scrollable::Scroller {
                color: Color { a: 0.25, ..p.gold },
                border: iced::Border { radius: 3.0.into(), ..Default::default() },
            },
        },
        gap: None,
    }
}

// ---------------------------------------------------------------------------
// v7.3 Grimoire layout primitives
// ---------------------------------------------------------------------------

/// Wrap content in a parchment-styled "book page" container.
#[allow(dead_code)]
pub fn book_page_container<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(theme::SPACE_LG as u16)
        .style(|_theme: &iced::Theme| {
            let p = theme::palette();
            container::Style {
                background: Some(iced::Background::Color(p.bg)),
                border: iced::Border {
                    color: p.rule,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}


// ---------------------------------------------------------------------------
// Gauge + macro helpers
// ---------------------------------------------------------------------------

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
            text(macro_fmt("Fed Funds", "FEDFUNDS", "%", "")).font(font::INTER).size(theme::text_base()),
            text(macro_fmt("CPI YoY", "CPIAUCSL_YOY", "%", "")).font(font::INTER).size(theme::text_base()),
            text(macro_fmt("Unemploy", "UNRATE", "%", "")).font(font::INTER).size(theme::text_base()),
            text(macro_fmt("10Y", "GS10", "%", "")).font(font::INTER).size(theme::text_base()),
            text(macro_fmt("2Y", "GS2", "%", "")).font(font::INTER).size(theme::text_base()),
            text(macro_fmt("Spread", "T10Y2Y", "%", "")).font(font::INTER).size(theme::text_base()),
            text(macro_fmt("VIX", "VIXCLS", "", "")).font(font::INTER).size(theme::text_base()),
            text(macro_fmt("WTI Oil", "DCOILWTICO", "", "$")).font(font::INTER).size(theme::text_base()),
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
                text(macro_fmt("Euribor 3M", intl_ids[0], "%", "")).font(font::INTER).size(theme::text_base()),
                text(macro_fmt("PBoC", intl_ids[1], "%", "")).font(font::INTER).size(theme::text_base()),
                text(macro_fmt("EU CPI", intl_ids[2], "%", "")).font(font::INTER).size(theme::text_base()),
                text(macro_fmt("OECD CLI", intl_ids[3], "", "")).font(font::INTER).size(theme::text_base()),
                text(macro_fmt("Credit/GDP", intl_ids[4], "%", "")).font(font::INTER).size(theme::text_base()),
            ]
            .spacing(20);
            column![macro_strip_us, macro_strip_intl].spacing(4)
        } else {
            column![macro_strip_us]
        }
    }
}
