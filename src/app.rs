//! 为cli::Cli添加`run`方法，用于解析命令行参数并执行对应动作
use crate::{cli, compare, config};
use anyhow::Result;
use std::io::Write;

impl cli::Cli {
    /// `self` 表示命令行结构体本身
    ///
    /// `writer` 输出方
    pub fn run(self, writer: &mut dyn Write) -> Result<()> {
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

                if changed {
                    manager.save(&app_config)?;
                }
                // 挪到最后的位置，确保conf --list --code-width 50 会先保存配置文件再读取
                if c.list {
                    manager.list(writer)?;
                }
            }
            cli::Command::Diff(d) => {
                let (left_file, right_file) = match d.path {
                    Some(p) => (p.join(&d.left_file), p.join(&d.right_file)),
                    None => (d.left_file, d.right_file),
                };

                compare::compare_files_table_style(
                    &left_file,
                    &right_file,
                    app_config.code_width,
                    app_config.no_width,
                    writer,
                )?;
            }
        }
        Ok(())
    }
}
