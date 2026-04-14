use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub agent_token: Option<String>,
    pub supabase_key: Option<String>,
    pub scale_token: Option<String>,
    pub api_url: String,
    pub polling_interval_secs: u64,
    pub printers: Vec<PrinterConfig>,
    pub scale: Option<ScaleConfig>,
    pub drawer: Option<DrawerConfig>,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub sound_enabled: bool,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            agent_token: None,
            supabase_key: None,
            scale_token: None,
            api_url: "https://szlyzyflalerxuyxfxzh.supabase.co/functions/v1".to_string(),
            polling_interval_secs: 5,
            printers: Vec::new(),
            scale: None,
            drawer: None,
            auto_start: true,
            minimize_to_tray: true,
            sound_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterConfig {
    pub id: String,
    pub name: String,
    pub connection_type: ConnectionType,
    pub address: String, // COM port or IP:port
    pub paper_width: PaperWidth,
    pub is_default: bool,
    pub printer_types: Vec<String>, // "receipt", "kitchen", "label"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    Usb,
    Network,
    Bluetooth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaperWidth {
    #[serde(rename = "58mm")]
    Width58mm,
    #[serde(rename = "80mm")]
    Width80mm,
}

impl PaperWidth {
    pub fn chars_per_line(&self) -> usize {
        match self {
            PaperWidth::Width58mm => 32,
            PaperWidth::Width80mm => 48,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleConfig {
    pub port: String,
    pub baud_rate: u32,
    pub protocol: ScaleProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScaleProtocol {
    Toledo,
    Filizola,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawerConfig {
    pub connection_type: ConnectionType,
    pub address: String,
    pub pulse_on_print: bool,
}
