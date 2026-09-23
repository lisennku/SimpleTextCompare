//! 提供用于序列化与反序列化的配置文件结构体`AppConfig`，和管理配置文件的`ConfigManager`
use crate::consts;
use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

/// # `AppConfig`
/// ## 字段
/// - `code_width` 代码列宽
/// - `no_width` 行号列宽
/// - `less_path` `less`可执行程序的位置
/// - `inline` 是否启用行内对比
/// - `file_limit_bytes` 单个文件最大的字节数 用于大文件内存保护，防止`OOM`
/// ## 方法
/// - `new` 根据传入的参数返回一个`AppConfig`对象
/// - `validate` 校验配置项是否合法
/// ## `Default trait`
/// 实现了`default`函数，默认以`50_usize`/`4_usize`/`None`/`false`/`1GiB`传入
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub code_width: usize,
    pub no_width: usize,
    pub less_path: Option<PathBuf>,
    pub inline: bool,
    pub file_limit_bytes: u64,
}

impl AppConfig {
    #[allow(dead_code)]
    pub fn new(
        code_width: usize,
        no_width: usize,
        less_path: Option<PathBuf>,
        inline: bool,
        file_limit_bytes: u64,
    ) -> Self {
        Self {
            code_width,
            no_width,
            less_path,
            inline,
            file_limit_bytes,
        }
    }
    pub fn validate(&self) -> Result<()> {
        // 不再对less path进行校验，放到pager.rs里
        if self.code_width < 40 || self.code_width > 100 {
            bail!("代码列宽度需在40-100之间");
        }
        if self.no_width < 4 || self.no_width > 7 {
            bail!("行号列宽需在4-7")
        }
        if self.file_limit_bytes > consts::FILE_MAX_BYTES {
            bail!("单个文件最大不得超过1GiB")
        }
        Ok(())
    }
}

/// 为配置对象生成默认的配置
///
/// 代码列宽`50_usize`
///
/// 行号列宽`4_usize`
///
/// `less`可执行程序地址为`None`
///
/// 不启用行内对比
///
/// 文件限制`50MiB`
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            code_width: 50_usize,
            no_width: 4_usize,
            less_path: None,
            inline: false,
            file_limit_bytes: 50 * 1024 * 1024,
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

    pub fn list(&self) -> Result<()> {
        let config = self.load()?;

        println!("{:#?}", config);

        Ok(())
    }

    pub fn save(&self, config: &AppConfig) -> Result<()> {
        let content = toml::to_string_pretty(&config).with_context(|| "序列化配置项失败")?;
        if let Some(parent) = self.config_file.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        // 使用临时文件+rename保证原子性
        let tmp = self.config_file.with_extension("toml.tmp");
        fs::write(&tmp, &content)?;
        fs::rename(&tmp, &self.config_file)?;

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

        Ok(app_config)
    }
}

/// 公共方法，用于获取基于`home`目录的配置文件地址
pub fn get_config_dir() -> Result<PathBuf> {
    let home_dir = env::home_dir();
    let config_dir = home_dir
        .map(|d| {
            d.join(consts::CONFIG_FOLDER_NAME)
                .join(consts::CONFIG_FILE_NAME)
        })
        .ok_or(anyhow!("无法找到用户目录"))?;

    Ok(config_dir)
}
