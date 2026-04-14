//! Thermal printer communication

use super::PrinterInfo;
use crate::escpos;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrinterError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Write failed: {0}")]
    WriteFailed(String),
    #[error("Serial port error: {0}")]
    SerialError(#[from] serialport::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// List available printers (serial ports and common network addresses)
pub fn list_printers() -> Result<Vec<PrinterInfo>, PrinterError> {
    let mut printers = Vec::new();
    
    // List serial ports
    if let Ok(ports) = serialport::available_ports() {
        for port in ports {
            printers.push(PrinterInfo {
                name: port.port_name.clone(),
                port: port.port_name,
                connection_type: "usb".to_string(),
                status: "available".to_string(),
            });
        }
    }
    
    Ok(printers)
}

/// Send data to printer via USB (serial port)
pub async fn print_usb(data: &[u8], port: &str) -> Result<(), PrinterError> {
    let mut serial = serialport::new(port, 9600)
        .timeout(Duration::from_secs(10))
        .open()?;
    
    serial.write_all(data)?;
    serial.flush()?;
    
    Ok(())
}

/// Send data to printer via network (TCP/IP)
pub async fn print_network(data: &[u8], address: &str) -> Result<(), PrinterError> {
    let addr = if address.contains(':') {
        address.to_string()
    } else {
        format!("{}:9100", address)
    };
    
    let mut stream = TcpStream::connect(&addr)
        .map_err(|e| PrinterError::ConnectionFailed(format!("{}: {}", addr, e)))?;
    
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    stream.write_all(data)?;
    stream.flush()?;
    
    Ok(())
}

/// Print data to specified printer
pub async fn print(data: &[u8], connection_type: &str, address: &str) -> Result<(), PrinterError> {
    match connection_type {
        "usb" => print_usb(data, address).await,
        "network" => print_network(data, address).await,
        _ => Err(PrinterError::ConnectionFailed(format!(
            "Unknown connection type: {}",
            connection_type
        ))),
    }
}

/// Test printer connection with a test page
pub async fn test_print(connection_type: &str, address: &str) -> Result<(), PrinterError> {
    let test_data = escpos::format_test_page();
    print(&test_data, connection_type, address).await
}

/// Open cash drawer via printer
pub async fn open_drawer_via_printer(
    connection_type: &str,
    address: &str,
) -> Result<(), PrinterError> {
    let drawer_cmd = escpos::commands::open_drawer();
    print(&drawer_cmd, connection_type, address).await
}
