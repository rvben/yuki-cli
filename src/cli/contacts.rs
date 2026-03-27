use crate::client::contact::{Contact, ContactClient};
use crate::config::Config;
use crate::error::YukiError;
use crate::output::{OutputFormat, format_json, format_table, is_tty};

fn contacts_to_rows(contacts: &[Contact]) -> Vec<Vec<String>> {
    contacts
        .iter()
        .map(|c| {
            vec![
                c.id.clone(),
                c.name.clone(),
                c.contact_type.clone(),
                c.country.clone(),
                if c.is_supplier { "Yes" } else { "No" }.to_string(),
                if c.is_customer { "Yes" } else { "No" }.to_string(),
            ]
        })
        .collect()
}

pub async fn search(
    config: &Config,
    _admin: Option<&str>,
    query: &str,
    format: Option<&str>,
) -> Result<(), YukiError> {
    let mut client = ContactClient::new();
    client.authenticate(&config.api_key).await?;
    let contacts = client.search_contacts(query).await?;

    let headers = vec![
        "ID".into(),
        "Name".into(),
        "Type".into(),
        "Country".into(),
        "Supplier".into(),
        "Customer".into(),
    ];
    let rows = contacts_to_rows(&contacts);

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
    let contacts = client
        .get_suppliers_and_customers(contact_type.unwrap_or(""))
        .await?;

    let headers = vec![
        "ID".into(),
        "Name".into(),
        "Type".into(),
        "Country".into(),
        "Supplier".into(),
        "Customer".into(),
    ];
    let rows = contacts_to_rows(&contacts);

    let fmt = OutputFormat::from_flag(format, is_tty());
    match fmt {
        OutputFormat::Table => println!("{}", format_table(&headers, &rows)),
        OutputFormat::Json => println!("{}", format_json(&headers, &rows)),
    }
    Ok(())
}
