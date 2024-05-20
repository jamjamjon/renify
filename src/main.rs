use clap::Parser;

fn main() -> anyhow::Result<()> {
    renify::Cli::parse().run()?;
    Ok(())
}
