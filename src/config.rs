use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub bot: BotConfig,
    pub api: ApiConfig,
    pub rate_limit: RateLimitConfig,
    #[serde(default)]
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub features: FeaturesConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BotConfig {
    pub app_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub base_url: String,
    pub auth_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
    pub min_interval_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NotificationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub sound: bool,
    #[serde(default = "default_true")]
    pub show_preview: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sound: true,
            show_preview: true,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub file: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct FeaturesConfig {
    #[serde(default = "default_true")]
    pub auto_download_media: bool,
    #[serde(default = "default_media_dir")]
    pub media_dir: String,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            auto_download_media: true,
            media_dir: "./media".to_string(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_media_dir() -> String {
    "./media".to_string()
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("无法读取配置文件: {}", path))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| "配置文件格式错误")?;

        // 验证配置
        if config.bot.app_id.is_empty() || config.bot.app_id == "your_app_id_here" {
            anyhow::bail!("请在配置文件中设置正确的 app_id");
        }

        if config.bot.client_secret.is_empty() || config.bot.client_secret == "your_client_secret_here" {
            anyhow::bail!("请在配置文件中设置正确的 client_secret");
        }

        Ok(config)
    }
}
