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
/// Check if an invoice exists in the archive by amount and date (±7 days).
///
/// Searches within a month around the given date, then filters by amount (±0.01)
/// and date proximity (±7 days). Returns matching documents or exits with code 3.
pub async fn exists(
    config: &Config,
    _admin: Option<&str>,
    amount: f64,
    date: &str,
    contact: Option<&str>,
    format: Option<&str>,
) -> Result<(), YukiError> {
    // Search a month around the given date
    let (search_start, search_end) = date_window(date, 30)?;

    let mut client = ArchiveClient::new();
    client.authenticate(&config.api_key).await?;

    let search_text = contact.unwrap_or("");
    let docs = client
        .search_documents(search_text, &search_start, &search_end)
        .await?;

    let matched: Vec<_> = docs
        .into_iter()
        .filter(|d| {
            let amount_matches = d
                .amount
                .trim()
                .parse::<f64>()
                .map(|a| (a - amount).abs() <= 0.01)
                .unwrap_or(false);
            let date_matches = within_days(&d.document_date, date, 7);
            amount_matches && date_matches
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

/// Build a search window of ±`days` around a YYYY-MM-DD date string.
fn date_window(date: &str, days: i32) -> Result<(String, String), YukiError> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return Err(YukiError::Config(format!(
            "invalid date format: '{date}', expected YYYY-MM-DD"
        )));
    }
    let y: i32 = parts[0].parse().unwrap_or(0);
    let m: i32 = parts[1].parse().unwrap_or(0);
    let d: i32 = parts[2].parse().unwrap_or(0);

    // Approximate: just shift the month for the window
    let start_m = if d - days < 1 { m - 1 } else { m };
    let end_m = if d + days > 28 { m + 1 } else { m };

    let start = format!("{y:04}-{:02}-01", start_m.max(1));
    let end = format!("{y:04}-{:02}-28", end_m.min(12));
    Ok((start, end))
}

/// Check if two YYYY-MM-DD date strings are within `max_days` of each other.
fn within_days(date_a: &str, date_b: &str, max_days: i32) -> bool {
    let to_days = |s: &str| -> Option<i32> {
        let p: Vec<&str> = s.split('-').collect();
        if p.len() < 3 {
            return None;
        }
        let y: i32 = p[0].parse().ok()?;
        let m: i32 = p[1].parse().ok()?;
        let d: i32 = p[2].split('T').next()?.parse().ok()?;
        Some(y * 365 + m * 30 + d)
    };
    match (to_days(date_a), to_days(date_b)) {
        (Some(a), Some(b)) => (a - b).abs() <= max_days,
        _ => false,
    }
}
