use std::fmt;
use std::process;

use clap::Parser;
use yuki_cli::cli::Cli;
use yuki_cli::cli::Commands;
use yuki_cli::cli::{
    AccountCommands, AdminCommands, CheckCommands, ContactCommands, DocumentCommands,
    InvoiceCommands, UploadCommands, VatCommands,
};
use yuki_cli::config::Config;
use yuki_cli::error::YukiError;
use yuki_cli::output::{format_error_json, is_tty};

enum AppError {
    Yuki(YukiError),
    Other(anyhow::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yuki(e) => write!(f, "{e}"),
            Self::Other(e) => write!(f, "{e}"),
        }
    }
}

impl From<YukiError> for AppError {
    fn from(e: YukiError) -> Self {
        Self::Yuki(e)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self::Other(e)
    }
}

impl AppError {
    fn exit_code(&self) -> u8 {
        match self {
            Self::Yuki(e) => e.exit_code(),
            Self::Other(_) => 1,
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(err) = run(cli).await {
        let code = err.exit_code();
        if is_tty() {
            eprintln!("error: {err}");
        } else {
            let label = match code {
                2 => "auth_failed",
                3 => "not_found",
                4 => "rate_limited",
                _ => "error",
            };
            eprintln!("{}", format_error_json(&err.to_string(), label));
        }
        process::exit(code.into());
    }
}

async fn run(cli: Cli) -> Result<(), AppError> {
    let format = cli.format.as_deref();

    match cli.command {
        Commands::Init {
            api_key,
            default_admin,
        } => {
            yuki_cli::cli::init::run(api_key.as_deref(), default_admin.as_deref()).await?;
        }

        Commands::Admin { command } => {
            let config = Config::load()?;
            match command {
                AdminCommands::List => {
                    yuki_cli::cli::admin::list(&config, format).await?;
                }
                AdminCommands::Switch { name } => {
                    let mut config = config;
                    yuki_cli::cli::admin::switch(&mut config, &name)?;
                }
            }
        }

        Commands::Vat { command } => {
            let config = Config::load()?;
            let admin = cli.admin.as_deref();
            match command {
                VatCommands::Returns { year } => {
                    yuki_cli::cli::vat::returns(&config, admin, year.as_deref(), format).await?;
                }
                VatCommands::Codes => {
                    yuki_cli::cli::vat::codes(&config, admin, format).await?;
                }
            }
        }

        Commands::Contacts { command } => {
            let config = Config::load()?;
            let admin = cli.admin.as_deref();
            match command {
                ContactCommands::Search { query } => {
                    yuki_cli::cli::contacts::search(&config, admin, &query, format).await?;
                }
                ContactCommands::List { contact_type } => {
                    yuki_cli::cli::contacts::list(&config, admin, contact_type.as_deref(), format)
                        .await?;
                }
            }
        }

        Commands::Accounts { command } => {
            let config = Config::load()?;
            let admin = cli.admin.as_deref();
            match command {
                AccountCommands::Balance { account, period } => {
                    yuki_cli::cli::accounts::balance(
                        &config,
                        admin,
                        account.as_deref(),
                        period.as_deref(),
                        format,
                    )
                    .await?;
                }
                AccountCommands::Transactions { account, period } => {
                    yuki_cli::cli::accounts::transactions(
                        &config,
                        admin,
                        account.as_deref(),
                        period.as_deref(),
                        format,
                    )
                    .await?;
                }
                AccountCommands::Scheme => {
                    yuki_cli::cli::accounts::scheme(&config, admin, format).await?;
                }
            }
        }

        Commands::Invoices { command } => {
            let config = Config::load()?;
            let admin = cli.admin.as_deref();
            match command {
                InvoiceCommands::List {
                    period,
                    invoice_type,
                } => {
                    yuki_cli::cli::invoices::list(
                        &config,
                        admin,
                        period.as_deref(),
                        invoice_type.as_deref(),
                        format,
                    )
                    .await?;
                }
                InvoiceCommands::Show { id } => {
                    yuki_cli::cli::invoices::show(&config, admin, &id, format).await?;
                }
                InvoiceCommands::Document { id } => {
                    yuki_cli::cli::invoices::document(&config, admin, &id, format).await?;
                }
            }
        }

        Commands::Documents { command } => {
            let config = Config::load()?;
            let admin = cli.admin.as_deref();
            match command {
                DocumentCommands::List { folder, doc_type } => {
                    yuki_cli::cli::documents::list(
                        &config,
                        admin,
                        folder.as_deref(),
                        doc_type.as_deref(),
                        format,
                    )
                    .await?;
                }
                DocumentCommands::Search { query } => {
                    yuki_cli::cli::documents::search(&config, admin, &query, format).await?;
                }
                DocumentCommands::Exists {
                    amount,
                    date,
                    contact,
                } => {
                    yuki_cli::cli::documents::exists(
                        &config,
                        admin,
                        amount,
                        &date,
                        contact.as_deref(),
                        format,
                    )
                    .await?;
                }
            }
        }

        Commands::Check { command } => {
            let config = Config::load()?;
            let admin = cli.admin.as_deref();
            match command {
                CheckCommands::Btw { period } => {
                    yuki_cli::cli::check::btw(&config, admin, period.as_deref(), format, cli.quiet)
                        .await?;
                }
                CheckCommands::Unmatched {
                    period,
                    bank_account,
                } => {
                    yuki_cli::cli::check::unmatched(
                        &config,
                        admin,
                        period.as_deref(),
                        &bank_account,
                        format,
                        cli.quiet,
                    )
                    .await?;
                }
                CheckCommands::Outstanding { reference } => {
                    yuki_cli::cli::check::outstanding(&config, admin, &reference, format).await?;
                }
            }
        }

        Commands::Upload { command } => {
            let config = Config::load()?;
            let admin = cli.admin.as_deref();
            match command {
                UploadCommands::File {
                    file,
                    folder,
                    amount,
                    category,
                    payment_method,
                    project,
                    remarks,
                    currency,
                } => {
                    let options = yuki_cli::cli::upload::UploadOptions {
                        folder: &folder,
                        amount,
                        category: category.as_deref(),
                        payment_method: payment_method.as_deref(),
                        project: project.as_deref(),
                        remarks: remarks.as_deref(),
                        currency: &currency,
                    };
                    yuki_cli::cli::upload::run(&config, admin, &file, options, format, cli.quiet)
                        .await?;
                }
                UploadCommands::Categories => {
                    yuki_cli::cli::upload::categories(&config, format).await?;
                }
                UploadCommands::PaymentMethods => {
                    yuki_cli::cli::upload::payment_methods(&config, format).await?;
                }
            }
        }
    }

    Ok(())
}
