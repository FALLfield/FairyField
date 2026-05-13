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

use agent::PrimaryAgent;
use llm::provider::create_provider;
use llm::LlmProviderConfig;
use memory::store::MemoryStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Emitter;
use voice::VoicePipeline;

// ========== 子模块声明 ==========

pub mod agent;
pub mod config;
pub mod gateway;
pub mod growth;
pub mod llm;
pub mod mcp;
pub mod memory;
pub mod plugins;
pub mod security;
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
    config_dir, config_file_path, default_config, load_from_file, save_to_file, AppConfig,
    CharacterConfig, GatewayConfig, LlmConfig, MemoryConfig, ProviderPreset, UiConfig, VoiceConfig,
    WindowConfig,
};

// Re-export agent commands（由 soul Agent 定义）
pub use agent::{
    agent_chat, agent_chat_stream, agent_clear_history, agent_get_emotion, AgentState,
    ChatResponse, EmotionKind, EmotionState,
};

// ========== Tauri Commands ==========

/// 问候命令（保留）
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ---------- 语音命令（voice Agent stub，等 Phase 2 真正实现） ----------

/// 启动语音识别
///
/// Phase 2 stub：开始监听麦克风并进行语音识别。
/// 语音管道状态
struct VoiceState {
    pipeline: tokio::sync::Mutex<VoicePipeline>,
}

#[tauri::command]
async fn voice_start_asr(state: tauri::State<'_, VoiceState>) -> Result<String, String> {
    let pipeline = state.pipeline.lock().await;
    let samples = vec![0.0f32; 24000];
    let result = pipeline
        .recognize(&samples, 24000)
        .map_err(|e| e.to_string())?;
    Ok(result.text)
}

/// 启动语音合成
///
/// 将文本转为语音并播放，同时向前端发送 tts-started/tts-finished 事件
/// 用于驱动角色口型同步。
#[tauri::command]
async fn voice_start_tts(
    text: String,
    state: tauri::State<'_, VoiceState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // 通知前端：开始说话（驱动口型）
    let _ = app_handle.emit("tts-started", ());

    let pipeline = state.pipeline.lock().await;
    pipeline.speak(&text).await.map_err(|e| e.to_string())?;

    // 通知前端：说话结束（停止口型）
    let _ = app_handle.emit("tts-finished", ());

    Ok(())
}

/// 获取语音活动检测状态
///
/// Phase 2 stub：返回当前是否检测到语音活动。
#[tauri::command]
async fn voice_get_vad_state(state: tauri::State<'_, VoiceState>) -> Result<bool, String> {
    let pipeline = state.pipeline.lock().await;
    Ok(pipeline.vad_state())
}

// ---------- 平台配置命令 ----------

/// 加载应用配置
///
/// 从 `~/.fairyfield/config.json` 读取配置，文件不存在则创建默认配置。
#[tauri::command]
fn platform_load_config() -> Result<AppConfig, String> {
    load_from_file()
}

/// 保存应用配置
///
/// 将配置写入 `~/.fairyfield/config.json`。
#[tauri::command]
fn platform_save_config(config: AppConfig) -> Result<(), String> {
    save_to_file(&config)
}

// ---------- 窗口命令 ----------

