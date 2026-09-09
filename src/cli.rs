use clap::Parser;
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli{
    pub left_file: String,
    pub right_file: String,
    #[arg(short, long)]
    pub path: Option<String>,
}