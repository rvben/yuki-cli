use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

use owo_colors::OwoColorize;

use crate::client::accounting::{AccountingClient, Administration};
use crate::config::{AdminEntry, Config, Region};
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

/// Index discovered administrations by their config key.
fn to_entries(admins: &[Administration]) -> BTreeMap<String, AdminEntry> {
    admins
        .iter()
        .map(|a| {
            (
                safe_name(&a.name),
                AdminEntry::new(&a.domain_id, &a.id).with_name(&a.name),
            )
        })
        .collect()
}

fn read_line(stdin: &io::Stdin) -> String {
    stdin
        .lock()
        .lines()
        .next()
        .and_then(|l| l.ok())
        .map(|l| l.trim().to_string())
        .unwrap_or_default()
}

/// Authenticate with `api_key` and list the administrations it reaches.
async fn discover(
    api_key: &str,
    region: Region,
    http: reqwest::Client,
) -> Result<Vec<Administration>, YukiError> {
    eprint!("Authenticating...");
    io::stderr().flush().ok();
    let mut client = AccountingClient::with_region_and_client(region, http.clone());
    match client.authenticate(api_key).await {
        Ok(_) => eprintln!(" {}", sym_ok()),
        Err(e) => {
            eprintln!(" {}", sym_fail());
            return Err(e);
        }
    }

    eprint!("Fetching administrations...");
    io::stderr().flush().ok();
    match client.administrations().await {
        Ok(admins) => {
            eprintln!(" {}", sym_ok());
            Ok(admins)
        }
        Err(e) => {
            eprintln!(" {}", sym_fail());
            Err(e)
        }
    }
}

pub async fn run(
    api_key: Option<&str>,
    default_admin: Option<&str>,
    add: bool,
    region: Option<Region>,
) -> Result<(), YukiError> {
    run_at_with_client(
        api_key,
        default_admin,
        add,
        region,
        &Config::default_path(),
        reqwest::Client::new(),
    )
    .await
}

