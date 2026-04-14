//! Heartbeat service

use std::time::Duration;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HeartbeatPayload {
    pub is_online: bool,
    pub pending_sync_count: i32,
    pub agent_version: String,
    pub machine_name: String,
    pub platform: String,
    pub os_version: String,
    pub capabilities: Vec<String>,
}

pub async fn send_heartbeat(
    api_url: &str,
    token: &str,
    payload: &HeartbeatPayload,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/agent-heartbeat", api_url);
    
    client
        .post(&url)
        .header("x-api-key", token)
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(10))
        .json(payload)
        .send()
        .await?;
    
    Ok(())
}

pub fn get_machine_info() -> (String, String, String) {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    
    let platform = std::env::consts::OS.to_string();
    let os_version = os_info::get().version().to_string();
    
    (hostname, platform, os_version)
}
