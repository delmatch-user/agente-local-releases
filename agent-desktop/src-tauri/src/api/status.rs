//! Job status reporting

use std::time::Duration;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct JobStatusPayload {
    pub job_id: String,
    pub status: String,
    pub error_message: Option<String>,
    pub printed_at: Option<String>,
}

pub async fn report_status(
    api_url: &str,
    token: &str,
    payload: &JobStatusPayload,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/print-job-status", api_url);
    
    client
        .post(&url)
        .header("x-agent-token", token)
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(10))
        .json(payload)
        .send()
        .await?;
    
    Ok(())
}
