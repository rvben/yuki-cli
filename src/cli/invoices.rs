use crate::cli::setup_domain;
use crate::client::accounting_info::AccountingInfoClient;
use crate::client::sales::SalesClient;
use crate::config::Config;
use crate::error::YukiError;
use crate::output::{OutputFormat, format_json, format_table, is_tty};

pub async fn list(
    config: &Config,
    admin: Option<&str>,
    _period: Option<&str>,
    invoice_type: Option<&str>,
    format: Option<&str>,
) -> Result<(), YukiError> {
    let fmt = OutputFormat::from_flag(format, is_tty());

    match invoice_type {
        Some("purchase") | Some("creditor") => {
            let (client, entry) = setup_domain(config, admin).await?;
            let items = client.outstanding_creditor_items(&entry.admin_id).await?;

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
