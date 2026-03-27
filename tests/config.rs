use yuki_cli::config::{AdminEntry, Config};

#[test]
fn loads_valid_config() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        r#"
api_key = "00000000-0000-0000-0000-000000000000"
default_admin = "company_a"

[administrations.company_a]
domain_id = "uuid-1"
admin_id = "admin-1"

[administrations.company_b]
domain_id = "uuid-2"
admin_id = "admin-2"
"#,
    )
    .unwrap();

    let config = Config::load_from(&path).unwrap();
    assert_eq!(config.api_key, "00000000-0000-0000-0000-000000000000");
    assert_eq!(config.default_admin, "company_a");
    assert_eq!(config.administrations.len(), 2);
    assert_eq!(config.administrations["company_a"].domain_id, "uuid-1");
    assert_eq!(config.administrations["company_a"].admin_id, "admin-1");
}

#[test]
fn returns_error_for_missing_file() {
    let result = Config::load_from(std::path::Path::new("/nonexistent/config.toml"));
    assert!(result.is_err());
}

#[test]
fn returns_error_for_invalid_toml() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "not valid toml {{{{").unwrap();

    let result = Config::load_from(&path);
    assert!(result.is_err());
}

#[test]
fn saves_config_roundtrip() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("config.toml");

    let config = Config {
        api_key: "test-key".into(),
        default_admin: "my_company".into(),
        administrations: [(
            "my_company".into(),
            AdminEntry {
                domain_id: "uuid-123".into(),
                admin_id: "admin-123".into(),
            },
        )]
        .into(),
    };

    config.save_to(&path).unwrap();
    let loaded = Config::load_from(&path).unwrap();
    assert_eq!(loaded.api_key, "test-key");
    assert_eq!(loaded.default_admin, "my_company");
    assert_eq!(loaded.administrations["my_company"].domain_id, "uuid-123");
    assert_eq!(loaded.administrations["my_company"].admin_id, "admin-123");
}

#[test]
fn config_path_returns_xdg_path() {
    let path = Config::default_path();
    let path_str = path.to_string_lossy();
    assert!(path_str.ends_with(".config/yuki/config.toml") || path_str.contains("yuki"));
}

#[test]
fn resolve_admin_returns_entry_for_known_name() {
    let config = Config {
        api_key: "key".into(),
        default_admin: "co_a".into(),
        administrations: [(
            "co_a".into(),
            AdminEntry {
                domain_id: "uuid-a".into(),
                admin_id: "admin-a".into(),
            },
        )]
        .into(),
    };
    let entry = config.resolve_admin(None).unwrap();
    assert_eq!(entry.domain_id, "uuid-a");
    assert_eq!(entry.admin_id, "admin-a");
}

#[test]
fn resolve_admin_override_takes_precedence() {
    let config = Config {
        api_key: "key".into(),
        default_admin: "co_a".into(),
        administrations: [
            (
                "co_a".into(),
                AdminEntry {
                    domain_id: "uuid-a".into(),
                    admin_id: "admin-a".into(),
                },
            ),
            (
                "co_b".into(),
                AdminEntry {
                    domain_id: "uuid-b".into(),
                    admin_id: "admin-b".into(),
                },
            ),
        ]
        .into(),
    };
    let entry = config.resolve_admin(Some("co_b")).unwrap();
    assert_eq!(entry.domain_id, "uuid-b");
    assert_eq!(entry.admin_id, "admin-b");
}

#[test]
fn resolve_admin_errors_on_unknown_name() {
    let config = Config {
        api_key: "key".into(),
        default_admin: "co_a".into(),
        administrations: [(
            "co_a".into(),
            AdminEntry {
                domain_id: "uuid-a".into(),
                admin_id: "admin-a".into(),
            },
        )]
        .into(),
    };
    assert!(config.resolve_admin(Some("unknown")).is_err());
}
