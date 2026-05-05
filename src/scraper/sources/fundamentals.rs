//! Multi-source fundamentals fallback (Wave 6.A2).
//!
//! Cascade: FMP (primary, in scraper/fundamentals.rs) →
//!          Finnhub `/stock/metric?metric=all` →
//!          Alpha Vantage `OVERVIEW` function.
//!
//! Each fallback returns a normalized `SourcedFundamentals` struct which
//! the caller persists with the appropriate `data_source` tag.

use anyhow::{Context, Result};
use serde::Deserialize;

/// Normalized fundamentals fields covering what `fundamental_metrics` stores.
/// Optional everywhere — different sources omit different fields.
#[derive(Debug, Default, Clone)]
pub struct SourcedFundamentals {
    pub market_cap:         Option<i64>,
    pub pe_ratio:           Option<f64>,
    pub pb_ratio:           Option<f64>,
    pub ps_ratio:           Option<f64>,
    pub ev_ebitda:          Option<f64>,
    pub peg_ratio:          Option<f64>,
    pub price_to_fcf:       Option<f64>,
    pub roe:                Option<f64>,
    pub roa:                Option<f64>,
    pub net_margin:         Option<f64>,
    pub operating_margin:   Option<f64>,
    pub debt_equity:        Option<f64>,
    pub current_ratio:      Option<f64>,
    pub fcf:                Option<i64>,
    pub operating_cf:       Option<i64>,
    pub revenue:            Option<i64>,
    pub net_income:         Option<i64>,
    pub eps:                Option<f64>,
    pub dividend_yield:     Option<f64>,
    pub shares_outstanding: Option<i64>,
}

impl SourcedFundamentals {
    /// True when at least the core valuation triplet is populated.
    pub fn has_useful_data(&self) -> bool {
        self.pe_ratio.is_some() || self.pb_ratio.is_some() || self.market_cap.is_some()
    }
}

/// Try Finnhub then AV when FMP fails. Returns first source with useful data.
pub async fn fetch_fundamentals_fallback(
    ticker: &str,
    client: &reqwest::Client,
    finnhub_key: Option<&str>,
    av_key: &str,
) -> Result<(SourcedFundamentals, &'static str)> {
    if let Some(fh_key) = finnhub_key {
        match fetch_finnhub_metric(ticker, client, fh_key).await {
            Ok(f) if f.has_useful_data() => return Ok((f, "finnhub")),
            Ok(_) => eprintln!("[fundamentals fallback] Finnhub returned empty for {ticker}"),
            Err(e) => eprintln!("[fundamentals fallback] Finnhub failed for {ticker}: {e}"),
        }
    }

    match fetch_av_overview(ticker, client, av_key).await {
        Ok(f) if f.has_useful_data() => return Ok((f, "alpha_vantage")),
        Ok(_) => eprintln!("[fundamentals fallback] AV OVERVIEW returned empty for {ticker}"),
        Err(e) => eprintln!("[fundamentals fallback] AV OVERVIEW failed for {ticker}: {e}"),
    }

    anyhow::bail!("All fundamentals fallback sources exhausted for {ticker}")
}

