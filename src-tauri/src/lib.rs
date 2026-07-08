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
pub mod text;
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
    config_dir, config_file_path, default_config, load_from_file, save_llm_provider_api_key,
    save_to_file, AppConfig, CharacterConfig, GatewayConfig, LlmConfig, MemoryConfig,
    ProviderPreset, UiConfig, VoiceConfig, WindowConfig,
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

// ---------- 语音命令 ----------

/// 启动语音识别
///
/// 使用当前 VoicePipeline 执行一次 ASR。实时麦克风捕获由 voice/audio_input 模块处理。
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
    let result = pipeline.speak(&text).await.map_err(|e| e.to_string());

    // 通知前端：说话结束（停止口型）—— 无论成功或失败都发送，避免口型动画卡住
    let _ = app_handle.emit("tts-finished", ());

    result.map(|_| ())
}

/// 获取语音活动检测状态
///
/// 返回当前是否检测到语音活动。
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
    let mut config = load_from_file()?;
    redact_config_secrets(&mut config);
    Ok(config)
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
async fn chat(message: String, state: tauri::State<'_, AgentState>) -> Result<String, String> {
    // Phase 0-1 兼容：旧前端只需要 reply 字符串，新路径仍走真实 Agent。
    let response = agent_chat(message, state).await?;
    Ok(response.reply)
}

#[tauri::command]
async fn get_emotion_state(state: tauri::State<'_, AgentState>) -> Result<Emotion, String> {
    let emotion = agent_get_emotion(state).await?;
    Ok(emotion.to_emotion())
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

fn redact_config_secrets(config: &mut AppConfig) {
    config.llm.api_key.clear();
    for provider in &mut config.llm.providers {
        provider.api_key.clear();
    }
    config.gateway.telegram_token.clear();
}

/// 切换活跃的 LLM 提供商
///
/// 参数 `name` 对应 ProviderPreset.name。
/// 切换后会重建 PrimaryAgent 以使用新的提供商，保留工具、记忆和挖掘能力。
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
        api_key: preset.api_key.clone(),
        api_endpoint: preset.api_endpoint.clone(),
        model: preset.model.clone(),
    };

    // 3. 使用环境变量作为后备
    let provider_config = if provider_config.api_key.is_empty() {
        provider_config.with_env_api_key()
    } else {
        provider_config
    };

    if provider_config.api_key.is_empty() {
        return Err(format!("提供商 '{}' 未配置 API Key", name));
    }

    // 4. 创建新 provider 并通过 AgentState 重建 Agent（保留工具/记忆）
    let new_provider = create_provider(provider_config).map_err(|e| e.to_string())?;
    agent_state.recreate_agent(new_provider).await;

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

/// 保存 LLM 提供商的 API Key 到 config.json 并重新初始化 Agent
///
/// OnboardingWizard 完成时调用：
/// 1. 将用户输入的密钥写入内存和磁盘
/// 2. 用真实 API Key 重建 Agent，替换启动时的 MockProvider
#[tauri::command]
async fn llm_save_api_key(
    provider_name: String,
    api_key: String,
    provider_state: tauri::State<'_, LlmProviderState>,
    agent_state: tauri::State<'_, AgentState>,
) -> Result<(), String> {
    // 1. 更新内存中的配置并提取提供商信息
    let provider_config = {
        let mut config = provider_state.config.lock().await;
        let idx = config
            .llm
            .providers
            .iter()
            .position(|p| p.name == provider_name)
            .ok_or_else(|| format!("提供商 '{}' 未找到", provider_name))?;
        config.llm.providers[idx].api_key = api_key.clone();
        let p = &config.llm.providers[idx];
        let pc = LlmProviderConfig {
            provider: p.provider_type.clone(),
            api_key: api_key.clone(),
            api_endpoint: p.api_endpoint.clone(),
            model: p.model.clone(),
        };
        // 密钥持久化到独立 secrets.json；config.json 保持无密钥。
        save_llm_provider_api_key(&provider_name, &api_key)
            .map_err(|e| format!("保存密钥失败: {}", e))?;
        crate::config::save_to_file(&config).map_err(|e| format!("保存配置失败: {}", e))?;
        pc
    }; // 锁在此处释放

    // 2. 用真实 API Key 重建 Agent（替换 MockProvider）
    let new_provider = llm::provider::create_provider(provider_config)
        .map_err(|e| format!("LLM 初始化失败: {}", e))?;
    agent_state.recreate_agent(new_provider).await;
    tracing::info!(
        "LLM: API Key 已保存，Agent 已用提供商 '{}' 重建",
        provider_name
    );

    Ok(())
}

