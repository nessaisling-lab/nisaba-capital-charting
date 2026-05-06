//! v12.1 — Universal pill notification system.
//!
//! Generalizes the v11.9 fetching_pill / alert_pill chrome into a deque
//! of `Notification` records rendered as pills between the right spacer
//! and the gear icon. Replaces the inline `fetch_error_banner` that used
//! to push the page header layout down on every fetch.
//!
//! Variants:
//!   - Sparkly  → ShootingStar canvas (fetch / celebratory)
//!   - Alert    → pulsing star (Lagrange unread)
//!   - Transit  → static gold star (astrology event)
//!   - Error    → orange warning glyph
//!   - Success  → green check
//!   - Info     → neutral info glyph
//!
//! Each notification carries an optional TTL and `on_click` message.
//! View loop renders up to MAX_VISIBLE_PILLS at a time; the rest spill
//! into history (drawer-ready, future v12.2).

use iced::widget::{button, column, container, row, scrollable, text, Canvas, Space};
use iced::{Alignment, Color, Element, Length};
use std::time::{Duration, Instant};

use crate::icons;
use crate::ornaments::ShootingStar;
use crate::state::Message;
use crate::theme;

/// Maximum pills rendered side-by-side. Older active pills auto-dismiss
/// when this cap is exceeded so chrome never grows unbounded.
pub const MAX_VISIBLE_PILLS: usize = 3;
/// Maximum entries kept in `notification_history` for drawer / audit.
pub const MAX_HISTORY: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Transit + Info wired in v12.2 (transit emit + drawer)
pub enum NotificationVariant {
    Sparkly,
    Alert,
    Transit,
    Error,
    Success,
    Info,
}

