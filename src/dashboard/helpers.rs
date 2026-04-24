// ---------------------------------------------------------------------------
// Formatting helpers — comma-grouped, sign-prefixed, compact
// ---------------------------------------------------------------------------

/// Comma-group an integer string: 1234567 -> "1,234,567"
fn comma_group(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(','); }
        out.push(ch);
    }
    out.chars().rev().collect()
}

/// Format a share count with comma grouping: 1234567 -> "1,234,567"
pub fn format_shares(n: i64) -> String {
    comma_group(&n.to_string())
}

/// Format a price with comma grouping and 2 decimals: 1234.5 -> "$1,234.50"
pub fn format_price(v: f64) -> String {
    if v.is_nan() || v.is_infinite() { return "—".into(); }
    let abs = v.abs();
    let int_part = abs as u64;
    let frac = format!("{:.2}", abs - int_part as f64);
    let sign = if v < 0.0 { "-" } else { "" };
    format!("${sign}{}{}", comma_group(&int_part.to_string()), &frac[1..])
}

/// Format a percentage with sign prefix: 12.345 -> "+12.3%", -4.5 -> "-4.5%"
pub fn format_pct(v: f64) -> String {
    if v.is_nan() || v.is_infinite() { return "—".into(); }
    if v >= 0.0 { format!("+{v:.1}%") } else { format!("{v:.1}%") }
}

/// Compact number: 1.2B, 345M, 12.3K, or raw for small values.
pub fn format_compact(v: f64) -> String {
    let abs = v.abs();
    let sign = if v < 0.0 { "-" } else { "" };
    if abs >= 1_000_000_000.0 { format!("{sign}{:.1}B", abs / 1_000_000_000.0) }
    else if abs >= 1_000_000.0 { format!("{sign}{:.1}M", abs / 1_000_000.0) }
    else if abs >= 10_000.0    { format!("{sign}{:.1}K", abs / 1_000.0) }
    else { format!("{sign}{abs:.0}") }
}

pub fn format_market_value_i64(v: i64) -> String {
    let f = v as f64;
    if f >= 1_000_000_000.0 { format!("${:.1}B", f / 1_000_000_000.0) }
    else if f >= 1_000_000.0 { format!("${:.1}M", f / 1_000_000.0) }
    else { format!("${f:.0}") }
}

pub fn format_market_value(val: &rust_decimal::Decimal) -> String {
    let f: f64 = val.to_string().parse().unwrap_or(0.0);
    if f >= 1_000_000_000.0 { format!("${:.1}B", f / 1_000_000_000.0) }
    else if f >= 1_000_000.0 { format!("${:.1}M", f / 1_000_000.0) }
    else { format!("${f:.0}") }
}

pub fn describe_8k_items(items: &str) -> String {
    const CODES: &[(&str, &str)] = &[
        ("1.01", "Material Agreement"), ("1.02", "Agreement Terminated"),
        ("1.03", "Bankruptcy/Receivership"), ("2.01", "Acquisition/Disposition"),
        ("2.02", "Earnings Results"), ("2.03", "Off-Balance Sheet Obligation"),
        ("2.04", "Triggering Events"), ("2.05", "Cost Restructuring"),
        ("2.06", "Material Impairment"), ("3.01", "Delisting Notice"),
        ("3.02", "Unregistered Securities"), ("4.01", "Auditor Change"),
        ("4.02", "Non-Reliance on Financials"), ("5.01", "Change in Control"),
        ("5.02", "Officers/Directors Change"), ("5.03", "Charter Amendment"),
        ("5.07", "Voting Results"), ("7.01", "Regulation FD Disclosure"),
        ("8.01", "Other Events"), ("9.01", "Exhibits"),
    ];
    items.split(',').map(|code| {
        let code = code.trim();
        CODES.iter().find(|(k, _)| *k == code).map(|(_, v)| *v).unwrap_or(code)
    }).collect::<Vec<_>>().join(", ")
}
