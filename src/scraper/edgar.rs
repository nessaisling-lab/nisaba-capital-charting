use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Fetch EDGAR Form 4 (insider trades) and 8-K filings for priority + watchlist tickers.
///
/// Processes priority tickers first (astro-ranked), then watchlist tickers, skipping
/// any already processed. Uses the dynamic CIK map from SEC's company_tickers.json
/// instead of the hardcoded CIK_MAP, so any ticker with a SEC filing can be fetched.
pub async fn fetch_all_edgar(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    cik_map: &HashMap<String, u64>,
    priority_tickers: &[String],
) {
    // Build ordered list: priority tickers first, then watchlist, deduped
    let mut seen = std::collections::HashSet::new();
    let mut tickers_to_fetch: Vec<String> = Vec::new();

    for t in priority_tickers {
        let upper = t.to_uppercase();
        if seen.insert(upper.clone()) {
            tickers_to_fetch.push(upper);
        }
    }
    for t in crate::WATCHLIST {
        let upper = t.to_uppercase();
        if seen.insert(upper.clone()) {
            tickers_to_fetch.push(upper);
        }
    }

    let mut fetched = 0usize;
    let mut skipped_no_cik = 0usize;

    for ticker in &tickers_to_fetch {
        let cik_num = match cik_map.get(ticker.as_str()) {
            Some(c) => *c,
            None => {
                // Fall back to hardcoded CIK_MAP for the original 10 tickers
                match crate::CIK_MAP.iter().find(|(t, _)| *t == ticker.as_str()) {
                    Some((_, cik_str)) => cik_str.trim_start_matches('0').parse::<u64>().unwrap_or(0),
                    None => { skipped_no_cik += 1; continue; }
                }
            }
        };

        let cik_padded = format!("{cik_num:010}");

        tokio::time::sleep(Duration::from_millis(300)).await;
        match fetch_edgar_form4(ticker, &cik_padded, &pool, &client).await {
            Ok(n) => {
                println!("[{ticker}] EDGAR Form4: {n} new insider trades");
                crate::log_fetch(&pool, "edgar", Some(ticker), "form4", "ok", None).await;
            }
            Err(e) => {
                eprintln!("[{ticker}] EDGAR Form4 error (skipping): {e:#}");
                crate::log_fetch(&pool, "edgar", Some(ticker), "form4", "error", Some(&e.to_string())).await;
            }
        }

        tokio::time::sleep(Duration::from_millis(300)).await;
        match fetch_edgar_8k(ticker, &cik_padded, &pool, &client).await {
            Ok(n) => {
                println!("[{ticker}] EDGAR 8-K: {n} new filings");
                crate::log_fetch(&pool, "edgar", Some(ticker), "8k", "ok", None).await;
            }
            Err(e) => {
                eprintln!("[{ticker}] EDGAR 8-K error (skipping): {e:#}");
                crate::log_fetch(&pool, "edgar", Some(ticker), "8k", "error", Some(&e.to_string())).await;
            }
        }

        fetched += 1;
    }

    println!("[EDGAR] Fetched filings for {fetched} tickers ({skipped_no_cik} skipped: no CIK found)");
}

