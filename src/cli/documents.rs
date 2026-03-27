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

    let xml = match (folder, doc_type) {
        (Some(f), _) => client.documents_in_folder(f).await?,
        (None, Some(t)) => client.documents_by_type(t).await?,
        (None, None) => client.documents_in_folder("").await?,
    };

    let headers = vec!["Raw XML".into()];
    let rows = vec![vec![xml]];

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
    let xml = client.search_documents(query).await?;

    let headers = vec!["Raw XML".into()];
    let rows = vec![vec![xml]];

    let fmt = OutputFormat::from_flag(format, is_tty());
    match fmt {
        OutputFormat::Table => println!("{}", format_table(&headers, &rows)),
        OutputFormat::Json => println!("{}", format_json(&headers, &rows)),
    }
    Ok(())
}
