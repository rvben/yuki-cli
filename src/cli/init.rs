use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

use crate::client::accounting::AccountingClient;
use crate::config::{AdminEntry, Config};
use crate::error::YukiError;

/// Convert an administration name to a safe config key.
///
/// Lowercases the name and replaces any non-alphanumeric character with an underscore.
fn safe_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

pub async fn run() -> Result<(), YukiError> {
    let stdin = io::stdin();

    eprint!("Yuki API key: ");
    io::stderr().flush().ok();

    let api_key = stdin
        .lock()
        .lines()
        .next()
        .and_then(|l| l.ok())
        .map(|l| l.trim().to_string())
        .unwrap_or_default();

    if api_key.is_empty() {
        return Err(YukiError::Config("API key cannot be empty".to_string()));
    }

    eprintln!("Authenticating...");
    let mut client = AccountingClient::new();
    client.authenticate(&api_key).await?;

    eprintln!("Fetching administrations...");
    let admins = client.administrations().await?;

    if admins.is_empty() {
        return Err(YukiError::NotFound(
            "no administrations found for this API key".to_string(),
        ));
    }

    eprintln!("Found {} administration(s):", admins.len());
    for (i, a) in admins.iter().enumerate() {
        eprintln!("  [{}] {}", i + 1, a.name);
    }

    let default_name = if admins.len() == 1 {
        eprintln!(
            "Using \"{}\" as the default administration.",
            admins[0].name
        );
        safe_name(&admins[0].name)
    } else {
        eprint!("Select default administration [1]: ");
        io::stderr().flush().ok();

        let choice = stdin
            .lock()
            .lines()
            .next()
            .and_then(|l| l.ok())
            .map(|l| l.trim().to_string())
            .unwrap_or_default();

        let idx: usize = if choice.is_empty() {
            1
        } else {
            choice
                .parse::<usize>()
                .map_err(|_| YukiError::Config(format!("invalid selection: {choice}")))?
        };

        if idx == 0 || idx > admins.len() {
            return Err(YukiError::Config(format!("selection out of range: {idx}")));
        }
        safe_name(&admins[idx - 1].name)
    };

    // Store both domain_id and admin_id per administration
    let administrations: BTreeMap<String, AdminEntry> = admins
        .iter()
        .map(|a| {
            (
                safe_name(&a.name),
                AdminEntry {
                    domain_id: a.domain_id.clone(),
                    admin_id: a.id.clone(),
                },
            )
        })
        .collect();

    let config = Config {
        api_key,
        default_admin: default_name.clone(),
        administrations,
    };

    let path = Config::default_path();
    config.save_to(&path)?;

    eprintln!("Configuration saved to {}", path.display());
    eprintln!("Default administration: {default_name}");

    Ok(())
}
