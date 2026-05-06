use iced::widget::canvas::Canvas;
use iced::widget::{button, column, container, mouse_area, pick_list, pin, row, rule, stack, text, text_input, tooltip, Column, Shader};
use iced::{Alignment, Color, Element, Length};

// v11.3 — natal wheel overlay constants (must match WGSL shader)
// v11.6.C — reduce tilt 0.32 → 0.10 so wheel reads as a sphere not an oval.
// User feedback 2026-05-05 video review: "I don't know why it keeps on
// being oval, but that needs to be resolved."
const CAMERA_TILT: f32 = 0.10;
const R_NATAL: f32 = 0.644;
const R_TRANSIT: f32 = 0.810;
const GLYPH_SIZE: f32 = 14.0;

fn planet_glyph(name: &str) -> &'static str {
    match name {
        "Sun" => "\u{2609}", "Moon" => "\u{263D}", "Mercury" => "\u{263F}",
        "Venus" => "\u{2640}", "Mars" => "\u{2642}", "Jupiter" => "\u{2643}",
        "Saturn" => "\u{2644}", "Uranus" => "\u{2645}", "Neptune" => "\u{2646}",
        "Pluto" => "\u{2647}", "North Node" => "\u{260A}", "South Node" => "\u{260B}",
        "Chiron" => "\u{26B7}", _ => "\u{2022}",
    }
}

/// Compute pixel position for a planet glyph. Returns (x, y) for `pin().x(x).y(y)`.
/// Accounts for camera tilt (Y compression) and centers the glyph on its position.
fn planet_pixel_pos(longitude: f64, radius: f32, chart_px: f32) -> (f32, f32) {
    let angle = -(longitude as f32).to_radians();
    let chart_x = radius * angle.cos();
    let chart_y = radius * angle.sin();
    let screen_y = chart_y * (1.0 - CAMERA_TILT);
    let half = chart_px / 2.0;
    let px = (chart_x + 1.0) * half - GLYPH_SIZE / 2.0;
    let py = (screen_y + 1.0) * half - GLYPH_SIZE / 2.0;
    (px, py)
}

use crate::astrology::{build_transits_section, build_wheel_legend};
use crate::calendar::AstroCalendar;
use crate::shaders::NatalWheel3DProgram;
use crate::font;
use crate::icons;
use crate::state::{BacktestWindowChoice, ChartSize, Dashboard, Message};
use super::shared::{icon_eyebrow, section_rule};
use crate::strategy::Condition;
use crate::theme;

impl Dashboard {
    pub(crate) fn view_astrology(&self) -> Element<'_, Message> {
        // ── Natal wheel + transits + horoscope ──────────────
        let moon_phase = self
            .astro_score
            .as_ref()
            .and_then(|s| s.moon_phase.as_deref());
        let moon_deg = self.astro_score.as_ref().and_then(|s| s.moon_phase_deg);
        let mercury_rx = self
            .astro_score
            .as_ref()
            .and_then(|s| s.mercury_rx)
            .unwrap_or(false);

