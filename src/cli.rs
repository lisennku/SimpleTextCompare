//! 结构体`Cli`用于命令行参数
//! - `left_file` 用于指定左文件
//! - `right_file` 用于指定右文件
//! - `path` 用于指定左右文件相同的路径，可选
//!
use clap::{self, Parser};
use std::path::PathBuf;
/// 比较两个文件的差异
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    /// 左侧文件
    pub left_file: PathBuf,
    /// 右侧文件
    pub right_file: PathBuf,
    /// 路径，当左右文件在统一路径下时使用
    #[arg(short, long)]
    pub path: Option<PathBuf>,
    /// 代码列宽度 默认50 在40-60之间
    #[arg(short, long, default_value_t = 50, value_parser = code_width_validate)]
    pub code_width: usize,
    /// 行号列宽度，默认为4，在3-6之间
    #[arg(short, long, default_value_t = 4, value_parser = no_width_validate)]
    pub no_width: usize,
    /// 是否u启用`less`控制显示
    /// 显式输入--long时才启用
    #[arg(long)]
    pub less: bool,
}

fn code_width_validate(w: &str) -> Result<usize, String> {
    let width = w
        .parse::<usize>()
        .map_err(|_| "宽度必须是正整数".to_string())?;
    if width < 40 || width > 60 {
        return Err("宽度需在40-60之间".to_string());
    }
    Ok(width)
}
fn no_width_validate(w: &str) -> Result<usize, String> {
    let width = w
        .parse::<usize>()
        .map_err(|_| "宽度必须是正整数".to_string())?;
    if width < 4 || width > 7 {
        return Err("宽度需在4-7之间".to_string());
    }
    Ok(width)
}