impl NotificationVariant {
    /// Default TTL for transient pills. `None` = persistent (must be
    /// dismissed explicitly, e.g. fetch progress driven by `fetching_ticker`).
    fn default_ttl(self) -> Option<Duration> {
        match self {
            Self::Sparkly => None, // caller sets TTL or holds open
            Self::Alert   => Some(Duration::from_secs(8)),
            Self::Transit => Some(Duration::from_secs(12)),
            Self::Error   => Some(Duration::from_secs(15)),
            Self::Success => Some(Duration::from_secs(4)),
            Self::Info    => Some(Duration::from_secs(8)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id:         u64,
    pub variant:    NotificationVariant,
    /// Display text (rendered after the icon).
    pub text:       String,
    /// Optional emphasized prefix (rendered bold, e.g. ticker symbol).
    pub emphasis:   Option<String>,
    #[allow(dead_code)] // surfaced in v12.2 drawer
    pub created_at: Instant,
    /// Absolute expiry instant. `None` = sticky (won't auto-dismiss).
    pub expires_at: Option<Instant>,
    /// Optional message dispatched on pill click.
    pub on_click:   Option<Message>,
}

impl Notification {
    pub fn new(id: u64, variant: NotificationVariant, text: impl Into<String>) -> Self {
        let now = Instant::now();
        let expires_at = variant.default_ttl().map(|d| now + d);
        Self { id, variant, text: text.into(), emphasis: None, created_at: now, expires_at, on_click: None }
    }
    pub fn with_emphasis(mut self, e: impl Into<String>) -> Self { self.emphasis = Some(e.into()); self }
    pub fn with_click(mut self, msg: Message) -> Self { self.on_click = Some(msg); self }
    pub fn sticky(mut self) -> Self { self.expires_at = None; self }
    #[allow(dead_code)]
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.expires_at = Some(self.created_at + ttl); self
    }
    pub fn is_expired(&self, now: Instant) -> bool {
        self.expires_at.map(|t| now >= t).unwrap_or(false)
    }
}

/// Render one notification as a pill element. `shader_time` drives
/// twinkle animations (sparkly + alert pulse).
pub fn render_pill<'a>(n: &Notification, shader_time: f32) -> Element<'a, Message> {
    let p = theme::palette();

    // ── pick icon + colors per variant ─────────────────────────
    let (icon_char, icon_color, border_color) = match n.variant {
        NotificationVariant::Sparkly => {
            let phase = (shader_time * 3.4).sin().abs();
            let alpha = 0.55 + 0.45 * phase;
            (icons::STAR, Color { a: alpha, ..p.gold }, Color { a: 0.80, ..p.gold })
        }
        NotificationVariant::Alert => {
            let phase = (shader_time * 2.6).sin().abs();
            let alpha = 0.65 + 0.35 * phase;
            (icons::STAR, Color { a: alpha, ..p.gold }, Color { a: 0.55, ..p.gold })
        }
        NotificationVariant::Transit => {
            (icons::MOON_STARS, p.gold, Color { a: 0.55, ..p.gold })
        }
        NotificationVariant::Error => {
            let bad = Color { r: 0.76, g: 0.35, b: 0.24, a: 1.0 };
            (icons::EXCLAMATION_TRI, bad, Color { a: 0.7, ..bad })
        }
        NotificationVariant::Success => {
            let good = Color { r: 0.43, g: 0.66, b: 0.42, a: 1.0 };
            (icons::CHECK, good, Color { a: 0.7, ..good })
        }
        NotificationVariant::Info => {
            (icons::INFO_CIRCLE, p.gold, Color { a: 0.45, ..p.gold })
        }
    };

    // ── label assembly: optional emphasis + text ─────────────────
    let label_color = Color { r: 0.95, g: 0.90, b: 0.80, a: 1.0 };
    let label_emph_color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

    let mut label_row = row![].spacing(4).align_y(Alignment::Center);
    if let Some(e) = &n.emphasis {
        label_row = label_row.push(
            text(e.clone())
                .font(crate::font::BODY_BOLD)
                .size(theme::text_xs())
                .color(label_emph_color),
        );
    }
    label_row = label_row.push(
        text(n.text.clone())
            .size(theme::text_xs())
            .color(label_color),
    );

    // ── icon: sparkly variant uses ShootingStar canvas; others a glyph
    let icon_el: Element<'a, Message> = if n.variant == NotificationVariant::Sparkly {
        Canvas::new(ShootingStar { time: shader_time })
            .width(Length::Fixed(56.0))
            .height(Length::Fixed(18.0))
            .into()
    } else {
        text(icon_char.to_string())
            .font(icons::PHOSPHOR_BOLD)
            .size(12.0)
            .color(icon_color)
            .into()
    };

    let body = row![icon_el, label_row]
        .spacing(6)
        .align_y(Alignment::Center);

    // v12.2.3 — every pill is now a button that emits NotificationClicked.
    // The handler dismisses the pill and dispatches the pill's stored
    // `on_click` (if any). Click-to-dismiss for plain pills, click-to-
    // route-and-dismiss for clickable ones — single uniform interaction.
    let pill_id = n.id;
    let pill_style = move |_t: &iced::Theme, status: button::Status| {
        let bg_alpha = match status {
            button::Status::Hovered | button::Status::Pressed => 0.98,
            _ => 0.92,
        };
        button::Style {
            background: Some(iced::Background::Color(
                Color { r: 0.12, g: 0.10, b: 0.08, a: bg_alpha },
            )),
            text_color: Color::TRANSPARENT,
            border: iced::Border {
                color: border_color,
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: iced::Shadow {
                color: Color { a: 0.30, ..Color::BLACK },
                offset: iced::Vector::new(0.0, 1.5),
                blur_radius: 6.0,
            },
            snap: false,
        }
    };
    button(body)
        .on_press(Message::NotificationClicked(pill_id))
        .padding([3, 10])
        .style(pill_style)
        .into()
}

/// Render the active pill stack as one row element. Honors
/// MAX_VISIBLE_PILLS by showing only the most-recent N (others remain
/// in the deque until expiry — newer pills push older ones off-screen,
/// not out of memory).
pub fn render_pill_stack<'a>(
    notifications: &std::collections::VecDeque<Notification>,
    shader_time: f32,
) -> Element<'a, Message> {
    if notifications.is_empty() {
        return Space::new().into();
    }
    let mut r = row![].spacing(6).align_y(Alignment::Center);
    // most-recent first → render newest on the right (closest to gear)
    let take = notifications.len().min(MAX_VISIBLE_PILLS);
    let start = notifications.len() - take;
    for i in start..notifications.len() {
        if let Some(n) = notifications.get(i) {
            r = r.push(render_pill(n, shader_time));
        }
    }
    r.into()
}

/// v12.2.4 — Format relative time-ago string ("just now", "5m", "2h", "1d ago").
fn time_ago(created: Instant) -> String {
    let secs = created.elapsed().as_secs();
    if secs < 30 { "just now".into() }
    else if secs < 60 { format!("{secs}s ago") }
    else if secs < 3600 { format!("{}m ago", secs / 60) }
    else if secs < 86400 { format!("{}h ago", secs / 3600) }
    else { format!("{}d ago", secs / 86400) }
}

