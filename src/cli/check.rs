use crate::cli::setup_domain;
use crate::client::accounting::AccountingClient;
use crate::client::accounting_info::AccountingInfoClient;
use crate::client::archive::ArchiveClient;
use crate::client::vat::VatClient;
use crate::config::Config;
use crate::error::YukiError;
use crate::output::{OutputFormat, format_json, format_table, is_tty};
use crate::period::parse_period;

pub async fn btw(
    config: &Config,
    admin: Option<&str>,
    period: Option<&str>,
    format: Option<&str>,
    quiet: bool,
) -> Result<(), YukiError> {
    let (start, end) = resolve_period(period)?;

    if !quiet {
        eprintln!("[1/5] Fetching VAT return list...");
    }
    let mut vat_client = VatClient::new();
    vat_client.authenticate(&config.api_key).await?;
    let (accounting_client, entry) = setup_domain(config, admin).await?;
    let vat_returns = vat_client.vat_return_list(&entry.admin_id).await?;

    if !quiet {
        eprintln!("[2/5] Fetching GL account transactions...");
    }
    let _transactions = accounting_client
        .gl_account_transactions(&entry.admin_id, "", &start, &end)
        .await?;

    if !quiet {
        eprintln!("[3/5] Fetching outstanding debtor items...");
    }
    let debtors = accounting_client
        .outstanding_debtor_items_by_date(&entry.admin_id, &start, &end)
        .await?;

    if !quiet {
        eprintln!("[4/5] Fetching outstanding creditor items...");
    }
    let creditors = accounting_client
        .outstanding_creditor_items_by_date(&entry.admin_id, &start, &end)
        .await?;

    if !quiet {
        eprintln!("[5/5] Fetching modified documents...");
    }
    let mut archive_client = ArchiveClient::new();
    archive_client.authenticate(&config.api_key).await?;
    let _modified = archive_client.modified_documents_by_type(0, &start).await?;

    if !quiet {
        let api_calls = 5;
        eprintln!("API calls made: {api_calls}");
    }

    // Build report: VAT returns in period + outstanding items
    let headers = vec![
        "Type".into(),
        "Contact".into(),
        "Description".into(),
        "Date".into(),
        "Amount".into(),
        "Open".into(),
    ];
    let mut rows: Vec<Vec<String>> = Vec::new();

    for r in &vat_returns {
        if r.start_date >= start && r.end_date <= end {
            rows.push(vec![
                "VAT Return".into(),
                String::new(),
                format!("Period {} ({})", r.period, r.status),
                r.start_date.clone(),
                String::new(),
                String::new(),
            ]);
        }
    }

    for item in &debtors {
        rows.push(vec![
            "Debtor".into(),
            item.contact_name.clone(),
            item.description.clone(),
            item.date.clone(),
            item.amount.clone(),
            item.open_amount.clone(),
        ]);
    }

    for item in &creditors {
        rows.push(vec![
            "Creditor".into(),
            item.contact_name.clone(),
            item.description.clone(),
            item.date.clone(),
            item.amount.clone(),
            item.open_amount.clone(),
        ]);
    }

    let fmt = OutputFormat::from_flag(format, is_tty());
    match fmt {
        OutputFormat::Table => println!("{}", format_table(&headers, &rows)),
        OutputFormat::Json => println!("{}", format_json(&headers, &rows)),
    }
    Ok(())
}

pub async fn jaarwerk(
    config: &Config,
    admin: Option<&str>,
    year: Option<&str>,
    format: Option<&str>,
    quiet: bool,
) -> Result<(), YukiError> {
    let year_str = year.unwrap_or("2025");
    let start = format!("{year_str}-01-01");
    let end = format!("{year_str}-12-31");

    if !quiet {
        eprintln!("[1/4] Fetching period date table...");
    }
    let mut info_client = AccountingInfoClient::new();
    info_client.authenticate(&config.api_key).await?;
    let _period_table = info_client.get_period_date_table(year_str).await?;

    if !quiet {
        eprintln!("[2/4] Fetching GL account balances...");
    }
    let (accounting_client, entry) = setup_domain(config, admin).await?;
    let balance = accounting_client
        .gl_account_balance(&entry.admin_id, "", &end)
        .await?;

    if !quiet {
        eprintln!("[3/4] Fetching outstanding items...");
    }
    let debtors = accounting_client
        .outstanding_debtor_items_by_date(&entry.admin_id, &start, &end)
        .await?;
    let creditors = accounting_client
        .outstanding_creditor_items_by_date(&entry.admin_id, &start, &end)
        .await?;

    if !quiet {
        eprintln!("[4/4] Fetching modified documents...");
    }
    let mut archive_client = ArchiveClient::new();
    archive_client.authenticate(&config.api_key).await?;
    let _modified = archive_client.modified_documents_by_type(0, &start).await?;

    if !quiet {
        let api_calls = 5;
        eprintln!("API calls made: {api_calls}");
    }

    let headers = vec![
        "Type".into(),
        "Contact".into(),
        "Description".into(),
        "Date".into(),
        "Amount".into(),
        "Open".into(),
    ];
    let mut rows: Vec<Vec<String>> = Vec::new();

    rows.push(vec![
        "GL Balance".into(),
        String::new(),
        format!("Year {year_str}"),
        start,
        balance,
        String::new(),
    ]);

    for item in &debtors {
        rows.push(vec![
            "Debtor".into(),
            item.contact_name.clone(),
            item.description.clone(),
            item.date.clone(),
            item.amount.clone(),
            item.open_amount.clone(),
        ]);
    }

    for item in &creditors {
        rows.push(vec![
            "Creditor".into(),
            item.contact_name.clone(),
            item.description.clone(),
            item.date.clone(),
            item.amount.clone(),
            item.open_amount.clone(),
        ]);
    }

    let fmt = OutputFormat::from_flag(format, is_tty());
    match fmt {
        OutputFormat::Table => println!("{}", format_table(&headers, &rows)),
        OutputFormat::Json => println!("{}", format_json(&headers, &rows)),
    }
    Ok(())
}

