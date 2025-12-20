//! Zebra Label Printer Client
//!
//! Async TCP client for Zebra printers using ZPL (Zebra Programming Language).
//! Supports printing labels for samples, libraries, pools, and boxes.

use std::time::Duration;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, error, info};

/// Errors that can occur during printer operations.
#[derive(Debug, Error)]
pub enum PrinterError {
    #[error("Failed to connect to printer at {host}:{port}: {source}")]
    ConnectionFailed {
        host: String,
        port: u16,
        source: std::io::Error,
    },

    #[error("Connection timed out after {timeout_secs}s")]
    ConnectionTimeout { timeout_secs: u64 },

    #[error("Failed to send print job: {0}")]
    SendFailed(#[from] std::io::Error),

    #[error("Invalid label template: {0}")]
    InvalidTemplate(String),
}

/// Configuration for the Zebra printer client.
#[derive(Debug, Clone)]
pub struct PrinterConfig {
    /// Printer hostname or IP address
    pub host: String,
    /// Printer port (default: 9100 for raw printing)
    pub port: u16,
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
    /// Default label width in dots (at 203 DPI: 203 dots = 1 inch)
    pub label_width_dots: u32,
    /// Default label height in dots
    pub label_height_dots: u32,
    /// Print darkness (0-30, default: 15)
    pub darkness: u8,
    /// Print speed (1-14, default: 6)
    pub speed: u8,
}

impl Default for PrinterConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 9100,
            connect_timeout_secs: 5,
            label_width_dots: 406, // ~2 inch at 203 DPI
            label_height_dots: 203, // ~1 inch at 203 DPI
            darkness: 15,
            speed: 6,
        }
    }
}

impl PrinterConfig {
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

    /// Sets the label dimensions in dots.
    pub fn label_size(mut self, width: u32, height: u32) -> Self {
        self.label_width_dots = width;
        self.label_height_dots = height;
        self
    }
}

/// Barcode types supported by ZPL.
#[derive(Debug, Clone, Copy)]
pub enum BarcodeType {
    /// Code 128 (most common for lab use)
    Code128,
    /// Code 39
    Code39,
    /// DataMatrix (2D)
    DataMatrix,
    /// QR Code (2D)
    QrCode,
}

impl BarcodeType {
    /// Returns the ZPL command for this barcode type.
    fn zpl_command(&self) -> &'static str {
        match self {
            Self::Code128 => "^BC",
            Self::Code39 => "^B3",
            Self::DataMatrix => "^BX",
            Self::QrCode => "^BQ",
        }
    }
}

/// A label field (text, barcode, etc.).
#[derive(Debug, Clone)]
pub enum LabelField {
    /// Plain text
    Text {
        x: u32,
        y: u32,
        text: String,
        font: char,
        height: u32,
        width: u32,
    },
    /// 1D or 2D barcode
    Barcode {
        x: u32,
        y: u32,
        data: String,
        barcode_type: BarcodeType,
        height: u32,
        /// Whether to print human-readable text below
        show_text: bool,
    },
    /// Horizontal line
    Line {
        x: u32,
        y: u32,
        width: u32,
        thickness: u32,
    },
    /// Box/rectangle
    Box {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        border: u32,
    },
}

/// Builder for creating ZPL label templates.
#[derive(Debug, Clone)]
pub struct LabelBuilder {
    fields: Vec<LabelField>,
    width: u32,
    height: u32,
    copies: u32,
}