// ---------------------------------------------------------------------------
// Finnhub /stock/metric?metric=all
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct FinnhubMetricResp {
    metric: Option<FinnhubMetricBag>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FinnhubMetricBag {
    #[serde(rename = "marketCapitalization")]
    market_cap_m:        Option<f64>, // in millions USD
    pe_ttm:              Option<f64>,
    pb:                  Option<f64>,
    ps_ttm:              Option<f64>,
    ev_ebitda_ttm:       Option<f64>,
    peg_ttm:             Option<f64>,
    pfcf_ttm:            Option<f64>,
    roe_ttm:             Option<f64>,
    roa_ttm:             Option<f64>,
    net_margin_ttm:      Option<f64>,
    operating_margin_ttm:Option<f64>,
    #[serde(rename = "totalDebt/totalEquityAnnual")]
    debt_equity:         Option<f64>,
    current_ratio_annual: Option<f64>,
    eps_ttm:              Option<f64>,
    dividend_yield_indicated_annual: Option<f64>,
}

async fn fetch_finnhub_metric(
    ticker: &str,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<SourcedFundamentals> {
    let url = format!(
        "https://finnhub.io/api/v1/stock/metric?symbol={ticker}&metric=all&token={api_key}"
    );
    let resp = client.get(&url).send().await
        .context("Finnhub metric request failed")?;
    if !resp.status().is_success() {
        anyhow::bail!("Finnhub HTTP {}", resp.status());
    }

    let body: FinnhubMetricResp = resp.json().await
        .context("Finnhub metric JSON parse failed")?;
    let m = body.metric.unwrap_or_default();

    Ok(SourcedFundamentals {
        market_cap:       m.market_cap_m.map(|mm| (mm * 1_000_000.0) as i64),
        pe_ratio:         m.pe_ttm,
        pb_ratio:         m.pb,
        ps_ratio:         m.ps_ttm,
        ev_ebitda:        m.ev_ebitda_ttm,
        peg_ratio:        m.peg_ttm,
        price_to_fcf:     m.pfcf_ttm,
        roe:              m.roe_ttm.map(|v| v / 100.0),  // Finnhub returns percentages
        roa:              m.roa_ttm.map(|v| v / 100.0),
        net_margin:       m.net_margin_ttm.map(|v| v / 100.0),
        operating_margin: m.operating_margin_ttm.map(|v| v / 100.0),
        debt_equity:      m.debt_equity,
        current_ratio:    m.current_ratio_annual,
        eps:              m.eps_ttm,
        dividend_yield:   m.dividend_yield_indicated_annual.map(|v| v / 100.0),
        ..Default::default()
    })
}

// ---------------------------------------------------------------------------
// Alpha Vantage OVERVIEW (third-tier fallback)
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AvOverview {
    #[serde(rename = "MarketCapitalization")]
    market_cap:        Option<String>,
    #[serde(rename = "PERatio")]
    pe_ratio:          Option<String>,
    #[serde(rename = "PriceToBookRatio")]
    pb_ratio:          Option<String>,
    #[serde(rename = "PriceToSalesRatioTTM")]
    ps_ratio:          Option<String>,
    #[serde(rename = "EVToEBITDA")]
    ev_ebitda:         Option<String>,
    #[serde(rename = "PEGRatio")]
    peg_ratio:         Option<String>,
    #[serde(rename = "ReturnOnEquityTTM")]
    roe:               Option<String>,
    #[serde(rename = "ReturnOnAssetsTTM")]
    roa:               Option<String>,
    #[serde(rename = "ProfitMargin")]
    net_margin:        Option<String>,
    #[serde(rename = "OperatingMarginTTM")]
    operating_margin:  Option<String>,
    #[serde(rename = "RevenueTTM")]
    revenue:           Option<String>,
    #[serde(rename = "EPS")]
    eps:               Option<String>,
    #[serde(rename = "DividendYield")]
    dividend_yield:    Option<String>,
    #[serde(rename = "SharesOutstanding")]
    shares_outstanding:Option<String>,
}

async fn fetch_av_overview(
    ticker: &str,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<SourcedFundamentals> {
    let url = format!(
        "https://www.alphavantage.co/query?function=OVERVIEW&symbol={ticker}&apikey={api_key}"
    );
    let resp = client.get(&url).send().await
        .context("AV OVERVIEW HTTP request failed")?;
    if !resp.status().is_success() {
        anyhow::bail!("AV OVERVIEW HTTP {}", resp.status());
    }

    let body: serde_json::Value = resp.json().await
        .context("AV OVERVIEW JSON parse failed")?;

    if body.get("Note").is_some() || body.get("Information").is_some() {
        anyhow::bail!("AV OVERVIEW rate-limited for {ticker}");
    }

    let parsed: AvOverview = serde_json::from_value(body)
        .context("AV OVERVIEW deserialize failed")?;

    let parse_f = |s: Option<String>| -> Option<f64> {
        s.and_then(|v| if v == "None" || v == "-" { None } else { v.parse().ok() })
    };
    let parse_i = |s: Option<String>| -> Option<i64> {
        s.and_then(|v| if v == "None" || v == "-" { None } else { v.parse().ok() })
    };

    Ok(SourcedFundamentals {
        market_cap:        parse_i(parsed.market_cap),
        pe_ratio:          parse_f(parsed.pe_ratio),
        pb_ratio:          parse_f(parsed.pb_ratio),
        ps_ratio:          parse_f(parsed.ps_ratio),
        ev_ebitda:         parse_f(parsed.ev_ebitda),
        peg_ratio:         parse_f(parsed.peg_ratio),
        roe:               parse_f(parsed.roe),
        roa:               parse_f(parsed.roa),
        net_margin:        parse_f(parsed.net_margin),
        operating_margin:  parse_f(parsed.operating_margin),
        revenue:           parse_i(parsed.revenue),
        eps:               parse_f(parsed.eps),
        dividend_yield:    parse_f(parsed.dividend_yield),
        shares_outstanding:parse_i(parsed.shares_outstanding),
        ..Default::default()
    })
}