/// Extract the counterparty name from a SEPA bank transaction description.
///
/// SEPA descriptions encode counterparty info in the format:
/// `/CNTP/<iban>/<bic>/<name>/`
///
/// Falls back to the first 50 characters of the description when the field is absent.
fn parse_counterparty(description: &str) -> String {
    if let Some(cntp_start) = description.find("/CNTP/") {
        let after_cntp = &description[cntp_start + 6..];
        let parts: Vec<&str> = after_cntp.splitn(4, '/').collect();
        if parts.len() >= 3 {
            return parts[2].trim().to_string();
        }
    }
    description
        .chars()
        .take(50)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Find bank transactions on GL account 11001 that have no matching invoice.
///
/// Each bank debit (negative amount) is first matched by absolute amount against outstanding
/// creditor items. Any remaining unmatched debits are then checked against booked invoices in
/// the archive for the same period. Only transactions that match neither source are reported.
pub async fn unmatched(
    config: &Config,
    admin: Option<&str>,
    period: Option<&str>,
    format: Option<&str>,
    quiet: bool,
) -> Result<(), YukiError> {
    let (start, end) = resolve_period(period)?;

    if !quiet {
        eprintln!("[1/3] Fetching bank transactions (GL 11001)...");
    }
    let mut accounting_client = AccountingClient::new();
    accounting_client.authenticate(&config.api_key).await?;
    let (accounting_client, entry) = {
        let entry = config.resolve_admin(admin)?;
        accounting_client
            .set_current_domain(&entry.domain_id)
            .await?;
        (accounting_client, entry)
    };

    let raw = accounting_client
        .gl_account_transactions(&entry.admin_id, "11001", &start, &end)
        .await?;
    let transactions = AccountingClient::parse_gl_transactions(&raw)?;

    if !quiet {
        eprintln!("[2/3] Fetching outstanding creditor items...");
    }
    let creditor_items = accounting_client
        .outstanding_creditor_items(&entry.admin_id)
        .await?;

    if !quiet {
        eprintln!("[3/3] Fetching booked invoices from archive...");
    }
    let mut archive_client = ArchiveClient::new();
    archive_client.authenticate(&config.api_key).await?;
    let archive_docs = archive_client.search_documents("", &start, &end).await?;

    if !quiet {
        eprintln!("API calls made: 4");
    }

    // Build a pool of creditor open amounts for single-pass matching.
    // Key: canonical amount string (absolute value), value: remaining count.
    let mut creditor_pool: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for item in &creditor_items {
        let key = item.open_amount.trim().to_string();
        *creditor_pool.entry(key).or_insert(0) += 1;
    }

    // Build a pool of archive document amounts for matching booked invoices.
    // Only positive amounts (purchase invoices) are relevant.
    let mut archive_pool: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for doc in &archive_docs {
        if let Ok(amt) = doc.amount.trim().parse::<f64>()
            && amt > 0.0
        {
            let key = format!("{amt:.2}");
            *archive_pool.entry(key).or_insert(0) += 1;
        }
    }

    let mut unmatched_rows: Vec<Vec<String>> = Vec::new();

    for tx in &transactions {
        let amount: f64 = tx.amount.trim().parse().unwrap_or(0.0);
        if amount >= 0.0 {
            // Only debits (payments out) are relevant.
            continue;
        }
        // Represent the absolute value as a rounded-cent string for matching.
        let abs_amount = format!("{:.2}", amount.abs());

        // Check outstanding creditor items first.
        if let Some(count) = creditor_pool.get_mut(&abs_amount)
            && *count > 0
        {
            *count -= 1;
            continue;
        }

        // Check booked invoices in the archive.
        if let Some(count) = archive_pool.get_mut(&abs_amount)
            && *count > 0
        {
            *count -= 1;
            continue;
        }

        let counterparty = parse_counterparty(&tx.description);
        unmatched_rows.push(vec![
            tx.date.clone(),
            format!("-{abs_amount}"),
            counterparty,
            tx.description.chars().take(80).collect::<String>(),
        ]);
    }

    let headers = vec![
        "Date".into(),
        "Amount".into(),
        "Counterparty".into(),
        "Description".into(),
    ];

    let fmt = OutputFormat::from_flag(format, is_tty());
    match fmt {
        OutputFormat::Table => println!("{}", format_table(&headers, &unmatched_rows)),
        OutputFormat::Json => println!("{}", format_json(&headers, &unmatched_rows)),
    }
    Ok(())
}

/// Resolve an optional period string to (start_date, end_date).
///
/// Defaults to the current calendar year when no period is given.
fn resolve_period(period: Option<&str>) -> Result<(String, String), YukiError> {
    match period {
        Some(p) => parse_period(p),
        None => {
            let year = current_year();
            Ok((format!("{year}-01-01"), format!("{year}-12-31")))
        }
    }
}

fn current_year() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    1970 + (secs / 31_557_600) as u32
}
