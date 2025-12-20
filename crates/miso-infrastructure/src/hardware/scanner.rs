//! VisionMate 2D Barcode Scanner Client
//!
//! Async TCP client for Thermo Scientific VisionMate scanners.
//! Supports high-speed scanning of 96-well and 384-well plates.

use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Errors that can occur during scanner operations.
#[derive(Debug, Error)]
pub enum ScannerError {
    #[error("Failed to connect to scanner at {host}:{port}: {source}")]
    ConnectionFailed {
        host: String,
        port: u16,
        source: std::io::Error,
    },

    #[error("Connection timed out after {timeout_secs}s")]
    ConnectionTimeout { timeout_secs: u64 },

    #[error("Failed to send command: {0}")]
    SendFailed(#[from] std::io::Error),

    #[error("Read timed out after {timeout_secs}s")]
    ReadTimeout { timeout_secs: u64 },

    #[error("Scanner returned error: {0}")]
    DeviceError(String),

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("Scanner not ready")]
    NotReady,

    #[error("No rack detected in scanner")]
    NoRackDetected,
}

/// The result of a rack scan.
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// The rack barcode (if readable)
    pub rack_barcode: Option<String>,
    /// Map of position (e.g., "A01") to barcode
    pub positions: HashMap<String, String>,
    /// Positions that had no tube or unreadable barcodes
    pub empty_positions: Vec<String>,
    /// Positions with read errors
    pub error_positions: Vec<String>,
    /// Raw response from scanner (for debugging)
    pub raw_response: String,
}

impl ScanResult {
    /// Returns the number of successfully scanned tubes.
    pub fn tube_count(&self) -> usize {
        self.positions.len()
    }

    /// Returns true if all positions were successfully read.
    pub fn is_complete(&self, expected_positions: usize) -> bool {
        self.positions.len() == expected_positions
    }

    /// Gets the barcode at a specific position.
    pub fn get_barcode(&self, position: &str) -> Option<&String> {
        self.positions.get(position)
    }

    /// Returns all barcodes as a vector.
    pub fn all_barcodes(&self) -> Vec<&String> {
        self.positions.values().collect()
    }
}

/// Configuration for the VisionMate client.
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    /// Scanner hostname or IP address
    pub host: String,
    /// Scanner port (default: 8000)
    pub port: u16,
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
    /// Read timeout in seconds
    pub read_timeout_secs: u64,
    /// Number of retry attempts on failure
    pub max_retries: u32,
    /// Delay between retries in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8000,
            connect_timeout_secs: 5,
            read_timeout_secs: 10,
            max_retries: 3,
            retry_delay_ms: 500,
        }
    }
}

impl ScannerConfig {
    /// Creates a new configuration for the given host.
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            ..Default::default()
        }
    }

    /// Sets the port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the connection timeout.
    pub fn connect_timeout(mut self, secs: u64) -> Self {
        self.connect_timeout_secs = secs;
        self
    }

    /// Sets the read timeout.
    pub fn read_timeout(mut self, secs: u64) -> Self {
        self.read_timeout_secs = secs;
        self
    }
}

/// VisionMate scanner client commands.
mod commands {
    /// Trigger a scan
    pub const SCAN: &[u8] = b"S\r";
    /// Get scanner status
    pub const STATUS: &[u8] = b"G\r";
    /// Reset scanner
    pub const RESET: &[u8] = b"R\r";
    /// Get version info
    pub const VERSION: &[u8] = b"V\r";
}

/// Response prefixes from the scanner.
mod responses {
    pub const OK_SCAN: &str = "OKS";
    pub const OK_STATUS: &str = "OKG";
    pub const OK_RESET: &str = "OKR";
    pub const ERROR: &str = "ERR";
    pub const NO_READ: &str = "NO READ";
    pub const EMPTY: &str = "EMPTY";
}

/// Async client for VisionMate 2D barcode scanners.
///
/// # Example
///
/// ```no_run
/// use miso_infrastructure::hardware::scanner::{VisionMateClient, ScannerConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = ScannerConfig::new("192.168.1.100").port(8000);
///     let client = VisionMateClient::new(config);
///     
///     let result = client.scan().await?;
///     println!("Scanned {} tubes", result.tube_count());
///     
///     for (pos, barcode) in &result.positions {
///         println!("{}: {}", pos, barcode);
///     }
///     
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct VisionMateClient {
    config: ScannerConfig,
}

impl VisionMateClient {
    /// Creates a new VisionMate client with the given configuration.
    pub fn new(config: ScannerConfig) -> Self {
        Self { config }
    }

    /// Creates a client for the given host with default settings.
    pub fn connect_to(host: impl Into<String>) -> Self {
        Self::new(ScannerConfig::new(host))
    }

    /// Establishes a connection to the scanner.
    async fn connect(&self) -> Result<TcpStream, ScannerError> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        debug!("Connecting to VisionMate at {}", addr);

        let stream = timeout(
            Duration::from_secs(self.config.connect_timeout_secs),
            TcpStream::connect(&addr),
        )
        .await
        .map_err(|_| ScannerError::ConnectionTimeout {
            timeout_secs: self.config.connect_timeout_secs,
        })?
        .map_err(|e| ScannerError::ConnectionFailed {
            host: self.config.host.clone(),
            port: self.config.port,
            source: e,
        })?;

