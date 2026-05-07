/// Configuration management for feishu-mcp-server

use serde::Deserialize;
use std::path::PathBuf;

/// Root configuration structure
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Feishu application configuration
    pub feishu: FeishuConfig,
    /// Server configuration
    pub server: ServerConfig,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Feishu-specific configuration
#[derive(Debug, Clone, Deserialize)]
pub struct FeishuConfig {
    /// Application ID from Feishu developer console
    pub app_id: String,
    /// Application secret (preferentially read from env var FEISHU_APP_SECRET)
    pub app_secret: String,
    /// Base URL for Feishu API
    #[serde(default = "default_base_url")]
    pub base_url: String,
}

/// Server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Server bind host
    #[serde(default = "default_host")]
    pub host: String,
    /// Server bind port
    #[serde(default = "default_port")]
    pub port: u16,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per minute per user
    #[serde(default = "default_max_requests_per_minute")]
    pub max_requests_per_minute: u64,
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Whether to redact tokens in logs
    #[serde(default = "default_redact_tokens")]
    pub redact_tokens: bool,
}

fn default_base_url() -> String {
    "https://open.feishu.cn/open-apis".to_string()
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_max_requests_per_minute() -> u64 {
    100
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_redact_tokens() -> bool {
    true
}

impl Config {
    /// Load configuration from file and environment variables
    ///
    /// File: `~/.feishu-mcp/config.yaml`
    ///
    /// Environment variables (override file values):
    /// - `FEISHU_APP_SECRET`: Overrides feishu.app_secret
    /// - `FEISHU_ENCRYPTION_KEY`: Used for token store encryption (32 bytes)
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path()?;

        // Use default config if file doesn't exist
        if !config_path.exists() {
            tracing::info!("Config file not found at {:?}, using defaults", config_path);
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let mut config: Config = serde_yaml::from_str(&content)?;

        // Override app_secret from environment variable if set
        if let Ok(env_secret) = std::env::var("FEISHU_APP_SECRET") {
            if !env_secret.is_empty() {
                tracing::info!("Using app_secret from FEISHU_APP_SECRET environment variable");
                config.feishu.app_secret = env_secret;
            }
        }

        Ok(config)
    }

    /// Get default configuration
    fn default() -> Self {
        Self {
            feishu: FeishuConfig {
                app_id: String::new(),
                app_secret: String::new(),
                base_url: default_base_url(),
            },
            server: ServerConfig {
                host: default_host(),
                port: default_port(),
            },
            rate_limit: RateLimitConfig {
                max_requests_per_minute: default_max_requests_per_minute(),
            },
            logging: LoggingConfig {
                level: default_log_level(),
                redact_tokens: default_redact_tokens(),
            },
        }
    }

    /// Get config file path
    fn config_path() -> anyhow::Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".feishu-mcp");
        Ok(config_dir.join("config.yaml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path() {
        let path = Config::config_path().unwrap();
        assert!(path.ends_with(".feishu-mcp/config.yaml"));
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.rate_limit.max_requests_per_minute, 100);
        assert_eq!(config.logging.level, "info");
        assert!(config.logging.redact_tokens);
    }

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
feishu:
  app_id: "test_app_id"
  app_secret: "test_secret"
  base_url: "https://open.feishu.cn/open-apis"
server:
  host: "127.0.0.1"
  port: 8080
rate_limit:
  max_requests_per_minute: 50
logging:
  level: "debug"
  redact_tokens: false
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.feishu.app_id, "test_app_id");
        assert_eq!(config.feishu.app_secret, "test_secret");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.rate_limit.max_requests_per_minute, 50);
        assert_eq!(config.logging.level, "debug");
        assert!(!config.logging.redact_tokens);
    }

    #[test]
    fn test_default_base_url() {
        assert_eq!(default_base_url(), "https://open.feishu.cn/open-apis");
    }
}