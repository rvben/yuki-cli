use serde_json::Value;
use yuki_cli::output::{ListOptions, OutputFormat, format_json, format_table, select_fields};

#[test]
fn format_json_produces_items_envelope() {
    let data = vec![
        vec!["Alice".to_string(), "100".to_string()],
        vec!["Bob".to_string(), "200".to_string()],
    ];
    let headers = vec!["Name".to_string(), "Amount".to_string()];
    let result = format_json(&headers, &data);
    let parsed: Value = serde_json::from_str(&result).unwrap();
    // Must use items envelope per clispec v0.2
    let items = parsed["items"].as_array().expect("must have 'items' array");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["Name"], "Alice");
    assert_eq!(items[0]["Amount"], "100");
    assert_eq!(parsed["total"], 2);
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
fn format_json_empty_data_uses_envelope() {
    let data: Vec<Vec<String>> = vec![];
    let headers = vec!["Name".to_string()];
    let result = format_json(&headers, &data);
    let parsed: Value = serde_json::from_str(&result).unwrap();
    let items = parsed["items"].as_array().expect("must have 'items' array");
    assert_eq!(items.len(), 0);
    assert_eq!(parsed["total"], 0);
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
fn output_format_from_flag_text() {
    let fmt = OutputFormat::from_flag(Some("text"), false);
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
fn output_format_auto_flag_defers_to_tty() {
    let fmt_tty = OutputFormat::from_flag(Some("auto"), true);
    assert!(matches!(fmt_tty, OutputFormat::Table));
    let fmt_pipe = OutputFormat::from_flag(Some("auto"), false);
    assert!(matches!(fmt_pipe, OutputFormat::Json));
}

#[test]
fn format_error_json_uses_spec_envelope() {
    let result = yuki_cli::output::format_error_json("something broke", "auth_failed");
    let parsed: Value = serde_json::from_str(&result).unwrap();
    // Must match clispec v0.2 structured error envelope
    let err = &parsed["error"];
    assert_eq!(err["kind"], "auth_failed");
    assert_eq!(err["message"], "something broke");
    // Must not use old flat format
    assert!(
        parsed.get("code").is_none(),
        "must not have top-level 'code' field"
    );
}

// --- select_fields -------------------------------------------------------
// Regression: `--fields` was parsed by clap but never read, so every list
// command returned all columns and `--fields bogus` exited 0.

fn sample() -> (Vec<String>, Vec<Vec<String>>) {
    let headers = vec!["ID".to_string(), "Date".to_string(), "Amount".to_string()];
    let rows = vec![
        vec![
            "a1".to_string(),
            "2026-01-01".to_string(),
            "10.00".to_string(),
        ],
        vec![
            "b2".to_string(),
            "2026-02-02".to_string(),
            "20.00".to_string(),
        ],
    ];
    (headers, rows)
}

#[test]
fn select_fields_narrows_columns_and_keeps_requested_order() {
    let (mut headers, mut rows) = sample();
    let opts = ListOptions {
        fields: Some("Amount,ID"),
        ..Default::default()
    };
    select_fields(&mut headers, &mut rows, &opts).unwrap();
    assert_eq!(headers, vec!["Amount".to_string(), "ID".to_string()]);
    assert_eq!(rows[0], vec!["10.00".to_string(), "a1".to_string()]);
    assert_eq!(rows[1], vec!["20.00".to_string(), "b2".to_string()]);
}

#[test]
fn select_fields_is_case_insensitive_and_trims() {
    let (mut headers, mut rows) = sample();
    let opts = ListOptions {
        fields: Some(" date , AMOUNT "),
        ..Default::default()
    };
    select_fields(&mut headers, &mut rows, &opts).unwrap();
    assert_eq!(headers, vec!["Date".to_string(), "Amount".to_string()]);
}

#[test]
fn select_fields_rejects_unknown_field() {
    let (mut headers, mut rows) = sample();
    let opts = ListOptions {
        fields: Some("bogus"),
        ..Default::default()
    };
    let err = select_fields(&mut headers, &mut rows, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown field"), "got: {msg}");
    assert!(
        msg.contains("Amount"),
        "error should list valid fields: {msg}"
    );
}

#[test]
fn select_fields_rejects_partially_unknown_selection() {
    let (mut headers, mut rows) = sample();
    let opts = ListOptions {
        fields: Some("ID,nope"),
        ..Default::default()
    };
    assert!(select_fields(&mut headers, &mut rows, &opts).is_err());
}

#[test]
fn select_fields_without_spec_is_a_noop() {
    let (mut headers, mut rows) = sample();
    let opts = ListOptions::default();
    select_fields(&mut headers, &mut rows, &opts).unwrap();
    assert_eq!(headers.len(), 3);
    assert_eq!(rows[0].len(), 3);
}
