use anyhow::Result;
use clap::Parser;
mod ansi_config;
mod app;
mod cli;
mod compare;
mod config;
mod line_status;
mod output;
mod pagers;

fn main() -> Result<()> {
    cli::Cli::parse().run()?;
    Ok(())
}