impl LabelBuilder {
    /// Creates a new label builder with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            fields: Vec::new(),
            width,
            height,
            copies: 1,
        }
    }

    /// Sets the number of copies to print.
    pub fn copies(mut self, copies: u32) -> Self {
        self.copies = copies;
        self
    }

    /// Adds a text field.
    pub fn text(
        mut self,
        x: u32,
        y: u32,
        text: impl Into<String>,
        font: char,
        height: u32,
    ) -> Self {
        self.fields.push(LabelField::Text {
            x,
            y,
            text: text.into(),
            font,
            height,
            width: height, // Square font by default
        });
        self
    }

    /// Adds a text field with custom width.
    pub fn text_sized(
        mut self,
        x: u32,
        y: u32,
        text: impl Into<String>,
        font: char,
        height: u32,
        width: u32,
    ) -> Self {
        self.fields.push(LabelField::Text {
            x,
            y,
            text: text.into(),
            font,
            height,
            width,
        });
        self
    }

    /// Adds a barcode field.
    pub fn barcode(
        mut self,
        x: u32,
        y: u32,
        data: impl Into<String>,
        barcode_type: BarcodeType,
        height: u32,
        show_text: bool,
    ) -> Self {
        self.fields.push(LabelField::Barcode {
            x,
            y,
            data: data.into(),
            barcode_type,
            height,
            show_text,
        });
        self
    }

    /// Adds a Code128 barcode (most common).
    pub fn code128(
        mut self,
        x: u32,
        y: u32,
        data: impl Into<String>,
        height: u32,
    ) -> Self {
        self.fields.push(LabelField::Barcode {
            x,
            y,
            data: data.into(),
            barcode_type: BarcodeType::Code128,
            height,
            show_text: true,
        });
        self
    }

    /// Adds a DataMatrix barcode (2D).
    pub fn datamatrix(
        mut self,
        x: u32,
        y: u32,
        data: impl Into<String>,
    ) -> Self {
        self.fields.push(LabelField::Barcode {
            x,
            y,
            data: data.into(),
            barcode_type: BarcodeType::DataMatrix,
            height: 0, // 2D barcodes auto-size
            show_text: false,
        });
        self
    }

    /// Adds a horizontal line.
    pub fn line(mut self, x: u32, y: u32, width: u32, thickness: u32) -> Self {
        self.fields.push(LabelField::Line {
            x,
            y,
            width,
            thickness,
        });
        self
    }

    /// Adds a box/rectangle.
    pub fn rect(mut self, x: u32, y: u32, width: u32, height: u32, border: u32) -> Self {
        self.fields.push(LabelField::Box {
            x,
            y,
            width,
            height,
            border,
        });
        self
    }

    /// Builds the ZPL command string.
    pub fn build(&self) -> String {
        let mut zpl = String::new();

        // Start label
        zpl.push_str("^XA\n");

        // Set print quantity
        if self.copies > 1 {
            zpl.push_str(&format!("^PQ{}\n", self.copies));
        }

        // Add fields
        for field in &self.fields {
            match field {
                LabelField::Text {
                    x,
                    y,
                    text,
                    font,
                    height,
                    width,
                } => {
                    zpl.push_str(&format!(
                        "^FO{},{}^A{},{},{}^FD{}^FS\n",
                        x, y, font, height, width, text
                    ));
                }
                LabelField::Barcode {
                    x,
                    y,
                    data,
                    barcode_type,
                    height,
                    show_text,
                } => {
                    let cmd = barcode_type.zpl_command();
                    let print_text = if *show_text { "Y" } else { "N" };
                    match barcode_type {
                        BarcodeType::Code128 => {
                            zpl.push_str(&format!(
                                "^FO{},{}{}N,{},{}^FD{}^FS\n",
                                x, y, cmd, height, print_text, data
                            ));
                        }
                        BarcodeType::DataMatrix => {
                            zpl.push_str(&format!(
                                "^FO{},{}{}N,4,200^FD{}^FS\n",
                                x, y, cmd, data
                            ));
                        }
                        BarcodeType::QrCode => {
                            zpl.push_str(&format!(
                                "^FO{},{}{}N,2,4^FDQA,{}^FS\n",
                                x, y, cmd, data
                            ));
                        }
                        BarcodeType::Code39 => {
                            zpl.push_str(&format!(
                                "^FO{},{}{}N,N,{},{}^FD{}^FS\n",
                                x, y, cmd, height, print_text, data
                            ));
                        }
                    }
                }
                LabelField::Line {
                    x,
                    y,
                    width,
                    thickness,
                } => {
                    zpl.push_str(&format!("^FO{},{}^GB{},{},{}^FS\n", x, y, width, thickness, thickness));
                }
                LabelField::Box {
                    x,
                    y,
                    width,
                    height,
                    border,
                } => {
                    zpl.push_str(&format!("^FO{},{}^GB{},{},{}^FS\n", x, y, width, height, border));
                }
            }
        }

        // End label
        zpl.push_str("^XZ\n");

        zpl
    }
}

