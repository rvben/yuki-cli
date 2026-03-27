use clap::Parser;
use yuki_cli::cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();

    // Command dispatch is not yet implemented.
    eprintln!("not yet implemented");

    Ok(())
}
