use std::process;

use crate::client::archive::ArchiveClient;
use crate::config::Config;
use crate::error::YukiError;
use crate::output::{OutputFormat, format_json, format_table, is_tty};

pub async fn list(
    config: &Config,
    _admin: Option<&str>,
    folder: Option<&str>,
    doc_type: Option<&str>,
    format: Option<&str>,
) -> Result<(), YukiError> {
    let mut client = ArchiveClient::new();
    client.authenticate(&config.api_key).await?;

    let docs = match (folder, doc_type) {
        (Some(f), _) => {
            let folder_id: i32 = f.parse().unwrap_or(0);
            client
                .documents_in_folder(folder_id, "2000-01-01", "2099-12-31")
                .await?
        }
        (None, Some(t)) => {
            let doc_type_id: i32 = t.parse().unwrap_or(0);
            let xml = client.documents_by_type(doc_type_id).await?;
            // documents_by_type returns raw XML; wrap it for display
            let headers = vec!["Raw XML".into()];
            let rows = vec![vec![xml]];
            let fmt = OutputFormat::from_flag(format, is_tty());
            match fmt {
                OutputFormat::Table => println!("{}", format_table(&headers, &rows)),
                OutputFormat::Json => println!("{}", format_json(&headers, &rows)),
            }
            return Ok(());
        }
        (None, None) => {
            client
                .documents_in_folder(0, "2000-01-01", "2099-12-31")
                .await?
        }
    };

    let headers = vec![
        "ID".into(),
        "Date".into(),
        "Amount".into(),
        "Contact".into(),
        "Subject".into(),
        "File".into(),
    ];
    let rows: Vec<Vec<String>> = docs
        .into_iter()
        .map(|d| {
            vec![
                d.id,
                d.document_date,
                d.amount,
                d.contact_name,
                d.subject,
                d.file_name,
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

pub async fn search(
    config: &Config,
    _admin: Option<&str>,
    query: &str,
    format: Option<&str>,
) -> Result<(), YukiError> {
    let mut client = ArchiveClient::new();
    client.authenticate(&config.api_key).await?;

    // Use current year as default search range when no date is specified.
    let year = current_year();
    let start = format!("{year}-01-01");
    let end = format!("{year}-12-31");

    let docs = client.search_documents(query, &start, &end).await?;

    let headers = vec![
        "ID".into(),
        "Date".into(),
        "Amount".into(),
        "Contact".into(),
        "Subject".into(),
        "File".into(),
    ];
    let rows: Vec<Vec<String>> = docs
        .into_iter()
        .map(|d| {
            vec![
                d.id,
                d.document_date,
                d.amount,
                d.contact_name,
                d.subject,
                d.file_name,
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

/// Check if an invoice exists in the archive by amount and optional contact name.
///
/// Searches the archive for documents in the given period, then filters by amount
/// with a ±0.01 tolerance. Outputs matching documents and exits with code 3 if none found.
pub async fn exists(
    config: &Config,
    _admin: Option<&str>,
    amount: f64,
    contact: Option<&str>,
    period: Option<&str>,
    format: Option<&str>,
) -> Result<(), YukiError> {
    let year = current_year();
    let (start, end) = match period {
        Some(p) => crate::period::parse_period(p)?,
        None => (format!("{year}-01-01"), format!("{year}-12-31")),
    };

    let mut client = ArchiveClient::new();
    client.authenticate(&config.api_key).await?;

    let search_text = contact.unwrap_or("");
    let docs = client.search_documents(search_text, &start, &end).await?;

    let matched: Vec<_> = docs
        .into_iter()
        .filter(|d| {
            d.amount
                .trim()
                .parse::<f64>()
                .map(|a| (a - amount).abs() <= 0.01)
                .unwrap_or(false)
        })
        .collect();

    let headers = vec![
        "ID".into(),
        "Date".into(),
        "Amount".into(),
        "Contact".into(),
        "Subject".into(),
        "File".into(),
    ];
    let rows: Vec<Vec<String>> = matched
        .iter()
        .map(|d| {
            vec![
                d.id.clone(),
                d.document_date.clone(),
                d.amount.clone(),
                d.contact_name.clone(),
                d.subject.clone(),
                d.file_name.clone(),
            ]
        })
        .collect();

    let fmt = OutputFormat::from_flag(format, is_tty());
    match fmt {
        OutputFormat::Table => println!("{}", format_table(&headers, &rows)),
        OutputFormat::Json => println!("{}", format_json(&headers, &rows)),
    }

    if rows.is_empty() {
        process::exit(3);
    }

    Ok(())
}

fn current_year() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    1970 + (secs / 31_557_600) as u32
}