/// Async client for Zebra label printers.
///
/// # Example
///
/// ```no_run
/// use miso_infrastructure::hardware::printer::{ZebraPrinter, PrinterConfig, LabelBuilder};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = PrinterConfig::new("192.168.1.50");
///     let printer = ZebraPrinter::new(config);
///     
///     let label = LabelBuilder::new(406, 203)
///         .text(10, 10, "Sample: SAM-001", '0', 30)
///         .code128(10, 50, "SAM-001", 60)
///         .build();
///     
///     printer.print_raw(&label).await?;
///     
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ZebraPrinter {
    config: PrinterConfig,
}

impl ZebraPrinter {
    /// Creates a new Zebra printer client.
    pub fn new(config: PrinterConfig) -> Self {
        Self { config }
    }

    /// Creates a client for the given host with default settings.
    pub fn connect_to(host: impl Into<String>) -> Self {
        Self::new(PrinterConfig::new(host))
    }

    /// Establishes a connection to the printer.
    async fn connect(&self) -> Result<TcpStream, PrinterError> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        debug!("Connecting to Zebra printer at {}", addr);

        let stream = timeout(
            Duration::from_secs(self.config.connect_timeout_secs),
            TcpStream::connect(&addr),
        )
        .await
        .map_err(|_| PrinterError::ConnectionTimeout {
            timeout_secs: self.config.connect_timeout_secs,
        })?
        .map_err(|e| PrinterError::ConnectionFailed {
            host: self.config.host.clone(),
            port: self.config.port,
            source: e,
        })?;

        info!("Connected to Zebra printer at {}", addr);
        Ok(stream)
    }

    /// Prints a raw ZPL command string.
    pub async fn print_raw(&self, zpl: &str) -> Result<(), PrinterError> {
        let mut stream = self.connect().await?;
        stream.write_all(zpl.as_bytes()).await?;
        stream.flush().await?;
        debug!("Sent ZPL to printer ({} bytes)", zpl.len());
        Ok(())
    }

    /// Prints a label built with LabelBuilder.
    pub async fn print_label(&self, label: &LabelBuilder) -> Result<(), PrinterError> {
        let zpl = label.build();
        self.print_raw(&zpl).await
    }

    /// Creates a label builder with the printer's default dimensions.
    pub fn label(&self) -> LabelBuilder {
        LabelBuilder::new(
            self.config.label_width_dots,
            self.config.label_height_dots,
        )
    }

    /// Prints a simple sample label.
    pub async fn print_sample_label(
        &self,
        barcode: &str,
        name: &str,
        project: &str,
    ) -> Result<(), PrinterError> {
        let label = self.label()
            .text(10, 10, name, '0', 25)
            .text(10, 40, project, '0', 20)
            .code128(10, 70, barcode, 50)
            .build();

        self.print_raw(&label).await
    }

    /// Prints multiple copies of a label.
    pub async fn print_labels(
        &self,
        label: &LabelBuilder,
        copies: u32,
    ) -> Result<(), PrinterError> {
        let label_with_copies = LabelBuilder {
            fields: label.fields.clone(),
            width: label.width,
            height: label.height,
            copies,
        };
        self.print_label(&label_with_copies).await
    }

    /// Tests printer connectivity.
    pub async fn ping(&self) -> bool {
        match self.connect().await {
            Ok(_) => {
                info!("Printer ping successful");
                true
            }
            Err(e) => {
                error!("Printer ping failed: {}", e);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_builder_simple() {
        let label = LabelBuilder::new(400, 200)
            .text(10, 10, "Test", '0', 30)
            .code128(10, 50, "12345", 60)
            .build();

        assert!(label.contains("^XA"));
        assert!(label.contains("^XZ"));
        assert!(label.contains("Test"));
        assert!(label.contains("12345"));
    }

    #[test]
    fn test_label_builder_with_copies() {
        let label = LabelBuilder::new(400, 200)
            .copies(5)
            .text(10, 10, "Test", '0', 30)
            .build();

        assert!(label.contains("^PQ5"));
    }

    #[test]
    fn test_label_with_datamatrix() {
        let label = LabelBuilder::new(400, 200)
            .datamatrix(10, 10, "SAM-001")
            .build();

        assert!(label.contains("^BX")); // DataMatrix command
        assert!(label.contains("SAM-001"));
    }

    #[test]
    fn test_config_builder() {
        let config = PrinterConfig::new("192.168.1.50")
            .port(9100)
            .label_size(812, 406);

        assert_eq!(config.host, "192.168.1.50");
        assert_eq!(config.port, 9100);
        assert_eq!(config.label_width_dots, 812);
    }
}

