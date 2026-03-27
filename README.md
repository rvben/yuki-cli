# yuki

CLI client for the [Yuki](https://www.yukiworks.nl) bookkeeping SOAP API.

Query invoices, VAT returns, GL accounts, contacts, and projects. Find missing invoices by cross-referencing bank transactions against booked items. Upload documents with metadata.

## Install

```
make install
```

Or directly:

```
cargo install --path .
```

This installs the `yuki` binary to `~/.cargo/bin/`.

## Setup

```
yuki init
```

Prompts for your Yuki API key, discovers available administrations, and writes the config to `~/.config/yuki/config.toml`.

Non-interactive setup:

```
yuki init --api-key <key> --default-admin <name>
```

## Commands

### Querying

```sh
yuki vat returns                          # List all VAT return periods
yuki vat returns --year 2025              # Filter by year
yuki vat codes                            # List active VAT codes

yuki invoices list --invoice-type purchase # Outstanding purchase invoices
yuki invoices show <transaction-id>       # Transaction details
yuki invoices document <transaction-id>   # Document linked to a transaction

yuki contacts search "Hetzner"            # Search contacts
yuki contacts list                        # List all suppliers and customers

yuki accounts balance --account 11001 --period 2025-Q1
yuki accounts transactions --account 11001 --period 2025-Q1
yuki accounts scheme                      # Chart of accounts (GL scheme)
yuki accounts revenue --period 2025-Q1    # Net revenue for a period
yuki accounts start-balance --year 2025   # Opening balances per GL account

yuki projects list                        # List all projects
yuki projects balance <code> --period 2025  # Project balance

yuki documents list --folder inkoop       # List documents in a folder
yuki documents search "factuur"           # Full-text search
yuki documents exists --amount 7.28 --date 2025-03  # Check if invoice exists

yuki admin list                           # List administrations
yuki admin switch <name>                  # Change default administration
```

### Gap analysis

```sh
yuki check btw 2025-Q4                    # VAT period check: outstanding items
yuki check unmatched --period 2026-Q1     # Bank debits without matching invoices
yuki check outstanding <reference>        # Check if a reference is still outstanding
```

### Uploading

```sh
yuki upload file invoice.pdf                           # Upload to uitzoeken (auto-sorted)
yuki upload file invoice.pdf --folder inkoop            # Upload to specific folder
yuki upload file invoice.pdf --amount 114.27 \
  --category 45100 --payment-method 4 \
  --remarks "Hosting"                                   # Upload with metadata

yuki upload categories                                  # List cost category IDs
yuki upload payment-methods                             # List payment method IDs
```

### Global flags

| Flag | Description |
|------|-------------|
| `--admin <name>` | Override default administration |
| `--format table\|json` | Output format (auto-detects TTY) |
| `--quiet` | Suppress informational output |

## Periods

The `--period` flag accepts:

- `2025` — full year
- `2025-Q1` — quarter
- `2025-03` — single month

## Agent use

When stdout is not a TTY (piped or called by an agent), output defaults to JSON. Errors are also structured JSON on stderr. Exit codes: 0 success, 1 general error, 2 auth error, 3 not found, 4 rate limited.

The `documents exists` command exits with code 3 when no matching document is found, making it easy to use in scripts and agent workflows.

## Config

`~/.config/yuki/config.toml`:

```toml
api_key = "your-api-key"
default_admin = "company_name"

# Skip these counterparties in `check unmatched` (case-insensitive substring match)
unmatched_ignore = [
  "Belastingdienst",
  "ING bankkosten",
]

[administrations.company_name]
domain_id = "domain-uuid"
admin_id = "admin-uuid"
```

## Development

```
make check    # Run clippy + fmt check + tests
make build    # Debug build
make release  # Release build
make fmt      # Format code
make install  # Install to ~/.cargo/bin/
```

## License

MIT