// Keep transport and filesystem boundaries injectable without weakening TLS or
// adding a test-only URL override to the CLI.
async fn run_at_with_client(
    api_key: Option<&str>,
    default_admin: Option<&str>,
    add: bool,
    region: Option<Region>,
    path: &std::path::Path,
    http: reqwest::Client,
) -> Result<(), YukiError> {
    let stdin = io::stdin();
    let existing = if path.exists() {
        Some(Config::load_from(path)?)
    } else {
        None
    };
    let region = region.unwrap_or_else(|| existing.as_ref().map(|c| c.region).unwrap_or_default());

    // Rotating the shared key touches nothing else, so it skips discovery entirely.
    if !add
        && let (Some(key), None) = (api_key, default_admin)
        && let Some(mut config) = existing.clone()
        && config.region == region
    {
        let key = key.trim().to_string();
        if key.is_empty() {
            return Err(YukiError::Config("API key cannot be empty".to_string()));
        }

        eprintln!("Verifying new API key...");
        let mut client = AccountingClient::with_region_and_client(region, http.clone());
        client.authenticate(&key).await?;

        config.api_key = key;
        config.region = region;
        config.save_to(path)?;
        eprintln!("{} API key updated in {}", sym_ok(), path.display());
        return Ok(());
    }

    let api_key = match api_key {
        Some(k) => k.trim().to_string(),
        None => {
            eprintln!("  {} Yuki Portal → Settings → API keys", dim("→"));
            eprint!("Yuki API key: ");
            io::stderr().flush().ok();
            read_line(&stdin)
        }
    };

    if api_key.is_empty() {
        return Err(YukiError::Config("API key cannot be empty".to_string()));
    }

    if add {
        return add_key(path, &api_key, default_admin, region, http).await;
    }

    let admins = discover(&api_key, region, http).await?;
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
        // Use the provided name directly, verifying it exists.
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

        let choice = read_line(&stdin);
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

    let administrations = to_entries(&admins);

    // A plain init replaces the administration map, so anything this key cannot reach
    // is about to disappear. Say which, rather than let a second set of books vanish.
    if let Some(previous) = &existing {
        let dropped: Vec<&str> = previous
            .administrations
            .iter()
            .filter(|(name, entry)| {
                administrations.get(*name).is_none_or(|new| {
                    new.admin_id != entry.admin_id
                        || entry.region.unwrap_or(previous.region) != region
                })
            })
            .map(|(name, _)| name.as_str())
            .collect();
        if !dropped.is_empty() {
            eprintln!();
            eprintln!(
                "{} this key does not reach {}, which init removes from the config.",
                sym_fail(),
                dropped.join(", ")
            );
            eprintln!(
                "  {}",
                dim("Run 'yuki init --add --api-key <key>' instead to keep both.")
            );
            eprintln!();
        }
    }

    let config = Config {
        api_key,
        region,
        default_admin: default_name.clone(),
        administrations,
        // Preserve unmatched_ignore from the existing config if present.
        unmatched_ignore: existing.map(|c| c.unmatched_ignore).unwrap_or_default(),
    };

    config.save_to(path)?;

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

/// Merge a second key's administrations into an existing configuration.
///
/// A Yuki access key is scoped to the administration it was created in, so this is
/// how a second set of books becomes reachable: the key is stored against the
/// administrations it opens, and every command that targets one authenticates with it.
async fn add_key(
    path: &std::path::Path,
    api_key: &str,
    default_admin: Option<&str>,
    region: Region,
    http: reqwest::Client,
) -> Result<(), YukiError> {
    // Distinguish "nothing to add to" from a config that exists but will not parse;
    // the second is a different problem and keeps its own error.
    if !path.exists() {
        return Err(YukiError::Config(format!(
            "no configuration at {}. Run 'yuki init' first; --add extends an existing one.",
            path.display()
        )));
    }
    let mut config = Config::load_from(path)?;

    let admins = discover(api_key, region, http).await?;
    if admins.is_empty() {
        return Err(YukiError::NotFound(
            "no administrations found for this API key".to_string(),
        ));
    }

    let (added, updated) = config.merge_administrations(
        to_entries(&admins)
            .into_iter()
            .map(|(name, entry)| (name, entry.with_region(region))),
        api_key,
    );

    if let Some(name) = default_admin {
        let key = safe_name(name);
        // Resolve only among the profiles just discovered. A name collision can
        // suffix the new profile; selecting the unsuffixed existing name would
        // silently activate a different region's books.
        let selected = added
            .iter()
            .chain(&updated)
            .find(|profile| {
                *profile == &key
                    || config.administrations[*profile]
                        .name
                        .as_deref()
                        .is_some_and(|display| safe_name(display) == key)
            })
            .ok_or_else(|| {
                YukiError::Config(format!("administration not discovered by this key: {name}"))
            })?;
        config.default_admin = selected.clone();
    }

    config.save_to(path)?;

    eprintln!();
    for name in &added {
        eprintln!("{} added {name}", sym_ok());
    }
    for name in &updated {
        eprintln!("{} updated {name}", sym_ok());
    }
    eprintln!(
        "{} Configuration saved to {} (default: {})",
        sym_ok(),
        path.display(),
        config.default_admin
    );
    eprintln!();
    eprintln!("{}:", bold("Next steps"));
    eprintln!("  yuki admin list                {}", dim("# verify both"));
    for name in added.iter().chain(updated.iter()) {
        eprintln!("  yuki --admin {name} documents list");
    }
    eprintln!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_name_lowercases_and_replaces_punctuation() {
        assert_eq!(safe_name("Example Holding B.V."), "example_holding_b_v_");
        // Digits survive; only non-alphanumerics become underscores.
        assert_eq!(safe_name("Acme 42 B.V."), "acme_42_b_v_");
    }

    #[test]
    fn to_entries_records_the_display_name() {
        let admins = vec![Administration {
            name: "Example Holding B.V.".into(),
            id: "admin-1".into(),
            domain_id: "domain-1".into(),
        }];
        let entries = to_entries(&admins);
        let entry = &entries["example_holding_b_v_"];
        assert_eq!(entry.name.as_deref(), Some("Example Holding B.V."));
        assert_eq!(entry.admin_id, "admin-1");
        assert_eq!(entry.domain_id, "domain-1");
        // Discovery does not decide which key an entry belongs to; merging does.
        assert_eq!(entry.api_key, None);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::cli::test_support::{Reply, SoapServer};
    const BE: &str = "api.yukiworks.be";
    const NL: &str = "api.yukiworks.nl";

    #[tokio::test]
    async fn belgian_setup_persists_region_then_doctor_and_scheme_use_saved_credentials() {
        let server = SoapServer::start(vec![
            Reply::auth(BE, "Accounting", "be-key"),
            Reply::admins(BE, "Belgian Company", "be-domain", "be-admin"),
            Reply::auth(BE, "Accounting", "be-key"),
            Reply::new(BE, "Accounting", "SetCurrentDomain", "").containing("<yuki:domainID>be-domain</yuki:domainID>"),
            Reply::auth(BE, "AccountingInfo", "be-key"),
            Reply::new(BE, "AccountingInfo", "GetGLAccountScheme", "<GlAccount><code>400</code><type>1</type><descripton>Customers</descripton></GlAccount>")
                .containing("<yuki:administrationID>be-admin</yuki:administrationID>"),
        ]);
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        run_at_with_client(
            Some("be-key"),
            None,
            false,
            Some(Region::Be),
            &path,
            server.http.clone(),
        )
        .await
        .unwrap();
        let config = Config::load_from(&path).unwrap();
        assert_eq!(config.region, Region::Be);
        assert_eq!(config.default_admin, "belgian_company");
        assert_eq!(config.target(None).unwrap().admin_id, "be-admin");
        crate::cli::account::doctor_with_client(
            &config,
            None,
            false,
            Some("json"),
            true,
            server.http.clone(),
        )
        .await
        .unwrap();
        crate::cli::accounts::scheme_with_client(&config, None, Some("json"), server.http.clone())
            .await
            .unwrap();
        server.finish();
    }

    #[tokio::test]
    async fn adding_belgian_profile_and_rotating_shared_dutch_key_keeps_regions_isolated() {
        let server = SoapServer::start(vec![
            Reply::auth(NL, "Accounting", "nl-key"),
            Reply::admins(NL, "Company", "nl-domain", "same-admin-id"),
            Reply::auth(BE, "Accounting", "be-key"),
            Reply::admins(BE, "Company", "be-domain", "same-admin-id"),
            Reply::auth(NL, "Accounting", "rotated-nl-key"),
            Reply::auth(BE, "Accounting", "be-key"),
            Reply::new(BE, "Accounting", "SetCurrentDomain", "")
                .containing("<yuki:domainID>be-domain</yuki:domainID>"),
            Reply::auth(NL, "Accounting", "rotated-nl-key"),
            Reply::new(NL, "Accounting", "SetCurrentDomain", "")
                .containing("<yuki:domainID>nl-domain</yuki:domainID>"),
        ]);
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        run_at_with_client(
            Some("nl-key"),
            None,
            false,
            None,
            &path,
            server.http.clone(),
        )
        .await
        .unwrap();
        run_at_with_client(
            Some("be-key"),
            Some("Company"),
            true,
            Some(Region::Be),
            &path,
            server.http.clone(),
        )
        .await
        .unwrap();
        run_at_with_client(
            Some("rotated-nl-key"),
            None,
            false,
            None,
            &path,
            server.http.clone(),
        )
        .await
        .unwrap();
        let config = Config::load_from(&path).unwrap();
        assert_eq!(config.administrations.len(), 2);
        assert_eq!(config.region, Region::Nl);
        assert_eq!(config.default_admin, "company_2");
        assert_eq!(config.target(Some("company_2")).unwrap().api_key, "be-key");
        crate::cli::setup_domain_with_client(&config, Some("company_2"), server.http.clone())
            .await
            .unwrap();
        crate::cli::setup_domain_with_client(&config, Some("company"), server.http.clone())
            .await
            .unwrap();
        server.finish();
    }

    #[tokio::test]
    async fn changing_region_rediscovers_ids_and_failed_setup_preserves_file() {
        let server = SoapServer::start(vec![
            Reply::auth(NL, "Accounting", "nl-key"),
            Reply::admins(NL, "Company", "nl-domain", "nl-admin"),
            Reply::rejected(BE, "bad-be-key"),
            Reply::auth(BE, "Accounting", "be-key"),
            Reply::admins(BE, "Company", "be-domain", "be-admin"),
        ]);
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        run_at_with_client(
            Some("nl-key"),
            None,
            false,
            None,
            &path,
            server.http.clone(),
        )
        .await
        .unwrap();
        let original = std::fs::read(&path).unwrap();
        let error = run_at_with_client(
            Some("bad-be-key"),
            None,
            false,
            Some(Region::Be),
            &path,
            server.http.clone(),
        )
        .await
        .unwrap_err();
        assert!(matches!(error, YukiError::AuthFailed(_)), "{error}");
        assert_eq!(std::fs::read(&path).unwrap(), original);
        run_at_with_client(
            Some("be-key"),
            None,
            false,
            Some(Region::Be),
            &path,
            server.http.clone(),
        )
        .await
        .unwrap();
        let config = Config::load_from(&path).unwrap();
        assert_eq!(config.region, Region::Be);
        assert_eq!(config.target(None).unwrap().domain_id, "be-domain");
        assert_eq!(config.target(None).unwrap().admin_id, "be-admin");
        server.finish();
    }
}