        let astrology_section: Element<Message> = if self.natal_positions.is_empty() {
            column![
                text(format!("{} Astrology", self.selected_ticker)).font(font::DISPLAY).size(theme::text_lg()),
                rule::horizontal(1),
                text(format!("No birth chart yet for {}.", self.selected_ticker)).size(theme::text_base()),
                text("The scraper enriches ~50 tickers per day via SEC EDGAR.").size(theme::text_sm()),
                text("Once an IPO date is found, the natal chart is computed automatically.")
                    .size(theme::text_sm()),
            ]
            .spacing(6)
            .into()
        } else {
            let p = theme::palette();
            // Active zodiac sign from Sun's transit longitude (first transit = Sun)
            let active_sign = self.daily_transits.first()
                .map(|sun| (sun.longitude as f32 / 30.0).floor().clamp(0.0, 11.0))
                .unwrap_or(0.0);

            let shader_widget = Shader::new(NatalWheel3DProgram {
                time: self.shader_time,
                natal_positions: self.natal_positions.clone(),
                transit_positions: self.daily_transits.clone(),
                bg_color: [p.bg.r, p.bg.g, p.bg.b, p.bg.a],
                gold_color: [
                    theme::NATAL_GOLD.r, theme::NATAL_GOLD.g,
                    theme::NATAL_GOLD.b, theme::NATAL_GOLD.a,
                ],
                transit_color: [
                    theme::TRANSIT_BLUE.r, theme::TRANSIT_BLUE.g,
                    theme::TRANSIT_BLUE.b, theme::TRANSIT_BLUE.a,
                ],
                retro_color: [
                    theme::RETROGRADE_RED.r, theme::RETROGRADE_RED.g,
                    theme::RETROGRADE_RED.b, theme::RETROGRADE_RED.a,
                ],
                active_sign,
                show_natal: self.show_natal_planets,
                show_transit: self.show_transit_planets,
                show_aspects: self.show_aspects,
                show_retrogrades: self.show_retrogrades,
            })
            // v11.5.D2 — mouse-wheel zoom: chart_size base * runtime zoom factor
            .width(Length::Fixed(self.chart_size.pixels() * self.natal_zoom))
            .height(Length::Fixed(self.chart_size.pixels() * self.natal_zoom));
            let chart_px = self.chart_size.pixels() * self.natal_zoom;

            // v11.3 — overlay planet glyphs + hover tooltips (3e + 3f)
            fn tip_style(_t: &iced::Theme) -> container::Style {
                let p = theme::palette();
                container::Style {
                    background: Some(iced::Background::Color(p.surface)),
                    border: iced::Border { color: p.gold, width: 1.0, radius: 3.0.into() },
                    ..Default::default()
                }
            }
            let mut layers: Vec<Element<Message>> = vec![shader_widget.into()];

            if self.show_natal_planets {
                for np in &self.natal_positions {
                    let (px, py) = planet_pixel_pos(np.longitude, R_NATAL, chart_px);
                    let glyph_color = theme::NATAL_GOLD;
                    // Wave 9.5.2 — enrich planet hover with decan + Sabian
                    // + critical/OOB flags so each glyph reveals its full
                    // narrative + precision context.
                    let decan = pursuit_week4_automation::astrology::decans::decan_for_longitude(np.longitude);
                    let sabian = pursuit_week4_automation::astrology::sabian::sabian_for_longitude(np.longitude);
                    let crit = pursuit_week4_automation::astrology::critical::is_critical_degree(np.longitude);
                    let mut info = format!(
                        "{} in {} {:.1}°{}",
                        np.planet, np.sign, np.degree,
                        if np.retrograde { "  R" } else { "" }
                    );
                    info.push_str(&format!(
                        "\n• Decan: {}-{} {}",
                        decan.ruler.name(), decan.sub_ruler.name(), decan.theme,
                    ));
                    info.push_str(&format!(
                        "\n• Sabian {}°: {}",
                        sabian.degree, sabian.image,
                    ));
                    if let Some(c) = crit {
                        info.push_str(&format!("\n• {}", c.label()));
                    }
                    // Declination not stored in natal_positions table —
                    // approximate from longitude assuming β = 0 (sufficient
                    // accuracy for OOB tagging on outer planets).
                    let approx_dec = pursuit_week4_automation::astrology::ephemeris::ecliptic_to_declination(np.longitude, 0.0);
                    if pursuit_week4_automation::astrology::ephemeris::is_out_of_bounds(approx_dec) {
                        let dir = if approx_dec > 0.0 { "north" } else { "south" };
                        info.push_str(&format!("\n• Out-of-bounds {dir} ({:.1}°)", approx_dec));
                    }
                    let glyph = text(planet_glyph(&np.planet))
                        .size(GLYPH_SIZE)
                        .color(glyph_color);
                    let hoverable = tooltip(
                        glyph,
                        container(text(info).size(theme::text_xs()))
                            .padding([4, 8]).style(tip_style),
                        tooltip::Position::Top,
                    );
                    layers.push(pin(hoverable).x(px).y(py).into());
                }
            }

            if self.show_transit_planets {
                for tp in &self.daily_transits {
                    let (px, py) = planet_pixel_pos(tp.longitude, R_TRANSIT, chart_px);
                    let is_retro = tp.retrograde && self.show_retrogrades;
                    let glyph_color = if is_retro {
                        theme::RETROGRADE_RED
                    } else {
                        theme::TRANSIT_BLUE
                    };
                    let info = format!(
                        "{} transiting {}{}",
                        tp.planet, tp.sign,
                        if tp.retrograde { "  (Rx)" } else { "" }
                    );
                    let glyph = text(planet_glyph(&tp.planet))
                        .size(GLYPH_SIZE)
                        .color(glyph_color);
                    let hoverable = tooltip(
                        glyph,
                        container(text(info).size(theme::text_xs()))
                            .padding([4, 8]).style(tip_style),
                        tooltip::Position::Top,
                    );
                    layers.push(pin(hoverable).x(px).y(py).into());
                }
            }

            // ── v11.5.D1 — aspect line hover hit zones (Approach A) ──
            // Place an invisible 26×26 mouse_area at each aspect line midpoint
            // and at 30% from each end so single-click coverage spans the line.
            // Tooltip shows full aspect detail without obscuring the chart.
            if self.show_aspects {
                const HIT_PX: f32 = 26.0;
                const HIT_OFFSET: f32 = HIT_PX / 2.0;
                let aspect_tip_style = |_t: &iced::Theme| {
                    let p = theme::palette();
                    container::Style {
                        background: Some(iced::Background::Color(p.surface)),
                        border: iced::Border { color: p.gold, width: 1.0, radius: 3.0.into() },
                        ..Default::default()
                    }
                };
                for obj in &self.astro_aspects {
                    let Some(transit_planet) = obj["transit_planet"].as_str() else { continue };
                    let Some(natal_planet)   = obj["natal_planet"].as_str()   else { continue };
                    let Some(aspect_name)    = obj["aspect"].as_str()         else { continue };
                    let symbol     = obj["aspect_symbol"].as_str().unwrap_or("");
                    let orb        = obj["orb"].as_f64().unwrap_or(0.0);
                    let applying   = obj["applying"].as_bool().unwrap_or(true);
                    let dignity    = obj["dignity"].as_str().unwrap_or("");
                    let effect     = obj["effect"].as_str().unwrap_or("—");
                    let delta      = obj["score_delta"].as_f64().unwrap_or(0.0);

                    let Some(np) = self.natal_positions.iter().find(|p| p.planet == natal_planet) else { continue };
                    let Some(tp) = self.daily_transits.iter().find(|p| p.planet == transit_planet) else { continue };

                    let (nx, ny) = planet_pixel_pos(np.longitude, R_NATAL, chart_px);
                    let (tx, ty) = planet_pixel_pos(tp.longitude, R_TRANSIT, chart_px);

                    let info = format!(
                        "{} {} {} {} {} (orb {:.1}°, {}{}{})\n→ {} ({:+.1})",
                        transit_planet, symbol, natal_planet,
                        aspect_name,
                        if applying { "applying" } else { "separating" },
                        orb,
                        if dignity.is_empty() { "" } else { dignity },
                        if dignity.is_empty() { "" } else { ", " },
                        if applying { "tightening" } else { "loosening" },
                        effect, delta,
                    );

                    // Three sample points: 30% from transit, midpoint, 30% from natal
                    let samples = [0.30_f32, 0.50, 0.70];
                    for t in samples {
                        let mx = tx + (nx - tx) * t;
                        let my = ty + (ny - ty) * t;
                        let info_clone = info.clone();
                        let hit = mouse_area(
                            container(iced::widget::Space::new())
                                .width(Length::Fixed(HIT_PX))
                                .height(Length::Fixed(HIT_PX)),
                        );
                        let tip = tooltip(
                            hit,
                            container(text(info_clone).size(theme::text_xs()))
                                .padding([4, 8]).style(aspect_tip_style),
                            tooltip::Position::Top,
                        );
                        layers.push(
                            pin(tip)
                                .x(mx + GLYPH_SIZE / 2.0 - HIT_OFFSET)
                                .y(my + GLYPH_SIZE / 2.0 - HIT_OFFSET)
                                .into(),
                        );
                    }
                }
            }

            // v11.5.D2 — mouse_area captures wheel scroll for runtime zoom
            let wheel_stack = stack(layers)
                .width(Length::Fixed(chart_px))
                .height(Length::Fixed(chart_px));
            let natal_wheel: Element<Message> = mouse_area(wheel_stack)
                .on_scroll(|delta| {
                    let dy = match delta {
                        iced::mouse::ScrollDelta::Lines { y, .. } => y * 0.10,
                        iced::mouse::ScrollDelta::Pixels { y, .. } => y * 0.005,
                    };
                    Message::NatalWheelZoom(dy)
                })
                .into();

            // v11.0: Sun/Moon/Rising "Big Three" summary
            let big_three_row: Element<Message> = {
                let sun_sign = self.natal_positions.iter()
                    .find(|p| p.planet == "Sun")
                    .map(|p| p.sign.as_str())
                    .unwrap_or("—");
                let moon_sign = self.natal_positions.iter()
                    .find(|p| p.planet == "Moon")
                    .map(|p| p.sign.as_str())
                    .unwrap_or("—");
                let rising_sign = self.natal_angles.as_ref()
                    .map(|a| {
                        let (sign, _) = pursuit_week4_automation::astrology::ephemeris::longitude_to_sign(a.ascendant);
                        sign
                    })
                    .unwrap_or("—");

                let gold = theme::palette().gold;
                row![
                    text(format!("☉ Sun: {sun_sign}")).font(font::INTER).size(theme::text_base()).color(gold),
                    text("  ·  ").size(theme::text_base()),
                    text(format!("☽ Moon: {moon_sign}")).font(font::INTER).size(theme::text_base()).color(gold),
                    text("  ·  ").size(theme::text_base()),
                    text(format!("↑ Rising: {rising_sign}")).font(font::INTER).size(theme::text_base()).color(gold),
                ]
                .spacing(0)
                .align_y(Alignment::Center)
                .into()
            };

            // v11.1: Layer visibility toggle buttons
            let make_toggle = |label: &str, visible: bool, msg: Message| -> Element<Message> {
                let ico = if visible { icons::EYE } else { icons::EYE_SLASH };
                let alpha = if visible { 1.0 } else { 0.4 };
                let clr = iced::Color { a: alpha, ..p.gold };
                button(
                    row![
                        text(ico.to_string()).font(icons::PHOSPHOR).size(theme::text_xs()).color(clr),
                        text(label.to_string()).size(theme::text_xs()).color(clr),
                    ].spacing(3).align_y(Alignment::Center)
                )
                .on_press(msg)
                .padding([2, 6])
                .style(|_theme: &iced::Theme, _status| iced::widget::button::Style {
                    background: None,
                    border: iced::Border::default(),
                    text_color: iced::Color::TRANSPARENT,
                    ..Default::default()
                })
                .into()
            };
            let size_picker = pick_list(
                ChartSize::all().to_vec(),
                Some(self.chart_size),
                Message::SetChartSize,
            )
            .text_size(theme::text_xs());

            // v11.5.D2 — zoom readout + reset button
            let zoom_pct = (self.natal_zoom * 100.0).round() as i32;
            let zoom_btn = button(
                row![
                    text(format!("{}%", zoom_pct)).size(theme::text_xs()),
                ]
                .spacing(2),
            )
            .on_press(Message::NatalWheelZoomReset)
            .padding([2, 6]);

            let layer_toggles = row![
                make_toggle("Natal",   self.show_natal_planets,   Message::ToggleChartNatal),
                make_toggle("Transit", self.show_transit_planets,  Message::ToggleChartTransit),
                make_toggle("Aspects", self.show_aspects,          Message::ToggleChartAspects),
                make_toggle("Retro",   self.show_retrogrades,      Message::ToggleChartRetrogrades),
                iced::widget::Space::new().width(Length::Fixed(8.0)),
                size_picker,
                iced::widget::Space::new().width(Length::Fixed(8.0)),
                zoom_btn,
            ]
            .spacing(4)
            .align_y(Alignment::Center);

            // Wave 9.5.1 — Year of [Lord] badge. Computes the Hellenistic
            // annual time-lord from natal IPO date + ascendant. Renders a
            // gold-outline pill below the chart title.
            let year_of_lord_badge: Element<Message> = if let (Some(ipo), Some(angles)) =
                (self.natal_ipo_date, self.natal_angles.as_ref())
            {
                let target = chrono::Local::now().date_naive();
                let prof = pursuit_week4_automation::astrology::profections::compute_profection(
                    ipo, angles.ascendant, target,
                );
                let line = pursuit_week4_automation::astrology::profections::summary_line(&prof);
                let p_b = theme::palette();
                container(
                    text(line)
                        .size(theme::text_sm())
                        .color(Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),
                )
                .padding([4, 12])
                .style(move |_t: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(
                        Color { r: 0.12, g: 0.10, b: 0.08, a: 0.85 },
                    )),
                    border: iced::Border {
                        color: Color { a: 0.65, ..p_b.gold },
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    ..Default::default()
                })
                .into()
            } else {
                iced::widget::Space::new().into()
            };