/// 临时测试指定提供商和 API Key，不持久化。
#[tauri::command]
async fn llm_test_provider(
    provider_name: String,
    api_key: String,
    provider_state: tauri::State<'_, LlmProviderState>,
) -> Result<(), String> {
    if api_key.trim().is_empty() {
        return Err("API Key 不能为空".into());
    }

    let provider_config = {
        let config = provider_state.config.lock().await;
        let preset = config
            .llm
            .providers
            .iter()
            .find(|p| p.name == provider_name)
            .ok_or_else(|| format!("提供商 '{}' 未找到", provider_name))?;

        LlmProviderConfig {
            provider: preset.provider_type.clone(),
            api_key,
            api_endpoint: preset.api_endpoint.clone(),
            model: preset.model.clone(),
        }
    };

    let provider = llm::provider::create_provider(provider_config).map_err(|e| format!("{e}"))?;
    provider
        .chat(
            vec![llm::Message {
                role: llm::MessageRole::User,
                content: "Reply with OK.".into(),
                timestamp: 0,
                tool_call_id: None,
                tool_calls: None,
            }],
            llm::ChatConfig {
                temperature: 0.0,
                max_tokens: 8,
                top_p: 1.0,
            },
        )
        .await
        .map(|_| ())
        .map_err(|e| format!("连接测试失败: {e}"))
}

// ========== 应用入口 ==========

/// 依次尝试所有提供商，找到第一个 API Key 非空的（优先从环境变量读取）
fn create_provider_with_fallback(
    llm: &crate::config::LlmConfig,
) -> Result<Arc<dyn llm::provider::LlmProvider>, llm::LlmError> {
    use llm::provider::create_provider;
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
        return Err(LlmError(
            "无提供商配置，请在 config.json 的 llm.providers 中添加至少一个提供商".into(),
        ));
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
                "提供商 '{}' 未配置 API Key（跳过）",
                preset.name
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
    let voice_config = config.voice.clone();
    let agent_config = agent::AgentConfig::default();
    let agent_config_for_state = agent_config.clone();

    // 创建共享记忆层（Arc 在 Agent 和 MemoryState 之间共享）
    let db_path = config_dir().join("fairyfield.db");
    let db_path_str = db_path.to_str().unwrap_or(":memory:").to_string();

    let shared_memory_layers = Arc::new(std::sync::Mutex::new(memory::MemoryLayers::new(
        MemoryStore::new(&db_path_str).expect("Failed to create MemoryStore for layers"),
    )));
    let shared_miner = Arc::new(std::sync::Mutex::new(memory::ConversationMiner::new(
        MemoryStore::new(&db_path_str).expect("Failed to create MemoryStore for miner"),
    )));

    // 创建工具系统（注入真实记忆层）
    let registry =
        tools::registry::ToolRegistry::new_with_memory(Some(Arc::clone(&shared_memory_layers)));
    let executor = Arc::new(registry.into_executor());
    let executor_for_mcp = Arc::clone(&executor);
    let guard = Arc::new(security::CommandGuard::new());
    let injection_detector = Arc::new(security::InjectionDetector::new());
    let toolset = Arc::new(agent::Toolset::new(executor, guard, injection_detector));
    let toolset_for_state = Arc::clone(&toolset);

    // 创建 Agent（带工具和记忆）
    // 尝试所有提供商，找到第一个 API Key 有效的
    let provider = match create_provider_with_fallback(&config.llm) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("所有 LLM 提供商初始化失败: {}", e);
            tracing::error!("请在 ~/.fairyfield/config.json 中配置至少一个提供商的 api_key");
            Arc::new(llm::provider::MockProvider::new(
                "⚠️ LLM 未配置。请在 ~/.fairyfield/config.json 中设置 api_key。",
            ))
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
            // Coding Agent 命令（Phase 6）
            agent::coding_agent::coding_agent_list,
            agent::coding_agent::coding_agent_dispatch,
            // Development Loop 命令（Manager/Coding/Testing/Goal agents）
            agent::development_loop::development_loop_plan,
            agent::development_loop::development_loop_run,
            // 语音命令
            voice_start_asr,
            voice_start_tts,
            voice_get_vad_state,
            // 平台配置命令
            platform_load_config,
            platform_save_config,
            // 用户配置命令（Phase 6）
            config::user::user_load_config,
            config::user::user_save_config,
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
            llm_save_api_key,
            llm_test_provider,
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
            mcp::server::mcp_fairy_execute_tool,
            mcp::server::mcp_fairy_get_context,
        ])
        .manage({
            agent::AgentState {
                agent: tokio::sync::Mutex::new(Arc::new(agent)),
                agent_config: agent_config_for_state,
                toolset: Some(toolset_for_state),
                memory_layers: Some(shared_memory_layers.clone()),
                miner: Some(shared_miner.clone()),
            }
        })
        .manage({
            LlmProviderState {
                config: tokio::sync::Mutex::new(config),
            }
        })
        .manage({
            // 按用户配置创建真实语音管道；模型或音频设备不可用时再安全回退。
            let pipeline = VoicePipeline::from_config(&voice_config)
                .or_else(|e| {
                    tracing::warn!("Voice: 配置语音管道初始化失败，回退到 mock 管道: {}", e);
                    VoicePipeline::new_mock()
                })
                .expect("Failed to create VoicePipeline");
            tracing::info!("Voice: TTS engine = {}", pipeline.tts_engine_name());
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
                registry: std::sync::Mutex::new(tools::registry::ToolRegistry::new_with_memory(
                    Some(shared_memory_layers.clone()),
                )),
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
                executor: executor_for_mcp,
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
