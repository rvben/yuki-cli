use crate::cli::setup_domain;
use crate::config::Config;
use crate::error::YukiError;
use crate::output::{OutputFormat, format_json, format_table, is_tty};
use crate::period::parse_period;

pub async fn balance(
    config: &Config,
    admin: Option<&str>,
    account: &str,
    period: Option<&str>,
    format: Option<&str>,
) -> Result<(), YukiError> {
    let (start, end) = resolve_period(period)?;
    let client = setup_domain(config, admin).await?;
    let balance = client.gl_account_balance(account, &start, &end).await?;

    let headers = vec![
        "Account".into(),
        "Start".into(),
        "End".into(),
        "Balance".into(),
    ];
    let rows = vec![vec![account.to_string(), start, end, balance]];

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
    account: &str,
    period: Option<&str>,
    format: Option<&str>,
) -> Result<(), YukiError> {
    let (start, end) = resolve_period(period)?;
    let client = setup_domain(config, admin).await?;
    let xml = client
        .gl_account_transactions(account, &start, &end)
        .await?;

    let headers = vec!["Raw XML".into()];
    let rows = vec![vec![xml]];

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
