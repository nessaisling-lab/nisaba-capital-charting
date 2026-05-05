use iced::widget::{button, column, container, row, rule, text, text_input, Column, Row};
use iced::{Alignment, Element, Length};

use crate::agents::{AgentMode, AgentPersona, AgentVerdict};
use crate::font;
use crate::helpers::{format_market_value_i64, format_shares};

use crate::state::{Dashboard, Message};
use crate::theme;
use super::shared::{explain, eyebrow, gutter_scroll, section_rule};

impl Dashboard {
    pub(crate) fn view_fundamentals(&self) -> Element<'_, Message> {
        // ── Fundamental metrics grid ────────────────────────
        let fundamentals_section: Element<'_, Message> = if let Some(ref f) = self.fundamentals {
            let fr = |v: Option<f64>| -> String {
                v.map(|x| format!("{x:.2}"))
                    .unwrap_or_else(|| "---".to_string())
            };
            let fm = |v: Option<i64>| -> String {
                v.map(format_market_value_i64)
                    .unwrap_or_else(|| "---".to_string())
            };
            let fp = |v: Option<f64>| -> String {
                v.map(|x| format!("{:.1}%", x * 100.0))
                    .unwrap_or_else(|| "---".to_string())
            };

            let val_col: Column<Message> = column![
                text("Valuation").font(font::DISPLAY).size(theme::text_md()),
                row![text("Market Cap").size(theme::text_sm()).width(Length::FillPortion(2)), text(fm(f.market_cap)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("P/E Ratio").size(theme::text_sm()).width(Length::FillPortion(2)), text(fr(f.pe_ratio)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("P/B Ratio").size(theme::text_sm()).width(Length::FillPortion(2)), text(fr(f.pb_ratio)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("P/S Ratio").size(theme::text_sm()).width(Length::FillPortion(2)), text(fr(f.ps_ratio)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("EV/EBITDA").size(theme::text_sm()).width(Length::FillPortion(2)), text(fr(f.ev_ebitda)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("PEG Ratio").size(theme::text_sm()).width(Length::FillPortion(2)), text(fr(f.peg_ratio)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("P/FCF").size(theme::text_sm()).width(Length::FillPortion(2)), text(fr(f.price_to_fcf)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("EPS").size(theme::text_sm()).width(Length::FillPortion(2)), text(fr(f.eps)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("Div Yield").size(theme::text_sm()).width(Length::FillPortion(2)), text(fp(f.dividend_yield)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
            ].spacing(4);

            let prof_col: Column<Message> = column![
                text("Profitability & Health").font(font::DISPLAY).size(theme::text_md()),
                row![text("ROE").size(theme::text_sm()).width(Length::FillPortion(2)), text(fp(f.roe)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("ROA").size(theme::text_sm()).width(Length::FillPortion(2)), text(fp(f.roa)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("Net Margin").size(theme::text_sm()).width(Length::FillPortion(2)), text(fp(f.net_margin)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("Op Margin").size(theme::text_sm()).width(Length::FillPortion(2)), text(fp(f.operating_margin)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("Debt/Equity").size(theme::text_sm()).width(Length::FillPortion(2)), text(fr(f.debt_equity)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("Current Ratio").size(theme::text_sm()).width(Length::FillPortion(2)), text(fr(f.current_ratio)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("Revenue").size(theme::text_sm()).width(Length::FillPortion(2)), text(fm(f.revenue)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("Net Income").size(theme::text_sm()).width(Length::FillPortion(2)), text(fm(f.net_income)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
                row![text("FCF").size(theme::text_sm()).width(Length::FillPortion(2)), text(fm(f.fcf)).size(theme::text_sm()).width(Length::FillPortion(3))].spacing(4),
            ].spacing(4);

            let grid = row![val_col, prof_col].spacing(20);
            let header_text = format!("Fundamentals — {} (as of {})", f.ticker, f.fetch_date);
            column![text(header_text).font(font::DISPLAY).size(theme::text_lg()), grid,]
                .spacing(8)
                .into()
        } else {
            column![
                text("Fundamental Metrics").font(font::DISPLAY).size(theme::text_lg()),
                text("No fundamental data yet. Run the scraper with an FMP API key to fetch valuation ratios, profitability metrics, and balance sheet data.")
                    .size(theme::text_sm()),
                button(
                    text(format!("Fetch {} data now", self.selected_ticker)).size(theme::text_sm())
                ).on_press(Message::FetchThisTicker),
                text("Source: Financial Modeling Prep /v3/key-metrics-ttm + /v3/ratios-ttm")
                    .size(theme::text_xs()),
            ]
            .spacing(6)
            .into()
        };

        // ── DCF Calculator ──────────────────────────────────
        let dcf_section: Element<'_, Message> = {
            let input_row = row![
                column![
                    text("Growth %").size(theme::text_xs()),
                    text_input("10", &self.dcf_growth_rate)
                        .on_input(Message::DcfGrowthRateInput)
                        .on_submit(Message::DcfCompute)
                        .size(theme::text_sm())
                        .width(Length::Fixed(60.0)),
                ].spacing(2),
                column![
                    text("Years").size(theme::text_xs()),
                    text_input("5", &self.dcf_growth_years)
                        .on_input(Message::DcfGrowthYearsInput)
                        .on_submit(Message::DcfCompute)
                        .size(theme::text_sm())
                        .width(Length::Fixed(50.0)),
                ].spacing(2),
                column![
                    text("Terminal %").size(theme::text_xs()),
                    text_input("2.5", &self.dcf_terminal_growth)
                        .on_input(Message::DcfTerminalGrowthInput)
                        .on_submit(Message::DcfCompute)
                        .size(theme::text_sm())
                        .width(Length::Fixed(60.0)),
                ].spacing(2),
                column![
                    text("WACC %").size(theme::text_xs()),
                    text_input("10", &self.dcf_discount_rate)
                        .on_input(Message::DcfDiscountRateInput)
                        .on_submit(Message::DcfCompute)
                        .size(theme::text_sm())
                        .width(Length::Fixed(60.0)),
                ].spacing(2),
                button(text("Compute").size(theme::text_sm()))
                    .on_press(Message::DcfCompute),
            ].spacing(8).align_y(Alignment::End);

            if let Some(ref dcf) = self.dcf_result {
                let margin_color = if dcf.margin_of_safety_pct > 0.0 { theme::ZONE_OPTIMAL } else { theme::ZONE_MISALIGNED };
                let margin_label = if dcf.margin_of_safety_pct > 0.0 { "UNDERVALUED" } else { "OVERVALUED" };
                let result_row = row![
                    column![
                        text(format!("Intrinsic Value: ${:.2}", dcf.intrinsic_per_share)).font(font::INTER).size(theme::text_md()),
                        text(format!("Enterprise Value: {}", format_market_value_i64(dcf.enterprise_value as i64))).font(font::INTER).size(theme::text_sm()),
                    ].spacing(2),
                    column![
                        text(format!("Margin of Safety: {:.1}%", dcf.margin_of_safety_pct)).font(font::INTER).size(theme::text_md()).color(margin_color),
                        text(margin_label).size(theme::text_sm()).color(margin_color),
                    ].spacing(2),
                ].spacing(20);
                column![
                    text("DCF Intrinsic Value Calculator").font(font::DISPLAY).size(theme::text_md()),
                    input_row, result_row,
                ].spacing(6).into()
            } else {
                column![
                    text("DCF Intrinsic Value Calculator").font(font::DISPLAY).size(theme::text_md()),
                    input_row,
                    text("Requires FCF + shares outstanding data. Run scraper with FMP key.").size(theme::text_xs()),
                ].spacing(6).into()
            }
        };

        // ── Options Greeks Calculator ────────────────────────
        let greeks_section: Element<'_, Message> = {
            let type_label = if self.greeks_is_call { "Call" } else { "Put" };
            let input_row = row![
                column![
                    text("Spot $").size(theme::text_xs()),
                    text_input("auto", &self.greeks_spot)
                        .on_input(Message::GreeksSpotInput)
                        .on_submit(Message::GreeksCompute)
                        .size(theme::text_sm())
                        .width(Length::Fixed(70.0)),
                ].spacing(2),
                column![
                    text("Strike $").size(theme::text_xs()),
                    text_input("100", &self.greeks_strike)
                        .on_input(Message::GreeksStrikeInput)
                        .on_submit(Message::GreeksCompute)
                        .size(theme::text_sm())
                        .width(Length::Fixed(70.0)),
                ].spacing(2),
                column![
                    text("Days").size(theme::text_xs()),
                    text_input("30", &self.greeks_expiry_days)
                        .on_input(Message::GreeksExpiryInput)
                        .on_submit(Message::GreeksCompute)
                        .size(theme::text_sm())
                        .width(Length::Fixed(50.0)),
                ].spacing(2),
                column![
                    text("Rate %").size(theme::text_xs()),
                    text_input("4.5", &self.greeks_rate)
                        .on_input(Message::GreeksRateInput)
                        .on_submit(Message::GreeksCompute)
                        .size(theme::text_sm())
                        .width(Length::Fixed(50.0)),
                ].spacing(2),
                column![
                    text("Vol %").size(theme::text_xs()),
                    text_input("25", &self.greeks_vol)
                        .on_input(Message::GreeksVolInput)
                        .on_submit(Message::GreeksCompute)
                        .size(theme::text_sm())
                        .width(Length::Fixed(50.0)),
                ].spacing(2),
                column![
                    text("Type").size(theme::text_xs()),
                    button(text(type_label).size(theme::text_sm()))
                        .on_press(Message::GreeksToggleType),
                ].spacing(2),
                column![
                    text(" ").size(theme::text_xs()),
                    button(text("Compute").size(theme::text_sm()))
                        .on_press(Message::GreeksCompute),
                ].spacing(2),
            ].spacing(6).align_y(Alignment::End);

            let iv_row = row![
                column![
                    text("Mkt Price $").size(theme::text_xs()),
                    text_input("0.00", &self.greeks_market_price)
                        .on_input(Message::GreeksMarketPriceInput)
                        .on_submit(Message::GreeksSolveIV)
                        .size(theme::text_sm())
                        .width(Length::Fixed(80.0)),
                ].spacing(2),
                column![
                    text(" ").size(theme::text_xs()),
                    button(text("Solve IV").size(theme::text_sm()))
                        .on_press(Message::GreeksSolveIV),
                ].spacing(2),
                {
                    let iv_label: Element<'_, Message> = if let Some(iv) = self.greeks_iv {
                        text(format!("IV: {:.1}%", iv * 100.0)).size(theme::text_md()).color(theme::ZONE_OPTIMAL).into()
                    } else {
                        text("").size(theme::text_xs()).into()
                    };
                    iv_label
                },
            ].spacing(6).align_y(Alignment::Center);

            if let Some(ref g) = self.greeks_result {
                let delta_color = if g.delta.abs() > 0.5 { theme::ZONE_OPTIMAL } else { theme::ZONE_NEUTRAL };
                let results = row![
                    column![
                        text(format!("Price: ${:.4}", g.price)).font(font::INTER).size(theme::text_md()),
                        text(format!("Delta: {:.4}", g.delta)).font(font::INTER).size(theme::text_sm()).color(delta_color),
                        text(format!("Gamma: {:.4}", g.gamma)).font(font::INTER).size(theme::text_sm()),
                    ].spacing(2),
                    column![
                        text(format!("Theta: {:.4}/day", g.theta)).font(font::INTER).size(theme::text_sm()).color(theme::ZONE_MISALIGNED),
                        text(format!("Vega:  {:.4}/1%", g.vega)).font(font::INTER).size(theme::text_sm()),
                        text(format!("Rho:   {:.4}/1%", g.rho)).font(font::INTER).size(theme::text_sm()),
                    ].spacing(2),
                ].spacing(20);
                column![
                    text("Options Greeks (Black-Scholes)").font(font::DISPLAY).size(theme::text_md()),
                    input_row, iv_row, results,
                ].spacing(6).into()
            } else {
                column![
                    text("Options Greeks (Black-Scholes)").font(font::DISPLAY).size(theme::text_md()),
                    input_row, iv_row,
                    text("Enter strike price and click Compute. Spot auto-fills from current price.").size(theme::text_xs()),
                ].spacing(6).into()
            }
        };

        // ── Agent Mode Toggle ────────────────────────────────
        let template_label = if self.agent_mode == AgentMode::Template { "[Template]" } else { "Template" };
        let llm_label = if self.agent_mode == AgentMode::Llm { "[LLM]" } else { "LLM" };
        let mode_toggle = row![
            text("Analysis Mode:").size(theme::text_sm()),
            button(text(template_label).size(theme::text_sm()))
                .on_press(Message::SetAgentMode(AgentMode::Template)),
            button(text(llm_label).size(theme::text_sm()))
                .on_press(Message::SetAgentMode(AgentMode::Llm)),
        ].spacing(6).align_y(Alignment::Center);

        // ── Agent Personas ──────────────────────────────────
        let agent_buttons: Row<Message> = AgentPersona::all().iter().fold(
            row![text("Ask the Council:").size(theme::text_base())]
                .spacing(6)
                .align_y(Alignment::Center),
            |r, &persona| {
                let label = if self.active_agent == Some(persona) {
                    format!("[{}]", persona.short_name())
                } else {
                    persona.short_name().to_string()
                };
                r.push(
                    button(text(label).size(theme::text_base()))
                        .on_press(Message::AgentSelected(persona)),
                )
            },
        );

        let agent_section: Element<'_, Message> = if self.agent_loading {
            let persona_name = self.active_agent.map(|p| p.name()).unwrap_or("Agent");
            column![
                text(format!("Consulting the council... {} is thinking...", persona_name)).size(theme::text_base()),
            ].spacing(6).into()
        } else if let Some(ref analysis) = self.agent_analysis {
            let verdict_color = match analysis.verdict {
                AgentVerdict::StrongBuy | AgentVerdict::Buy => theme::ZONE_OPTIMAL,
                AgentVerdict::Hold | AgentVerdict::InsufficientData => theme::ZONE_NEUTRAL,
                AgentVerdict::Sell | AgentVerdict::StrongSell => theme::ZONE_MISALIGNED,
            };
            let mode_badge = match self.agent_mode {
                AgentMode::Llm => " (LLM)",
                AgentMode::Template => " (Template)",
            };
            let metrics_rows: Vec<Element<Message>> = analysis.key_metrics.iter().map(|(metric, value, assessment)| {
                row![
                    text(metric.clone()).size(theme::text_sm()).width(Length::Fixed(110.0)),
                    text(value.clone()).size(theme::text_sm()).width(Length::Fixed(80.0)),
                    text(assessment.clone()).size(theme::text_xs()).width(Length::Fill),
                ].spacing(8).into()
            }).collect();
            // v11.5.C6 — verdict + persona tooltip explains reasoning style
            let persona_explanation = match analysis.persona {
                AgentPersona::Buffett => "Warren Buffett's lens: durable competitive moats, owner earnings, free cash flow, capital allocation track record. Skeptical of leverage and margin compression.",
                AgentPersona::Graham => "Benjamin Graham's lens: margin of safety, net-net working capital, P/B + P/E together, cash + securities vs market cap. Buys obvious bargains.",
                AgentPersona::Lynch => "Peter Lynch's lens: PEG ratio under 1.0, growth at reasonable price, earnings momentum, 'know what you own'. Favors stories you can summarize in two sentences.",
                AgentPersona::Munger => "Charlie Munger's lens: quality first, durable franchise, mental models, inversion. 'A great business at a fair price beats a fair business at a great price.'",
            };
            let mut content = column![
                explain(
                    text(format!("{}{} — {}", analysis.persona.name(), mode_badge, analysis.persona.philosophy())).size(theme::text_sm()),
                    persona_explanation,
                ),
                rule::horizontal(1),
                text(analysis.headline.clone()).size(theme::text_base()),
                explain(
                    text(format!("Verdict: {}", analysis.verdict.label())).size(theme::text_md()).color(verdict_color),
                    "Verdict synthesizes fundamental + astrology + macro signals through this persona's framework. StrongBuy/Buy = conviction; Hold = mixed; Sell/StrongSell = thesis breaks.",
                ),
                text(analysis.analysis.clone()).size(theme::text_sm()),
                rule::horizontal(1),
                Column::with_children(metrics_rows).spacing(3),
                rule::horizontal(1),
                text(format!("Closing thought: {}", analysis.astro_take)).size(theme::text_xs()),
            ].spacing(6);
            if let Some(ref err) = self.agent_llm_error {
                content = content.push(text(format!("LLM error: {}", err)).size(theme::text_xs()).color(theme::ZONE_MISALIGNED));
            }
            content.into()
        } else {
            column![
                text("Select an agent to get their investment analysis:").size(theme::text_sm()),
                text("Buffett — moat, FCF, owner earnings").size(theme::text_xs()),
                text("Graham — margin of safety, deep value").size(theme::text_xs()),
                text("Lynch — PEG ratio, know what you own").size(theme::text_xs()),
                text("Munger — quality, mental models, durability").size(theme::text_xs()),
            ].spacing(3).into()
        };

        // ── Comparative Analysis ────────────────────────────
        let compare_section = {
            let compare_input_row = row![
                text("Compare:").size(theme::text_base()),
                text_input("Add ticker (max 4)…", &self.compare_input)
                    .on_input(Message::CompareInput)
                    .on_submit(Message::CompareAdd)
                    .size(theme::text_sm())
                    .width(Length::Fixed(140.0)),
                button(text("Add").size(theme::text_sm())).on_press(Message::CompareAdd),
            ].spacing(6).align_y(Alignment::Center);

            let chip_row: Row<Message> = self.compare_tickers.iter().fold(
                row![].spacing(6),
                |r, t| r.push(
                    button(text(format!("{t} ✕")).size(theme::text_sm()))
                        .on_press(Message::CompareRemove(t.clone()))
                ),
            );

            // Sector peer suggestions
            let peer_row: Element<'_, Message> = if self.sector_peers.is_empty() || self.compare_tickers.len() >= 4 {
                text("").into()
            } else {
                let peers: Row<Message> = self.sector_peers.iter()
                    .filter(|p| !self.compare_tickers.contains(p) && **p != self.selected_ticker)
                    .take(4)
                    .fold(
                        row![text("Peers:").size(theme::text_xs())].spacing(4).align_y(Alignment::Center),
                        |r, peer| {
                            r.push(
                                button(text(format!("+{peer}")).size(theme::text_xs()))
                                    .on_press(Message::CompareAddDirect(peer.clone()))
                            )
                        },
                    );
                peers.into()
            };

            let compare_table: Element<'_, Message> = if self.compare_data.is_empty() {
                text("Add tickers above to compare side by side.").size(theme::text_sm()).into()
            } else {
                let hdr = row![
                    text("Metric").size(theme::text_sm()).width(Length::Fixed(100.0)),
                ].spacing(8);
                let hdr = self.compare_data.iter().fold(hdr, |r, d| {
                    r.push(text(d.ticker.clone()).size(theme::text_sm()).width(Length::FillPortion(1)))
                });

                let metric_row = |label: &str, f: &dyn Fn(&crate::db::CompareRow) -> String| -> Element<'_, Message> {
                    let r = row![
                        text(label.to_string()).size(theme::text_xs()).width(Length::Fixed(100.0)),
                    ];
                    let r = self.compare_data.iter().fold(r, |r, d| {
                        r.push(text(f(d)).size(theme::text_xs()).width(Length::FillPortion(1)))
                    });
                    r.spacing(8).into()
                };

                let fr = |v: Option<f64>| v.map(|x| format!("{x:.2}")).unwrap_or_else(|| "---".into());
                let fp = |v: Option<f64>| v.map(|x| format!("{:.1}%", x * 100.0)).unwrap_or_else(|| "---".into());
                let fm = |v: Option<i64>| v.map(|x| format_market_value_i64(x)).unwrap_or_else(|| "---".into());

                column![
                    hdr,
                    rule::horizontal(1),
                    metric_row("P/E",         &|d| fr(d.pe_ratio)),
                    metric_row("P/B",         &|d| fr(d.pb_ratio)),
                    metric_row("P/S",         &|d| fr(d.ps_ratio)),
                    metric_row("EV/EBITDA",   &|d| fr(d.ev_ebitda)),
                    metric_row("PEG",         &|d| fr(d.peg_ratio)),
                    metric_row("ROE",         &|d| fp(d.roe)),
                    metric_row("Net Margin",  &|d| fp(d.net_margin)),
                    metric_row("Debt/Equity", &|d| fr(d.debt_equity)),
                    metric_row("FCF",         &|d| fm(d.fcf)),
                    metric_row("Market Cap",  &|d| fm(d.market_cap)),
                    metric_row("Astro Score", &|d| d.astro_score.map(|s| format!("{s:.0}")).unwrap_or_else(|| "---".into())),
                    metric_row("Astro Zone",  &|d| d.astro_label.clone().unwrap_or_else(|| "---".into())),
                ].spacing(3).into()
            };

            column![
                text("Comparative Analysis").font(font::DISPLAY).size(theme::text_md()),
                compare_input_row,
                peer_row,
                chip_row,
                compare_table,
            ].spacing(6)
        };

        // ── Earnings + Price Table ──────────────────────────
        let earnings_section = self.build_earnings_section();

        let price_toggle_label = if self.show_price_table {
            format!("▼ Price History ({} rows) — click to collapse", self.rows.len())
        } else {
            format!("▶ Price History ({} rows) — click to expand", self.rows.len())
        };
        let price_toggle = button(text(price_toggle_label).size(theme::text_sm()))
            .on_press(Message::TogglePriceTable);

        let price_section: Element<'_, Message> = if self.show_price_table {
            let (price_header, data_rows) = self.build_price_table();
            container(
                column![
                    price_toggle,
                    price_header,
                    gutter_scroll(data_rows, 300.0),
                ]
                .spacing(4),
            )
            .into()
        } else {
            container(price_toggle).into()
        };

        column![
            eyebrow("VALUATION"),
            fundamentals_section,
            section_rule(),
            eyebrow("DCF CALCULATOR"),
            dcf_section,
            section_rule(),
            eyebrow("OPTIONS GREEKS"),
            greeks_section,
            section_rule(),
            eyebrow("THE COUNCIL"),
            mode_toggle,
            agent_buttons,
            agent_section,
            section_rule(),
            eyebrow("COMPARATIVE ANALYSIS"),
            compare_section,
            section_rule(),
            eyebrow("EARNINGS"),
            earnings_section,
            section_rule(),
            eyebrow("PRICE HISTORY"),
            price_section,
        ]
        .spacing(theme::SPACE_SM)
        .into()
    }

    /// Build the earnings calendar section.
    pub(crate) fn build_earnings_section(&self) -> Column<'_, Message> {
        if self.earnings.is_empty() {
            column![
                text(format!("{} Earnings", self.selected_ticker)).font(font::DISPLAY).size(theme::text_md()),
                text(format!("No earnings dates found for {}.", self.selected_ticker)).size(theme::text_base()),
                text("The scraper fetches earnings dates from Finnhub.").size(theme::text_sm()),
            ].spacing(4)
        } else {
            let today = chrono::Utc::now().date_naive();
            let hdr = row![
                text("Date").width(Length::Fixed(90.0)),
                text("Ticker").width(Length::Fixed(60.0)),
                text("EPS Est").width(Length::Fixed(80.0)),
                text("EPS Actual").width(Length::Fixed(90.0)),
                text("Rev Est").width(Length::Fill),
            ].spacing(8);
            let items: Vec<Element<Message>> = self.earnings.iter().map(|e| {
                let is_upcoming = e.earnings_date >= today;
                let eps_est = e.eps_estimate.as_ref().map(|v| format!("${v:.2}")).unwrap_or_else(|| "—".into());
                let eps_act = e.eps_actual.as_ref().map(|v| format!("${v:.2}")).unwrap_or_else(|| "—".into());
                let rev_est = e.revenue_estimate.map(format_market_value_i64).unwrap_or_else(|| "—".into());
                let date_str = if is_upcoming { format!(">> {}", e.earnings_date) } else { e.earnings_date.to_string() };
                row![
                    text(date_str).width(Length::Fixed(90.0)),
                    text(&e.ticker).width(Length::Fixed(60.0)),
                    text(eps_est).width(Length::Fixed(80.0)),
                    text(eps_act).width(Length::Fixed(90.0)),
                    text(rev_est).width(Length::Fill),
                ].spacing(8).into()
            }).collect();
            column![
                text("Earnings Calendar").font(font::DISPLAY).size(theme::text_md()),
                hdr,
                gutter_scroll(Column::with_children(items).spacing(4), 130.0),
            ].spacing(4)
        }
    }

    /// Build the OHLCV price table.
    pub(crate) fn build_price_table(&self) -> (Row<'_, Message>, Column<'_, Message>) {
        let price_header = row![
            text("Date").width(Length::FillPortion(2)),
            text("Open").width(Length::FillPortion(2)),
            text("High").width(Length::FillPortion(2)),
            text("Low").width(Length::FillPortion(2)),
            text("Close").width(Length::FillPortion(2)),
            text("Volume").width(Length::FillPortion(3)),
        ].spacing(10);

        let data_rows: Column<Message> = if self.rows.is_empty() {
            column![text(&self.status).size(theme::text_md())]
        } else {
            let price_rows: Vec<Element<Message>> = self.rows.iter().map(|r| {
                row![
                    text(r.date.to_string()).font(font::INTER).width(Length::FillPortion(2)),
                    text(format!("{:.2}", r.open)).font(font::INTER).width(Length::FillPortion(2)),
                    text(format!("{:.2}", r.high)).font(font::INTER).width(Length::FillPortion(2)),
                    text(format!("{:.2}", r.low)).font(font::INTER).width(Length::FillPortion(2)),
                    text(format!("{:.2}", r.close)).font(font::INTER).width(Length::FillPortion(2)),
                    text(format_shares(r.volume)).font(font::INTER).width(Length::FillPortion(3)),
                ].spacing(10).into()
            }).collect();
            Column::with_children(price_rows).spacing(4)
        };

        (price_header, data_rows)
    }
}
