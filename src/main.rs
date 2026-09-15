use anyhow::Result;
use clap::Parser;
mod app;
mod cli;
mod compare;
mod config;
mod line_status;
mod output;

fn main() -> Result<()> {
    let mut w = std::io::stdout();
    cli::Cli::parse().run(&mut w)?;
    Ok(())
}
