//! Serial port utilities

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerialError {
    #[error("Serial port error: {0}")]
    Port(#[from] serialport::Error),
}

/// List available serial ports
pub fn list_ports() -> Result<Vec<String>, SerialError> {
    let ports = serialport::available_ports()?;
    Ok(ports.into_iter().map(|p| p.port_name).collect())
}

/// Get detailed port info
pub fn get_port_info(port_name: &str) -> Option<serialport::SerialPortInfo> {
    serialport::available_ports()
        .ok()?
        .into_iter()
        .find(|p| p.port_name == port_name)
}
