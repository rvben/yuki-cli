pub mod accounts;
pub mod admin;
pub mod check;
pub mod contacts;
pub mod documents;
pub mod init;
pub mod invoices;
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
    Init,

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

    /// Run year-end (jaarwerk) checks for a given year.
    Jaarwerk {
        /// Fiscal year (e.g. 2025).
        year: Option<String>,
    },

    /// Find bank transactions without matching booked invoices.
    Unmatched {
        /// Accounting period (e.g. 2025-Q1).
        #[arg(long)]
        period: Option<String>,
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
