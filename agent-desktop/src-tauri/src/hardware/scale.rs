//! Scale (balance) reading — Serial and TCP/IP support

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScaleError {
    #[error("Serial port error: {0}")]
    SerialError(#[from] serialport::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Timeout reading weight")]
    Timeout,
    #[error("Weight not stable")]
    NotStable,
}

/// Read weight from scale using Toledo protocol
pub async fn read_weight(port: &str) -> Result<f64, ScaleError> {
    let mut serial = serialport::new(port, 9600)
        .timeout(Duration::from_secs(5))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()?;
    
    // Send weight request (ENQ)
    serial.write(&[0x05])?;
    serial.flush()?;
    
    // Read response
    let mut buffer = [0u8; 32];
    let mut response = Vec::new();
    let start = std::time::Instant::now();
    
    loop {
        if start.elapsed() > Duration::from_secs(3) {
            return Err(ScaleError::Timeout);
        }
        
        match serial.read(&mut buffer) {
            Ok(n) if n > 0 => {
                response.extend_from_slice(&buffer[..n]);
                // Check for end of message (CR or ETX)
                if response.contains(&0x0D) || response.contains(&0x03) {
                    break;
                }
            }
            Ok(_) => continue,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
            Err(e) => return Err(e.into()),
        }
    }
    
    parse_toledo_response(&response)
}

/// Parse Toledo protocol response
/// Format: STX + Status + Weight (6 digits) + Unit + CR
fn parse_toledo_response(data: &[u8]) -> Result<f64, ScaleError> {
    if data.len() < 8 {
        return Err(ScaleError::ParseError("Response too short".to_string()));
    }
    
    // Find weight digits in response
    let weight_str: String = data
        .iter()
        .filter(|b| b.is_ascii_digit() || **b == b'.' || **b == b',')
        .map(|&b| b as char)
        .collect();
    
    if weight_str.is_empty() {
        return Err(ScaleError::ParseError("No weight found".to_string()));
    }
    
    // Replace comma with dot for parsing
    let weight_str = weight_str.replace(',', ".");
    
    weight_str
        .parse::<f64>()
        .map_err(|e| ScaleError::ParseError(format!("Cannot parse '{}': {}", weight_str, e)))
}

/// Read weight using Filizola protocol
pub async fn read_weight_filizola(port: &str) -> Result<f64, ScaleError> {
    let mut serial = serialport::new(port, 2400)
        .timeout(Duration::from_secs(5))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()?;
    
    // Filizola sends weight continuously
    let mut buffer = [0u8; 64];
    let mut response = Vec::new();
    let start = std::time::Instant::now();
    
    loop {
        if start.elapsed() > Duration::from_secs(3) {
            return Err(ScaleError::Timeout);
        }
        
        match serial.read(&mut buffer) {
            Ok(n) if n > 0 => {
                response.extend_from_slice(&buffer[..n]);
                if response.len() >= 15 {
                    break;
                }
            }
            Ok(_) => continue,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
            Err(e) => return Err(e.into()),
        }
    }
    
    parse_filizola_response(&response)
}

fn parse_filizola_response(data: &[u8]) -> Result<f64, ScaleError> {
    // Filizola format: varies by model, typically ASCII weight
    let text: String = data
        .iter()
        .filter(|b| b.is_ascii_graphic() || **b == b' ')
        .map(|&b| b as char)
        .collect();
    
    // Extract digits and decimal
    let weight_str: String = text
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .collect();
    
    if weight_str.is_empty() {
        return Err(ScaleError::ParseError("No weight found".to_string()));
    }
    
    let weight_str = weight_str.replace(',', ".");
    
    weight_str
        .parse::<f64>()
        .map_err(|e| ScaleError::ParseError(format!("Cannot parse '{}': {}", weight_str, e)))
}

/// Read weight from scale via TCP/IP (Ethernet)
/// Used for Toledo Prix and similar scales with TCP converters
pub async fn read_weight_tcp(ip: &str, port: u16) -> Result<f64, ScaleError> {
    let addr = format!("{}:{}", ip, port);
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().map_err(|e| ScaleError::ParseError(format!("Invalid address: {}", e)))?,
        Duration::from_secs(5),
    ).map_err(|e| ScaleError::IoError(e))?;
    
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    
    // Send ENQ (0x05) — Toledo weight request
    stream.write_all(&[0x05])?;
    stream.flush()?;
    
    // Read response
    let mut buffer = [0u8; 64];
    let mut response = Vec::new();
    let start = std::time::Instant::now();
    
    loop {
        if start.elapsed() > Duration::from_secs(3) {
            if response.is_empty() {
                return Err(ScaleError::Timeout);
            }
            break;
        }
        
        match stream.read(&mut buffer) {
            Ok(n) if n > 0 => {
                response.extend_from_slice(&buffer[..n]);
                // Check for CR (end of Toledo response)
                if response.contains(&0x0D) || response.contains(&0x03) {
                    break;
                }
            }
            Ok(_) => break,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                if !response.is_empty() {
                    break;
                }
                continue;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if !response.is_empty() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
    
    parse_toledo_response(&response)
}

/// Test TCP connectivity to a scale and return weight + metadata
pub async fn test_tcp_connection(ip: &str, port: u16) -> Result<(f64, bool, Vec<u8>), ScaleError> {
    let addr = format!("{}:{}", ip, port);
    let start = std::time::Instant::now();
    
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().map_err(|e| ScaleError::ParseError(format!("Invalid address: {}", e)))?,
        Duration::from_secs(5),
    ).map_err(|e| ScaleError::IoError(e))?;
    
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.write_all(&[0x05])?;
    stream.flush()?;
    
    let mut buffer = [0u8; 64];
    let mut response = Vec::new();
    
    loop {
        if start.elapsed() > Duration::from_secs(5) {
            if response.is_empty() {
                return Err(ScaleError::Timeout);
            }
            break;
        }
        match stream.read(&mut buffer) {
            Ok(n) if n > 0 => {
                response.extend_from_slice(&buffer[..n]);
                if response.contains(&0x0D) || response.contains(&0x03) {
                    break;
                }
            }
            Ok(_) => break,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut || e.kind() == std::io::ErrorKind::WouldBlock => {
                if !response.is_empty() { break; }
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
    
    let weight = parse_toledo_response(&response)?;
    // Check stability: look for 'M' (motion) after STX
    let is_stable = !response.iter().any(|&b| b == b'M');
    
    Ok((weight, is_stable, response))
}
