use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use std::sync::Arc;
use std::time::Duration;

pub async fn fetch_all_13f(pool: Arc<sqlx::PgPool>, client: Arc<reqwest::Client>) {
    for (institution_cik, institution_name) in crate::institution_map() {
        tokio::time::sleep(Duration::from_millis(400)).await;
        match fetch_13f_holdings(institution_cik, institution_name, &pool, &client).await {
            Ok(n) => {
                println!("[13F] {institution_name}: {n} new watchlist holdings");
                crate::log_fetch(&pool, "edgar", None, "13f", "ok", None).await;
            }
            Err(e) => {
                eprintln!("[13F] {institution_name} error (skipping): {e:#}");
                crate::log_fetch(&pool, "edgar", None, "13f", "error", Some(&e.to_string())).await;
            }
        }
    }
}

async fn fetch_13f_holdings(
    institution_cik: &str,
    institution_name: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
) -> Result<u64> {
    let url = format!("https://data.sec.gov/submissions/CIK{institution_cik}.json");
    let resp = client.get(&url).send().await
        .context("EDGAR submissions request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("EDGAR returned HTTP {}", resp.status());
    }

    let body: serde_json::Value = resp.json().await
        .context("Failed to parse EDGAR submissions JSON")?;

    let recent       = &body["filings"]["recent"];
    let forms        = recent["form"].as_array().context("No forms array")?;
    let accessions   = recent["accessionNumber"].as_array().context("No accessionNumber array")?;
    let filing_dates = recent["filingDate"].as_array().context("No filingDate array")?;
    let report_dates = recent["reportDate"].as_array().context("No reportDate array")?;

    let idx = forms
        .iter()
        .enumerate()
        .find(|(_, f)| f.as_str() == Some("13F-HR"))
        .map(|(i, _)| i)
        .context("No 13F-HR found for this institution")?;

    let accession = accessions.get(idx).and_then(|v| v.as_str())
        .context("Missing accessionNumber")?;

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM filings WHERE accession_number = $1)",
    )
    .bind(accession)
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if exists { return Ok(0); }

    let filing_date_str = filing_dates.get(idx).and_then(|v| v.as_str()).unwrap_or("");
    let report_date_str = report_dates.get(idx).and_then(|v| v.as_str()).unwrap_or("");
    let filing_date = NaiveDate::parse_from_str(filing_date_str, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    let report_date = NaiveDate::parse_from_str(report_date_str, "%Y-%m-%d")
        .unwrap_or(filing_date);

    let cik_numeric      = institution_cik.trim_start_matches('0');
    let accession_nodash = accession.replace('-', "");

    tokio::time::sleep(Duration::from_millis(200)).await;
    let index_url = format!(
        "https://www.sec.gov/Archives/edgar/data/{cik_numeric}/{accession_nodash}/"
    );
    let index_html = client.get(&index_url).send().await
        .context("Index fetch failed")?
        .text().await
        .context("Failed to read index")?;

    let xml_path  = find_infotable_in_index(&index_html)
        .context("No information table XML found in filing index")?;
    let edgar_url = format!("https://www.sec.gov{xml_path}");

    sqlx::query(
        "INSERT INTO filings \
         (cik, ticker, form_type, filed_date, period_of_report, accession_number, edgar_url) \
         VALUES ($1, NULL, $2, $3, $4, $5, $6) \
         ON CONFLICT (accession_number) DO NOTHING",
    )
    .bind(institution_cik)
    .bind("13F-HR")
    .bind(filing_date)
    .bind(report_date)
    .bind(accession)
    .bind(&edgar_url)
    .execute(pool)
    .await
    .context("Failed to insert 13F filing")?;

    tokio::time::sleep(Duration::from_millis(200)).await;
    let xml_resp = client.get(&edgar_url).send().await
        .context("XML fetch failed")?;

    if !xml_resp.status().is_success() {
        anyhow::bail!("XML returned HTTP {}", xml_resp.status());
    }

    let xml = xml_resp.text().await.context("Failed to read XML")?;

    parse_and_insert_13f(&xml, accession, institution_cik, institution_name, report_date, pool).await
}

async fn parse_and_insert_13f(
    xml: &str,
    accession_number: &str,
    institution_cik: &str,
    institution_name: &str,
    report_period: NaiveDate,
    pool: &sqlx::PgPool,
) -> Result<u64> {
    let opts = roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() };
    let doc  = roxmltree::Document::parse_with_options(xml, opts)
        .context("Failed to parse 13F XML")?;
    let root = doc.root_element();

    let mut inserted = 0u64;

    for entry in root.descendants().filter(|n| n.has_tag_name("infoTable")) {
        let cusip = match entry.descendants()
            .find(|n| n.has_tag_name("cusip"))
            .and_then(|n| n.text())
            .map(|s| s.trim().to_string())
        {
            Some(c) => c,
            None => continue,
        };

        let ticker = match crate::cusip_map().iter().find(|(c, _)| *c == cusip.as_str()) {
            Some((_, t)) => *t,
            None => continue,
        };

        let value_str = entry.descendants()
            .find(|n| n.has_tag_name("value"))
            .and_then(|n| n.text())
            .unwrap_or("0");
        let value_thousands: i64 = value_str.trim().replace(',', "").parse().unwrap_or(0);
        let market_value = rust_decimal::Decimal::from(value_thousands)
            * rust_decimal::Decimal::from(1000);

        let shares_str = entry.descendants()
            .find(|n| n.has_tag_name("sshPrnamt"))
            .and_then(|n| n.text())
            .unwrap_or("0");
        let shares_held: i64 = shares_str.trim().replace(',', "").parse().unwrap_or(0);

        let investment_discretion = entry.descendants()
            .find(|n| n.has_tag_name("investmentDiscretion"))
            .and_then(|n| n.text())
            .map(|s| s.trim().to_string());

        sqlx::query(
            "INSERT INTO institutional_holdings \
             (accession_number, institution_cik, institution_name, report_period, \
              ticker, cusip, shares_held, market_value, investment_discretion) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(accession_number)
        .bind(institution_cik)
        .bind(institution_name)
        .bind(report_period)
        .bind(ticker)
        .bind(&cusip)
        .bind(shares_held)
        .bind(market_value)
        .bind(investment_discretion)
        .execute(pool)
        .await
        .context("Failed to insert institutional holding")?;

        inserted += 1;
    }

    Ok(inserted)
}

fn find_infotable_in_index(html: &str) -> Option<String> {
    let mut fallback: Option<String> = None;
    let mut search = html;
    while let Some(pos) = search.find("href=\"") {
        let rest = &search[pos + 6..];
        if let Some(end) = rest.find('"') {
            let href  = &rest[..end];
            let lower = href.to_lowercase();
            if lower.ends_with(".xml") && !lower.contains("-index") {
                if lower.contains("infotable") || lower.contains("information") {
                    return Some(href.to_string());
                }
                if fallback.is_none() {
                    fallback = Some(href.to_string());
                }
            }
        }
        search = &search[pos + 6..];
    }
    fallback
}
