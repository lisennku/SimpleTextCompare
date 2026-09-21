//! 提供`Pager`枚举类型，用于选择具体输出位置
//!
//! - `Stdout`变体 表示输出到标准输出
//!
//! - `Less`变体 表示通过`less`输出

use anyhow::{Result, anyhow};
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};

/// `Pager`枚举
///
/// `Stdout`变体 表示使用标准输出 当未启用`--less`参数 或者重定向输出时启用
///
/// `Less`变体 表示将输出转到`less`里 启用`--less`参数时使用
/// - `Less`变体是一个结构体，包含
///     - `child` 子进程句柄
///     - `writer` 子进程的标准输入
///
/// 因需要显式调用子进程的`wait`以使子进程完全完成后退出，所以保留子进程句柄
pub enum Pager {
    Stdout(io::Stdout),
    Less { child: Child, writer: ChildStdin },
}

impl Pager {
    /// 构造函数 因启用`less`可能会产生失败，所以返回`Result`
    pub fn new(is_less: bool, path: Option<PathBuf>) -> Result<Pager> {
        if !is_less || !io::stdout().is_terminal() {
            return Ok(Pager::Stdout(io::stdout()));
        }

        let less_path = path.unwrap_or_else(|| PathBuf::from("less"));

        let mut child = Command::new(less_path)
            .arg("-FRSX")
            .stdin(Stdio::piped()) // 父进程输出通过管道进入子进程输入
            .spawn()?;

        let writer = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("无法获取less子进程的标准输入"))?;
        Ok(Pager::Less { child, writer })
    }

    ///统一返回各个变体的`writer`
    pub fn writer(&mut self) -> &mut dyn Write {
        match self {
            Pager::Stdout(w) => w,
            Pager::Less { writer, .. } => writer,
        }
    }

    /// 最后的工作
    ///
    /// 1. 刷新缓冲区
    /// 2. 对于`less`子进程，关掉`stdin`，并阻塞父进程待子进程完成
    pub fn finish(self) -> Result<()> {
        match self {
            Pager::Stdout(mut w) => {
                w.flush()?;
                Ok(())
            }
            Pager::Less {
                mut child,
                mut writer,
            } => {
                writer.flush()?;
                drop(writer);
                child.wait()?;
                Ok(())
            }
        }
    }
}
