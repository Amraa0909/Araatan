//! ARAATAN Configuration Loader

use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::tickerall::TickerallConfig;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub system: SystemConfig,
    #[serde(default)]
    pub tickerall: TickerallSettings,
    #[serde(default)]
    pub risk: RiskConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub name: String,
    pub version: String,
    pub symbols: Vec<String>,
    pub timeframe: String,
    pub timeframe_ms: u64,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            name: "ARAATAN".to_string(),
            version: "0.6.1".to_string(),
            symbols: vec!["XAUUSD".to_string(), "USDJPY".to_string()],
            timeframe: "M5".to_string(),
            timeframe_ms: 300_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerallSettings {
    #[serde(default = "default_base_url")]
    pub base_url: String,
    pub api_key: Option<String>,
    pub account_id: Option<String>,
    pub broker: Option<String>,
    pub server: Option<String>,
    pub account: Option<u64>,
    pub password: Option<String>,
    #[serde(default = "default_warmup_bars")]
    pub warmup_bars: usize,
}

fn default_base_url() -> String {
    "https://api.tickerall.com".to_string()
}

fn default_warmup_bars() -> usize {
    600
}

impl Default for TickerallSettings {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            api_key: std::env::var("TICKERALL_API_KEY").ok(),
            account_id: std::env::var("TICKERALL_ACCOUNT_ID").ok(),
            broker: Some("mt5".to_string()),
            server: None,
            account: None,
            password: None,
            warmup_bars: default_warmup_bars(),
        }
    }
}

impl TickerallSettings {
    pub fn to_tickerall_config(&self) -> TickerallConfig {
        TickerallConfig {
            api_key: self.api_key.clone().or_else(|| std::env::var("TICKERALL_API_KEY").ok()),
            account_id: self.account_id.clone().or_else(|| std::env::var("TICKERALL_ACCOUNT_ID").ok()),
            base_url: if self.base_url.is_empty() {
                default_base_url()
            } else {
                self.base_url.clone()
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub risk_pct: f64,
    pub max_loss: f64,
    pub max_slippage: f64,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            risk_pct: 0.01,
            max_loss: 0.02,
            max_slippage: 3.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_url: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite://araatan.db".to_string(),
        }
    }
}

impl AppConfig {
    /// Loads configuration from araatan.toml or returns default
    pub fn load() -> Result<Self> {
        let path = Path::new("araatan.toml");
        if path.exists() {
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let mut cfg: AppConfig = toml::from_str(&content)
                .with_context(|| format!("Failed to parse {}", path.display()))?;

            // Environment overrides
            if let Ok(key) = std::env::var("TICKERALL_API_KEY") {
                if !key.trim().is_empty() {
                    cfg.tickerall.api_key = Some(key);
                }
            }
            if let Ok(acc) = std::env::var("TICKERALL_ACCOUNT_ID") {
                if !acc.trim().is_empty() {
                    cfg.tickerall.account_id = Some(acc);
                }
            }
            Ok(cfg)
        } else {
            Ok(Self {
                system: SystemConfig::default(),
                tickerall: TickerallSettings::default(),
                risk: RiskConfig::default(),
                storage: StorageConfig::default(),
            })
        }
    }
}
