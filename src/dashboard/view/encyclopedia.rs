//! v11.5.E4 — Encyclopedia tab. Wikipedia summary card per ticker.

use iced::widget::{button, column, container, image, row, text, Space};
use iced::{Alignment, Color, Element, Length};

use crate::font;
use crate::icons;
use crate::state::{Dashboard, Message};
use crate::theme;
use super::shared::{card, eyebrow, gutter_scroll};

impl Dashboard {
    pub(crate) fn view_encyclopedia(&self) -> Element<'_, Message> {
        let p = theme::palette();

        let body: Element<'_, Message> = match &self.wiki_summary {
            Some(w) => {
                let title = text(w.title.clone())
                    .font(font::DISPLAY)
                    .size(theme::text_2xl())
                    .color(p.gold);

                let extract: Element<'_, Message> = match &w.extract {
                    Some(e) if !e.is_empty() => text(e.clone())
                        .size(theme::text_base())
                        .color(p.ink)
                        .into(),
                    _ => text("Wikipedia returned no summary for this title. Try another ticker, or run the scraper to refresh.")
                        .size(theme::text_sm())
                        .color(Color { a: 0.7, ..p.ink })
                        .into(),
                };

                let age_days = (chrono::Utc::now() - w.fetched_at).num_days();
                let age_label = if age_days == 0 {
                    "fetched today".to_string()
                } else if age_days == 1 {
                    "fetched yesterday".to_string()
                } else {
                    format!("fetched {age_days} days ago")
                };
                let meta = text(format!("Source: Wikipedia · {age_label}"))
                    .size(theme::text_xs())
                    .color(Color { a: 0.65, ..p.ink });

                let link_btn: Element<'_, Message> = match &w.wikipedia_url {
                    Some(url) if !url.is_empty() => button(
                        row![
                            text(icons::BOOK_OPEN.to_string()).font(icons::PHOSPHOR).size(theme::text_sm()),
                            text("Read full article on Wikipedia").size(theme::text_sm()),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    )
                    .on_press(Message::OpenUrl(url.clone()))
                    .padding([6, 12])
                    .into(),
                    _ => Space::new().into(),
                };

                // v11.5.E4 (extension) — Iced image widget renders the
                // Wikipedia thumbnail when bytes are cached in state.
                let thumb: Element<'_, Message> = match &self.wiki_thumbnail_bytes {
                    Some(bytes) => container(
                        image(image::Handle::from_bytes(bytes.clone()))
                            .width(Length::Fixed(220.0))
                            .height(Length::Fixed(220.0))
                            .content_fit(iced::ContentFit::Contain),
                    )
                    .style(|_t: &iced::Theme| {
                        let p = theme::palette();
                        container::Style {
                            background: Some(iced::Background::Color(Color { a: 0.06, ..p.gold })),
                            border: iced::Border {
                                color: Color { a: 0.4, ..p.gold },
                                width: 1.0,
                                radius: 3.0.into(),
                            },
                            ..Default::default()
                        }
                    })
                    .padding(6)
                    .into(),
                    None => match &w.thumbnail_url {
                        Some(_) => text("(loading thumbnail…)")
                            .size(theme::text_xs())
                            .color(Color { a: 0.5, ..p.ink })
                            .into(),
                        None => Space::new().into(),
                    },
                };

                let body_block = column![
                    title,
                    Space::new().height(Length::Fixed(theme::SPACE_SM)),
                    meta,
                    Space::new().height(Length::Fixed(theme::SPACE_SM)),
                    extract,
                    Space::new().height(Length::Fixed(theme::SPACE_MD)),
                    link_btn,
                ]
                .spacing(4);

                row![thumb, body_block].spacing(theme::SPACE_MD).into()
            }
            None => column![
                text("No encyclopedia entry cached for this ticker yet.")
                    .size(theme::text_base()),
                Space::new().height(Length::Fixed(theme::SPACE_SM)),
                text("Run the scraper to fetch Wikipedia summaries (30-day TTL). The dashboard will pick up the new entry on the next ticker selection.")
                    .size(theme::text_sm())
                    .color(Color { a: 0.7, ..p.ink }),
            ]
            .spacing(4)
            .into(),
        };

        let main = card(column![
            eyebrow("ENCYCLOPEDIA"),
            Space::new().height(Length::Fixed(theme::SPACE_XS)),
            body,
        ]
        .spacing(4));

        container(gutter_scroll(main, 600.0))
            .width(Length::Fill)
            .padding([10, 14])
            .into()
    }
}
