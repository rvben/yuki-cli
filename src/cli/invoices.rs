use crate::cli::setup_domain;
use crate::client::accounting_info::AccountingInfoClient;
use crate::client::sales::SalesClient;
use crate::config::Config;
use crate::error::YukiError;
use crate::output::{OutputFormat, format_json, format_table, is_tty};
use crate::period::parse_period;

pub async fn list(
    config: &Config,
    admin: Option<&str>,
    period: Option<&str>,
    invoice_type: Option<&str>,
    format: Option<&str>,
) -> Result<(), YukiError> {
    let fmt = OutputFormat::from_flag(format, is_tty());

    match invoice_type {
        Some("purchase") | Some("creditor") => {
            let (start, end) = resolve_period(period)?;
            let client = setup_domain(config, admin).await?;
            let items = client.outstanding_creditor_items_by_date(&end).await?;

            let items: Vec<_> = items
                .into_iter()
                .filter(|i| i.date >= start && i.date <= end)
                .collect();

            let headers = vec![
                "Contact".into(),
                "Description".into(),
                "Date".into(),
                "Amount".into(),
                "Open".into(),
            ];
            let rows: Vec<Vec<String>> = items
                .iter()
                .map(|i| {
                    vec![
                        i.contact_name.clone(),
                        i.description.clone(),
                        i.date.clone(),
                        i.amount.clone(),
                        i.open_amount.clone(),
                    ]
                })
                .collect();

            match fmt {
                OutputFormat::Table => println!("{}", format_table(&headers, &rows)),
                OutputFormat::Json => println!("{}", format_json(&headers, &rows)),
            }
        }

        // Default to sales invoices when type is "sales", "debtor", or unspecified
        _ => {
            let mut client = SalesClient::new();
            client.authenticate(&config.api_key).await?;
            let xml = client.get_sales_items().await?;

            let headers = vec!["Raw XML".into()];
            let rows = vec![vec![xml]];

            match fmt {
                OutputFormat::Table => println!("{}", format_table(&headers, &rows)),
                OutputFormat::Json => println!("{}", format_json(&headers, &rows)),
            }
        }
    }

    Ok(())
}

pub async fn show(
    config: &Config,
    _admin: Option<&str>,
    id: &str,
    format: Option<&str>,
) -> Result<(), YukiError> {
    let mut client = AccountingInfoClient::new();
    client.authenticate(&config.api_key).await?;
    let xml = client.get_transaction_details(id).await?;

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
