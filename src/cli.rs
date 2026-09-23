//! # 结构体`Cli`用于命令行参数
//!
//! 命令行分为两个子命令
//!
//! - `conf` 用于进行配置
//! - `diff` 用于文件比较
//!
//! # `conf`子命令
//! `conf`子命令目前拥有`--list`/`--init`/`--code-width`/`--no-width`/`--less-path`/`--inline`参数
//! - `--list`
//!     - 展示配置
//! - `--init`
//!     - 初始化配置
//! - `--code-width`
//!     - 代码列宽设置
//! - `--no-width`
//!     - 行号列宽设置
//! - `--less-path`
//!     - `less`可执行程序位置
//! - `--inline`
//!     - 是否开启行内比较 将`Replace`的差异放到一行
//!
//! `--list`与`--init`互斥，不可同时使用
//!
//! `--list`与其他参数一起使用时，先保存最新设置，再展示配置
//!
//! `--init`与其他参数一起使用时，先生成默认配置，再覆盖指定值并保存
//! # `diff`子命令
//! `diff`子命令目前拥有`left_file`/`right_file`/`--path`/`--less`参数
//! - `left_file` 指定左文件
//! - `right_file` 指定右文件
//! - `--path` 路径，当左右文件在统一路径下时使用，以便简略输入所有文件名
//! - `--less` 是否启用`less`控制显示
//! - `--style` 对比样式
//!
use crate::bytes_unit::BytesUnit;
use crate::consts;
use anyhow::{Context, Result, anyhow, bail};
use clap::{self, ArgAction, ArgGroup, Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

fn code_width_validate(w: &str) -> Result<usize> {
    let width = w
        .parse::<usize>()
        .map_err(|_| anyhow!("宽度必须是正整数"))?;
    if width < 40 || width > 100 {
        bail!("宽度需在40-100之间");
    }
    Ok(width)
}
fn no_width_validate(w: &str) -> Result<usize> {
    let width = w
        .parse::<usize>()
        .map_err(|_| anyhow!("宽度必须是正整数"))?;
    if width < 4 || width > 7 {
        bail!("宽度需在4-7之间");
    }
    Ok(width)
}
fn less_path_validate(p: &str) -> Result<PathBuf> {
    if p.is_empty() {
        bail!("路径不能为空");
    }
    let path = PathBuf::from(p);
    let Ok(is_exists) = path.try_exists() else {
        bail!("无法确认路径是否存在");
    };

    if !is_exists {
        bail!("路径不存在");
    };

    Ok(path)
}
fn file_limit_bytes_vlaidate(b: &str) -> Result<u64> {
    let non_num_first_pos = b
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or_else(|| b.len());

    let digits: String = b.get(0..non_num_first_pos).unwrap_or("0").to_string();
    let suffixes: String = b.get(non_num_first_pos..).unwrap_or("").to_string();

    let unit =
        BytesUnit::new(&suffixes).with_context(|| anyhow!("{}不是合法的字节单位", suffixes))?;
    let limit = unit.multiply(digits.parse::<u64>().map_err(|e| anyhow!("{}", e))?)?;

    if limit > consts::FILE_MAX_BYTES {
        bail!("文件大小不可以超过1GiB");
    }
    Ok(limit)
}

/// 用于`Diff`的`style`变体的枚举
/// - `Git` 启用类似`git diff`的输出样式
/// - `Table` 启用左右对比的输出方式
#[derive(ValueEnum, Debug, Clone)]
pub enum DiffStyle {
    Git,
    Table,
}
/// 比较两个文件的差异
///
/// `conf`子命令 用于配置宽度等配置项
///
/// `diff`子命令 用于显示两个文件的差异
#[derive(Parser, Debug)]
#[command(version, about)]
// #[command(flatten_help = true)] // 是否将子命令的help信息展平到应用本身的--help
pub struct Cli {
    /// 子命令
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
    /// 是否启用`less`控制显示
    ///
    /// 显式输入--less时才启用
    #[arg(long)]
    pub less: bool,
    /// 文本对比方式，只允许`git`/`table`
    #[arg(long, value_enum, default_value = "git")]
    pub style: DiffStyle,
}

#[derive(Args, Debug)]
// #[group(required = true, multiple = false)]  该语句会将下方整个组中的参数设置为互斥
// 增加了宽度配置后，之后list/init才需要互斥
#[command(group(ArgGroup::new("config_action").required(false).multiple(false)))] // 此处新建一个对应的ArgGroup，并设定参数
pub struct Config {
    /// 展示当前配置项内容
    #[arg(long, group = "config_action")]
    pub list: bool,
    /// 强制进行配置初始化
    #[arg(long, group = "config_action")]
    pub init: bool,
    /// 代码列宽度 默认50 在40-100之间
    #[arg(long, value_parser = code_width_validate)]
    pub code_width: Option<usize>,
    /// 行号列宽度，默认为4，在4-7之间
    #[arg(long, value_parser = no_width_validate)]
    pub no_width: Option<usize>,
    /// less执行程序的路径
    #[arg(long, value_parser = less_path_validate)]
    pub less_path: Option<PathBuf>,
    /// 是否启用行内比较
    /// --inline参数要区分如下场景
    ///
    /// - 不输入参数`--inline` 表示从配置表取数
    /// - 输入`--inline` 表示设置为`true`
    /// - 输入`--inline=false` 表示设置为`false`
    ///
    /// `ArgAction::Set + num_args + default_missing_value`组合起来的意思就是
    /// 开启参数，接收0或1个对应值参数，如果为0，则用默认值
    #[arg(long, action = ArgAction::Set, num_args = 0..=1, default_missing_value = "true",require_equals = true)]
    pub inline: Option<bool>,
    #[arg(
        long,
        value_name = "SIZE",
        value_parser = file_limit_bytes_vlaidate,
        help = "单个文件大小上限，可带 K/M/G 后缀（如 10MiB、1G）",
        long_help = concat!(
            "单个文件大小上限，可带 K/M/G 后缀。\n",
            "\n",
            "支持“数字+后缀”或纯字节数：10MiB / 500K / 1G / 52428800 均可。\n",
            "\n",
            "单位（均为 1024 进制，大小写不敏感）：\n",
            "    K = KB = KiB = 1024\n",
            "    M = MB = MiB = 1024^2\n",
            "    G = GB = GiB = 1024^3\n",
            "注意：此处 KB/MB/GB 也按 1024 计算，而非 1000。\n",
            "\n",
            "只认整数（1.5G 会被拒绝）；数字与后缀之间不能有空格\n",
            "（10 MiB 会被拒绝，请写 10MiB）。上限不得超过 1GiB。",
        ),
    )]
    pub file_limit_bytes: Option<u64>,
}
