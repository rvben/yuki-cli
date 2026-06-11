use crate::client::accounting::AccountingClient;
use crate::config::Config;
use crate::error::YukiError;
use crate::output::{OutputFormat, format_json, format_table, is_tty};

pub async fn list(
    config: &Config,
    format: Option<&str>,
    limit: Option<usize>,
    offset: Option<usize>,
    _fields: Option<&str>,
) -> Result<(), YukiError> {
    let mut client = AccountingClient::new();
    client.authenticate(&config.api_key).await?;
    let admins = client.administrations().await?;

    let headers = vec!["Name".into(), "Admin ID".into(), "Domain ID".into()];
    let mut rows: Vec<Vec<String>> = admins
        .iter()
        .map(|a| vec![a.name.clone(), a.id.clone(), a.domain_id.clone()])
        .collect();

    apply_pagination(&mut rows, offset, limit);

    let fmt = OutputFormat::from_flag(format, is_tty());
    match fmt {
        OutputFormat::Table => println!("{}", format_table(&headers, &rows)),
        OutputFormat::Json => println!("{}", format_json(&headers, &rows)),
    }
    Ok(())
}

fn apply_pagination(rows: &mut Vec<Vec<String>>, offset: Option<usize>, limit: Option<usize>) {
    if let Some(off) = offset {
        if off >= rows.len() {
            rows.clear();
            return;
        }
        rows.drain(..off);
    }
    if let Some(lim) = limit {
        rows.truncate(lim);
    }
}

pub fn switch(config: &mut Config, name: &str) -> Result<(), YukiError> {
    if !config.administrations.contains_key(name) {
        return Err(YukiError::Config(format!(
            "unknown administration: {name}. Run 'yuki admin list' to see available administrations."
        )));
    }
    config.default_admin = name.to_string();
    config.save_to(&Config::default_path())?;
    eprintln!("Switched default administration to: {name}");
    Ok(())
}