async fn fetch_edgar_form4(
    ticker: &str,
    cik: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
) -> Result<u64> {
    let url = format!("https://data.sec.gov/submissions/CIK{cik}.json");
    let resp = client.get(&url).send().await
        .context("EDGAR submissions request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("EDGAR submissions returned HTTP {}", resp.status());
    }

    let body: serde_json::Value = resp.json().await
        .context("Failed to parse EDGAR submissions JSON")?;

    let recent = &body["filings"]["recent"];
    let forms       = recent["form"].as_array().context("No forms array")?;
    let accessions  = recent["accessionNumber"].as_array().context("No accessionNumber array")?;
    let filing_dates = recent["filingDate"].as_array().context("No filingDate array")?;
    let report_dates = recent["reportDate"].as_array().context("No reportDate array")?;

    let form4_indices: Vec<usize> = forms
        .iter()
        .enumerate()
        .filter(|(_, f)| f.as_str() == Some("4"))
        .map(|(i, _)| i)
        .take(5)
        .collect();

    let cik_numeric = cik.trim_start_matches('0');
    let mut total_inserted = 0u64;

    for idx in form4_indices {
        let accession = match accessions.get(idx).and_then(|v| v.as_str()) {
            Some(a) => a,
            None => continue,
        };
        let filing_date_str = filing_dates.get(idx).and_then(|v| v.as_str()).unwrap_or("");
        let report_date_str = report_dates.get(idx).and_then(|v| v.as_str()).unwrap_or("");

        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM filings WHERE accession_number = $1)",
        )
        .bind(accession)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if exists { continue; }

        let filing_date = NaiveDate::parse_from_str(filing_date_str, "%Y-%m-%d")
            .unwrap_or_else(|_| Utc::now().date_naive());
        let report_date = NaiveDate::parse_from_str(report_date_str, "%Y-%m-%d").ok();
        let accession_nodash = accession.replace('-', "");

        tokio::time::sleep(Duration::from_millis(150)).await;
        let index_url = format!(
            "https://www.sec.gov/Archives/edgar/data/{cik_numeric}/{accession_nodash}/"
        );
        let index_html = match client.get(&index_url).send().await {
            Ok(r) => match r.text().await { Ok(t) => t, Err(_) => continue },
            Err(e) => { eprintln!("[{ticker}] Index fetch failed ({accession}): {e}"); continue; }
        };
        let xml_path = match find_xml_in_index(&index_html) {
            Some(p) => p,
            None => { eprintln!("[{ticker}] No XML doc in index for {accession}"); continue; }
        };
        let edgar_url = format!("https://www.sec.gov{xml_path}");

        sqlx::query(
            "INSERT INTO filings \
             (cik, ticker, form_type, filed_date, period_of_report, accession_number, edgar_url) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (accession_number) DO NOTHING",
        )
        .bind(cik)
        .bind(ticker)
        .bind("4")
        .bind(filing_date)
        .bind(report_date)
        .bind(accession)
        .bind(&edgar_url)
        .execute(pool)
        .await
        .context("Failed to insert filing")?;

        tokio::time::sleep(Duration::from_millis(150)).await;
        let xml_resp = match client.get(&edgar_url).send().await {
            Ok(r) => r,
            Err(e) => { eprintln!("[{ticker}] XML fetch failed ({accession}): {e}"); continue; }
        };

        if !xml_resp.status().is_success() {
            eprintln!("[{ticker}] XML returned HTTP {} for {accession}", xml_resp.status());
            continue;
        }

        let xml = match xml_resp.text().await {
            Ok(t) => t,
            Err(e) => { eprintln!("[{ticker}] Failed to read XML body: {e}"); continue; }
        };

        match parse_and_insert_form4(&xml, ticker, accession, pool).await {
            Ok(n) => total_inserted += n,
            Err(e) => eprintln!("[{ticker}] Form 4 parse error ({accession}): {e:#}"),
        }
    }

    Ok(total_inserted)
}

async fn fetch_edgar_8k(
    ticker: &str,
    cik: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
) -> Result<u64> {
    let url = format!("https://data.sec.gov/submissions/CIK{cik}.json");
    let resp = client.get(&url).send().await
        .context("EDGAR submissions request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("EDGAR submissions returned HTTP {}", resp.status());
    }

    let body: serde_json::Value = resp.json().await
        .context("Failed to parse EDGAR submissions JSON")?;

    let recent = &body["filings"]["recent"];
    let forms        = recent["form"].as_array().context("No forms array")?;
    let accessions   = recent["accessionNumber"].as_array().context("No accessionNumber array")?;
    let filing_dates = recent["filingDate"].as_array().context("No filingDate array")?;
    let report_dates = recent["reportDate"].as_array().context("No reportDate array")?;
    let primary_docs = recent["primaryDocument"].as_array().context("No primaryDocument array")?;
    let items_arr    = recent["items"].as_array();

    let form8k_indices: Vec<usize> = forms
        .iter()
        .enumerate()
        .filter(|(_, f)| f.as_str() == Some("8-K"))
        .map(|(i, _)| i)
        .take(5)
        .collect();

    let cik_numeric = cik.trim_start_matches('0');
    let mut inserted = 0u64;

    for idx in form8k_indices {
        let accession = match accessions.get(idx).and_then(|v| v.as_str()) {
            Some(a) => a,
            None => continue,
        };

        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM filings WHERE accession_number = $1)",
        )
        .bind(accession)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if exists { continue; }

        let filing_date_str = filing_dates.get(idx).and_then(|v| v.as_str()).unwrap_or("");
        let report_date_str = report_dates.get(idx).and_then(|v| v.as_str()).unwrap_or("");
        let primary_doc     = primary_docs.get(idx).and_then(|v| v.as_str()).unwrap_or("");
        let items = items_arr
            .and_then(|arr| arr.get(idx))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let filing_date = NaiveDate::parse_from_str(filing_date_str, "%Y-%m-%d")
            .unwrap_or_else(|_| Utc::now().date_naive());
        let report_date = NaiveDate::parse_from_str(report_date_str, "%Y-%m-%d").ok();
        let accession_nodash = accession.replace('-', "");

        let edgar_url = format!(
            "https://www.sec.gov/Archives/edgar/data/{cik_numeric}/{accession_nodash}/{primary_doc}"
        );

        sqlx::query(
            "INSERT INTO filings \
             (cik, ticker, form_type, filed_date, period_of_report, accession_number, edgar_url, items) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (accession_number) DO NOTHING",
        )
        .bind(cik)
        .bind(ticker)
        .bind("8-K")
        .bind(filing_date)
        .bind(report_date)
        .bind(accession)
        .bind(&edgar_url)
        .bind(items)
        .execute(pool)
        .await
        .context("Failed to insert 8-K filing")?;

        inserted += 1;
    }

    Ok(inserted)
}

