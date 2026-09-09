//! 结构体`Cli`用于命令行参数
//! - `left_file` 用于指定左文件
//! - `right_file` 用于指定右文件
//! - `path` 用于指定左右文件相同的路径，可选
//!
use clap::Parser;
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    pub left_file: String,
    pub right_file: String,
    #[arg(short, long)]
    pub path: Option<String>,
}
