use anyhow::Result;
use clap::Parser;
mod ansi_config;
mod app;
mod bytes_unit;
mod cli;
mod compare;
mod config;
mod consts;
mod line_status;
mod output;
mod pagers;
mod row;

fn main() -> Result<()> {
    cli::Cli::parse().run()?;
    Ok(())
}
