use iced::widget::{button, canvas::Canvas, column, container, mouse_area, row, rule, scrollable, text, tooltip, Column, Space};
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
        rule::horizontal(1),
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

/// Eyebrow with a leading Phosphor icon (v11.3).
pub fn icon_eyebrow<'a>(icon_char: char, label: &str) -> Element<'a, Message> {
    let gold = theme::palette().gold;
    row![
        text(icon_char.to_string()).font(icons::PHOSPHOR).size(theme::text_xs()).color(gold),
        eyebrow(label),
    ]
    .spacing(4)
    .align_y(Alignment::Center)
    .into()
}

/// Section divider — horizontal rule with vertical breathing room.
/// Use between major content sections (not inside cards).
pub fn section_rule<'a>() -> Element<'a, Message> {
    column![
        Space::new().height(Length::Fixed(theme::SPACE_SM)),
        rule::horizontal(1),
        Space::new().height(Length::Fixed(theme::SPACE_SM)),
    ]
    .into()
}

// ---------------------------------------------------------------------------
// v11.5.A — Foundation: explanation tooltips + right-click primitive
// ---------------------------------------------------------------------------

/// Gold-bordered surface used as the tooltip body. Reused everywhere a
/// hover-explanation appears so the visual language is consistent.
#[allow(dead_code)]
pub fn tip_style(_t: &iced::Theme) -> container::Style {
    let p = theme::palette();
    container::Style {
        background: Some(iced::Background::Color(p.surface)),
        border: iced::Border { color: p.gold, width: 1.0, radius: 3.0.into() },
        ..Default::default()
    }
}

/// Wrap any element in a hover-explanation tooltip. Gold-bordered card
/// with body text. Position defaults to Bottom (most-readable for column
/// headers + inline glyphs).
#[allow(dead_code)]
pub fn explain<'a>(
    content: impl Into<Element<'a, Message>>,
    explanation: &str,
) -> Element<'a, Message> {
    explain_at(content, explanation, tooltip::Position::Bottom)
}

/// Same as `explain` but lets the caller pick tooltip position. Use Top
/// when the element sits at the bottom of a card and the popup would
/// otherwise spill off-screen.
#[allow(dead_code)]
pub fn explain_at<'a>(
    content: impl Into<Element<'a, Message>>,
    explanation: &str,
    position: tooltip::Position,
) -> Element<'a, Message> {
    tooltip(
        content,
        container(text(explanation.to_string()).size(theme::text_xs()))
            .padding([4, 8])
            .style(tip_style),
        position,
    )
    .into()
}

/// Inline label with a trailing Phosphor info-circle that triggers a
/// hover tooltip. Use for gauge titles, FRED indicator labels, or any
/// place a column header would feel cramped — the icon is a quiet hint
/// that the term has more depth.
#[allow(dead_code)]
pub fn label_with_explanation<'a>(label: &str, explanation: &str) -> Element<'a, Message> {
    let p = theme::palette();
    let icon = text(icons::INFO_CIRCLE.to_string())
        .font(icons::PHOSPHOR)
        .size(theme::text_xs())
        .color(Color { a: 0.7, ..p.gold });
    explain(
        row![
            text(label.to_string()).size(theme::text_sm()),
            icon,
        ]
        .spacing(4)
        .align_y(Alignment::Center),
        explanation,
    )
}

/// Right-click primitive — wrap any element so that secondary-clicking it
/// emits a Message. The caller owns the popup state; this helper only
/// provides the hit-testing surface. Pair with a state-managed overlay
/// (e.g. `Dashboard.context_menu: Option<ContextMenu>`) when consumed.
///
/// Example:
/// ```ignore
/// right_click(score_cell, Message::ShowContextMenu(MenuKind::AstroScore))
/// ```
#[allow(dead_code)]
pub fn right_click<'a>(
    content: impl Into<Element<'a, Message>>,
    on_right_click: Message,
) -> Element<'a, Message> {
    mouse_area(content).on_right_press(on_right_click).into()
}

// ---------------------------------------------------------------------------
// Clickable text link — gold text, transparent button, opens URL
// ---------------------------------------------------------------------------

/// Render a name as a clickable gold text link that opens a search URL.
pub fn link_button<'a>(label: &str, url: String) -> Element<'a, Message> {
    button(
        text(label.to_string())
            .size(theme::text_base())
            .color(theme::palette().gold),
    )
    .on_press(Message::OpenUrl(url))
    .padding(0)
    .style(|_theme: &iced::Theme, status: button::Status| {
        let p = theme::palette();
        let text_color = match status {
            button::Status::Hovered | button::Status::Pressed => {
                Color { a: 1.0, ..p.gold }
            }
            _ => Color { a: 0.85, ..p.gold },
        };
        button::Style {
            background: None,
            text_color,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            snap: false,
        }
    })
    .into()
}

// ---------------------------------------------------------------------------
// v7.6 Gold scrollbar style — reusable for sub-scrollables
// ---------------------------------------------------------------------------

