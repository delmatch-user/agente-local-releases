#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod config;
mod db;
mod escpos;
mod hardware;

use std::sync::Arc;
use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, CustomMenuItem};
use tokio::sync::Mutex;

use crate::config::AppConfig;
use crate::db::Database;
use crate::api::polling::PollingService;

pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub db: Arc<Database>,
    pub polling: Arc<Mutex<Option<PollingService>>>,
}

#[tauri::command]
async fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

#[tauri::command]
async fn save_config(
    state: tauri::State<'_, AppState>,
    new_config: AppConfig,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    *config = new_config.clone();
    state.db.save_config(&new_config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn set_agent_token(
    state: tauri::State<'_, AppState>,
    token: String,
    supabase_key: Option<String>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    config.agent_token = Some(token);
    if let Some(key) = supabase_key {
        config.supabase_key = Some(key);
    }
    state.db.save_config(&config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn set_scale_token(
    state: tauri::State<'_, AppState>,
    token: String,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    config.scale_token = Some(token);
    state.db.save_config(&config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_scale_status(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let config = state.config.lock().await;
    let has_token = config.scale_token.is_some();
    let scale_config = config.scale.clone();
    Ok(serde_json::json!({
        "has_token": has_token,
        "scale_configured": scale_config.is_some(),
        "scale_config": scale_config,
    }))
}

#[tauri::command]
async fn start_polling(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let config = state.config.lock().await;
    
    if config.agent_token.is_none() {
        return Err("Token não configurado".to_string());
    }
    
    let token = config.agent_token.clone().unwrap();
    let supabase_key = config.supabase_key.clone().unwrap_or_default();
    let api_url = config.api_url.clone();
    let db = state.db.clone();

    drop(config);

    let polling = PollingService::new(token, supabase_key, api_url, db, app_handle, state.config.clone());
    
    let mut polling_guard = state.polling.lock().await;
    *polling_guard = Some(polling);
    
    if let Some(ref mut p) = *polling_guard {
        p.start().await;
    }
    
    Ok(())
}

#[tauri::command]
async fn stop_polling(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut polling = state.polling.lock().await;
    if let Some(ref mut p) = *polling {
        p.stop().await;
    }
    *polling = None;
    Ok(())
}

#[tauri::command]
async fn get_printers() -> Result<Vec<hardware::PrinterInfo>, String> {
    hardware::printer::list_printers().map_err(|e| e.to_string())
}

#[tauri::command]
async fn test_printer(
    connection_type: String,
    address: String,
) -> Result<(), String> {
    hardware::printer::test_print(&connection_type, &address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_serial_ports() -> Result<Vec<String>, String> {
    hardware::serial::list_ports().map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_scale_weight(port: String) -> Result<f64, String> {
    hardware::scale::read_weight(&port)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_cash_drawer(
    connection_type: String,
    address: String,
) -> Result<(), String> {
    hardware::drawer::open(&connection_type, &address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_recent_jobs(
    state: tauri::State<'_, AppState>,
    limit: i32,
) -> Result<Vec<db::PrintJobRecord>, String> {
    state.db.get_recent_jobs(limit).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_stats(state: tauri::State<'_, AppState>) -> Result<db::AgentStats, String> {
    state.db.get_stats().map_err(|e| e.to_string())
}

fn main() {
    env_logger::init();
    
    let quit = CustomMenuItem::new("quit".to_string(), "Sair");
    let show = CustomMenuItem::new("show".to_string(), "Abrir");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(quit);
    
    let system_tray = SystemTray::new().with_menu(tray_menu);
    
    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                if let Some(window) = app.get_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "show" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            },
            _ => {}
        })
        .setup(|app| {
            let app_dir = app.path_resolver().app_config_dir().unwrap();
            std::fs::create_dir_all(&app_dir).ok();
            
            let db_path = app_dir.join("agent.db");
            let db = Database::new(&db_path).expect("Falha ao criar banco de dados");
            let db = Arc::new(db);
            
            let config = db.load_config().unwrap_or_default();
            
            let state = AppState {
                config: Arc::new(Mutex::new(config)),
                db,
                polling: Arc::new(Mutex::new(None)),
            };
            
            app.manage(state);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            set_agent_token,
            set_scale_token,
            get_scale_status,
            start_polling,
            stop_polling,
            get_printers,
            test_printer,
            get_serial_ports,
            read_scale_weight,
            open_cash_drawer,
            get_recent_jobs,
            get_stats,
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao executar aplicação");
}
