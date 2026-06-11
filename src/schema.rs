use serde_json::{Value, json};

pub fn generate() -> Value {
    json!({
        "clispec": "0.2",
        "name": "yuki",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "CLI client for the Yuki bookkeeping API",
        "global_args": [
            {
                "name": "--output",
                "type": "string",
                "description": "Output format: auto, text, or json.",
                "enum": ["auto", "text", "json"],
                "default": "auto"
            },
            {
                "name": "--admin",
                "type": "string",
                "description": "Override the active administration by name."
            },
            {
                "name": "--quiet",
                "type": "boolean",
                "description": "Suppress all output except errors.",
                "default": false
            },
            {
                "name": "--yes",
                "type": "boolean",
                "description": "Skip confirmation prompts (for use in scripts and pipelines).",
                "default": false
            }
        ],
        "commands": [
            {
                "name": "admin list",
                "description": "List all available administrations.",
                "mutating": false,
                "args": [
                    {"name": "--limit", "type": "integer", "required": false, "description": "Maximum number of results to return."},
                    {"name": "--offset", "type": "integer", "required": false, "description": "Number of results to skip (for pagination)."},
                    {"name": "--fields", "type": "string", "required": false, "description": "Comma-separated list of fields to include in output."}
                ],
                "output_fields": [
                    {"name": "name", "type": "string"},
                    {"name": "domain_id", "type": "string"},
                    {"name": "admin_id", "type": "string"},
                    {"name": "default", "type": "boolean"}
                ]
            },
            {
                "name": "admin switch",
                "description": "Switch the active administration.",
                "mutating": true,
                "args": [
                    {"name": "name", "type": "string", "required": true, "description": "Name of the administration to activate."}
                ]
            },
            {
                "name": "vat returns",
                "description": "List VAT returns for a given year.",
                "mutating": false,
                "args": [
                    {"name": "year", "type": "string", "required": false, "description": "Fiscal year (e.g. 2025)."}
                ],
                "output_fields": [
                    {"name": "period", "type": "string"},
                    {"name": "status", "type": "string"},
                    {"name": "amount", "type": "string"}
                ]
            },
            {
                "name": "vat codes",
                "description": "List active VAT codes.",
                "mutating": false,
                "args": [
                    {"name": "--limit", "type": "integer", "required": false, "description": "Maximum number of results to return."},
                    {"name": "--offset", "type": "integer", "required": false, "description": "Number of results to skip (for pagination)."},
                    {"name": "--fields", "type": "string", "required": false, "description": "Comma-separated list of fields to include in output."}
                ],
                "output_fields": [
                    {"name": "code", "type": "string"},
                    {"name": "description", "type": "string"},
                    {"name": "percentage", "type": "string"}
                ]
            },
            {
                "name": "contacts search",
                "description": "Search contacts by name or other criteria.",
                "mutating": false,
                "args": [
                    {"name": "query", "type": "string", "required": true, "description": "Search query."}
                ],
                "output_fields": [
                    {"name": "id", "type": "string"},
                    {"name": "name", "type": "string"},
                    {"name": "type", "type": "string"}
                ]
            },
            {
                "name": "contacts list",
                "description": "List contacts filtered by type.",
                "mutating": false,
                "args": [
                    {"name": "--contact-type", "type": "string", "required": false, "description": "Contact type (e.g. customer, supplier)."},
                    {"name": "--limit", "type": "integer", "required": false, "description": "Maximum number of results to return."},
                    {"name": "--offset", "type": "integer", "required": false, "description": "Number of results to skip (for pagination)."},
                    {"name": "--fields", "type": "string", "required": false, "description": "Comma-separated list of fields to include in output."}
                ],
                "output_fields": [
                    {"name": "id", "type": "string"},
                    {"name": "name", "type": "string"},
                    {"name": "type", "type": "string"},
                    {"name": "email", "type": "string"}
                ]
            },
            {
                "name": "accounts balance",
                "description": "Show the balance of a general ledger account for a period.",
                "mutating": false,
                "args": [
                    {"name": "--account", "type": "string", "required": false, "description": "GL account code."},
                    {"name": "--period", "type": "string", "required": false, "description": "Accounting period (e.g. 2025-01)."}
                ],
                "output_fields": [
                    {"name": "account", "type": "string"},
                    {"name": "description", "type": "string"},
                    {"name": "balance", "type": "string"}
                ]
            },
            {
                "name": "accounts transactions",
                "description": "List transactions for a general ledger account.",
                "mutating": false,
                "args": [
                    {"name": "--account", "type": "string", "required": false, "description": "GL account code."},
                    {"name": "--period", "type": "string", "required": false, "description": "Accounting period (e.g. 2025-01)."},
                    {"name": "--limit", "type": "integer", "required": false, "description": "Maximum number of results to return."},
                    {"name": "--offset", "type": "integer", "required": false, "description": "Number of results to skip (for pagination)."},
                    {"name": "--fields", "type": "string", "required": false, "description": "Comma-separated list of fields to include in output."}
                ],
                "output_fields": [
                    {"name": "date", "type": "string"},
                    {"name": "description", "type": "string"},
                    {"name": "amount", "type": "string"},
                    {"name": "reference", "type": "string"}
                ]
            },
            {
                "name": "accounts scheme",
                "description": "Show the chart of accounts (GL account scheme).",
                "mutating": false,
                "args": [
                    {"name": "--limit", "type": "integer", "required": false, "description": "Maximum number of results to return."},
                    {"name": "--offset", "type": "integer", "required": false, "description": "Number of results to skip (for pagination)."},
                    {"name": "--fields", "type": "string", "required": false, "description": "Comma-separated list of fields to include in output."}
                ],
                "output_fields": [
                    {"name": "code", "type": "string"},
                    {"name": "description", "type": "string"},
                    {"name": "type", "type": "string"}
                ]
            },
            {
                "name": "accounts revenue",
                "description": "Show net revenue for a period.",
                "mutating": false,
                "args": [
                    {"name": "--period", "type": "string", "required": false, "description": "Accounting period (e.g. 2025, 2025-Q1, 2025-01)."}
                ],
                "output_fields": [
                    {"name": "period", "type": "string"},
                    {"name": "revenue", "type": "string"}
                ]
            },
            {
                "name": "accounts start-balance",
                "description": "Show opening balances per GL account for a book year.",
                "mutating": false,
                "args": [
                    {"name": "--year", "type": "string", "required": false, "description": "Book year (e.g. 2025)."},
                    {"name": "--limit", "type": "integer", "required": false, "description": "Maximum number of results to return."},
                    {"name": "--offset", "type": "integer", "required": false, "description": "Number of results to skip (for pagination)."},
                    {"name": "--fields", "type": "string", "required": false, "description": "Comma-separated list of fields to include in output."}
                ],
                "output_fields": [
                    {"name": "account", "type": "string"},
                    {"name": "description", "type": "string"},
                    {"name": "balance", "type": "string"}
                ]
            },
            {
                "name": "projects list",
                "description": "List all projects.",
                "mutating": false,
                "args": [
                    {"name": "--limit", "type": "integer", "required": false, "description": "Maximum number of results to return."},
                    {"name": "--offset", "type": "integer", "required": false, "description": "Number of results to skip (for pagination)."},
                    {"name": "--fields", "type": "string", "required": false, "description": "Comma-separated list of fields to include in output."}
                ],
                "output_fields": [
                    {"name": "code", "type": "string"},
                    {"name": "name", "type": "string"},
                    {"name": "status", "type": "string"}
                ]
            },
            {
                "name": "projects balance",
                "description": "Show balance for a project.",
                "mutating": false,
                "args": [
                    {"name": "project", "type": "string", "required": true, "description": "Project code."},
                    {"name": "--account", "type": "string", "required": false, "description": "GL account code filter."},
                    {"name": "--period", "type": "string", "required": false, "description": "Accounting period (e.g. 2025, 2025-Q1)."}
                ],
                "output_fields": [
                    {"name": "account", "type": "string"},
                    {"name": "description", "type": "string"},
                    {"name": "balance", "type": "string"}
                ]
            },
            {
                "name": "invoices list",
                "description": "List invoices, optionally filtered by period and type.",
                "mutating": false,
                "args": [
                    {"name": "--period", "type": "string", "required": false, "description": "Accounting period (e.g. 2025-01)."},
                    {"name": "--invoice-type", "type": "string", "required": false, "description": "Invoice type filter (e.g. sales, purchase)."},
                    {"name": "--limit", "type": "integer", "required": false, "description": "Maximum number of results to return."},
                    {"name": "--offset", "type": "integer", "required": false, "description": "Number of results to skip (for pagination)."},
                    {"name": "--fields", "type": "string", "required": false, "description": "Comma-separated list of fields to include in output."}
                ],
                "output_fields": [
                    {"name": "id", "type": "string"},
                    {"name": "date", "type": "string"},
                    {"name": "contact", "type": "string"},
                    {"name": "amount", "type": "string"},
                    {"name": "status", "type": "string"}
                ]
            },
            {
                "name": "invoices show",
                "description": "Show details for a single invoice.",
                "mutating": false,
                "args": [
                    {"name": "id", "type": "string", "required": true, "description": "Invoice ID."}
                ],
                "output_fields": [
                    {"name": "id", "type": "string"},
                    {"name": "date", "type": "string"},
                    {"name": "contact", "type": "string"},
                    {"name": "amount", "type": "string"},
                    {"name": "status", "type": "string"}
                ]
            },
            {
                "name": "invoices document",
                "description": "Show the document linked to a transaction.",
                "mutating": false,
                "args": [
                    {"name": "id", "type": "string", "required": true, "description": "Transaction ID."}
                ],
                "output_fields": [
                    {"name": "id", "type": "string"},
                    {"name": "filename", "type": "string"},
                    {"name": "url", "type": "string"}
                ]
            },
            {
                "name": "documents list",
                "description": "List documents in a folder or of a given type.",
                "mutating": false,
                "args": [
                    {"name": "--folder", "type": "string", "required": false, "description": "Archive folder name."},
                    {"name": "--doc-type", "type": "string", "required": false, "description": "Document type filter."},
                    {"name": "--limit", "type": "integer", "required": false, "description": "Maximum number of results to return."},
                    {"name": "--offset", "type": "integer", "required": false, "description": "Number of results to skip (for pagination)."},
                    {"name": "--fields", "type": "string", "required": false, "description": "Comma-separated list of fields to include in output."}
                ],
                "output_fields": [
                    {"name": "id", "type": "string"},
                    {"name": "filename", "type": "string"},
                    {"name": "folder", "type": "string"},
                    {"name": "date", "type": "string"}
                ]
            },
            {
                "name": "documents search",
                "description": "Search documents by a query string.",
                "mutating": false,
                "args": [
                    {"name": "query", "type": "string", "required": true, "description": "Search query."}
                ],
                "output_fields": [
                    {"name": "id", "type": "string"},
                    {"name": "filename", "type": "string"},
                    {"name": "folder", "type": "string"},
                    {"name": "date", "type": "string"}
                ]
            },
            {
                "name": "documents exists",
                "description": "Check if an invoice exists in the archive (by amount, date, and optional contact).",
                "mutating": false,
                "args": [
                    {"name": "--amount", "type": "number", "required": true, "description": "Invoice amount to search for."},
                    {"name": "--date", "type": "string", "required": true, "description": "Invoice date (YYYY-MM-DD). Matches within +/-7 days."},
                    {"name": "--contact", "type": "string", "required": false, "description": "Contact/supplier name to narrow the search."}
                ],
                "output_fields": [
                    {"name": "exists", "type": "boolean"},
                    {"name": "id", "type": "string"},
                    {"name": "filename", "type": "string"}
                ]
            },
            {
                "name": "check btw",
                "description": "Check outstanding BTW (VAT) items for a period.",
                "mutating": false,
                "args": [
                    {"name": "period", "type": "string", "required": false, "description": "Accounting period (e.g. 2025-01)."}
                ],
                "output_fields": [
                    {"name": "account", "type": "string"},
                    {"name": "description", "type": "string"},
                    {"name": "amount", "type": "string"}
                ]
            },
            {
                "name": "check unmatched",
                "description": "Find bank transactions without matching booked invoices.",
                "mutating": false,
                "args": [
                    {"name": "--period", "type": "string", "required": false, "description": "Accounting period (e.g. 2025-Q1)."},
                    {"name": "--bank-account", "type": "string", "required": false, "description": "GL account code for the bank account.", "default": "11001"}
                ],
                "output_fields": [
                    {"name": "date", "type": "string"},
                    {"name": "description", "type": "string"},
                    {"name": "amount", "type": "string"}
                ]
            },
            {
                "name": "check outstanding",
                "description": "Check if a specific invoice reference is still outstanding.",
                "mutating": false,
                "args": [
                    {"name": "reference", "type": "string", "required": true, "description": "Invoice reference to check."}
                ],
                "output_fields": [
                    {"name": "reference", "type": "string"},
                    {"name": "outstanding", "type": "boolean"},
                    {"name": "amount", "type": "string"}
                ]
            },
            {
                "name": "upload file",
                "description": "Upload a document with optional invoice metadata.",
                "mutating": true,
                "args": [
                    {"name": "file", "type": "path", "required": true, "description": "Path to the file to upload."},
                    {"name": "--folder", "type": "string", "required": false, "description": "Target folder.", "default": "uitzoeken"},
                    {"name": "--amount", "type": "number", "required": false, "description": "Invoice amount (e.g. 114.27)."},
                    {"name": "--category", "type": "string", "required": false, "description": "Cost category ID (e.g. 45100)."},
                    {"name": "--payment-method", "type": "string", "required": false, "description": "Payment method ID."},
                    {"name": "--project", "type": "string", "required": false, "description": "Project ID."},
                    {"name": "--remarks", "type": "string", "required": false, "description": "Remarks or notes."},
                    {"name": "--currency", "type": "string", "required": false, "description": "Currency code.", "default": "EUR"}
                ],
                "output_fields": [
                    {"name": "id", "type": "string"},
                    {"name": "filename", "type": "string"},
                    {"name": "folder", "type": "string"}
                ]
            },
            {
                "name": "upload categories",
                "description": "List available cost categories.",
                "mutating": false,
                "output_fields": [
                    {"name": "id", "type": "string"},
                    {"name": "description", "type": "string"}
                ]
            },
            {
                "name": "upload payment-methods",
                "description": "List available payment methods.",
                "mutating": false,
                "output_fields": [
                    {"name": "id", "type": "string"},
                    {"name": "description", "type": "string"}
                ]
            },
            {
                "name": "init",
                "description": "Initialize yuki configuration for this machine.",
                "mutating": true,
                "args": [
                    {"name": "--api-key", "type": "string", "required": false, "description": "API key (skips interactive prompt if provided)."},
                    {"name": "--default-admin", "type": "string", "required": false, "description": "Default administration name (auto-selects if only one available)."}
                ]
            },
            {
                "name": "schema",
                "description": "Output JSON schema for agent integration.",
                "mutating": false
            },
            {
                "name": "completions",
                "description": "Generate shell completions.",
                "mutating": false,
                "args": [
                    {"name": "shell", "type": "string", "required": true, "description": "Shell to generate completions for.", "enum": ["bash", "fish", "zsh", "powershell", "elvish"]}
                ]
            }
        ],
        "errors": [
            {
                "kind": "auth_failed",
                "exit_code": 2,
                "retryable": false,
                "description": "Authentication failed: invalid or expired API key."
            },
            {
                "kind": "not_found",
                "exit_code": 3,
                "retryable": false,
                "description": "The requested resource was not found."
            },
            {
                "kind": "rate_limited",
                "exit_code": 4,
                "retryable": true,
                "description": "API rate limit exceeded (1000 calls/day)."
            },
            {
                "kind": "config_error",
                "exit_code": 1,
                "retryable": false,
                "description": "Configuration error: missing or invalid config file."
            },
            {
                "kind": "conflict",
                "exit_code": 5,
                "retryable": false,
                "description": "The operation conflicts with existing data (e.g. duplicate document)."
            },
            {
                "kind": "confirmation_required",
                "exit_code": 2,
                "retryable": false,
                "description": "A mutating command was invoked non-interactively without --yes."
            },
            {
                "kind": "error",
                "exit_code": 1,
                "retryable": false,
                "description": "An unexpected error occurred."
            }
        ]
    })
}

