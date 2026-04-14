//! Polling service for fetching print jobs and commands

use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tokio::sync::Mutex;

use crate::db::Database;
use crate::escpos::receipt::{ReceiptContent, ReceiptFormatter};
use crate::hardware::printer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResponse {
    pub print_jobs: Vec<PrintJob>,
    pub scale_requests: Vec<ScaleRequest>,
    pub commands: Vec<AgentCommand>,
    pub config: PollConfig,
    pub server_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJob {
    pub id: String,
    pub order_id: Option<String>,
    pub printer_type: Option<String>,
    pub printer_id: Option<String>,
    pub content: serde_json::Value,
    pub copies: Option<i32>,
    pub job_type: Option<String>,
    pub pickup_code: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleRequest {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCommand {
    pub id: String,
    pub command_type: String,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollConfig {
    pub printers: Vec<serde_json::Value>,
    pub scales: Vec<serde_json::Value>,
    pub settings: Option<serde_json::Value>,
}

pub struct PollingService {
    token: String,
    supabase_key: String,
    api_url: String,
    db: Arc<Database>,
    app_handle: tauri::AppHandle,
    running: Arc<Mutex<bool>>,
}

impl PollingService {
    pub fn new(
        token: String,
        supabase_key: String,
        api_url: String,
        db: Arc<Database>,
        app_handle: tauri::AppHandle,
    ) -> Self {
        Self {
            token,
            supabase_key,
            api_url,
            db,
            app_handle,
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    pub async fn start(&mut self) {
        let mut running = self.running.lock().await;
        if *running {
            return;
        }
        *running = true;
        drop(running);
        
        let token = self.token.clone();
        let supabase_key = self.supabase_key.clone();
        let api_url = self.api_url.clone();
        let db = self.db.clone();
        let app_handle = self.app_handle.clone();
        let running = self.running.clone();
        
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            
            loop {
                {
                    let is_running = running.lock().await;
                    if !*is_running {
                        break;
                    }
                }
                
                match poll_once(&client, &api_url, &token, &supabase_key).await {
                    Ok(response) => {
                        // Process print jobs
                        for job in response.print_jobs {
                            process_print_job(&job, &db, &app_handle, &client, &api_url, &token, &supabase_key).await;
                        }
                        
                        // Process scale requests
                        for scale_req in &response.scale_requests {
                            if scale_req.status == "pending" || scale_req.status == "reading" {
                                process_scale_request(scale_req, &app_handle, &client, &api_url, &token, &supabase_key, &response.config).await;
                            }
                        }
                        
                        // Process commands
                        for cmd in response.commands {
                            process_command(&cmd, &app_handle, &client, &api_url, &token, &supabase_key).await;
                        }
                        
                        // Emit status update
                        let _ = app_handle.emit_all("poll_success", &response.server_time);
                    }
                    Err(e) => {
                        log::error!("Polling error: {}", e);
                        let _ = app_handle.emit_all("poll_error", e.to_string());
                    }
                }
                
                // Sync pending offline jobs
                sync_pending(&db, &client, &api_url, &token, &supabase_key).await;
                
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
    }
    
    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;
    }
}

async fn poll_once(
    client: &reqwest::Client,
    api_url: &str,
    token: &str,
    supabase_key: &str,
) -> Result<PollResponse, reqwest::Error> {
    let url = format!("{}/agent-unified-poll", api_url);
    
    let mut request = client.get(&url)
        .header("x-api-key", token)
        .header("Content-Type", "application/json");

    if !supabase_key.is_empty() {
        request = request.header("apikey", supabase_key)
                        .header("Authorization", format!("Bearer {}", supabase_key));
    }

    let response = request
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .json::<PollResponse>()
        .await?;
    
    Ok(response)
}

async fn process_print_job(
    job: &PrintJob,
    db: &Database,
    app_handle: &tauri::AppHandle,
    client: &reqwest::Client,
    api_url: &str,
    token: &str,
    supabase_key: &str,
) {
    log::info!("Processing print job: {}", job.id);
    
    // Parse content
    let content: ReceiptContent = match serde_json::from_value(job.content.clone()) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to parse job content: {}", e);
            report_job_status(client, api_url, token, supabase_key, &job.id, "failed", Some(&e.to_string())).await;
            return;
        }
    };
    
    // Format receipt
    let formatter = ReceiptFormatter::new(80); // TODO: get from config
    let job_type = job.job_type.as_deref().unwrap_or("receipt");
    
    let data = match job_type {
        "kitchen" => formatter.format_sector_receipt(&content, "COZINHA"),
        "bar" => formatter.format_sector_receipt(&content, "BAR"),
        "delivery" => formatter.format_sector_receipt(&content, "ENTREGA"),
        "cashier" => formatter.format_sector_receipt(&content, "BALCÃO"),
        t if t != "order" && t != "receipt" && t != "drawer" => {
            formatter.format_sector_receipt(&content, &t.to_uppercase())
        }
        _ => formatter.format_customer_receipt(&content),
    };
    
    // Print (TODO: get printer config)
    let result = printer::print_network(&data, "192.168.1.100").await;
    
    match result {
        Ok(_) => {
            log::info!("Job {} printed successfully", job.id);
            report_job_status(client, api_url, token, supabase_key, &job.id, "printed", None).await;
            let _ = app_handle.emit_all("job_printed", &job.id);
        }
        Err(e) => {
            log::error!("Job {} failed: {}", job.id, e);
            report_job_status(client, api_url, token, supabase_key, &job.id, "failed", Some(&e.to_string())).await;
            let _ = app_handle.emit_all("job_failed", (&job.id, e.to_string()));
        }
    }
    
    // Save to local DB
    let _ = db.save_job(&crate::db::PrintJobRecord {
        id: job.id.clone(),
        order_id: job.order_id.clone(),
        job_type: job_type.to_string(),
        status: "printed".to_string(),
        error_message: None,
        created_at: job.created_at.clone(),
        printed_at: Some(chrono::Utc::now().to_rfc3339()),
    });
}

async fn process_command(
    cmd: &AgentCommand,
    app_handle: &tauri::AppHandle,
    client: &reqwest::Client,
    api_url: &str,
    token: &str,
    supabase_key: &str,
) {
    log::info!("Processing command: {} ({})", cmd.id, cmd.command_type);
    
    match cmd.command_type.as_str() {
        "open_drawer" => {
            // TODO: get drawer config
            match crate::hardware::drawer::open("network", "192.168.1.100").await {
                Ok(_) => {
                    let _ = app_handle.emit_all("drawer_opened", ());
                }
                Err(e) => {
                    log::error!("Failed to open drawer: {}", e);
                }
            }
        }
        "read_weight" => {
            let payload = cmd.payload.as_ref();
            let conn_type = payload
                .and_then(|p| p.get("connection_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("tcp");

            let weight_result = match conn_type {
                "usb" | "serial" => {
                    let serial_port = payload
                        .and_then(|p| p.get("serial_port"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("COM9");
                    let baud_rate = payload
                        .and_then(|p| p.get("baud_rate"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(9600) as u32;
                    log::info!("Reading scale via serial: {} @ {}", serial_port, baud_rate);
                    crate::hardware::scale::read_weight(serial_port).await
                }
                _ => {
                    let ip = payload
                        .and_then(|p| p.get("ip"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("192.168.1.100");
                    let port = payload
                        .and_then(|p| p.get("port"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(4001) as u16;
                    log::info!("Reading scale via TCP: {}:{}", ip, port);
                    crate::hardware::scale::read_weight_tcp(ip, port).await
                }
            };

            match weight_result {
                Ok(weight) => {
                    let _ = app_handle.emit_all("scale_weight_read", serde_json::json!({
                        "weight": weight,
                        "command_id": cmd.id,
                    }));
                    report_scale_weight(client, api_url, token, supabase_key, &cmd.id, weight, None).await;
                }
                Err(e) => {
                    log::error!("Failed to read scale: {}", e);
                    report_scale_weight(client, api_url, token, supabase_key, &cmd.id, 0.0, Some(&e.to_string())).await;
                }
            }
        }
        "test_printer" => {
            // TODO: implement
        }
        _ => {
            log::warn!("Unknown command type: {}", cmd.command_type);
        }
    }
}

async fn report_job_status(
    client: &reqwest::Client,
    api_url: &str,
    token: &str,
    supabase_key: &str,
    job_id: &str,
    status: &str,
    error: Option<&str>,
) {
    let url = format!("{}/print-job-status", api_url);
    
    let body = serde_json::json!({
        "job_id": job_id,
        "status": status,
        "error_message": error,
    });
    
    let mut request = client.post(&url)
        .header("x-api-key", token)
        .header("Content-Type", "application/json")
        .json(&body);

    if !supabase_key.is_empty() {
        request = request.header("apikey", supabase_key)
                        .header("Authorization", format!("Bearer {}", supabase_key));
    }
    
    let _ = request.send().await;
}

async fn sync_pending(
    db: &Database,
    client: &reqwest::Client,
    api_url: &str,
    token: &str,
    supabase_key: &str,
) {
    if let Ok(pending) = db.get_pending_sync() {
        for (id, job_id, status, error) in pending {
            report_job_status(client, api_url, token, supabase_key, &job_id, &status, error.as_deref()).await;
            let _ = db.remove_pending_sync(id);
        }
    }
}

async fn process_scale_request(
    scale_req: &ScaleRequest,
    app_handle: &tauri::AppHandle,
    client: &reqwest::Client,
    api_url: &str,
    token: &str,
    supabase_key: &str,
    config: &PollConfig,
) {
    log::info!("Processing scale request: {}", scale_req.id);

    // Get scale config from poll config
    let scale_cfg = config.scales.first();
    let conn_type = scale_cfg
        .and_then(|cfg| cfg.get("connection_type"))
        .and_then(|v| v.as_str())
        .unwrap_or("tcp");

    let weight_result = match conn_type {
        "usb" | "serial" => {
            let serial_port = scale_cfg
                .and_then(|cfg| cfg.get("serial_port"))
                .and_then(|v| v.as_str())
                .unwrap_or("COM9");
            let baud_rate = scale_cfg
                .and_then(|cfg| cfg.get("baud_rate"))
                .and_then(|v| v.as_u64())
                .unwrap_or(9600) as u32;
            log::info!("Reading scale via serial: {} @ {} baud", serial_port, baud_rate);
            crate::hardware::scale::read_weight(serial_port).await
        }
        _ => {
            let ip = scale_cfg
                .and_then(|cfg| cfg.get("ip_address"))
                .and_then(|v| v.as_str())
                .unwrap_or("192.168.1.100");
            let port = scale_cfg
                .and_then(|cfg| cfg.get("port"))
                .and_then(|v| v.as_u64())
                .unwrap_or(4001) as u16;
            log::info!("Reading scale via TCP: {}:{}", ip, port);
            crate::hardware::scale::read_weight_tcp(ip, port).await
        }
    };

    match weight_result {
        Ok(weight) => {
            log::info!("Scale read: {}g for request {}", weight, scale_req.id);
            let _ = app_handle.emit_all("scale_weight_read", serde_json::json!({
                "weight": weight,
                "request_id": scale_req.id,
                "stable": true,
            }));
            report_scale_weight(client, api_url, token, supabase_key, &scale_req.id, weight, None).await;
        }
        Err(e) => {
            log::error!("Scale read failed for {}: {}", scale_req.id, e);
            let _ = app_handle.emit_all("scale_weight_error", serde_json::json!({
                "request_id": scale_req.id,
                "error": e.to_string(),
            }));
            report_scale_weight(client, api_url, token, supabase_key, &scale_req.id, 0.0, Some(&e.to_string())).await;
        }
    }
}

async fn report_scale_weight(
    client: &reqwest::Client,
    api_url: &str,
    token: &str,
    supabase_key: &str,
    request_id: &str,
    weight: f64,
    error: Option<&str>,
) {
    let url = format!("{}/scale-weight-receive", api_url);

    let body = if error.is_some() {
        serde_json::json!({
            "request_id": request_id,
            "peso": 0,
            "unidade": "g",
            "status": "error",
            "error_message": error,
        })
    } else {
        // weight comes in grams from read_weight, convert to kg for the endpoint
        serde_json::json!({
            "request_id": request_id,
            "peso": weight / 1000.0,
            "unidade": "kg",
        })
    };

    let mut request = client.post(&url)
        .header("x-api-key", token)
        .header("Content-Type", "application/json")
        .json(&body);

    if !supabase_key.is_empty() {
        request = request.header("apikey", supabase_key)
                        .header("Authorization", format!("Bearer {}", supabase_key));
    }

    let _ = request.send().await;
}
