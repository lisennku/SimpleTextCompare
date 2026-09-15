use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use toml;

const CONFIG_FILE_NAME: &str = r"stc.toml";
const CONFIG_FOLDER_NAME: &str = "stc_conf";

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub code_width: usize,
    pub no_width: usize,
    pub less_path: Option<PathBuf>,
}
impl AppConfig {
    pub fn new(code_width: usize, no_width: usize, less_path: Option<PathBuf>) -> Self {
        Self {
            code_width,
            no_width,
            less_path,
        }
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
        println!("{:#?}", config);
        Ok(config)
    }

    pub fn remove(&self) -> Result<()> {
        if let Some(parent) = self.config_file.parent() {
            if !parent.as_os_str().is_empty() {
                fs::remove_dir_all(parent)?;
            }
        }
        Ok(())
    }
}

fn get_config_dir() -> Result<PathBuf> {
    let home_dir = env::home_dir();
    let config_dir = home_dir
        .map(|d| d.join(CONFIG_FOLDER_NAME).join(CONFIG_FILE_NAME))
        .ok_or(anyhow!("无法找到用户目录"))?;

    Ok(config_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_get_config_dir() {
        println!("{}", get_config_dir().unwrap().display());
    }

    #[test]
    fn test_config_new_app_config() {
        let p = AppConfig::new(5, 4, Some(PathBuf::new()));
        println!("{:?}", p.code_width);
        println!("{:?}", p.no_width);
        println!("{:?}", p.less_path.unwrap().display());
    }

    #[test]
    fn test_config_store() {
        let conf_manage = ConfigManager::new(get_config_dir().unwrap().into());
        conf_manage.init().unwrap();
    }

    // #[test]
    // fn test_config_remove() {
    //     let conf_manage = ConfigManager::new(get_config_dir().unwrap().into());
    //     conf_manage.init().unwrap();
    //     conf_manage.remove().unwrap();
    // }

    #[test]
    fn test_config_load() {
        let conf_manage = ConfigManager::new(get_config_dir().unwrap().into());
        conf_manage.init().unwrap();
        conf_manage.load().unwrap();
    }
}
