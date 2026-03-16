use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub vpn: VpnConfig,
    pub system: SystemConfig,
    pub delays: DelaysConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub allowed_subnet: String, // e.g., "192.168.0.0/24"
    pub macrodroid_webhook_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VpnConfig {
    /// IP prefix used to detect the corporate VPN tunnel interface.
    /// e.g. "10." matches any 10.x.x.x, "10.0." matches only 10.0.x.x
    pub detection_prefix: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemConfig {
    pub autostart: bool,
    pub check_interval: u64,
    pub code_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DelaysConfig {
    pub menu_open: f64,       // Ожидание открытия контекстного меню после клика (сек)
    pub dialog_wait: f64,     // Ожидание окна endpoint security (сек)
    pub connection_wait: f64, // Ожидание окна пароля после ввода кода (сек)
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                port: 8337,
                allowed_subnet: "192.168.0.0/24".to_string(), // Default local subnet
                macrodroid_webhook_url: String::new(),
            },
            vpn: VpnConfig {
                detection_prefix: "10.".to_string(),
            },
            system: SystemConfig {
                autostart: true,
                check_interval: 2,
                code_timeout: 30,
            },
            delays: DelaysConfig {
                menu_open: 1.0,       // Ожидание открытия меню
                dialog_wait: 3.0,     // Ожидание окна endpoint security
                connection_wait: 5.0, // Ожидание окна пароля
            },
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "esconnect", "esconnect")
            .ok_or_else(|| anyhow!("Could not determine config directory"))?;

        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;

        Ok(Self {
            config_path: config_dir.join("config.json"),
        })
    }

    pub fn load(&self) -> Result<Config> {
        if !self.config_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }
}
