pub mod drawer;
pub mod printer;
pub mod scale;
pub mod serial;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterInfo {
    pub name: String,
    pub port: String,
    pub connection_type: String,
    pub status: String,
}
