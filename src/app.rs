//! 为cli::Cli添加`run`方法，用于解析命令行参数并执行对应动作

use crate::{cli, compare, config, pagers};
use anyhow::Result;
use std::io::{self, ErrorKind, IsTerminal};

impl cli::Cli {
    /// `self` 表示命令行结构体本身
    ///
    /// 进行初始化时不需要判断配置路径&配置文件是否存在，因此优先特殊处理
    ///
    /// 除`init`外，其他操作要先判断配置文件的存在，以及配置项的正确
    ///
    /// 输出时判断是否在终端，以配合`less`参数控制是否启用`less`
    ///
    /// 同时判断是否重定向，以便于输入文件时不启用着色
    pub fn run(self) -> Result<()> {
        // 构造配置文件管理者
        let manager = config::ConfigManager::new(config::get_config_dir()?);
        // 用户使用`--init`的话，表示需要初始化，此时不需要检查配置文件路径是否存在，也不需要检查配置内容
        if let cli::Command::Conf(c) = &self.command {
            if c.init {
                manager.init()?;
            }
        }
        // 除了--init之外，其他操作需要进行检查
        let mut app_config = manager.open()?;

        match self.command {
            cli::Command::Conf(c) => {
                let mut changed = false;

                if let Some(code_width) = c.code_width {
                    app_config.code_width = code_width;
                    changed = true;
                }
                if let Some(no_width) = c.no_width {
                    app_config.no_width = no_width;
                    changed = true;
                }
                if let Some(less_path) = c.less_path {
                    app_config.less_path = Some(less_path);
                    changed = true;
                }

                if let Some(inline) = c.inline {
                    app_config.inline = inline;
                    changed = true;
                }

                if changed {
                    manager.save(&app_config)?;
                }
                // 挪到最后的位置，确保conf --list --code-width 50 会先保存配置文件再读取
                if c.list {
                    manager.list()?;
                }
            }
            cli::Command::Diff(d) => {
                let (left_file, right_file) = match d.path {
                    Some(p) => (p.join(&d.left_file), p.join(&d.right_file)),
                    None => (d.left_file, d.right_file),
                };

                let enable_inline = app_config.inline;

                let mut p = pagers::Pager::new(d.less, app_config.less_path)?;

                // 判断是否重定向
                let use_color = io::stdout().is_terminal();

                let res = compare::compare_files_table_style(
                    &left_file,
                    &right_file,
                    app_config.code_width,
                    app_config.no_width,
                    p.writer(),
                    enable_inline,
                    use_color,
                ); // 此处不再使用?解析Result

                if let Err(err) = res {
                    // anyhow Error 降级
                    let is_broken_pipe_err = err
                        .downcast_ref::<io::Error>()
                        .map(|ioe| ioe.kind() == ErrorKind::BrokenPipe)
                        .unwrap_or(false);
                    if !is_broken_pipe_err {
                        return Err(err.into());
                    }
                }

                p.finish()?;
            }
        }
        Ok(())
    }
}
