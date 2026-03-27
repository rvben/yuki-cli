use crate::cli::setup_domain;
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