pub(crate) async fn parse_and_insert_form4(
    xml: &str,
    ticker: &str,
    accession_number: &str,
    pool: &sqlx::PgPool,
) -> Result<u64> {
    let opts = roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() };
    let doc  = roxmltree::Document::parse_with_options(xml, opts)
        .context("Failed to parse Form 4 XML")?;
    let root = doc.root_element();

    let insider_name  = find_text_tag(root, "rptOwnerName")
        .unwrap_or_else(|| "Unknown".to_string());
    let insider_title = find_text_tag(root, "officerTitle");

    let mut inserted = 0u64;

    for txn in root.descendants().filter(|n| n.has_tag_name("nonDerivativeTransaction")) {
        let date_str = match find_val(txn, "transactionDate") {
            Some(d) => d,
            None => continue,
        };
        let date = match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };

        let shares_str = match find_val(txn, "transactionShares") {
            Some(s) => s,
            None => continue,
        };
        let shares: Decimal = match shares_str.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let price_str = find_val(txn, "transactionPricePerShare")
            .unwrap_or_else(|| "0".to_string());
        let price: Decimal = price_str.parse().unwrap_or_default();

        let txn_type = find_val(txn, "transactionAcquiredDisposedCode")
            .unwrap_or_else(|| "U".to_string());

        let shares_after: Option<Decimal> =
            find_val(txn, "sharesOwnedFollowingTransaction").and_then(|s| s.parse().ok());

        sqlx::query(
            "INSERT INTO insider_trades \
             (accession_number, ticker, insider_name, insider_title, \
              transaction_date, transaction_type, shares, price_per_share, shares_owned_after) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(accession_number)
        .bind(ticker)
        .bind(&insider_name)
        .bind(&insider_title)
        .bind(date)
        .bind(&txn_type)
        .bind(shares)
        .bind(price)
        .bind(shares_after)
        .execute(pool)
        .await
        .context("Failed to insert insider trade")?;

        inserted += 1;
    }

    Ok(inserted)
}

// ---------------------------------------------------------------------------
// XML parsing helpers
// ---------------------------------------------------------------------------

pub(crate) fn find_xml_in_index(html: &str) -> Option<String> {
    let mut search = html;
    while let Some(pos) = search.find("href=\"") {
        let rest = &search[pos + 6..];
        if let Some(end) = rest.find('"') {
            let href = &rest[..end];
            if href.ends_with(".xml") && !href.contains("-index") {
                return Some(href.to_string());
            }
        }
        search = &search[pos + 6..];
    }
    None
}

pub(crate) fn find_val(node: roxmltree::Node<'_, '_>, container: &str) -> Option<String> {
    node.descendants()
        .find(|n| n.has_tag_name(container))?
        .children()
        .find(|n| n.has_tag_name("value"))?
        .text()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub(crate) fn find_text_tag(node: roxmltree::Node<'_, '_>, tag: &str) -> Option<String> {
    node.descendants()
        .find(|n| n.has_tag_name(tag))?
        .text()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
