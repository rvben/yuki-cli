use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

use owo_colors::OwoColorize;

use crate::client::accounting::AccountingClient;
use crate::config::{AdminEntry, Config};
use crate::error::YukiError;
use crate::output::is_tty;

fn sym_ok() -> String {
    if is_tty() {
        "✔".green().to_string()
    } else {
        "✔".to_owned()
    }
}

fn sym_fail() -> String {
    if is_tty() {
        "✖".red().to_string()
    } else {
        "✖".to_owned()
    }
}

fn bold(s: &str) -> String {
    if is_tty() {
        s.bold().to_string()
    } else {
        s.to_owned()
    }
}

fn dim(s: &str) -> String {
    if is_tty() {
        s.dimmed().to_string()
    } else {
        s.to_owned()
    }
}

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

pub async fn run(api_key: Option<&str>, default_admin: Option<&str>) -> Result<(), YukiError> {
    let stdin = io::stdin();
    let path = Config::default_path();

    // If config exists and only --api-key is provided, update the key in place.
    if let (Some(key), None) = (api_key, default_admin)
        && let Ok(mut config) = Config::load_from(&path)
    {
        let key = key.trim().to_string();
        if key.is_empty() {
            return Err(YukiError::Config("API key cannot be empty".to_string()));
        }

        eprintln!("Verifying new API key...");
        let mut client = AccountingClient::new();
        client.authenticate(&key).await?;

        config.api_key = key;
        config.save_to(&path)?;
        eprintln!("{} API key updated in {}", sym_ok(), path.display());
        return Ok(());
    }

    let api_key = match api_key {
        Some(k) => k.trim().to_string(),
        None => {
            eprintln!("  {} Yuki Portal → Settings → API keys", dim("→"));
            eprint!("Yuki API key: ");
            io::stderr().flush().ok();
            stdin
                .lock()
                .lines()
                .next()
                .and_then(|l| l.ok())
                .map(|l| l.trim().to_string())
                .unwrap_or_default()
        }
    };

    if api_key.is_empty() {
        return Err(YukiError::Config("API key cannot be empty".to_string()));
    }

    eprint!("Authenticating...");
    io::stderr().flush().ok();
    let mut client = AccountingClient::new();
    match client.authenticate(&api_key).await {
        Ok(_) => eprintln!(" {}", sym_ok()),
        Err(e) => {
            eprintln!(" {}", sym_fail());
            return Err(e);
        }
    }

    eprint!("Fetching administrations...");
    io::stderr().flush().ok();
    let admins = match client.administrations().await {
        Ok(admins) => {
            eprintln!(" {}", sym_ok());
            admins
        }
        Err(e) => {
            eprintln!(" {}", sym_fail());
            return Err(e);
        }
    };

    if admins.is_empty() {
        return Err(YukiError::NotFound(
            "no administrations found for this API key".to_string(),
        ));
    }

    eprintln!("Found {} administration(s):", admins.len());
    for (i, a) in admins.iter().enumerate() {
        eprintln!("  [{}] {}", i + 1, a.name);
    }

    let default_name = if let Some(name) = default_admin {
        // Use the provided name directly — verify it exists.
        let key = safe_name(name);
        if !admins.iter().any(|a| safe_name(&a.name) == key) {
            return Err(YukiError::Config(format!(
                "administration not found: {name}"
            )));
        }
        eprintln!("Using \"{name}\" as the default administration.");
        key
    } else if admins.len() == 1 {
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

    // Preserve unmatched_ignore from existing config if present.
    let unmatched_ignore = Config::load_from(&path)
        .map(|c| c.unmatched_ignore)
        .unwrap_or_default();

    let config = Config {
        api_key,
        default_admin: default_name.clone(),
        administrations,
        unmatched_ignore,
    };

    config.save_to(&path)?;

    eprintln!();
    eprintln!("{} Configuration saved to {}", sym_ok(), path.display());
    eprintln!();
    eprintln!("{}:", bold("Next steps"));
    eprintln!(
        "  yuki documents list  {}",
        dim("# list archived documents")
    );
    eprintln!("  yuki contacts list   {}", dim("# list contacts"));
    eprintln!("  yuki invoices list   {}", dim("# list invoices"));
    eprintln!("  yuki completions zsh {}", dim("# shell completions"));
    eprintln!();

    Ok(())
}
