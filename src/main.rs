use clap::Parser;

#[derive(Parser)]
#[command(name = "yuki", about = "CLI client for the Yuki bookkeeping API")]
struct Cli {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();
    Ok(())
}
