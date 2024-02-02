use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    renify::Cli::parse().run()?;
    Ok(())
}