        info!("Connected to VisionMate at {}", addr);
        Ok(stream)
    }

    /// Sends a command and reads the response.
    async fn send_command(
        &self,
        stream: &mut TcpStream,
        command: &[u8],
    ) -> Result<String, ScannerError> {
        // Send command
        stream.write_all(command).await?;
        stream.flush().await?;

        debug!("Sent command: {:?}", String::from_utf8_lossy(command).trim());

        // Read response
        let mut reader = BufReader::new(stream);
        let mut response = String::new();

        timeout(
            Duration::from_secs(self.config.read_timeout_secs),
            reader.read_line(&mut response),
        )
        .await
        .map_err(|_| ScannerError::ReadTimeout {
            timeout_secs: self.config.read_timeout_secs,
        })??;

        let response = response.trim().to_string();
        debug!("Received response: {}", response);

        Ok(response)
    }

    /// Triggers a scan and returns the results.
    pub async fn scan(&self) -> Result<ScanResult, ScannerError> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                warn!("Retry attempt {} after previous failure", attempt);
                tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
            }

            match self.scan_once().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    error!("Scan attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Single scan attempt without retries.
    async fn scan_once(&self) -> Result<ScanResult, ScannerError> {
        let mut stream = self.connect().await?;

        // Send scan command
        let response = self.send_command(&mut stream, commands::SCAN).await?;

        // Parse response
        self.parse_scan_response(&response)
    }

    /// Parses the scan response into a ScanResult.
    fn parse_scan_response(&self, response: &str) -> Result<ScanResult, ScannerError> {
        // Check for error response
        if response.starts_with(responses::ERROR) {
            return Err(ScannerError::DeviceError(response.to_string()));
        }

        // Expected format: OKS,RackBarcode,A01:barcode1,A02:barcode2,...
        if !response.starts_with(responses::OK_SCAN) {
            return Err(ScannerError::InvalidResponse(format!(
                "Expected OKS prefix, got: {}",
                response
            )));
        }

        let parts: Vec<&str> = response.split(',').collect();
        if parts.len() < 2 {
            return Err(ScannerError::InvalidResponse(
                "Response too short".to_string(),
            ));
        }

        let mut result = ScanResult {
            rack_barcode: None,
            positions: HashMap::new(),
            empty_positions: Vec::new(),
            error_positions: Vec::new(),
            raw_response: response.to_string(),
        };

        // Skip "OKS" and parse rack barcode
        if parts.len() > 1 && !parts[1].contains(':') {
            let rack = parts[1].trim();
            if !rack.is_empty() && rack != responses::EMPTY && rack != responses::NO_READ {
                result.rack_barcode = Some(rack.to_string());
            }
        }

        // Parse position:barcode pairs
        for part in parts.iter().skip(1) {
            if let Some((pos, barcode)) = part.split_once(':') {
                let pos = pos.trim().to_uppercase();
                let barcode = barcode.trim();

                match barcode {
                    "" | "EMPTY" => {
                        result.empty_positions.push(pos);
                    }
                    "NO READ" | "ERROR" => {
                        result.error_positions.push(pos);
                    }
                    _ => {
                        result.positions.insert(pos, barcode.to_string());
                    }
                }
            }
        }

        Ok(result)
    }

    /// Gets the scanner status.
    pub async fn get_status(&self) -> Result<String, ScannerError> {
        let mut stream = self.connect().await?;
        self.send_command(&mut stream, commands::STATUS).await
    }

    /// Gets the scanner version information.
    pub async fn get_version(&self) -> Result<String, ScannerError> {
        let mut stream = self.connect().await?;
        self.send_command(&mut stream, commands::VERSION).await
    }

    /// Resets the scanner.
    pub async fn reset(&self) -> Result<(), ScannerError> {
        let mut stream = self.connect().await?;
        let response = self.send_command(&mut stream, commands::RESET).await?;

        if response.starts_with(responses::OK_RESET) {
            Ok(())
        } else {
            Err(ScannerError::DeviceError(response))
        }
    }

    /// Checks if the scanner is reachable.
    pub async fn ping(&self) -> bool {
        self.get_status().await.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scan_response_success() {
        let client = VisionMateClient::connect_to("localhost");
        let response = "OKS,RACK123,A01:TUBE001,A02:TUBE002,A03:EMPTY,B01:NO READ";

        let result = client.parse_scan_response(response).unwrap();

        assert_eq!(result.rack_barcode, Some("RACK123".to_string()));
        assert_eq!(result.positions.len(), 2);
        assert_eq!(result.get_barcode("A01"), Some(&"TUBE001".to_string()));
        assert_eq!(result.get_barcode("A02"), Some(&"TUBE002".to_string()));
        assert!(result.empty_positions.contains(&"A03".to_string()));
        assert!(result.error_positions.contains(&"B01".to_string()));
    }

    #[test]
    fn test_parse_scan_response_error() {
        let client = VisionMateClient::connect_to("localhost");
        let response = "ERR,Scanner not ready";

        let result = client.parse_scan_response(response);
        assert!(matches!(result, Err(ScannerError::DeviceError(_))));
    }

    #[test]
    fn test_config_builder() {
        let config = ScannerConfig::new("192.168.1.100")
            .port(9000)
            .connect_timeout(10)
            .read_timeout(30);

        assert_eq!(config.host, "192.168.1.100");
        assert_eq!(config.port, 9000);
        assert_eq!(config.connect_timeout_secs, 10);
        assert_eq!(config.read_timeout_secs, 30);
    }
}

