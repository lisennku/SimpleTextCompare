//! 结构体`Cli`用于命令行参数
use clap::{self, ArgGroup, Args, Parser, Subcommand};
use std::path::PathBuf;

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
fn less_path_validate(p: &str) -> Result<PathBuf, String> {
    if p.is_empty() {
        return Err("路径不能为空".to_string());
    }
    let path = PathBuf::from(p);
    let Ok(is_exists) = path.try_exists() else {
        return Err("无法确认路径是否存在".to_string());
    };

    if !is_exists {
        return Err("路径不存在".to_string());
    };

    Ok(path)
}
/// 比较两个文件的差异
///
/// conf子命令 用于配置宽度等配置项
///
/// diff子命令 用于显示两个文件的差异
#[derive(Parser, Debug)]
#[command(version, about)]
// #[command(flatten_help = true)] 是否将子命令的help信息展平到应用本身的--help
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// 配置读写
    Conf(Config),
    /// 文件比较
    Diff(Compare),
}
#[derive(Args, Debug)]
pub struct Compare {
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
    /// 是否启用`less`控制显示
    /// 显式输入--long时才启用
    #[arg(long)]
    pub less: bool,
}

#[derive(Args, Debug)]
// #[group(required = true, multiple = false)]  该语句会将下方整个组中的参数设置为互斥
// 增加了宽度配置后，之后list/init才需要互斥
#[command(group(ArgGroup::new("config_action").required(false).multiple(false)))] // 此处新建一个对应的ArgGroup，并设定参数
pub struct Config {
    /// 展示当前配置项内容
    #[arg(long, group = "config_action")]
    pub list: bool,
    /// 进行配置初始化
    #[arg(long, group = "config_action")]
    pub init: bool,
    /// 代码列宽度 默认50 在40-60之间
    #[arg(long, default_value_t = 50, value_parser = code_width_validate)]
    pub code_width: usize,
    /// 行号列宽度，默认为4，在3-6之间
    #[arg(long, default_value_t = 4, value_parser = no_width_validate)]
    pub no_width: usize,
    /// less执行程序的路径
    #[arg(long, value_parser = less_path_validate)]
    pub less_path: Option<PathBuf>,
}
