use anyhow::Result;
use clap::Parser;
mod cli;
mod compare;
mod config;
mod line_status;
mod output;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Command::Diff(d) => {
            if let Some(path) = d.path {
                let left_file = path.join(d.left_file);
                let right_file = path.join(d.right_file);
                compare::compare_files_table_style(
                    &left_file,
                    &right_file,
                    d.code_width,
                    d.no_width,
                )?;
            } else {
                compare::compare_files_table_style(
                    &d.left_file,
                    &d.right_file,
                    d.code_width,
                    d.no_width,
                )?;
            }
        }
        cli::Command::Conf(c) => {
            println!("{:#?}", c);
        }
    }

    Ok(())
}
