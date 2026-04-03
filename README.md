# yuki

[![codecov](https://codecov.io/gh/rvben/yuki-cli/graph/badge.svg)](https://codecov.io/gh/rvben/yuki-cli)

CLI client for the [Yuki](https://www.yukiworks.nl) bookkeeping SOAP API.

[Yuki](https://www.yukiworks.nl) is a Dutch bookkeeping SaaS used for accounting, VAT returns, and document archiving. This CLI lets you query your administration, find missing invoices, and upload documents — from the terminal or as part of automated workflows.

> **Note:** This project is not affiliated with or endorsed by Yuki Software.

## Install

```sh
cargo install yuki-cli
```

Or via pip:

```sh
pip install yuki-cli
```

Both install the `yuki` binary.

## Setup

1. Get a Yuki API key from your Yuki portal under **Settings > API keys**.
2. Run `yuki init` and paste your key when prompted. The CLI discovers your administrations and writes the config to `~/.config/yuki/config.toml`.

```sh
yuki init
```

Non-interactive (for scripting):

```sh
yuki init --api-key <key> --default-admin <name>
```

To rotate your API key later:

```sh
yuki init --api-key <new-key>
```

## Quick start: find missing invoices

The main workflow is finding bank transactions that don't have a matching invoice in Yuki:

```sh
# Show bank debits without matching invoices for Q1 2025
yuki check unmatched --period 2025-Q1
```

This cross-references bank transactions against outstanding creditor items, booked archive documents, and known counterparty names. The output shows unmatched transactions with their date, amount, counterparty, and description.

For each unmatched item, you can check if the invoice is already in the archive, and upload it if not:

```sh
# Check if an invoice already exists
yuki documents exists --amount 7.28 --date 2025-03

# Upload an invoice (Yuki auto-sorts it)
yuki upload file invoice.pdf

# Or upload to a specific folder with metadata
yuki upload file invoice.pdf --folder inkoop --amount 7.28 --remarks "Hetzner hosting"
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
