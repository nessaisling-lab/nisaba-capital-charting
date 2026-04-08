// Iced 0.13 uses Task instead of Command, and application() builder instead of Application trait
use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Element, Length, Subscription, Task};
use pursuit_week4_automation::models::PriceRow;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Data flow:
//
//  iced::time::every(30s) → Message::Tick
//       │
//       ▼
//  Task::perform(fetch_prices(pool)) → Message::DataLoaded(rows)
//       │
//       ▼
//  update() stores rows in state
//       │
//       ▼
//  view() renders scrollable OHLCV table
//
//  "Refresh Now" button → Message::RefreshNow
//  → same Task::perform one-shot query
// ---------------------------------------------------------------------------

pub fn main() -> iced::Result {
    iced::application("Financial Dashboard", Dashboard::update, Dashboard::view)
        .subscription(Dashboard::subscription)
        .run_with(Dashboard::new)
}

#[derive(Default)]
struct Dashboard {
    pool: Option<Arc<PgPool>>,
    rows: Vec<PriceRow>,
    status: String,
    refreshing: bool,
}

#[derive(Debug, Clone)]
enum Message {
    PoolReady(Result<Arc<PgPool>, String>),
    DataLoaded(Result<Vec<PriceRow>, String>),
    RefreshNow,
    Tick,
}

impl Dashboard {
    fn new() -> (Self, Task<Message>) {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:dev@localhost:5432/financial_dashboard".to_string()
        });

        (
            Dashboard {
                status: "Connecting to database...".to_string(),
                ..Default::default()
            },
            Task::perform(connect_db(database_url), Message::PoolReady),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PoolReady(Ok(pool)) => {
                self.status = "Connected. Loading data...".to_string();
                let p = Arc::clone(&pool);
                self.pool = Some(pool);
                Task::perform(fetch_prices(p), Message::DataLoaded)
            }
            Message::PoolReady(Err(e)) => {
                self.status = format!("DB connection failed: {e}");
                Task::none()
            }
            Message::DataLoaded(Ok(rows)) => {
                self.refreshing = false;
                self.status = if rows.is_empty() {
                    "No data yet — run the scraper first.".to_string()
                } else {
                    format!("Loaded {} rows for AAPL", rows.len())
                };
                self.rows = rows;
                Task::none()
            }
            Message::DataLoaded(Err(e)) => {
                self.refreshing = false;
                // Keep showing stale data — don't clear self.rows
                self.status = format!("Query error (showing stale data): {e}");
                Task::none()
            }
            Message::RefreshNow => {
                if let Some(pool) = &self.pool {
                    self.refreshing = true;
                    self.status = "Refreshing...".to_string();
                    Task::perform(fetch_prices(Arc::clone(pool)), Message::DataLoaded)
                } else {
                    Task::none()
                }
            }
            Message::Tick => {
                if let Some(pool) = &self.pool {
                    Task::perform(fetch_prices(Arc::clone(pool)), Message::DataLoaded)
                } else {
                    Task::none()
                }
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        // Poll the database every 30 seconds
        iced::time::every(Duration::from_secs(30)).map(|_| Message::Tick)
    }

    fn view(&self) -> Element<'_, Message> {
        let header = row![
            text("Date").width(Length::FillPortion(2)),
            text("Open").width(Length::FillPortion(2)),
            text("High").width(Length::FillPortion(2)),
            text("Low").width(Length::FillPortion(2)),
            text("Close").width(Length::FillPortion(2)),
            text("Volume").width(Length::FillPortion(3)),
        ]
        .spacing(10);

        let data_rows: Column<Message> = if self.rows.is_empty() {
            column![text(&self.status).size(16)]
        } else {
            let rows: Vec<Element<Message>> = self
                .rows
                .iter()
                .map(|r| {
                    row![
                        text(r.date.to_string()).width(Length::FillPortion(2)),
                        text(format!("{:.2}", r.open)).width(Length::FillPortion(2)),
                        text(format!("{:.2}", r.high)).width(Length::FillPortion(2)),
                        text(format!("{:.2}", r.low)).width(Length::FillPortion(2)),
                        text(format!("{:.2}", r.close)).width(Length::FillPortion(2)),
                        text(r.volume.to_string()).width(Length::FillPortion(3)),
                    ]
                    .spacing(10)
                    .into()
                })
                .collect();
            Column::with_children(rows).spacing(4)
        };

        let refresh_label = if self.refreshing {
            "Refreshing..."
        } else {
            "Refresh Now"
        };

        let content = column![
            text("AAPL — Daily Price Data").size(24),
            text(&self.status).size(14),
            button(refresh_label).on_press(Message::RefreshNow),
            header,
            scrollable(data_rows).height(Length::Fill),
        ]
        .spacing(12)
        .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

async fn connect_db(url: String) -> Result<Arc<PgPool>, String> {
    PgPoolOptions::new()
        .max_connections(3)
        .connect(&url)
        .await
        .map(Arc::new)
        .map_err(|e| e.to_string())
}

// Non-macro query: no DATABASE_URL required at compile time
async fn fetch_prices(pool: Arc<PgPool>) -> Result<Vec<PriceRow>, String> {
    sqlx::query_as::<_, PriceRow>(
        "SELECT ticker, date, open, high, low, close, volume \
         FROM price_data \
         WHERE ticker = 'AAPL' \
         ORDER BY date DESC \
         LIMIT 100",
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| e.to_string())
}
