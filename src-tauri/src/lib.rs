//! FairyField 后端核心库
//!
//! Tauri v2 应用的 Rust 后端，负责：
//! - AI Agent 系统（Phase 3）
//! - 语音管道（Phase 2）
//! - LLM 客户端（Phase 3）
//! - 工具执行器（Phase 3）
//! - 持久化记忆（Phase 2）
//! - 配置管理（Phase 0）
//! - 插件系统（Phase 4）

use serde::{Deserialize, Serialize};

// ========== 子模块声明 ==========

pub mod agent;
pub mod config;
pub mod llm;
pub mod memory;
pub mod plugins;
pub mod tools;
pub mod voice;

// ========== 共享类型 ==========

/// 情绪状态枚举
///
/// Agent 感知到的用户情绪，用于驱动角色表情和语气变化。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Emotion {
    Happy,
    Sad,
    Angry,
    Surprised,
    Neutral,
    Thinking,
}

// Re-export 配置类型，方便其他模块和 Tauri commands 使用
pub use config::{
    default_config, AppConfig, CharacterConfig, LlmConfig, VoiceConfig, WindowConfig,
};

// ========== Tauri Commands ==========

/// 问候命令（保留）
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 发送聊天消息
///
/// Phase 3 stub：接收用户消息，返回 AI 回复。
#[tauri::command]
async fn chat(message: String) -> Result<String, String> {
    // TODO: Phase 3 实现 — 调用 PrimaryAgent 处理消息
    Ok(format!("(Phase 3 stub) 收到消息: {}", message))
}

/// 获取当前情绪状态
///
/// Phase 3 stub：返回 Agent 感知到的当前情绪。
#[tauri::command]
async fn get_emotion_state() -> Emotion {
    // TODO: Phase 3 实现 — 从 PrimaryAgent 获取情绪状态
    Emotion::Neutral
}

/// 启动语音识别
///
/// Phase 2 stub：开始监听麦克风并进行语音识别。
#[tauri::command]
async fn start_asr() -> Result<(), String> {
    // TODO: Phase 2 实现 — 启动 AsrEngine
    Err("ASR 尚未实现（Phase 2）".to_string())
}

/// 启动语音合成
///
/// Phase 2 stub：将文本转为语音并播放。
#[tauri::command]
async fn start_tts(text: String) -> Result<(), String> {
    // TODO: Phase 2 实现 — 调用 TtsEngine 合成语音
    let _ = text;
    Ok(())
}

/// 获取语音活动检测状态
///
/// Phase 2 stub：返回当前是否检测到语音活动。
#[tauri::command]
async fn get_vad_state() -> bool {
    // TODO: Phase 2 实现 — 从 VadEngine 获取状态
    false
}

/// 加载应用配置
///
/// 读取配置文件或返回默认配置。
#[tauri::command]
fn load_config() -> Result<AppConfig, String> {
    Ok(default_config())
}

/// 窗口拖拽命令（保留）
#[tauri::command]
async fn start_drag(window: tauri::Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

/// 设置鼠标事件穿透（保留）
#[tauri::command]
async fn set_ignore_cursor_events(window: tauri::Window, ignore: bool) -> Result<(), String> {
    window
        .set_ignore_cursor_events(ignore)
        .map_err(|e| e.to_string())
}

// ========== 应用入口 ==========

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            chat,
            get_emotion_state,
            start_asr,
            start_tts,
            get_vad_state,
            load_config,
            start_drag,
            set_ignore_cursor_events,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
