//! 提供用于序列化与反序列化的配置文件结构体`AppConfig`，和管理配置文件的`ConfigManger`

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use toml;
/// 指定存放配置文件的目录名
const CONFIG_FILE_NAME: &str = r"stc.toml";
/// 指定配置文件名
const CONFIG_FOLDER_NAME: &str = "stc_conf";

/// # `AppConfig`
/// ## 字段
/// - `code_width` 代码列宽
/// - `no_width` 行号列宽
/// - `less_path` `less`可执行程序的位置
/// ## 方法
/// - `new` 根据传入的参数返回一个`AppConfig`对象
/// - `validate` 校验配置项是否合法
/// ## `Default trait`
/// 实现了`default`函数，默认以`50_usize`/`4_usize`/`None`传入
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub code_width: usize,
    pub no_width: usize,
    pub less_path: Option<PathBuf>,
}

impl AppConfig {
    #[allow(dead_code)]
    pub fn new(code_width: usize, no_width: usize, less_path: Option<PathBuf>) -> Self {
        Self {
            code_width,
            no_width,
            less_path,
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.code_width < 40 || self.code_width > 60 {
            bail!("代码列宽度需在40-60之间");
        }
        if self.no_width < 4 || self.no_width > 7 {
            bail!("行号列宽需在4-7")
        }
        if let Some(path) = &self.less_path {
            let Ok(is_exists) = path.try_exists() else {
                bail!("无法访问文件系统");
            };

            if !is_exists {
                bail!("less可执行程序路径不存在")
            }
        }
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            code_width: 50_usize,
            no_width: 4_usize,
            less_path: None,
        }
    }
}

/// # `ConfigManager`
/// 用于实际操作配置文件的结构体
/// ## 字段
/// - `config_file` 存储配置文件路径，通过函数`get_config_dir`获取
/// ## 方法
/// - `new` 关联方法
/// - `init` 初始化配置文件
/// - `list` 展示配置文件
/// - `save` 将更改后的`AppConfig`保存回配置文件
/// - `load` 读取配置文件并返回`AppConfig`
/// - `remove` 删除配置文件及其目录， 非`pub`
/// - `open` 负责判断在非初始化的场景下，配置文件及其目录是否存在，当前配置项是否合法
pub struct ConfigManager {
    pub config_file: PathBuf,
}

impl ConfigManager {
    pub fn new(config_file: PathBuf) -> Self {
        Self { config_file }
    }
    pub fn init(&self) -> Result<()> {
        let app_config = AppConfig::default();
        self.save(&app_config)?;
        Ok(())
    }

    pub fn list(&self, writer: &mut dyn Write) -> Result<()> {
        let config = self.load()?;

        writeln!(writer, "{:#?}", config)?;

        Ok(())
    }

    pub fn save(&self, config: &AppConfig) -> Result<()> {
        let content = toml::to_string_pretty(&config).with_context(|| "序列化配置项失败")?;
        if let Some(parent) = self.config_file.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(&self.config_file, content)?;

        Ok(())
    }

    pub fn load(&self) -> Result<AppConfig> {
        let content = fs::read_to_string(&self.config_file)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    #[allow(dead_code)]
    fn remove(&self) -> Result<()> {
        if let Some(parent) = self.config_file.parent() {
            if !parent.as_os_str().is_empty() {
                fs::remove_dir_all(parent)?;
            }
        }
        Ok(())
    }

    pub fn open(&self) -> Result<AppConfig> {
        let path = &self.config_file;
        let Ok(is_exists) = path.try_exists() else {
            bail!("无法访问文件系统");
        };

        if !is_exists {
            bail!("尚未进行初始化，请使用conf --init进行初始化");
        }

        let app_config = self.load()?;
        app_config.validate()?;

        Ok(app_config)
    }
}

/// 公共方法，用于获取基于`home`目录的配置文件地址
pub fn get_config_dir() -> Result<PathBuf> {
    let home_dir = env::home_dir();
    let config_dir = home_dir
        .map(|d| d.join(CONFIG_FOLDER_NAME).join(CONFIG_FILE_NAME))
        .ok_or(anyhow!("无法找到用户目录"))?;

    Ok(config_dir)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_config_get_config_dir() {
//         println!("{}", get_config_dir().unwrap().display());
//     }
//
//     #[test]
//     fn test_config_new_app_config() {
//         let p = AppConfig::new(5, 4, Some(PathBuf::new()));
//         println!("{:?}", p.code_width);
//         println!("{:?}", p.no_width);
//         println!("{:?}", p.less_path.unwrap().display());
//     }
//
//     #[test]
//     fn test_config_store() {
//         let conf_manage = ConfigManager::new(get_config_dir().unwrap().into());
//         conf_manage.init().unwrap();
//     }
//
//     // #[test]
//     // fn test_config_remove() {
//     //     let conf_manage = ConfigManager::new(get_config_dir().unwrap().into());
//     //     conf_manage.init().unwrap();
//     //     conf_manage.remove().unwrap();
//     // }
//
//     #[test]
//     fn test_config_load() {
//         let conf_manage = ConfigManager::new(get_config_dir().unwrap().into());
//         conf_manage.init().unwrap();
//         conf_manage.load().unwrap();
//     }
// }
