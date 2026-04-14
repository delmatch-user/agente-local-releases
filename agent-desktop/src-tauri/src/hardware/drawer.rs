//! Cash drawer control

use crate::escpos::commands;
use super::printer;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DrawerError {
    #[error("Printer error: {0}")]
    PrinterError(#[from] printer::PrinterError),
}

/// Open cash drawer
pub async fn open(connection_type: &str, address: &str) -> Result<(), DrawerError> {
    let cmd = commands::open_drawer();
    printer::print(&cmd, connection_type, address).await?;
    Ok(())
}

/// Open cash drawer (pin 5)
pub async fn open_pin5(connection_type: &str, address: &str) -> Result<(), DrawerError> {
    let cmd = commands::open_drawer_pin5();
    printer::print(&cmd, connection_type, address).await?;
    Ok(())
}