/// Scrollable with gold style + right-side gutter padding (v11.3).
/// Wraps content in a container with 20px right padding so text never
/// hides behind the scrollbar thumb.
pub fn gutter_scroll<'a>(
    content: impl Into<Element<'a, Message>>,
    height: f32,
) -> Element<'a, Message> {
    scrollable(
        container(content)
            .width(Length::Fill)
            .padding(iced::Padding { top: 0.0, right: 20.0, bottom: 0.0, left: 0.0 })
    )
    .height(Length::Fixed(height))
    .style(gold_scrollbar_style)
    .into()
}

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
                background: iced::Background::Color(Color { a: 0.35, ..p.gold }),
                border: iced::Border { radius: 3.0.into(), ..Default::default() },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: None,
            border: iced::Border::default(),
            scroller: scrollable::Scroller {
                background: iced::Background::Color(Color { a: 0.25, ..p.gold }),
                border: iced::Border { radius: 3.0.into(), ..Default::default() },
            },
        },
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: iced::Background::Color(Color::TRANSPARENT),
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            icon: Color::TRANSPARENT,
        },
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
                .width(Length::Fixed(theme::sw(148.0)))
                .height(Length::Fixed(theme::sw(82.0))),
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
        // v11.5.C4 — explain() on each FRED indicator
        let macro_strip_us = row![
            explain(
                text(macro_fmt("Fed Funds", "FEDFUNDS", "%", "")).font(font::INTER).size(theme::text_base()),
                "Federal Funds Rate — overnight lending rate set by the Fed. The single most important number in finance. Higher = tighter credit, lower asset prices.",
            ),
            explain(
                text(macro_fmt("CPI YoY", "CPIAUCSL_YOY", "%", "")).font(font::INTER).size(theme::text_base()),
                "Consumer Price Index, year-over-year — headline inflation. Fed targets ~2%. Above = rate-hike pressure; below = rate-cut room.",
            ),
            explain(
                text(macro_fmt("Unemploy", "UNRATE", "%", "")).font(font::INTER).size(theme::text_base()),
                "Unemployment rate (U-3) — labor market slack. Below 4% = tight labor; above 5% = recession signal.",
            ),
            explain(
                text(macro_fmt("10Y", "GS10", "%", "")).font(font::INTER).size(theme::text_base()),
                "10-Year Treasury yield — the world's risk-free benchmark. Drives mortgage rates, equity discount rates, dollar strength.",
            ),
            explain(
                text(macro_fmt("2Y", "GS2", "%", "")).font(font::INTER).size(theme::text_base()),
                "2-Year Treasury yield — tracks Fed rate expectations 24 months out. Faster-moving than 10Y, more sensitive to policy.",
            ),
            explain(
                text(macro_fmt("Spread", "T10Y2Y", "%", "")).font(font::INTER).size(theme::text_base()),
                "10Y-2Y yield spread. Negative = inverted curve = recession signal (precedes every US recession since 1955).",
            ),
            explain(
                text(macro_fmt("VIX", "VIXCLS", "", "")).font(font::INTER).size(theme::text_base()),
                "CBOE Volatility Index — implied vol on S&P options, 30-day. Below 15 = calm; above 30 = fear; above 40 = panic.",
            ),
            explain(
                text(macro_fmt("WTI Oil", "DCOILWTICO", "", "$")).font(font::INTER).size(theme::text_base()),
                "West Texas Intermediate crude price. Inflation pressure, geopolitical proxy, energy-sector earnings driver.",
            ),
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
                explain(
                    text(macro_fmt("Euribor 3M", intl_ids[0], "%", "")).font(font::INTER).size(theme::text_base()),
                    "Euro Interbank 3-month rate — Eurozone's funding cost benchmark, ECB-policy proxy.",
                ),
                explain(
                    text(macro_fmt("PBoC", intl_ids[1], "%", "")).font(font::INTER).size(theme::text_base()),
                    "People's Bank of China policy rate — drives Chinese credit conditions, commodity demand, EM risk appetite.",
                ),
                explain(
                    text(macro_fmt("EU CPI", intl_ids[2], "%", "")).font(font::INTER).size(theme::text_base()),
                    "Eurozone Harmonised CPI year-over-year. ECB's 2% target. Diverges from US in regime shifts.",
                ),
                explain(
                    text(macro_fmt("OECD CLI", intl_ids[3], "", "")).font(font::INTER).size(theme::text_base()),
                    "OECD Composite Leading Indicator (US) — turning-point predictor for the business cycle. 100 = trend, above = expansion.",
                ),
                explain(
                    text(macro_fmt("Credit/GDP", intl_ids[4], "%", "")).font(font::INTER).size(theme::text_base()),
                    "Total credit to non-financial corporations as % of GDP. BIS gap measure flags credit booms before crashes.",
                ),
            ]
            .spacing(20);
            column![macro_strip_us, macro_strip_intl].spacing(4)
        } else {
            column![macro_strip_us]
        }
    }
}
