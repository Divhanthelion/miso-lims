//! Database connection management.
//!
//! Handles connection pooling and configuration for MySQL via SeaORM.

use sea_orm::{ConnectOptions, DatabaseConnection, DbErr};
use std::time::Duration;
use tracing::info;

/// Database configuration options.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database connection URL (e.g., mysql://user:pass@host/db)
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections to keep alive
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
    /// Idle timeout in seconds
    pub idle_timeout_secs: u64,
    /// Maximum lifetime of a connection in seconds
    pub max_lifetime_secs: u64,
    /// Whether to log SQL queries
    pub sqlx_logging: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "mysql://miso:miso@localhost:3306/miso".to_string(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 300,
            max_lifetime_secs: 3600,
            sqlx_logging: false,
        }
    }
}

impl DatabaseConfig {
    /// Creates a new configuration from a database URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Sets the maximum number of connections.
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Enables SQL query logging.
    pub fn with_logging(mut self) -> Self {
        self.sqlx_logging = true;
        self
    }

    /// Creates a configuration from environment variables.
    pub fn from_env() -> Self {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://miso:miso@localhost:3306/miso".to_string());

        let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        Self {
            url,
            max_connections,
            ..Default::default()
        }
    }
}

/// Database connection wrapper.
///
/// Provides a managed connection pool for MySQL via SeaORM.
#[derive(Debug, Clone)]
pub struct Database {
    connection: DatabaseConnection,
}

impl Database {
    /// Creates a new database connection with the given configuration.
    pub async fn connect(config: DatabaseConfig) -> Result<Self, DbErr> {
        info!("Connecting to database...");

        let mut opts = ConnectOptions::new(config.url);
        opts.max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect_timeout(Duration::from_secs(config.connect_timeout_secs))
            .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
            .max_lifetime(Duration::from_secs(config.max_lifetime_secs))
            .sqlx_logging(config.sqlx_logging);

        let connection = sea_orm::Database::connect(opts).await?;

        info!("Database connected successfully");

        Ok(Self { connection })
    }

    /// Creates a connection from environment variables.
    pub async fn from_env() -> Result<Self, DbErr> {
        Self::connect(DatabaseConfig::from_env()).await
    }

    /// Returns a reference to the underlying connection.
    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    /// Pings the database to check connectivity.
    pub async fn ping(&self) -> Result<(), DbErr> {
        // Execute a simple query to test the connection
        use sea_orm::ConnectionTrait;
        self.connection
            .execute_unprepared("SELECT 1")
            .await
            .map(|_| ())
    }

    /// Closes the database connection pool.
    pub async fn close(self) -> Result<(), DbErr> {
        self.connection.close().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert!(!config.sqlx_logging);
    }

    #[test]
    fn test_config_builder() {
        let config = DatabaseConfig::new("mysql://test:test@localhost/test")
            .max_connections(20)
            .with_logging();

        assert_eq!(config.max_connections, 20);
        assert!(config.sqlx_logging);
    }
}

