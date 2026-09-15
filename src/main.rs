use anyhow::Result;
use clap::Parser;

mod cli;
mod compare;
mod line_status;
mod output;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let mut w = std::io::stdout();
    if let Some(path) = cli.path {
        let left_file = path.join(cli.left_file);
        let right_file = path.join(cli.right_file);
        compare::compare_files_table_style(
            &left_file,
            &right_file,
            cli.code_width,
            cli.no_width,
            &mut w,
        )?;
    } else {
        compare::compare_files_table_style(
            &cli.left_file,
            &cli.right_file,
            cli.code_width,
            cli.no_width,
            &mut w,
        )?;
    }

    Ok(())
}