/// 获取窗口尺寸
#[tauri::command]
async fn get_window_size(window: tauri::Window) -> Result<WindowConfig, String> {
    let size = window.inner_size().map_err(|e| e.to_string())?;
    Ok(WindowConfig {
        width: size.width as f64,
        height: size.height as f64,
        transparent: true,
        always_on_top: true,
        decorations: false,
        click_through: true,
    })
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

// ---------- 兼容 Phase 0-1 的命令别名 ----------
// 保留旧的 command 名称以免前端崩溃，内部委托给新实现。

#[tauri::command]
async fn chat(message: String) -> Result<String, String> {
    // Phase 0-1 兼容：返回简单文本（不经过 AgentState）
    Ok(format!("(Phase 2 stub) 收到消息: {}", message))
}

#[tauri::command]
async fn get_emotion_state() -> Emotion {
    Emotion::Neutral
}

#[tauri::command]
async fn start_asr(state: tauri::State<'_, VoiceState>) -> Result<String, String> {
    voice_start_asr(state).await
}

#[tauri::command]
async fn start_tts(
    text: String,
    state: tauri::State<'_, VoiceState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    voice_start_tts(text, state, app_handle).await
}

#[tauri::command]
async fn get_vad_state(state: tauri::State<'_, VoiceState>) -> Result<bool, String> {
    voice_get_vad_state(state).await
}

#[tauri::command]
fn load_config() -> Result<AppConfig, String> {
    platform_load_config()
}

#[tauri::command]
fn save_config(config: AppConfig) -> Result<(), String> {
    platform_save_config(config)
}

// ---------- LLM 提供商管理命令 ----------

/// LLM 提供商管理状态（Tauri managed state）
///
/// 包装 AppConfig 的 Mutex，支持运行时动态切换提供商。
struct LlmProviderState {
    config: tokio::sync::Mutex<AppConfig>,
}

/// 列出所有可用的 LLM 提供商预设
///
/// 返回所有已配置的提供商预设列表（不含 API 密钥）。
#[tauri::command]
async fn llm_list_providers(
    state: tauri::State<'_, LlmProviderState>,
) -> Result<Vec<ProviderPreset>, String> {
    let config = state.config.lock().await;
    let presets: Vec<ProviderPreset> = config
        .llm
        .providers
        .iter()
        .map(|p| ProviderPreset {
            name: p.name.clone(),
            provider_type: p.provider_type.clone(),
            api_endpoint: p.api_endpoint.clone(),
            model: p.model.clone(),
            api_key: String::new(), // 不暴露密钥给前端
        })
        .collect();
    Ok(presets)
}

/// 切换活跃的 LLM 提供商
///
/// 参数 `name` 对应 ProviderPreset.name。
/// 切换后会重建 PrimaryAgent 以使用新的提供商。
#[tauri::command]
async fn llm_switch_provider(
    name: String,
    provider_state: tauri::State<'_, LlmProviderState>,
    agent_state: tauri::State<'_, AgentState>,
) -> Result<(), String> {
    // 1. 更新配置中的 active_provider
    let mut config = provider_state.config.lock().await;
    config.llm.switch_provider(&name)?;

    // 2. 根据新配置构建 provider
    let preset = config.llm.active_preset();
    let provider_config = LlmProviderConfig {
        provider: preset.provider_type,
        api_key: preset.api_key,
        api_endpoint: preset.api_endpoint,
        model: preset.model,
    };

    // 3. 重建 PrimaryAgent
    let agent_config = agent::AgentConfig::default();
    let new_agent =
        PrimaryAgent::from_config(provider_config, agent_config).map_err(|e| e.to_string())?;

    // 4. 替换 AgentState 中的 agent（会丢失当前对话历史和情绪状态）
    {
        let mut agent = agent_state.agent.lock().await;
        *agent = Arc::new(new_agent);
    }

    // 5. 保存切换后的配置到文件
    let _ = save_to_file(&config);

    Ok(())
}

/// 获取当前活跃的 LLM 提供商信息
///
/// 返回当前正在使用的提供商预设（不含 API 密钥）。
#[tauri::command]
async fn llm_get_active_provider(
    state: tauri::State<'_, LlmProviderState>,
) -> Result<ProviderPreset, String> {
    let config = state.config.lock().await;
    let preset = config.llm.active_preset();
    Ok(ProviderPreset {
        name: preset.name,
        provider_type: preset.provider_type,
        api_endpoint: preset.api_endpoint,
        model: preset.model,
        api_key: String::new(), // 不暴露密钥
    })
}

// ========== 应用入口 ==========

/// 依次尝试所有提供商，找到第一个 API Key 非空的（优先从环境变量读取）
fn create_provider_with_fallback(
    llm: &crate::config::LlmConfig,
) -> Result<Arc<dyn llm::provider::LlmProvider>, llm::LlmError> {
    use llm::provider::{create_provider, LlmProvider};
    use llm::{LlmError, LlmProviderConfig};

    // 按顺序遍历：先 active_provider 匹配的，再其余
    let mut ordered: Vec<&crate::config::ProviderPreset> = Vec::new();
    if let Some(active) = llm.providers.iter().find(|p| p.name == llm.active_provider) {
        ordered.push(active);
    }
    for p in &llm.providers {
        if p.name != llm.active_provider {
            ordered.push(p);
        }
    }
    // 旧格式兼容（providers 为空时从旧字段构造）
    if ordered.is_empty() {
        // 借用 checker 需要一个 owned preset；这里直接用 active_preset()
        let _preset = llm.active_preset(); // 需要 owned — 但借用检查器有问题
        return Err(LlmError("无提供商配置，请在 config.json 的 llm.providers 中添加至少一个提供商".into()));
    }

    let mut last_err = None;
    for preset in &ordered {
        let mut config = LlmProviderConfig {
            provider: preset.provider_type.clone(),
            api_key: preset.api_key.clone(),
            api_endpoint: preset.api_endpoint.clone(),
            model: preset.model.clone(),
        };
        // 环境变量覆盖
        if config.api_key.is_empty() {
            config = config.with_env_api_key();
        }
        if config.api_key.is_empty() {
            last_err = Some(LlmError(format!(
                "提供商 '{}' 未配置 API Key（跳过）", preset.name
            )));
            continue;
        }
        match create_provider(config) {
            Ok(p) => {
                tracing::info!("LLM: 使用提供商 '{}'", preset.name);
                return Ok(p);
            }
            Err(e) => {
                tracing::warn!("LLM: 提供商 '{}' 初始化失败: {}", preset.name, e);
                last_err = Some(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| LlmError("无可用提供商".into())))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化结构化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = load_from_file().unwrap_or_else(|e| {
        tracing::warn!("配置文件加载失败，使用默认配置: {e}");
        default_config()
    });
    let agent_config = agent::AgentConfig::default();

    // 创建共享记忆层（Arc 在 Agent 和 MemoryState 之间共享）
    let db_path = config_dir().join("fairyfield.db");
    let db_path_str = db_path.to_str().unwrap_or(":memory:").to_string();

    let shared_memory_layers = Arc::new(std::sync::Mutex::new(
        memory::MemoryLayers::new(
            MemoryStore::new(&db_path_str).expect("Failed to create MemoryStore for layers"),
        ),
    ));
    let shared_miner = Arc::new(std::sync::Mutex::new(
        memory::ConversationMiner::new(
            MemoryStore::new(&db_path_str).expect("Failed to create MemoryStore for miner"),
        ),
    ));

    // 创建工具系统（注入真实记忆层）
    let registry = tools::registry::ToolRegistry::new_with_memory(Some(Arc::clone(
        &shared_memory_layers,
    )));
    let executor = Arc::new(registry.into_executor());
    let guard = Arc::new(security::CommandGuard::new());
    let injection_detector = Arc::new(security::InjectionDetector::new());
    let toolset = Arc::new(agent::Toolset::new(executor, guard, injection_detector));

    // 创建 Agent（带工具和记忆）
    // 尝试所有提供商，找到第一个 API Key 有效的
    let provider = match create_provider_with_fallback(&config.llm) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("所有 LLM 提供商初始化失败: {}", e);
            tracing::error!("请在 ~/.fairyfield/config.json 中配置至少一个提供商的 api_key");
            Arc::new(llm::provider::MockProvider::new("⚠️ LLM 未配置。请在 ~/.fairyfield/config.json 中设置 api_key。"))
        }
    };
    let mut agent = PrimaryAgent::with_tools_and_memory(
        provider,
        agent_config,
        toolset,
        shared_memory_layers.clone(),
        shared_miner.clone(),
    );

    // 设置 AppHandle 需要在 builder setup 之后，这里先用 None
    // （app_handle 在首次 chat_stream 时通过 Tauri command 注入）
    let _ = &mut agent; // agent 已准备好

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // 原始命令
            greet,
            // Agent 命令（来自 agent::primary，通过 re-export）
            agent_chat,
            agent_chat_stream,
            agent_get_emotion,
            agent_clear_history,
            // 语音命令（platform stub）
            voice_start_asr,
            voice_start_tts,
            voice_get_vad_state,
            // 平台配置命令
            platform_load_config,
            platform_save_config,
            // 窗口命令
            get_window_size,
            start_drag,
            set_ignore_cursor_events,
            // Phase 0-1 兼容别名
            chat,
            get_emotion_state,
            start_asr,
            start_tts,
            get_vad_state,
            load_config,
            save_config,
            // LLM 提供商管理
            llm_list_providers,
            llm_switch_provider,
            llm_get_active_provider,
            // Memory 命令（Phase 3）
            memory::commands::memory_wake_up,
            memory::commands::memory_search,
            memory::commands::memory_recall,
            memory::commands::memory_add_drawer,
            memory::commands::memory_mine,
            memory::commands::memory_set_meta,
            memory::commands::memory_list_wings,
            // Security 命令（Phase 3）
            security::commands::security_check_command,
            security::commands::security_approve_command,
            security::commands::security_check_injection,
            security::commands::security_redact,
            security::commands::security_check_url,
            // Tools 命令（Phase 3）
            tools::commands::tools_list,
            tools::commands::tools_execute,
            // 知识图谱命令（Phase 3 Wave 3）
            memory::commands::kg_add_fact,
            memory::commands::kg_invalidate,
            memory::commands::kg_query_entity,
            memory::commands::kg_query_relation,
            memory::commands::kg_search,
            // 向量搜索命令（Phase 3 Wave 3）
            memory::commands::memory_vector_search,
            // Gateway 命令（Phase 4）
            gateway::commands::gateway_send_message,
            gateway::commands::gateway_send_notification,
            gateway::commands::cron_add_job,
            gateway::commands::cron_remove_job,
            gateway::commands::cron_list_jobs,
            // Growth/Skill 命令（Phase 4）
            growth::commands::growth_save_experience,
            growth::commands::growth_get_user_profile,
            growth::commands::growth_get_recent_experiences,
            growth::commands::growth_get_category_stats,
            growth::commands::skill_list,
            growth::commands::skill_view,
            growth::commands::skill_create,
            growth::commands::skill_update,
            growth::commands::skill_delete,
            // MCP 命令（Phase 4）
            mcp::server::mcp_list_tools,
            mcp::server::mcp_fairy_memory_search,
            mcp::server::mcp_fairy_wake_up,
            mcp::server::mcp_fairy_recall_wing,
        ])
        .manage({
            agent::AgentState {
                agent: tokio::sync::Mutex::new(Arc::new(agent)),
            }
        })
        .manage({
            LlmProviderState {
                config: tokio::sync::Mutex::new(config),
            }
        })
        .manage({
            // 创建语音管道（使用 Mock 引擎，无需模型文件）
            let pipeline = VoicePipeline::new_mock().expect("Failed to create VoicePipeline");
            VoiceState {
                pipeline: tokio::sync::Mutex::new(pipeline),
            }
        })
        .manage({
            // 创建记忆子系统（使用与 Agent 共享的 Arc）
            memory::commands::MemoryState {
                layers: shared_memory_layers.clone(),
                miner: shared_miner,
                palace: std::sync::Mutex::new(memory::Palace::new(
                    MemoryStore::new(&db_path_str)
                        .expect("Failed to create MemoryStore for palace"),
                )),
                knowledge_graph: std::sync::Mutex::new(
                    memory::KnowledgeGraph::new(
                        rusqlite::Connection::open(&db_path_str)
                            .expect("Failed to open KG database"),
                    )
                    .expect("Failed to create KnowledgeGraph"),
                ),
                searcher: std::sync::Mutex::new(memory::embedding::EmbeddingSearcher::new()),
            }
        })
        .manage({
            // 创建安全子系统
            security::commands::SecurityState {
                guard: std::sync::Mutex::new(security::CommandGuard::new()),
                injection_detector: security::InjectionDetector::new(),
                redactor: security::SecretRedactor::new(),
            }
        })
        .manage({
            // 创建工具子系统
            tools::commands::ToolsState {
                registry: std::sync::Mutex::new(tools::registry::ToolRegistry::new()),
            }
        })
        .manage({
            // 创建通信网关子系统
            gateway::commands::GatewayState {
                discord: tokio::sync::Mutex::new(None),
                cron: std::sync::Mutex::new(gateway::cron::CronScheduler::new()),
            }
        })
        .manage({
            // 创建成长引擎 + 技能管理子系统（Phase 4）
            growth::commands::GrowthState {
                engine: std::sync::Mutex::new(
                    growth::GrowthEngine::new(
                        rusqlite::Connection::open(&db_path_str)
                            .expect("Failed to open GrowthEngine database"),
                    )
                    .expect("Failed to create GrowthEngine"),
                ),
                skills: std::sync::Mutex::new(
                    growth::SkillManager::new(
                        rusqlite::Connection::open(&db_path_str)
                            .expect("Failed to open SkillManager database"),
                    )
                    .expect("Failed to create SkillManager"),
                ),
            }
        })
        .manage({
            // 创建 MCP Server 子系统（Phase 4）— 共享记忆层
            mcp::server::McpState {
                layers: shared_memory_layers.clone(),
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
