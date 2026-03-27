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

fn current_year() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    1970 + (secs / 31_557_600) as u32
}
