use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tuxedo_common::types::AppConfig;

pub struct Config {
    pub data: AppConfig,
    config_path: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_dir = Self::config_dir()?;
        let config_path = config_dir.join("config.json");
        
        let data = if config_path.exists() {
            let json = fs::read_to_string(&config_path)?;
            serde_json::from_str(&json)?
        } else {
            AppConfig::default()
        };
        
        Ok(Self { data, config_path })
    }
    
    pub fn save(&self) -> Result<()> {
        let config_dir = self.config_path.parent().unwrap();
        fs::create_dir_all(config_dir)?;
        
        let json = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.config_path, json)?;
        
        Ok(())
    }
    
    fn config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")?;
        Ok(PathBuf::from(home).join(".config/tuxedo-control-center"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data: AppConfig::default(),
            config_path: Self::config_dir().unwrap().join("config.json"),
        }
    }
}