use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use thiserror::Error;

use crate::config::AppConfig;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct Database {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJobRecord {
    pub id: String,
    pub order_id: Option<String>,
    pub job_type: String,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub printed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    pub total_jobs: i64,
    pub successful_jobs: i64,
    pub failed_jobs: i64,
    pub today_jobs: i64,
    pub uptime_secs: i64,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS print_jobs (
                id TEXT PRIMARY KEY,
                order_id TEXT,
                job_type TEXT NOT NULL,
                content TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                error_message TEXT,
                created_at TEXT NOT NULL,
                printed_at TEXT,
                synced INTEGER DEFAULT 0
            );
            
            CREATE TABLE IF NOT EXISTS pending_sync (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                status TEXT NOT NULL,
                error_message TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (job_id) REFERENCES print_jobs(id)
            );
            
            CREATE INDEX IF NOT EXISTS idx_jobs_status ON print_jobs(status);
            CREATE INDEX IF NOT EXISTS idx_jobs_synced ON print_jobs(synced);
            CREATE INDEX IF NOT EXISTS idx_jobs_created ON print_jobs(created_at);
            "
        )?;
        
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    
    pub fn save_config(&self, config: &AppConfig) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();
        let json = serde_json::to_string(config)?;
        
        conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES ('app_config', ?)",
            params![json],
        )?;
        
        Ok(())
    }
    
    pub fn load_config(&self) -> Result<AppConfig, DbError> {
        let conn = self.conn.lock().unwrap();
        
        let result: Result<String, _> = conn.query_row(
            "SELECT value FROM config WHERE key = 'app_config'",
            [],
            |row| row.get(0),
        );
        
        match result {
            Ok(json) => Ok(serde_json::from_str(&json)?),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AppConfig::new()),
            Err(e) => Err(e.into()),
        }
    }
    
    pub fn save_job(&self, job: &PrintJobRecord) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT OR REPLACE INTO print_jobs 
             (id, order_id, job_type, status, error_message, created_at, printed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                job.id,
                job.order_id,
                job.job_type,
                job.status,
                job.error_message,
                job.created_at,
                job.printed_at,
            ],
        )?;
        
        Ok(())
    }
    
    pub fn update_job_status(
        &self,
        job_id: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "UPDATE print_jobs SET status = ?, error_message = ?, printed_at = ? WHERE id = ?",
            params![status, error, now, job_id],
        )?;
        
        Ok(())
    }
    
    pub fn get_recent_jobs(&self, limit: i32) -> Result<Vec<PrintJobRecord>, DbError> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, order_id, job_type, status, error_message, created_at, printed_at
             FROM print_jobs
             ORDER BY created_at DESC
             LIMIT ?"
        )?;
        
        let jobs = stmt.query_map(params![limit], |row| {
            Ok(PrintJobRecord {
                id: row.get(0)?,
                order_id: row.get(1)?,
                job_type: row.get(2)?,
                status: row.get(3)?,
                error_message: row.get(4)?,
                created_at: row.get(5)?,
                printed_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(jobs)
    }
    
    pub fn get_stats(&self) -> Result<AgentStats, DbError> {
        let conn = self.conn.lock().unwrap();
        
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM print_jobs",
            [],
            |row| row.get(0),
        )?;
        
        let successful: i64 = conn.query_row(
            "SELECT COUNT(*) FROM print_jobs WHERE status = 'printed'",
            [],
            |row| row.get(0),
        )?;
        
        let failed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM print_jobs WHERE status = 'failed'",
            [],
            |row| row.get(0),
        )?;
        
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let today_jobs: i64 = conn.query_row(
            "SELECT COUNT(*) FROM print_jobs WHERE created_at LIKE ?",
            params![format!("{}%", today)],
            |row| row.get(0),
        )?;
        
        Ok(AgentStats {
            total_jobs: total,
            successful_jobs: successful,
            failed_jobs: failed,
            today_jobs,
            uptime_secs: 0, // Will be calculated by frontend
        })
    }
    
    pub fn add_pending_sync(
        &self,
        job_id: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO pending_sync (job_id, status, error_message, created_at)
             VALUES (?, ?, ?, ?)",
            params![job_id, status, error, now],
        )?;
        
        Ok(())
    }
    
    pub fn get_pending_sync(&self) -> Result<Vec<(i64, String, String, Option<String>)>, DbError> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, job_id, status, error_message FROM pending_sync ORDER BY created_at"
        )?;
        
        let pending = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(pending)
    }
    
    pub fn remove_pending_sync(&self, id: i64) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM pending_sync WHERE id = ?", params![id])?;
        Ok(())
    }
}
