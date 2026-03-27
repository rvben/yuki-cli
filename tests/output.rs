use serde_json::Value;
use yuki_cli::output::{OutputFormat, format_json, format_table};

#[test]
fn format_json_produces_valid_json() {
    let data = vec![
        vec!["Alice".to_string(), "100".to_string()],
        vec!["Bob".to_string(), "200".to_string()],
    ];
    let headers = vec!["Name".to_string(), "Amount".to_string()];
    let result = format_json(&headers, &data);
    let parsed: Vec<Value> = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0]["Name"], "Alice");
    assert_eq!(parsed[0]["Amount"], "100");
}

#[test]
fn format_table_contains_headers_and_data() {
    let data = vec![vec!["Alice".to_string(), "100".to_string()]];
    let headers = vec!["Name".to_string(), "Amount".to_string()];
    let result = format_table(&headers, &data);
    assert!(result.contains("Name"));
    assert!(result.contains("Amount"));
    assert!(result.contains("Alice"));
    assert!(result.contains("100"));
}

#[test]
fn format_json_empty_data() {
    let data: Vec<Vec<String>> = vec![];
    let headers = vec!["Name".to_string()];
    let result = format_json(&headers, &data);
    let parsed: Vec<Value> = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed.len(), 0);
}

#[test]
fn output_format_from_flag_json() {
    let fmt = OutputFormat::from_flag(Some("json"), true);
    assert!(matches!(fmt, OutputFormat::Json));
}

#[test]
fn output_format_from_flag_table() {
    let fmt = OutputFormat::from_flag(Some("table"), false);
    assert!(matches!(fmt, OutputFormat::Table));
}

#[test]
fn output_format_auto_tty() {
    let fmt = OutputFormat::from_flag(None, true);
    assert!(matches!(fmt, OutputFormat::Table));
}

#[test]
fn output_format_auto_pipe() {
    let fmt = OutputFormat::from_flag(None, false);
    assert!(matches!(fmt, OutputFormat::Json));
}

#[test]
fn format_error_json() {
    let result = yuki_cli::output::format_error_json("something broke", "GENERAL_ERROR");
    let parsed: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["error"], "something broke");
    assert_eq!(parsed["code"], "GENERAL_ERROR");
}
