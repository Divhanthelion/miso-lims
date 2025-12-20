//! Server configuration.

use serde::Deserialize;

/// Server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Server host (default: 0.0.0.0)
    #[serde(default = "default_host")]
    pub host: String,

    /// Server port (default: 8080)
    #[serde(default = "default_port")]
    pub port: u16,

    /// Database connection URL
    pub database_url: String,

    /// JWT secret for token signing
    pub jwt_secret: String,

    /// JWT token expiration in hours (default: 24)
    #[serde(default = "default_jwt_expiration")]
    pub jwt_expiration_hours: u64,

    /// Enable CORS for development
    #[serde(default)]
    pub cors_enabled: bool,

    /// Log level (default: info)
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_jwt_expiration() -> u64 {
    24
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Config {
    /// Loads configuration from environment variables.
    pub fn from_env() -> Result<Self, config::ConfigError> {
        // Load .env file if present
        let _ = dotenvy::dotenv();

        config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .set_default("host", "0.0.0.0")?
            .set_default("port", 8080)?
            .set_default("jwt_expiration_hours", 24)?
            .set_default("cors_enabled", false)?
            .set_default("log_level", "info")?
            .build()?
            .try_deserialize()
    }

    /// Returns the server address.
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

