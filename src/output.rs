use comfy_table::{Table, presets::UTF8_FULL_CONDENSED};
use serde_json::{Map, Value, json};

pub enum OutputFormat {
    Table,
    Json,
}

impl OutputFormat {
    pub fn from_flag(flag: Option<&str>, is_tty: bool) -> Self {
        match flag {
            Some("json") => Self::Json,
            Some("table") => Self::Table,
            Some(_) => Self::Table,
            None if is_tty => Self::Table,
            None => Self::Json,
        }
    }
}

pub fn format_json(headers: &[String], rows: &[Vec<String>]) -> String {
    let items: Vec<Value> = rows
        .iter()
        .map(|row| {
            let mut map = Map::new();
            for (i, header) in headers.iter().enumerate() {
                let val = row.get(i).cloned().unwrap_or_default();
                map.insert(header.clone(), Value::String(val));
            }
            Value::Object(map)
        })
        .collect();
    serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".into())
}

pub fn format_table(headers: &[String], rows: &[Vec<String>]) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(headers);
    for row in rows {
        table.add_row(row);
    }
    table.to_string()
}

pub fn format_error_json(message: &str, code: &str) -> String {
    serde_json::to_string_pretty(&json!({
        "error": message,
        "code": code,
    }))
    .unwrap_or_else(|_| format!(r#"{{"error":"{message}","code":"{code}"}}"#))
}

pub fn is_tty() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stdout())
}
