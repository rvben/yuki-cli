pub mod accounts;
pub mod admin;
pub mod check;
pub mod contacts;
pub mod documents;
pub mod init;
pub mod invoices;
pub mod projects;
pub mod upload;
pub mod vat;

use clap::{Parser, Subcommand};

use crate::client::accounting::AccountingClient;
use crate::config::{AdminEntry, Config};
use crate::error::YukiError;

/// Authenticate a client and set the active administration domain.
///
/// Returns both the configured client and the resolved `AdminEntry` so callers
/// can pass `admin_id` to operations that require `administrationID`.
pub async fn setup_domain(
    config: &Config,
    admin: Option<&str>,
) -> Result<(AccountingClient, AdminEntry), YukiError> {
    let entry = config.resolve_admin(admin)?;
    let mut client = AccountingClient::new();
    client.authenticate(&config.api_key).await?;
    client.set_current_domain(&entry.domain_id).await?;
    Ok((client, entry))
}

/// Top-level CLI entry point for the Yuki bookkeeping API client.
#[derive(Parser)]
#[command(name = "yuki", about = "CLI client for the Yuki bookkeeping API")]
pub struct Cli {
    /// Override the active administration by name.
    #[arg(long = "admin", global = true)]
    pub admin: Option<String>,

    /// Output format: table, json, or csv.
    #[arg(long, global = true)]
    pub format: Option<String>,

    /// Suppress all output except errors.
    #[arg(long, short, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize yuki configuration for this machine.
    Init {
        /// API key (skips interactive prompt if provided).
        #[arg(long)]
        api_key: Option<String>,

        /// Default administration name (auto-selects if only one available).
        #[arg(long)]
        default_admin: Option<String>,
    },

    /// Manage Yuki administrations.
    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },

    /// Work with sales invoices.
    Invoices {
        #[command(subcommand)]
        command: InvoiceCommands,
    },

    /// Work with archived documents.
    Documents {
        #[command(subcommand)]
        command: DocumentCommands,
    },

    /// Work with contacts (customers and suppliers).
    Contacts {
        #[command(subcommand)]
        command: ContactCommands,
    },

    /// Work with general ledger accounts.
    Accounts {
        #[command(subcommand)]
        command: AccountCommands,
    },

    /// Work with VAT returns and codes.
    Vat {
        #[command(subcommand)]
        command: VatCommands,
    },

    /// Work with projects.
    Projects {
        #[command(subcommand)]
        command: ProjectCommands,
    },

    /// Run compliance and period checks.
    Check {
        #[command(subcommand)]
        command: CheckCommands,
    },

    /// Upload documents to the Yuki archive.
    Upload {
        #[command(subcommand)]
        command: UploadCommands,
    },
}

#[derive(Subcommand)]
pub enum AdminCommands {
    /// List all available administrations.
    List,

    /// Switch the active administration.
    Switch {
        /// Name of the administration to activate.
        name: String,
    },
}

#[derive(Subcommand)]
pub enum InvoiceCommands {
    /// List invoices, optionally filtered by period and type.
    List {
        /// Accounting period (e.g. 2025-01).
        #[arg(long)]
        period: Option<String>,

        /// Invoice type filter (e.g. sales, purchase).
        #[arg(long)]
        invoice_type: Option<String>,
    },

    /// Show details for a single invoice.
    Show {
        /// Invoice ID.
        id: String,
    },

    /// Show the document linked to a transaction.
    Document {
        /// Transaction ID.
        id: String,
    },
}

#[derive(Subcommand)]
pub enum DocumentCommands {
    /// List documents in a folder or of a given type.
    List {
        /// Archive folder name.
        #[arg(long)]
        folder: Option<String>,

        /// Document type filter.
        #[arg(long)]
        doc_type: Option<String>,
    },

    /// Search documents by a query string.
    Search {
        /// Search query.
        query: String,
    },

    /// Check if an invoice exists in the archive (by amount, date, and optional contact).
    Exists {
        /// Invoice amount to search for.
        #[arg(long)]
        amount: f64,
        /// Invoice date (YYYY-MM-DD). Matches within ±7 days.
        #[arg(long)]
        date: String,
        /// Contact/supplier name to narrow the search.
        #[arg(long)]
        contact: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ContactCommands {
    /// Search contacts by name or other criteria.
    Search {
        /// Search query.
        query: String,
    },

    /// List contacts filtered by type.
    List {
        /// Contact type (e.g. customer, supplier).
        #[arg(long)]
        contact_type: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum AccountCommands {
    /// Show the balance of a general ledger account for a period.
    Balance {
        /// GL account code.
        #[arg(long)]
        account: Option<String>,

        /// Accounting period (e.g. 2025-01).
        #[arg(long)]
        period: Option<String>,
    },

    /// List transactions for a general ledger account.
    Transactions {
        /// GL account code.
        #[arg(long)]
        account: Option<String>,

        /// Accounting period (e.g. 2025-01).
        #[arg(long)]
        period: Option<String>,
    },

    /// Show the chart of accounts (GL account scheme).
    Scheme,

    /// Show net revenue for a period.
    Revenue {
        /// Accounting period (e.g. 2025, 2025-Q1, 2025-01).
        #[arg(long)]
        period: Option<String>,
    },

    /// Show opening balances per GL account for a book year.
    StartBalance {
        /// Book year (e.g. 2025).
        #[arg(long)]
        year: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    /// List all projects.
    List,

    /// Show balance for a project.
    Balance {
        /// Project code.
        project: String,

        /// GL account code filter.
        #[arg(long)]
        account: Option<String>,

        /// Accounting period (e.g. 2025, 2025-Q1).
        #[arg(long)]
        period: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum VatCommands {
    /// List VAT returns for a given year.
    Returns {
        /// Fiscal year (e.g. 2025).
        year: Option<String>,
    },

    /// List active VAT codes.
    Codes,
}

#[derive(Subcommand)]
pub enum CheckCommands {
    /// Check outstanding BTW (VAT) items for a period.
    Btw {
        /// Accounting period (e.g. 2025-01).
        period: Option<String>,
    },

    /// Find bank transactions without matching booked invoices.
    Unmatched {
        /// Accounting period (e.g. 2025-Q1).
        #[arg(long)]
        period: Option<String>,
        /// GL account code for the bank account (default: 11001).
        #[arg(long, default_value = "11001")]
        bank_account: String,
    },

    /// Check if a specific invoice reference is still outstanding.
    Outstanding {
        /// Invoice reference to check.
        reference: String,
    },
}

#[derive(Subcommand)]
pub enum UploadCommands {
    /// Upload a document with optional invoice metadata.
    File {
        /// Path to the file to upload.
        file: String,

        /// Target folder: uitzoeken (default), inkoop, verkoop, bank, personeel, belasting, overig-financieel.
        #[arg(long, default_value = "uitzoeken")]
        folder: String,

        /// Invoice amount (e.g. 114.27); enables richer metadata upload.
        #[arg(long)]
        amount: Option<f64>,

        /// Cost category ID (e.g. 45100).
        #[arg(long)]
        category: Option<String>,

        /// Payment method ID (e.g. 4 for pinpas).
        #[arg(long = "payment-method")]
        payment_method: Option<String>,

        /// Project ID.
        #[arg(long)]
        project: Option<String>,

        /// Remarks or notes.
        #[arg(long)]
        remarks: Option<String>,

        /// Currency code (default: EUR).
        #[arg(long, default_value = "EUR")]
        currency: String,
    },

    /// List available cost categories.
    Categories,

    /// List available payment methods.
    PaymentMethods,
}
