//! Tauri IPC commands for gateway subsystem

use super::cron::CronScheduler;
use super::discord::{DiscordConfig, DiscordGateway};
use std::sync::Mutex;
use tauri::State;

/// Gateway subsystem managed state
pub struct GatewayState {
    pub discord: tokio::sync::Mutex<Option<DiscordGateway>>,
    pub cron: Mutex<CronScheduler>,
}

#[tauri::command]
pub async fn gateway_send_message(
    content: String,
    state: State<'_, GatewayState>,
) -> Result<(), String> {
    let discord = state.discord.lock().await;
    match discord.as_ref() {
        Some(gw) => gw.send_message(&content).await,
        None => Err("Discord 未配置".into()),
    }
}

#[tauri::command]
pub async fn gateway_send_notification(
    title: String,
    body: String,
    state: State<'_, GatewayState>,
) -> Result<(), String> {
    let discord = state.discord.lock().await;
    match discord.as_ref() {
        Some(gw) => gw.send_notification(&title, &body).await,
        None => Err("Discord 未配置".into()),
    }
}

#[tauri::command]
pub fn cron_add_job(
    name: String,
    interval_secs: u64,
    command: String,
    state: State<'_, GatewayState>,
) -> Result<String, String> {
    let mut cron = state.cron.lock().map_err(|e| e.to_string())?;
    let id = cron.add_job(&name, interval_secs, &command);
    Ok(id)
}

#[tauri::command]
pub fn cron_remove_job(id: String, state: State<'_, GatewayState>) -> Result<(), String> {
    let mut cron = state.cron.lock().map_err(|e| e.to_string())?;
    cron.remove_job(&id)
}

#[tauri::command]
pub fn cron_list_jobs(state: State<'_, GatewayState>) -> Result<Vec<serde_json::Value>, String> {
    let cron = state.cron.lock().map_err(|e| e.to_string())?;
    Ok(cron
        .list_jobs()
        .iter()
        .map(|j| serde_json::to_value(j).unwrap_or_default())
        .collect())
}