/// v12.2.4 — Notification drawer. Overlays via `stack!` in the main view
/// when `notifications_drawer_open == true`. Shows up to MAX_HISTORY entries
/// from `notification_history` newest-first. Each row is click-to-dismiss
/// (or click-to-route if the pill carried an on_click). Footer "Clear All"
/// wipes both active deque + history.
pub fn render_drawer<'a>(
    history: &[Notification],
) -> Element<'a, Message> {
    let p = theme::palette();

    let header = row![
        text(icons::BELL.to_string())
            .font(icons::PHOSPHOR_BOLD)
            .size(14.0)
            .color(p.gold),
        text("Recent notifications")
            .size(theme::text_xs())
            .color(Color { r: 0.95, g: 0.90, b: 0.80, a: 1.0 })
            .font(crate::font::BODY_BOLD),
        Space::new().width(Length::Fill),
        button(
            text("Clear all")
                .size(theme::text_xs())
                .color(Color { a: 0.85, ..p.gold }),
        )
        .padding([2, 8])
        .on_press(Message::ClearAllNotifications)
        .style(|_t: &iced::Theme, status: button::Status| {
            let p = theme::palette();
            let bg_alpha = match status {
                button::Status::Hovered => 0.18,
                _ => 0.0,
            };
            button::Style {
                background: Some(iced::Background::Color(Color { a: bg_alpha, ..p.gold })),
                text_color: Color::TRANSPARENT,
                border: iced::Border { radius: 4.0.into(), ..Default::default() },
                ..button::Style::default()
            }
        }),
        button(
            text(icons::X_LG.to_string())
                .font(icons::PHOSPHOR_BOLD)
                .size(11.0)
                .color(Color { a: 0.7, r: 0.95, g: 0.90, b: 0.80 }),
        )
        .padding([2, 6])
        .on_press(Message::ToggleNotificationDrawer)
        .style(|_t: &iced::Theme, status: button::Status| {
            let bg_alpha = match status {
                button::Status::Hovered => 0.15,
                _ => 0.0,
            };
            button::Style {
                background: Some(iced::Background::Color(Color { r: 1.0, g: 1.0, b: 1.0, a: bg_alpha })),
                text_color: Color::TRANSPARENT,
                border: iced::Border { radius: 4.0.into(), ..Default::default() },
                ..button::Style::default()
            }
        }),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let body: Element<'a, Message> = if history.is_empty() {
        container(
            text("No notifications yet")
                .size(theme::text_xs())
                .color(Color { a: 0.55, r: 0.95, g: 0.90, b: 0.80 }),
        )
        .padding(20)
        .center_x(Length::Fill)
        .into()
    } else {
        let mut col = column![].spacing(2);
        // Newest first
        for n in history.iter().rev().take(MAX_HISTORY) {
            col = col.push(render_drawer_row(n));
        }
        scrollable(col).height(Length::Fixed(280.0)).into()
    };

    let drawer_inner = column![
        header,
        iced::widget::rule::horizontal(1),
        body,
    ]
    .spacing(4);

    container(drawer_inner)
        .width(Length::Fixed(360.0))
        .padding(8)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(
                Color { r: 0.10, g: 0.08, b: 0.06, a: 0.97 },
            )),
            border: iced::Border {
                color: Color { a: 0.55, ..p.gold },
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow {
                color: Color { a: 0.45, ..Color::BLACK },
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 14.0,
            },
            ..Default::default()
        })
        .into()
}

fn render_drawer_row<'a>(n: &Notification) -> Element<'a, Message> {
    let p = theme::palette();
    let (icon_char, icon_color) = match n.variant {
        NotificationVariant::Sparkly | NotificationVariant::Alert | NotificationVariant::Transit => (icons::STAR, p.gold),
        NotificationVariant::Error => (icons::EXCLAMATION_TRI, Color { r: 0.76, g: 0.35, b: 0.24, a: 1.0 }),
        NotificationVariant::Success => (icons::CHECK, Color { r: 0.43, g: 0.66, b: 0.42, a: 1.0 }),
        NotificationVariant::Info => (icons::INFO_CIRCLE, p.gold),
    };
    let icon_el = text(icon_char.to_string())
        .font(icons::PHOSPHOR_BOLD)
        .size(11.0)
        .color(icon_color);

    let mut text_col = column![].spacing(1);
    let mut top_row = row![].spacing(4).align_y(Alignment::Center);
    if let Some(e) = &n.emphasis {
        top_row = top_row.push(
            text(e.clone())
                .font(crate::font::BODY_BOLD)
                .size(theme::text_xs())
                .color(Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),
        );
    }
    top_row = top_row.push(
        text(n.text.clone())
            .size(theme::text_xs())
            .color(Color { r: 0.95, g: 0.90, b: 0.80, a: 1.0 }),
    );
    text_col = text_col.push(top_row);
    text_col = text_col.push(
        text(time_ago(n.created_at))
            .size(9.0)
            .color(Color { r: 0.66, g: 0.60, b: 0.47, a: 1.0 }),
    );

    let body = row![icon_el, text_col]
        .spacing(8)
        .align_y(Alignment::Start);

    container(body)
        .padding([6, 10])
        .width(Length::Fill)
        .style(|_t: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(
                Color { r: 0.13, g: 0.11, b: 0.09, a: 0.5 },
            )),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
