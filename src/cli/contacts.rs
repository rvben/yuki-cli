use crate::client::contact::ContactClient;
use crate::config::Config;
use crate::error::YukiError;
use crate::output::{OutputFormat, format_json, format_table, is_tty};

pub async fn search(
    config: &Config,
    _admin: Option<&str>,
    query: &str,
    format: Option<&str>,
) -> Result<(), YukiError> {
    let mut client = ContactClient::new();
    client.authenticate(&config.api_key).await?;
    let xml = client.search_contacts(query).await?;

    let headers = vec!["Raw XML".into()];
    let rows = vec![vec![xml]];

    let fmt = OutputFormat::from_flag(format, is_tty());
    match fmt {
        OutputFormat::Table => println!("{}", format_table(&headers, &rows)),
        OutputFormat::Json => println!("{}", format_json(&headers, &rows)),
    }
    Ok(())
}

pub async fn list(
    config: &Config,
    _admin: Option<&str>,
    contact_type: Option<&str>,
    format: Option<&str>,
) -> Result<(), YukiError> {
    let mut client = ContactClient::new();
    client.authenticate(&config.api_key).await?;
    let xml = client
        .get_suppliers_and_customers(contact_type.unwrap_or(""))
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
