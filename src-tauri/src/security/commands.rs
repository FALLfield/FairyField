//! Tauri IPC commands for security subsystem

use super::guard::CommandGuard;
use super::injection::InjectionDetector;
use super::redaction::{is_url_safe, SecretRedactor};
use std::sync::Mutex;
use tauri::State;

/// Security subsystem managed state
pub struct SecurityState {
    pub guard: Mutex<CommandGuard>,
    pub injection_detector: InjectionDetector,
    pub redactor: SecretRedactor,
}

#[tauri::command]
pub fn security_check_command(
    command: String,
    state: State<'_, SecurityState>,
) -> Result<super::guard::GuardResult, String> {
    Ok(state
        .guard
        .lock()
        .map_err(|e| e.to_string())?
        .check_command(&command))
}

#[tauri::command]
pub fn security_approve_command(
    command: String,
    session_only: bool,
    state: State<'_, SecurityState>,
) -> Result<(), String> {
    state
        .guard
        .lock()
        .map_err(|e| e.to_string())?
        .approve(&command, session_only);
    Ok(())
}

#[tauri::command]
pub fn security_check_injection(
    text: String,
    state: State<'_, SecurityState>,
) -> Result<super::injection::InjectionResult, String> {
    Ok(state.injection_detector.check(&text))
}

#[tauri::command]
pub fn security_redact(
    text: String,
    state: State<'_, SecurityState>,
) -> Result<super::redaction::RedactionResult, String> {
    Ok(state.redactor.redact(&text))
}

#[tauri::command]
pub fn security_check_url(url: String) -> Result<bool, String> {
    match is_url_safe(&url) {
        Ok(()) => Ok(true),
        Err(e) => Err(e),
    }
}