            // v11.5.B3 — zodiac legend relocated ABOVE natal wheel
            let wheel_col = column![
                text(format!("{} Birth Chart", self.selected_ticker)).font(font::DISPLAY).size(theme::text_lg()),
                year_of_lord_badge,
                big_three_row,
                layer_toggles,
                build_wheel_legend(),
                natal_wheel,
            ]
            .spacing(4)
            .align_x(Alignment::Center);

            let transits_col =
                column![build_transits_section(
                    &self.astro_aspects,
                    moon_phase,
                    moon_deg,
                    mercury_rx
                ),]
                .width(Length::Fill);

            row![wheel_col, transits_col]
                .spacing(20)
                .align_y(Alignment::Start)
                .into()
        };

        // ── Horoscope Reading (v11.3: standalone narrative section) ──
        let horoscope_section: Element<'_, Message> = if let Some(ref h) = self.horoscope {
            let key_transit_items: Vec<Element<Message>> = h
                .key_transits
                .iter()
                .map(|t| {
                    row![
                        text(&t.transit_desc)
                            .size(theme::text_sm())
                            .width(Length::Fixed(180.0)),
                        text(&t.strength)
                            .size(theme::text_xs())
                            .width(Length::Fixed(120.0)),
                        text(&t.financial_implication)
                            .size(theme::text_xs())
                            .width(Length::Fill),
                    ]
                    .spacing(8)
                    .into()
                })
                .collect();

            let mercury_line: Element<Message> = if let Some(ref warn) = h.mercury_warning {
                text(format!("Mercury: {warn}"))
                    .size(theme::text_xs())
                    .color(theme::ZONE_UNFAVORABLE)
                    .into()
            } else {
                text("Mercury: Direct")
                    .size(theme::text_xs())
                    .into()
            };

            column![
                text(&h.overall_outlook).size(theme::text_base()),
                row![
                    text(format!("Theme: {}", h.dominant_theme)).size(theme::text_sm()),
                    text(format!("Confidence: {:.0}/100", h.confidence)).size(theme::text_sm()),
                ]
                .spacing(20),
                text("Key Transits:").size(theme::text_sm()),
                Column::with_children(key_transit_items).spacing(2),
                row![
                    text(&h.moon_guidance).size(theme::text_xs()),
                    mercury_line,
                    text(format!("Timing: {}", h.timing_window)).size(theme::text_xs()),
                ]
                .spacing(12),
            ]
            .spacing(6)
            .into()
        } else {
            text("Horoscope reading not yet generated for today. Run the scraper to compute.")
                .size(theme::text_sm())
                .into()
        };

        // ── Backtest Section ────────────────────────────────
        // v11.5.C6 — explanatory tooltips on threshold inputs
        let backtest_section: Element<'_, Message> = {
            let config_row = row![
                super::shared::explain(
                    text("Buy when astro >").size(theme::text_sm()),
                    "Astrology Score threshold above which a long position is opened. 65 = favorable; 75 = optimal-only; 50 = aggressive.",
                ),
                text_input("65", &self.backtest_buy_input)
                    .on_input(Message::BacktestBuyInput)
                    .width(Length::Fixed(50.0))
                    .size(theme::text_sm()),
                super::shared::explain(
                    text("Sell when astro <").size(theme::text_sm()),
                    "Astrology Score threshold below which the position is closed. 35 = wait for clear weakness; 45 = quicker exit; 25 = hold through neutral.",
                ),
                text_input("35", &self.backtest_sell_input)
                    .on_input(Message::BacktestSellInput)
                    .width(Length::Fixed(50.0))
                    .size(theme::text_sm()),
                // Wave 9.5.6 — TimeWindow picker. Wires `BacktestConfig.time_window`
                // through user choice. Cycle-aligned modes use the Wave 9.A2 returns
                // engine to find Saturn / Jupiter return dates per natal chart.
                pick_list(
                    BacktestWindowChoice::all(),
                    Some(self.backtest_window_choice),
                    Message::SetBacktestWindowChoice,
                )
                .text_size(theme::text_sm()),
                button(text("Run Backtest").size(theme::text_sm())).on_press(Message::RunBacktest),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            if let Some(ref bt) = self.backtest_result {
                let clear_btn = button(text("Clear Results").size(theme::text_sm()))
                    .on_press(Message::ClearBacktest);

                if let Some(ref msg) = bt.insufficient_data {
                    column![
                        text("Astro Backtest").font(font::DISPLAY).size(theme::text_md()),
                        config_row,
                        clear_btn,
                        rule::horizontal(1),
                        text(msg.as_str())
                            .size(theme::text_base())
                            .color(theme::ZONE_UNFAVORABLE),
                        text("Run the scraper to fetch more astro + price history.")
                            .size(theme::text_sm()),
                    ]
                    .spacing(6)
                    .into()
                } else {
                    let strat_color = if bt.total_return_pct > bt.buy_hold_return_pct {
                        theme::ZONE_OPTIMAL
                    } else {
                        theme::ZONE_MISALIGNED
                    };
                    let acc_color = if bt.signal_accuracy_pct >= 55.0 {
                        theme::ZONE_OPTIMAL
                    } else if bt.signal_accuracy_pct >= 45.0 {
                        theme::ZONE_NEUTRAL
                    } else {
                        theme::ZONE_MISALIGNED
                    };

                    let metrics = column![
                        row![
                            text(format!("Strategy: {:.1}%", bt.total_return_pct))
                                .size(theme::text_base())
                                .color(strat_color),
                            text("  vs  ").size(theme::text_sm()),
                            text(format!("Buy & Hold: {:.1}%", bt.buy_hold_return_pct))
                                .size(theme::text_base()),
                        ]
                        .spacing(4),
                        row![
                            text(format!("Trades: {}", bt.num_trades)).size(theme::text_sm()),
                            text(format!("Win Rate: {:.0}%", bt.win_rate_pct)).size(theme::text_sm()),
                            text(format!("Max DD: {:.1}%", bt.max_drawdown_pct)).size(theme::text_sm()),
                            text(format!("Final: ${:.0}", bt.final_capital)).size(theme::text_sm()),
                        ]
                        .spacing(12),
                        text(format!(
                            "Astro Signal Accuracy (30d): {:.1}%",
                            bt.signal_accuracy_pct
                        ))
                        .size(theme::text_base())
                        .color(acc_color),
                    ]
                    .spacing(4);

                    let trade_rows: Vec<Element<Message>> = bt
                        .trades
                        .iter()
                        .rev()
                        .take(10)
                        .flat_map(|t| {
                            let color = if t.return_pct > 0.0 {
                                theme::ZONE_OPTIMAL
                            } else {
                                theme::ZONE_MISALIGNED
                            };
                            let mut items: Vec<Element<Message>> = vec![
                                text(format!(
                                    "  {} @ ${:.2}  ->  {} @ ${:.2}  ({:+.1}%)",
                                    t.buy_date, t.buy_price, t.sell_date, t.sell_price, t.return_pct
                                ))
                                .size(theme::text_sm())
                                .color(color)
                                .into()
                            ];
                            // v11.0: Show correlated real-world events
                            for ev in &t.events {
                                items.push(
                                    text(format!("    \u{25AB} {ev}"))
                                        .size(theme::text_xs())
                                        .into()
                                );
                            }
                            items
                        })
                        .collect();

                    column![
                        text(format!(
                            "Backtest: {} ({} days)",
                            bt.ticker, bt.days_tested
                        ))
                        .size(theme::text_md()),
                        row![config_row, clear_btn].spacing(8).align_y(Alignment::Center),
                        rule::horizontal(1),
                        metrics,
                        rule::horizontal(1),
                        text("Recent Trades (last 10)").size(theme::text_sm()),
                        Column::with_children(trade_rows).spacing(1),
                    ]
                    .spacing(6)
                    .into()
                }
            } else {
                column![
                    text("Astro Backtest").font(font::DISPLAY).size(theme::text_md()),
                    config_row,
                    text("Press 'Run Backtest' to test the astro signal for this ticker.")
                        .size(theme::text_sm()),
                ]
                .spacing(6)
                .into()
            }
        };

        // ── Strategy Builder ────────────────────────────────
        let strategy_section: Element<'_, Message> = {
            let buy_logic_btn = button(
                text(self.strategy.buy_logic.label()).size(theme::text_sm()),
            )
            .on_press(Message::StrategyToggleBuyLogic);

            let buy_conds: Vec<Element<Message>> = self
                .strategy
                .buy_conditions
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    row![
                        text(c.label()).size(theme::text_sm()),
                        button(text("✕").size(theme::text_sm()))
                            .on_press(Message::StrategyRemoveBuyCond(i)),
                    ]
                    .spacing(4)
                    .into()
                })
                .collect();

            let sell_logic_btn = button(
                text(self.strategy.sell_logic.label()).size(theme::text_sm()),
            )
            .on_press(Message::StrategyToggleSellLogic);

            let sell_conds: Vec<Element<Message>> = self
                .strategy
                .sell_conditions
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    row![
                        text(c.label()).size(theme::text_sm()),
                        button(text("✕").size(theme::text_sm()))
                            .on_press(Message::StrategyRemoveSellCond(i)),
                    ]
                    .spacing(4)
                    .into()
                })
                .collect();

            let buy_add = row![
                button(text("+Astro>70").size(theme::text_sm()))
                    .on_press(Message::StrategyAddBuyCond(Condition::AstroAbove(70.0))),
                button(text("+RSI<70").size(theme::text_sm()))
                    .on_press(Message::StrategyAddBuyCond(Condition::RsiBelow(70.0))),
                button(text("+Astro>80").size(theme::text_sm()))
                    .on_press(Message::StrategyAddBuyCond(Condition::AstroAbove(80.0))),
                button(text("+P>SMA50").size(theme::text_sm()))
                    .on_press(Message::StrategyAddBuyCond(Condition::PriceAboveSma50)),
            ]
            .spacing(4);

            let sell_add = row![
                button(text("+Astro<30").size(theme::text_sm()))
                    .on_press(Message::StrategyAddSellCond(Condition::AstroBelow(30.0))),
                button(text("+RSI>80").size(theme::text_sm()))
                    .on_press(Message::StrategyAddSellCond(Condition::RsiAbove(80.0))),
                button(text("+Astro<20").size(theme::text_sm()))
                    .on_press(Message::StrategyAddSellCond(Condition::AstroBelow(20.0))),
                button(text("+P<SMA50").size(theme::text_sm()))
                    .on_press(Message::StrategyAddSellCond(Condition::PriceBelowSma50)),
            ]
            .spacing(4);

            let mut strat_col = column![
                text("Strategy Builder").font(font::DISPLAY).size(theme::text_md()),
                row![text("BUY when").size(theme::text_sm()), buy_logic_btn]
                    .spacing(6)
                    .align_y(Alignment::Center),
                Column::with_children(buy_conds).spacing(2),
                buy_add,
                row![text("SELL when").size(theme::text_sm()), sell_logic_btn]
                    .spacing(6)
                    .align_y(Alignment::Center),
                Column::with_children(sell_conds).spacing(2),
                sell_add,
                button(text("Run Strategy Backtest").size(theme::text_sm()))
                    .on_press(Message::RunStrategy),
            ]
            .spacing(6);

            if let Some(ref sr) = self.strategy_result {
                strat_col = strat_col.push(rule::horizontal(1));
                if let Some(ref msg) = sr.insufficient_data {
                    strat_col = strat_col.push(
                        text(msg.as_str())
                            .size(theme::text_base())
                            .color(theme::ZONE_UNFAVORABLE),
                    );
                } else {
                    let color = if sr.total_return_pct > sr.buy_hold_return_pct {
                        theme::ZONE_OPTIMAL
                    } else {
                        theme::ZONE_MISALIGNED
                    };
                    strat_col = strat_col.push(
                        row![
                            text(format!("Strategy: {:.1}%", sr.total_return_pct))
                                .size(theme::text_base())
                                .color(color),
                            text(format!("vs B&H: {:.1}%", sr.buy_hold_return_pct))
                                .size(theme::text_base()),
                            text(format!("Trades: {}", sr.num_trades)).size(theme::text_sm()),
                            text(format!("Win: {:.0}%", sr.win_rate_pct)).size(theme::text_sm()),
                        ]
                        .spacing(12),
                    );
                }
            }

            strat_col.into()
        };

        // ── Astro Calendar ──────────────────────────────────
        let calendar_section: Element<'_, Message> = {
            let nav = row![
                button(text("< Prev").size(theme::text_sm())).on_press(Message::CalendarPrevMonth),
                text("Astro Calendar").font(font::DISPLAY).size(theme::text_md()),
                button(text("Next >").size(theme::text_sm())).on_press(Message::CalendarNextMonth),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            let legend = row![
                text("■")
                    .size(theme::text_sm())
                    .color(theme::ZONE_OPTIMAL),
                text("Favorable (>50)").size(theme::text_xs()),
                text("■")
                    .size(theme::text_sm())
                    .color(theme::ZONE_NEUTRAL),
                text("Neutral (~50)").size(theme::text_xs()),
                text("■")
                    .size(theme::text_sm())
                    .color(theme::ZONE_MISALIGNED),
                text("Unfavorable (<50)").size(theme::text_xs()),
            ]
            .spacing(6)
            .align_y(Alignment::Center);

            // v11.6.D — render 3 month grids stacked (calendar_month + 0/1/2)
            // so the user sees the full quarter at once. Prev/Next steps the
            // window in 3-month increments.
            let mut grids: Vec<Element<'_, Message>> = Vec::new();
            for offset in 0..3 {
                let mut m = self.calendar_month as i32 + offset;
                let mut y = self.calendar_year;
                while m > 12 { m -= 12; y += 1; }
                grids.push(
                    Canvas::new(AstroCalendar {
                        year: y,
                        month: m as u32,
                        days: self.calendar_days.clone(),
                    })
                    .width(Length::Fill)
                    .height(Length::Fixed(170.0))
                    .into(),
                );
            }
            column![
                nav,
                Column::with_children(grids).spacing(8),
                legend,
            ]
            .spacing(6)
            .into()
        };

        // ── Forecast Timeline (v11.0) ─────────────────────
        let forecast_section: Element<'_, Message> = if self.forecast.is_empty() {
            text("Computing forecast...").size(theme::text_sm()).into()
        } else {
            let mut windows: Vec<String> = Vec::new();
            let mut i = 0;
            while i < self.forecast.len() && windows.len() < 6 {
                let day = &self.forecast[i];
                if day.score > 60.0 || day.score < 40.0 {
                    let zone = if day.score > 60.0 { "Favorable" } else { "Unfavorable" };
                    let start = day.date;
                    let mut end = start;
                    while i + 1 < self.forecast.len() {
                        let next = &self.forecast[i + 1];
                        if (day.score > 60.0 && next.score > 60.0) || (day.score < 40.0 && next.score < 40.0) {
                            end = next.date;
                            i += 1;
                        } else { break; }
                    }
                    let hint = day.key_aspect.as_deref().unwrap_or("");
                    if start == end {
                        windows.push(format!("{zone}: {start}  {hint}"));
                    } else {
                        windows.push(format!("{zone}: {start} \u{2192} {end}  {hint}"));
                    }
                }
                i += 1;
            }
            let window_items: Vec<Element<Message>> = windows.iter().map(|w| {
                let color = if w.starts_with("Favorable") { theme::ZONE_OPTIMAL } else { theme::ZONE_MISALIGNED };
                text(format!("  \u{2022} {w}")).size(theme::text_sm()).color(color).into()
            }).collect();
            let key_aspects: Vec<Element<Message>> = self.forecast.iter()
                .take(30)
                .filter_map(|d| d.key_aspect.as_ref().map(|a| (d.date, a)))
                .take(5)
                .map(|(date, aspect)| text(format!("  {date}: {aspect}")).size(theme::text_xs()).into())
                .collect();
            let mut col = column![
                text(format!("90-Day Forecast: {}", self.selected_ticker))
                    .font(font::DISPLAY).size(theme::text_md()),
                rule::horizontal(1),
            ].spacing(4);
            if window_items.is_empty() {
                col = col.push(text("  No strong signals in next 90 days.").size(theme::text_sm()));
            } else {
                col = col.push(Column::with_children(window_items).spacing(2));
            }
            if !key_aspects.is_empty() {
                col = col.push(text("Key Aspects (30d):").size(theme::text_sm()));
                col = col.push(Column::with_children(key_aspects).spacing(1));
            }
            col.into()
        };

        // ── Final assembly ─────────────────────────────────
        // v11.3 layout: Natal → row![Calendar | Forecast] → Horoscope → Backtest → Strategy
        let calendar_col = column![
            icon_eyebrow(icons::CALENDAR, "ASTRO CALENDAR"),
            container(calendar_section).padding([10, 14]),
        ]
        .spacing(theme::SPACE_XS)
        .width(Length::FillPortion(1));

        let forecast_col = column![
            icon_eyebrow(icons::GRAPH_UP, "FORECAST"),
            container(forecast_section).padding([10, 14]),
        ]
        .spacing(theme::SPACE_XS)
        .width(Length::FillPortion(1));

        // ── Wave 9.5.3 + 9.5.4 — Lifecycle section (Solar Return +
        // upcoming returns + progressed Sun). Builds only when IPO + ASC
        // are both available; otherwise renders a compact placeholder.
        let lifecycle_section: Element<Message> = self.build_lifecycle_section();

        column![
            icon_eyebrow(icons::GLOBE, "NATAL CHART"),
            astrology_section,
            section_rule(),
            icon_eyebrow(icons::CLOCK, "LIFECYCLE"),
            container(lifecycle_section).padding([10, 14]),
            section_rule(),
            row![calendar_col, forecast_col]
                .spacing(theme::SPACE_MD)
                .align_y(Alignment::Start),
            section_rule(),
            icon_eyebrow(icons::MOON_STARS, "HOROSCOPE READING"),
            container(horoscope_section).padding([10, 14]),
            section_rule(),
            icon_eyebrow(icons::LIGHTNING, "BACKTEST"),
            container(backtest_section).padding([10, 14]),
            section_rule(),
            container(strategy_section).padding([10, 14]),
        ]
        .spacing(theme::SPACE_SM)
        .into()
    }

    /// Wave 9.5.3 + 9.5.4 — Build the Lifecycle section.
    /// Combines current-year Solar Return summary, upcoming planetary
    /// returns (Saturn / Jupiter / Mars), and progressed Sun position.
    fn build_lifecycle_section(&self) -> Element<'_, Message> {
        use pursuit_week4_automation::astrology::ephemeris::Planet;
        let p = theme::palette();

        let (Some(ipo), Some(_angles)) = (self.natal_ipo_date, self.natal_angles.as_ref()) else {
            return text(
                "Lifecycle data requires natal IPO date + Ascendant. \
                 Run the scraper to populate company_metadata + natal_angles."
            ).size(theme::text_sm()).into();
        };

        // Build a NatalChart from stored positions.
        let ticker = self.selected_ticker.clone();
        let natal = pursuit_week4_automation::astrology::natal::NatalChart::compute(&ticker, ipo);

        let today = chrono::Local::now().date_naive();
        let target_year = chrono::Datelike::year(&today);

        // Solar Return for current calendar year.
        let sr_line: String = match pursuit_week4_automation::astrology::solar_return::compute_solar_return(&natal, target_year) {
            Ok(sr) => pursuit_week4_automation::astrology::solar_return::summary_line(&sr),
            Err(_) => "Solar Return unavailable.".to_string(),
        };

        // Upcoming returns — first one after `today` for each major.
        let format_return = |label: &str, planet: Planet| -> String {
            match pursuit_week4_automation::astrology::returns::next_return(&natal, planet, today, 60) {
                Ok(Some(ev)) => {
                    let days = (ev.return_date - today).num_days();
                    let when = if days < 365 {
                        format!("in {} days", days)
                    } else {
                        let years = days / 365;
                        let months = (days % 365) / 30;
                        format!("in {}y {}mo", years, months)
                    };
                    format!("Next {label}: {} ({when})", ev.return_date)
                }
                _ => format!("Next {label}: not in 60y window"),
            }
        };
        let saturn_line = format_return("Saturn return", Planet::Saturn);
        let jupiter_line = format_return("Jupiter return", Planet::Jupiter);
        let mars_line = format_return("Mars return", Planet::Mars);

        // Progressed Sun.
        let prog_line: String = match pursuit_week4_automation::astrology::progressions::compute_progressed_chart(&natal, today) {
            Ok(prog) => pursuit_week4_automation::astrology::progressions::summary_line(&prog),
            Err(_) => "Progressed chart unavailable.".to_string(),
        };

        let line_style = move |s: String| -> Element<'_, Message> {
            text(s).size(theme::text_sm())
                .color(Color { r: 0.95, g: 0.90, b: 0.80, a: 1.0 })
                .into()
        };

        column![
            text("Current Solar Return").size(theme::text_xs()).color(Color { a: 0.65, ..p.gold }),
            line_style(sr_line),
            iced::widget::Space::new().height(Length::Fixed(6.0)),
            text("Upcoming returns").size(theme::text_xs()).color(Color { a: 0.65, ..p.gold }),
            line_style(saturn_line),
            line_style(jupiter_line),
            line_style(mars_line),
            iced::widget::Space::new().height(Length::Fixed(6.0)),
            text("Progressed Sun").size(theme::text_xs()).color(Color { a: 0.65, ..p.gold }),
            line_style(prog_line),
        ]
        .spacing(2)
        .into()
    }
}
