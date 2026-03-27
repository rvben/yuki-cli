use crate::cli::setup_domain;
use crate::client::accounting::AccountingClient;
use crate::client::soap_client::SoapClient;
use crate::config::Config;
use crate::error::YukiError;
use crate::output::{OutputFormat, format_json, format_table, is_tty};
use crate::period::parse_period;

pub async fn balance(
    config: &Config,
    admin: Option<&str>,
    account: Option<&str>,
    period: Option<&str>,
    format: Option<&str>,
) -> Result<(), YukiError> {
    let (start, _end) = resolve_period(period)?;
    let gl_code = account.unwrap_or("");
    let (client, entry) = setup_domain(config, admin).await?;
    let xml = client
        .gl_account_balance(&entry.admin_id, gl_code, &start)
        .await?;
    let balance = SoapClient::parse_single_result(&xml, "GLAccountBalanceResult").unwrap_or(xml);

    let headers = vec!["Account".into(), "Date".into(), "Balance".into()];
    let rows = vec![vec![gl_code.to_string(), start, balance]];

    let fmt = OutputFormat::from_flag(format, is_tty());
    match fmt {
        OutputFormat::Table => println!("{}", format_table(&headers, &rows)),
        OutputFormat::Json => println!("{}", format_json(&headers, &rows)),
    }
    Ok(())
}

pub async fn transactions(
    config: &Config,
    admin: Option<&str>,
    account: Option<&str>,
    period: Option<&str>,
    format: Option<&str>,
) -> Result<(), YukiError> {
    let (start, end) = resolve_period(period)?;
    let gl_code = account.unwrap_or("");
    let (client, entry) = setup_domain(config, admin).await?;
    let xml = client
        .gl_account_transactions(&entry.admin_id, gl_code, &start, &end)
        .await?;
    let transactions = AccountingClient::parse_gl_transactions(&xml)?;

    let headers = vec![
        "ID".into(),
        "Date".into(),
        "Amount".into(),
        "Description".into(),
    ];
    let rows: Vec<Vec<String>> = transactions
        .iter()
        .map(|t| {
            vec![
                t.id.clone(),
                t.date.clone(),
                t.amount.clone(),
                t.description.clone(),
            ]
        })
        .collect();

    let fmt = OutputFormat::from_flag(format, is_tty());
    match fmt {
        OutputFormat::Table => println!("{}", format_table(&headers, &rows)),
        OutputFormat::Json => println!("{}", format_json(&headers, &rows)),
    }
    Ok(())
}

/// Resolve an optional period string to (start_date, end_date).
///
/// When no period is given, defaults to the current calendar year.
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
    // Approximate: seconds since epoch divided by seconds per year
    1970 + (secs / 31_557_600) as u32
}