pub fn print_schema() {
    let schema = generate();
    println!(
        "{}",
        serde_json::to_string_pretty(&schema).expect("serialize schema")
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonschema::Validator;
    use serde_json::Value;

    #[test]
    fn schema_is_valid_json() {
        let schema = generate();
        let serialized = serde_json::to_string_pretty(&schema).unwrap();
        let _: Value = serde_json::from_str(&serialized).unwrap();
    }

    #[test]
    fn schema_has_required_top_level_keys() {
        let schema = generate();
        assert!(schema.get("clispec").is_some(), "missing clispec field");
        assert!(schema.get("name").is_some(), "missing name field");
        assert!(schema.get("version").is_some(), "missing version field");
        assert!(schema.get("commands").is_some(), "missing commands field");
        assert!(schema.get("errors").is_some(), "missing errors field");
        assert!(
            schema.get("global_args").is_some(),
            "missing global_args field"
        );
    }

    #[test]
    fn schema_clispec_version() {
        let schema = generate();
        assert_eq!(schema["clispec"], "0.2");
    }

    #[test]
    fn schema_commands_is_array() {
        let schema = generate();
        assert!(schema["commands"].is_array(), "commands must be an array");
    }

    #[test]
    fn schema_all_commands_have_mutating_field() {
        let schema = generate();
        let commands = schema["commands"].as_array().unwrap();
        for cmd in commands {
            let name = cmd["name"].as_str().unwrap_or("unknown");
            assert!(
                cmd.get("mutating").is_some(),
                "command '{name}' is missing 'mutating' field"
            );
        }
    }

    #[test]
    fn schema_errors_has_conflict_kind() {
        let schema = generate();
        let errors = schema["errors"].as_array().unwrap();
        let has_conflict = errors.iter().any(|e| e["kind"] == "conflict");
        assert!(has_conflict, "errors array must include kind='conflict'");
    }

    #[test]
    fn schema_errors_have_exit_codes() {
        let schema = generate();
        let errors = schema["errors"].as_array().unwrap();
        for err in errors {
            let kind = err["kind"].as_str().unwrap_or("unknown");
            assert!(
                err.get("exit_code").is_some(),
                "error '{kind}' is missing 'exit_code'"
            );
        }
    }

    #[test]
    fn schema_global_args_has_output_flag() {
        let schema = generate();
        let global_args = schema["global_args"].as_array().unwrap();
        let has_output = global_args.iter().any(|a| a["name"] == "--output");
        assert!(has_output, "global_args must include --output flag");
    }

    #[test]
    fn schema_global_args_has_yes_flag() {
        let schema = generate();
        let global_args = schema["global_args"].as_array().unwrap();
        let has_yes = global_args.iter().any(|a| a["name"] == "--yes");
        assert!(has_yes, "global_args must include --yes flag");
    }

    #[test]
    fn schema_includes_leaf_commands() {
        let schema = generate();
        let commands = schema["commands"].as_array().unwrap();
        let names: Vec<&str> = commands
            .iter()
            .map(|c| c["name"].as_str().unwrap_or(""))
            .collect();
        assert!(names.contains(&"admin list"), "missing 'admin list'");
        assert!(names.contains(&"vat returns"), "missing 'vat returns'");
        assert!(names.contains(&"invoices list"), "missing 'invoices list'");
    }

    #[test]
    fn schema_validates_against_clispec_v02() {
        let schema_json: Value =
            serde_json::from_str(include_str!("../tests/fixtures/schema-v0.2.json"))
                .expect("parse clispec schema fixture");

        let validator = Validator::new(&schema_json).expect("compile clispec schema");
        let output = generate();
        if let Err(e) = validator.validate(&output) {
            panic!("Schema does not validate against clispec v0.2:\n{e}");
        }
    }

    #[test]
    fn schema_works_without_config() {
        // Must not panic or require any config file; generate() is purely static.
        let schema = generate();
        assert!(schema.get("name").is_some());
    }

    #[test]
    fn list_commands_have_limit_flag() {
        let schema = generate();
        let commands = schema["commands"].as_array().unwrap();
        let list_cmd = commands
            .iter()
            .find(|c| c["name"] == "contacts list")
            .unwrap();
        let args = list_cmd["args"].as_array().unwrap();
        let has_limit = args.iter().any(|a| a["name"] == "--limit");
        assert!(has_limit, "contacts list is missing --limit arg");
    }

    #[test]
    fn list_commands_have_offset_flag() {
        let schema = generate();
        let commands = schema["commands"].as_array().unwrap();
        let list_cmd = commands
            .iter()
            .find(|c| c["name"] == "contacts list")
            .unwrap();
        let args = list_cmd["args"].as_array().unwrap();
        let has_offset = args.iter().any(|a| a["name"] == "--offset");
        assert!(has_offset, "contacts list is missing --offset arg");
    }

    #[test]
    fn list_commands_have_fields_flag() {
        let schema = generate();
        let commands = schema["commands"].as_array().unwrap();
        let list_cmd = commands
            .iter()
            .find(|c| c["name"] == "contacts list")
            .unwrap();
        let args = list_cmd["args"].as_array().unwrap();
        let has_fields = args.iter().any(|a| a["name"] == "--fields");
        assert!(has_fields, "contacts list is missing --fields arg");
    }

    #[test]
    fn output_fields_declared_on_commands() {
        let schema = generate();
        let commands = schema["commands"].as_array().unwrap();
        let with_output_fields = commands
            .iter()
            .filter(|c| {
                c.get("output_fields")
                    .and_then(|f| f.as_array())
                    .map(|a| !a.is_empty())
                    .unwrap_or(false)
            })
            .count();
        assert!(
            with_output_fields > 0,
            "at least some commands must have output_fields"
        );
    }
}
