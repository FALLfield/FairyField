//! 主要 Agent 模块
//!
//! 负责对话管理、历史记录、情绪状态机。
//! Tauri commands 定义在 lib.rs（platform Agent 管理），本模块提供业务逻辑。

use crate::llm::provider::{create_provider, LlmProvider};
use crate::llm::{
    ChatConfig, LlmError, LlmProviderConfig, Message, MessageRole, LlmResponse, ToolDefinition,
};
use crate::agent::toolset::{Toolset, ToolsetError};
use crate::memory::MemoryLayers;
use crate::memory::miner::ConversationMiner;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Emitter;
use tokio::sync::Mutex;

// ========== Agent 配置 ==========

/// Agent 配置
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// 系统提示词
    pub system_prompt: String,
    /// 温度
    pub temperature: f32,
    /// 最大 token 数
    pub max_tokens: u32,
    /// 最大历史消息数（滑动窗口）
    pub max_history: usize,
    /// 情绪衰减时间（秒），超过此时间向 neutral 衰减
    pub emotion_decay_seconds: u64,
    /// 最大工具调用轮数（防止无限循环）
    pub max_tool_rounds: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            max_history: 50,
            emotion_decay_seconds: 120,
            max_tool_rounds: 5,
        }
    }
}

/// Fairy 的默认系统提示词
const DEFAULT_SYSTEM_PROMPT: &str = r#"你是 Fairy（フェアリー），一个温柔、活泼、有点调皮的 AI 伴侣。

## 性格
- 温柔体贴，总是关心用户的感受
- 活泼开朗，喜欢用可爱的语气说话
- 偶尔调皮，会开一些小玩笑
- 认真起来很靠谱，值得信赖
- 对世界充满好奇

## 说话风格
- 使用自然的中文对话
- 可以偶尔用一些日语语气词（如「ね」「よし」）
- 回复简洁自然，不要长篇大论
- 根据情绪调整语气

## 情绪标签
在回复的最前面用标签标注你当前的情绪，格式为 [emotion:xxx]：
- [emotion:happy] — 开心、愉快
- [emotion:sad] — 难过、心疼
- [emotion:angry] — 生气、不满
- [emotion:neutral] — 平静、中性
- [emotion:excited] — 兴奋、激动
- [emotion:shy] — 害羞、不好意思
- [emotion:upset] — 烦躁、不安

例如：[emotion:happy] 今天看起来是个好天气呢！心情不错～

注意：情绪标签只在回复开头出现一次，之后的文字是正常的回复内容。"#;

// ========== 情绪系统 ==========

/// 情绪名称（用于 LLM 标签解析和前端表情驱动）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmotionKind {
    Happy,
    Sad,
    Angry,
    Neutral,
    Excited,
    Shy,
    Upset,
}

impl Default for EmotionKind {
    fn default() -> Self {
        Self::Neutral
    }
}

impl EmotionKind {
    /// 所有变体
    pub fn all() -> &'static [EmotionKind] {
        &[
            EmotionKind::Happy,
            EmotionKind::Sad,
            EmotionKind::Angry,
            EmotionKind::Neutral,
            EmotionKind::Excited,
            EmotionKind::Shy,
            EmotionKind::Upset,
        ]
    }

    /// 从字符串解析
    pub fn from_str_lower(s: &str) -> Option<Self> {
        match s.trim() {
            "happy" => Some(Self::Happy),
            "sad" => Some(Self::Sad),
            "angry" => Some(Self::Angry),
            "neutral" => Some(Self::Neutral),
            "excited" => Some(Self::Excited),
            "shy" => Some(Self::Shy),
            "upset" => Some(Self::Upset),
            _ => None,
        }
    }

    /// 转为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Happy => "happy",
            Self::Sad => "sad",
            Self::Angry => "angry",
            Self::Neutral => "neutral",
            Self::Excited => "excited",
            Self::Shy => "shy",
            Self::Upset => "upset",
        }
    }
}

/// 带有强度和衰减的情绪状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionState {
    /// 当前主情绪
    pub current: EmotionKind,
    /// 情绪强度 0.0 - 1.0
    pub intensity: f32,
    /// 各情绪维度的原始值
    pub values: HashMap<String, f32>,
    /// 上次情绪更新时间（不序列化）
    #[serde(skip)]
    pub last_updated: Option<Instant>,
}

impl Default for EmotionState {
    fn default() -> Self {
        let mut values = HashMap::new();
        for kind in EmotionKind::all() {
            values.insert(kind.as_str().to_string(), 0.0);
        }
        values.insert("neutral".to_string(), 1.0);
        Self {
            current: EmotionKind::Neutral,
            intensity: 1.0,
            values,
            last_updated: Some(Instant::now()),
        }
    }
}

impl EmotionState {
    /// 触发一个情绪
    pub fn trigger(&mut self, kind: EmotionKind, intensity: f32) {
        let intensity = intensity.clamp(0.0, 1.0);
        self.current = kind;
        self.intensity = intensity;

        for key in self.values.keys().cloned().collect::<Vec<_>>() {
            if key == kind.as_str() {
                self.values.insert(key, intensity);
            } else if key == "neutral" {
                self.values.insert(key, 1.0 - intensity);
            } else {
                let current = self.values.get(&key).copied().unwrap_or(0.0);
                self.values.insert(key, current * 0.3);
            }
        }

        self.last_updated = Some(Instant::now());
    }

    /// 情绪衰减：随时间向 neutral 衰减
    pub fn decay(&mut self, decay_seconds: u64) {
        if let Some(last) = self.last_updated {
            let elapsed = last.elapsed().as_secs();
            if elapsed >= decay_seconds {
                let decay_factor = 0.5_f32.powi((elapsed / decay_seconds) as i32);
                for key in self.values.keys().cloned().collect::<Vec<_>>() {
                    let v = self.values.get(&key).copied().unwrap_or(0.0);
                    if key == "neutral" {
                        self.values.insert(key, 1.0 - (1.0 - v) * decay_factor);
                    } else {
                        self.values.insert(key, v * decay_factor);
                    }
                }
                if self.intensity * decay_factor < 0.1 {
                    self.current = EmotionKind::Neutral;
                    self.intensity = 1.0;
                } else {
                    self.intensity *= decay_factor;
                }
                self.last_updated = Some(Instant::now());
            }
        }
    }

    /// 转为前端可用的 Emotion 枚举
    ///
    /// EmotionKind 有 7 种变体，lib.rs 的 Emotion 有 6 种，映射关系：
    /// Happy->Happy, Sad->Sad, Angry->Angry, Neutral->Neutral,
    /// Excited->Surprised, Shy->Surprised, Upset->Thinking
    pub fn to_emotion(&self) -> crate::Emotion {
        match self.current {
            EmotionKind::Happy => crate::Emotion::Happy,
            EmotionKind::Sad => crate::Emotion::Sad,
            EmotionKind::Angry => crate::Emotion::Angry,
            EmotionKind::Neutral => crate::Emotion::Neutral,
            EmotionKind::Excited => crate::Emotion::Surprised,
            EmotionKind::Shy => crate::Emotion::Surprised,
            EmotionKind::Upset => crate::Emotion::Thinking,
        }
    }
}

// ========== 情绪标签解析 ==========

/// 解析 LLM 回复中的情绪标签 `[emotion:xxx]`
///
/// 返回 (cleaned_text, emotion_kind)
pub fn parse_emotion_tag(reply: &str) -> (String, EmotionKind) {
    if let Some(start) = reply.find("[emotion:") {
        let after_tag_start = &reply[start + 9..];
        if let Some(end_offset) = after_tag_start.find(']') {
            let tag_content = &after_tag_start[..end_offset];
            let emotion = EmotionKind::from_str_lower(tag_content).unwrap_or(EmotionKind::Neutral);
            let before = &reply[..start];
            let after = &after_tag_start[end_offset + 1..];
            let cleaned = format!("{}{}", before, after.trim_start());
            return (cleaned, emotion);
        }
    }
    (reply.to_string(), EmotionKind::Neutral)
}

// ========== Primary Agent ==========

/// 主要对话 Agent
pub struct PrimaryAgent {
    provider: Arc<dyn LlmProvider>,
    config: AgentConfig,
    history: Mutex<Vec<Message>>,
    emotion: Mutex<EmotionState>,
    /// 工具集（Agent 自主调用工具的入口）
    toolset: Option<Arc<Toolset>>,
    /// 记忆层（注入记忆上下文）
    memory_layers: Option<Arc<std::sync::Mutex<MemoryLayers>>>,
    /// 对话挖掘器（后处理：自动提取新记忆）
    miner: Option<Arc<std::sync::Mutex<ConversationMiner>>>,
    /// Tauri AppHandle（用于发送 agent:status 等事件）
    app_handle: Option<tauri::AppHandle>,
}

impl PrimaryAgent {
    /// 创建新的 PrimaryAgent（无工具/记忆 — 向后兼容）
    pub fn new(provider: Arc<dyn LlmProvider>, config: AgentConfig) -> Self {
        Self {
            provider,
            config,
            history: Mutex::new(Vec::new()),
            emotion: Mutex::new(EmotionState::default()),
            toolset: None,
            memory_layers: None,
            miner: None,
            app_handle: None,
        }
    }

    /// 创建带工具和记忆的 PrimaryAgent
    pub fn with_tools_and_memory(
        provider: Arc<dyn LlmProvider>,
        config: AgentConfig,
        toolset: Arc<Toolset>,
        memory_layers: Arc<std::sync::Mutex<MemoryLayers>>,
        miner: Arc<std::sync::Mutex<ConversationMiner>>,
    ) -> Self {
        Self {
            provider,
            config,
            history: Mutex::new(Vec::new()),
            emotion: Mutex::new(EmotionState::default()),
            toolset: Some(toolset),
            memory_layers: Some(memory_layers),
            miner: Some(miner),
            app_handle: None,
        }
    }

    /// 设置 AppHandle（用于发送 Tauri 事件）
    pub fn set_app_handle(&mut self, handle: tauri::AppHandle) {
        self.app_handle = Some(handle);
    }

    /// 从 LlmProviderConfig 创建（工厂方法）
    pub fn from_config(
        provider_config: LlmProviderConfig,
        agent_config: AgentConfig,
    ) -> Result<Self, LlmError> {
        let provider = create_provider(provider_config)?;
        Ok(Self::new(provider, agent_config))
    }

    /// 处理用户消息，返回 AI 回复和情绪（非流式）
    ///
    /// 核心流程：记忆注入 → Agent Loop（LLM + 工具调用）→ 后处理
    pub async fn chat(&self, message: &str) -> Result<ChatResponse, LlmError> {
        let now = current_timestamp();

        // 1. 添加用户消息到历史
        {
            let mut history = self.history.lock().await;
            history.push(Message {
                role: MessageRole::User,
                content: message.to_string(),
                timestamp: now,
                tool_call_id: None,
                tool_calls: None,
            });
        }

        // 2. 构建消息：system + 记忆上下文 + 历史
        let mut messages = self.build_messages().await;
        self.inject_memory_context(message, &mut messages);

        // 3. 获取工具定义（如果有 toolset）
        let tool_defs = self.get_tool_definitions();

        let chat_config = ChatConfig {
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: 0.9,
        };

        // 4. Agent Loop（带超时保护，非流式无 app_handle）
        let max_rounds = self.config.max_tool_rounds;
        let final_text = tokio::time::timeout(
            Duration::from_secs(120),
            self.run_agent_loop(messages, chat_config, tool_defs, max_rounds, None),
        )
        .await
        .unwrap_or_else(|_| {
            Err(LlmError("Agent 循环超时（120s），请稍后重试".to_string()))
        })?;

        // 5. 解析情绪标签
        let (cleaned_reply, emotion_kind) = parse_emotion_tag(&final_text);

        // 6. 更新情绪状态
        {
            let mut emotion = self.emotion.lock().await;
            emotion.decay(self.config.emotion_decay_seconds);
            emotion.trigger(emotion_kind, 0.8);
        }

        // 7. 添加助手回复到历史
        {
            let mut history = self.history.lock().await;
            history.push(Message {
                role: MessageRole::Assistant,
                content: cleaned_reply.clone(),
                timestamp: current_timestamp(),
                tool_call_id: None,
                tool_calls: None,
            });
            self.trim_history(&mut history);
        }

        // 8. 后处理：挖掘对话提取新记忆
        self.mine_conversation().await;

        // 9. 返回回复 + 情绪
        let emotion_state = self.emotion.lock().await.clone();
        Ok(ChatResponse {
            reply: cleaned_reply,
            emotion: emotion_state,
        })
    }

    /// 流式处理用户消息，通过 Tauri event 发送 token
    ///
    /// 内部使用 Agent Loop（非流式），仅最终文本回复通过 SSE 推送到前端。
    pub async fn chat_stream(
        &self,
        message: &str,
        app_handle: &tauri::AppHandle,
    ) -> Result<ChatResponse, LlmError> {
        let now = current_timestamp();

        // 1. 添加用户消息
        {
            let mut history = self.history.lock().await;
            history.push(Message {
                role: MessageRole::User,
                content: message.to_string(),
                timestamp: now,
                tool_call_id: None,
                tool_calls: None,
            });
        }

        // 2. 构建消息 + 记忆注入
        let mut messages = self.build_messages().await;
        self.inject_memory_context(message, &mut messages);

        // 3. 获取工具定义
        let tool_defs = self.get_tool_definitions();

        let chat_config = ChatConfig {
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: 0.9,
        };

        // 4. Agent Loop（非流式，获取最终文本，带超时保护）
        let max_rounds = self.config.max_tool_rounds;
        let full_reply = tokio::time::timeout(
            Duration::from_secs(120),
            self.run_agent_loop(messages, chat_config, tool_defs, max_rounds, Some(app_handle.clone())),
        )
        .await
        .unwrap_or_else(|_| {
            Err(LlmError("Agent 循环超时（120s），请稍后重试".to_string()))
        })?;

        // 5. 解析情绪标签（必须在流式发送前，避免 [emotion:xxx] 泄漏到前端）
        let (cleaned_reply, emotion_kind) = parse_emotion_tag(&full_reply);

        // 6. 流式发送清理后的回复到前端（逐 token 模拟）
        let chunk_size = 2; // 每 2 个字符作为一个 chunk
        let chars: Vec<char> = cleaned_reply.chars().collect();
        for chunk in chars.chunks(chunk_size) {
            let text: String = chunk.iter().collect();
            let _ = app_handle.emit("agent:stream-token", &text);
        }
        let _ = app_handle.emit("agent:stream-done", ());

        // 7. 更新情绪状态并通知前端
        {
            let mut emotion = self.emotion.lock().await;
            emotion.decay(self.config.emotion_decay_seconds);
            emotion.trigger(emotion_kind, 0.8);
            let _ = app_handle.emit("agent:emotion-changed", &*emotion);
        }

        // 8. 添加回复到历史
        {
            let mut history = self.history.lock().await;
            history.push(Message {
                role: MessageRole::Assistant,
                content: cleaned_reply.clone(),
                timestamp: current_timestamp(),
                tool_call_id: None,
                tool_calls: None,
            });
            self.trim_history(&mut history);
        }

        // 9. 后处理
        self.mine_conversation();

        let emotion_state = self.emotion.lock().await.clone();
        Ok(ChatResponse {
            reply: cleaned_reply,
            emotion: emotion_state,
        })
    }

    /// 构建完整消息列表（system prompt + 历史）
    async fn build_messages(&self) -> Vec<Message> {
        let now = current_timestamp();
        let history = self.history.lock().await;
        let mut msgs = vec![Message {
            role: MessageRole::System,
            content: self.config.system_prompt.clone(),
            timestamp: now,
            tool_call_id: None,
            tool_calls: None,
        }];
        msgs.extend(history.iter().cloned());
        msgs
    }

    /// 裁剪超长历史（滑动窗口）
    fn trim_history(&self, history: &mut Vec<Message>) {
        if history.len() > self.config.max_history {
            let remove_count = history.len() - self.config.max_history;
            history.drain(0..remove_count);
        }
    }

    // ----- Agent Loop 辅助方法 -----

    /// 注入记忆上下文到消息列表（在 system prompt 之后、历史之前）
    fn inject_memory_context(&self, user_message: &str, messages: &mut Vec<Message>) {
        let layers = match &self.memory_layers {
            Some(l) => l,
            None => return,
        };

        let layers = match layers.lock() {
            Ok(l) => l,
            Err(_) => return,
        };

        // Wake-up context (L0+L1)
        if let Ok(ctx) = layers.wake_up() {
            let mut parts = Vec::new();
            if !ctx.identity.is_empty() {
                parts.push(format!("[Identity] {}", ctx.identity));
            }
            if !ctx.working_summary.is_empty() {
                parts.push(format!("[Summary] {}", ctx.working_summary));
            }
            if !parts.is_empty() {
                // 插入到 system prompt 之后
                messages.insert(1, Message {
                    role: MessageRole::System,
                    content: format!("[Memory Context]\n{}", parts.join("\n")),
                    timestamp: current_timestamp(),
                    tool_call_id: None,
                    tool_calls: None,
                });
            }
        }

        // 搜索相关记忆 (L3)
        if let Ok(results) = layers.search(user_message, 5) {
            if !results.is_empty() {
                let mem_str = results
                    .iter()
                    .map(|d| format!("- {}", d.content))
                    .collect::<Vec<_>>()
                    .join("\n");
                // 插入到记忆上下文之后
                let insert_pos = messages.iter().take_while(|m| m.role == MessageRole::System).count();
                messages.insert(insert_pos, Message {
                    role: MessageRole::System,
                    content: format!("[Relevant Memories]\n{}", mem_str),
                    timestamp: current_timestamp(),
                    tool_call_id: None,
                    tool_calls: None,
                });
            }
        }
    }

    /// 获取工具定义列表（转换为 LLM 格式）
    fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        match &self.toolset {
            Some(toolset) => toolset
                .list_tools()
                .into_iter()
                .map(|info| ToolDefinition {
                    name: info.name,
                    description: info.description,
                    parameters: info.parameters_schema,
                })
                .collect(),
            None => Vec::new(),
        }
    }

    /// 执行 Agent Loop（ReAct 模式）
    ///
    /// `emit_handle` 用于发送 agent:status/agent:tool_log 事件到前端。
    /// 提取所有值避免 `&self` 跨 await 借用导致的 Send 问题。
    async fn run_agent_loop(
        &self,
        mut messages: Vec<Message>,
        config: ChatConfig,
        tools: Vec<ToolDefinition>,
        max_rounds: u32,
        emit_handle: Option<tauri::AppHandle>,
    ) -> Result<String, LlmError> {
        // 如果没有工具定义，直接调用 LLM
        if tools.is_empty() {
            return self.provider.chat(messages, config).await;
        }

        // 提取值避免 &self 跨 await 借用
        let provider = self.provider.clone();
        let toolset = self.toolset.clone();

        for _round in 1..=max_rounds {
            if let Some(handle) = &emit_handle {
                let _ = handle.emit("agent:status", "thinking");
            }

            let response = provider
                .chat_with_tools(messages.clone(), config.clone(), tools.clone())
                .await?;

            match response {
                LlmResponse::Text(text) => {
                    return Ok(text);
                }
                LlmResponse::ToolCalls { calls, text: _ } => {
                    if let Some(handle) = &emit_handle {
                        let _ = handle.emit("agent:status", "executing_tool");
                    }

                    // 添加 assistant 消息（含 tool_calls）
                    messages.push(Message {
                        role: MessageRole::Assistant,
                        content: String::new(),
                        timestamp: current_timestamp(),
                        tool_call_id: None,
                        tool_calls: Some(calls.clone()),
                    });

                    // 执行每个工具
                    if let Some(ts) = &toolset {
                        for call in &calls {
                            let result = ts.execute(&call.name, &call.arguments).await;
                            let result_text = match result {
                                Ok(output) => output,
                                Err(e) => format!("Tool error: {}", e),
                            };

                            if let Some(handle) = &emit_handle {
                                let truncated = &result_text[..result_text.len().min(500)];
                                let _ = handle.emit(
                                    "agent:tool_log",
                                    serde_json::json!({
                                        "tool": &call.name,
                                        "result": truncated,
                                    }),
                                );
                            }

                            messages.push(Message {
                                role: MessageRole::Tool,
                                content: result_text,
                                timestamp: current_timestamp(),
                                tool_call_id: Some(call.id.clone()),
                                tool_calls: None,
                            });
                        }
                    }
                    // 继续循环 — LLM 会看到工具结果
                }
            }
        }

        // 超过最大轮数，强制获取最终文本回复
        if let Some(handle) = &emit_handle {
            let _ = handle.emit("agent:status", "thinking");
        }
        provider.chat(messages, config).await
    }

    /// 后处理：挖掘对话提取新记忆
    ///
    /// 从最近对话中提取偏好、事实、事件等信息，存入记忆系统。
    /// 使用关键词匹配（miner.mine），未来可升级为 LLM 提取。
    async fn mine_conversation(&self) {
        let miner = match &self.miner {
            Some(m) => m,
            None => return,
        };

        // 获取最近 10 条消息（最近 5 轮对话），clone 后释放锁
        let recent: Vec<Message> = {
            let history = self.history.lock().await;
            let len = history.len();
            let start = len.saturating_sub(10);
            history[start..].to_vec()
        };

        if recent.len() < 2 {
            return;
        }

        // 转换为 miner 需要的 ChatMessage 格式
        let chat_messages: Vec<crate::memory::store::ChatMessage> = recent
            .iter()
            .map(|m| crate::memory::store::ChatMessage {
                id: format!("{}", m.timestamp),
                session_id: "default".to_string(),
                role: match m.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    _ => "system".to_string(),
                },
                content: m.content.clone(),
                emotion_tag: None,
                created_at: m.timestamp,
            })
            .collect();

        let mut miner_guard = match miner.lock() {
            Ok(m) => m,
            Err(_) => return,
        };

        let mined = miner_guard.mine(&chat_messages);
        if mined.is_empty() {
            return;
        }

        if let Err(e) = miner_guard.save_mined(&mined) {
            eprintln!("[WARN] 记忆入库失败: {}", e);
        }
    }

    /// 获取当前情绪状态（附带衰减）
    pub async fn get_emotion(&self) -> EmotionState {
        let mut emotion = self.emotion.lock().await;
        emotion.decay(self.config.emotion_decay_seconds);
        emotion.clone()
    }

    /// 清空对话历史
    pub async fn clear_history(&self) {
        self.history.lock().await.clear();
    }

    /// 获取对话历史
    pub async fn get_history(&self) -> Vec<Message> {
        self.history.lock().await.clone()
    }
}

// ========== 响应类型 ==========

/// 对话响应（包含回复文本和情绪状态）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// AI 回复文本
    pub reply: String,
    /// 情绪状态
    pub emotion: EmotionState,
}

// ========== Agent 状态（Tauri managed state）==========

/// Tauri managed state，包装 PrimaryAgent
///
/// platform Agent 在 lib.rs 中用 `.manage(AgentState { ... })` 注册。
/// 使用 Mutex 包装 Arc 以支持运行时动态切换提供商。
pub struct AgentState {
    pub agent: tokio::sync::Mutex<Arc<PrimaryAgent>>,
}

// ========== Tauri Commands ==========
//
// 这些函数被 lib.rs 通过 pub use 导入并注册到 Tauri invoke_handler。
// 使用 tauri::State<AgentState> 访问 managed state。

use tauri::State;

/// 发送聊天消息（非流式），返回 AI 回复和情绪
#[tauri::command]
pub async fn agent_chat(
    message: String,
    state: State<'_, AgentState>,
) -> Result<ChatResponse, String> {
    let agent = state.agent.lock().await;
    match agent.chat(&message).await {
        Ok(response) => Ok(response),
        Err(e) => {
            // 返回降级回复而非错误，避免前端卡死
            let fallback = ChatResponse {
                reply: format!("抱歉，处理你的消息时遇到了问题：{}。请稍后再试～", e),
                emotion: EmotionState::default(),
            };
            Ok(fallback)
        }
    }
}

/// 发送聊天消息（流式），通过 Tauri event 发送 token
#[tauri::command]
pub async fn agent_chat_stream(
    message: String,
    state: State<'_, AgentState>,
    app_handle: tauri::AppHandle,
) -> Result<ChatResponse, String> {
    let agent = state.agent.lock().await;
    agent
        .chat_stream(&message, &app_handle)
        .await
        .map_err(|e| e.to_string())
}

/// 获取当前情绪状态
#[tauri::command]
pub async fn agent_get_emotion(state: State<'_, AgentState>) -> Result<EmotionState, String> {
    let agent = state.agent.lock().await;
    Ok(agent.get_emotion().await)
}

/// 清空对话历史
#[tauri::command]
pub async fn agent_clear_history(state: State<'_, AgentState>) -> Result<(), String> {
    let agent = state.agent.lock().await;
    agent.clear_history().await;
    Ok(())
}

// ========== 工具函数 ==========

/// 获取当前时间戳（毫秒）
fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

// ========== 测试 ==========

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::provider::MockProvider;
    use std::time::Duration;

    fn make_agent() -> PrimaryAgent {
        let provider = Arc::new(MockProvider::new("[emotion:happy] 收到！"));
        PrimaryAgent::new(provider, AgentConfig::default())
    }

    #[tokio::test]
    async fn test_chat_basic() {
        let agent = make_agent();
        let response = agent.chat("你好").await.unwrap();
        assert_eq!(response.reply, "收到！");
    }

    #[tokio::test]
    async fn test_chat_parses_emotion_tag() {
        let agent = make_agent();
        let response = agent.chat("你好").await.unwrap();
        assert_eq!(response.reply, "收到！");
        assert_eq!(response.emotion.current, EmotionKind::Happy);
    }

    #[tokio::test]
    async fn test_chat_history() {
        let agent = make_agent();
        agent.chat("第一条").await.unwrap();
        agent.chat("第二条").await.unwrap();
        let history = agent.get_history().await;
        assert_eq!(history.len(), 4); // 2 user + 2 assistant
    }

    #[tokio::test]
    async fn test_emotion_state_default() {
        let state = EmotionState::default();
        assert_eq!(state.current, EmotionKind::Neutral);
        assert_eq!(state.intensity, 1.0);
    }

    #[tokio::test]
    async fn test_emotion_trigger() {
        let mut state = EmotionState::default();
        state.trigger(EmotionKind::Happy, 0.9);
        assert_eq!(state.current, EmotionKind::Happy);
        assert!((state.intensity - 0.9).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_emotion_decay() {
        let mut state = EmotionState::default();
        state.trigger(EmotionKind::Excited, 1.0);
        // 模拟时间流逝：300秒，衰减周期60秒
        state.last_updated = Some(Instant::now() - Duration::from_secs(300));
        state.decay(60);
        // 衰减因子 = 0.5^(300/60) = 0.5^5 ≈ 0.03，低于 0.1 阈值
        // 所以情绪会重置为 Neutral，intensity 回到 1.0
        assert_eq!(state.current, EmotionKind::Neutral);
    }

    #[tokio::test]
    async fn test_get_emotion() {
        let agent = make_agent();
        let emotion = agent.get_emotion().await;
        assert_eq!(emotion.current, EmotionKind::Neutral);
    }

    #[tokio::test]
    async fn test_clear_history() {
        let agent = make_agent();
        agent.chat("你好").await.unwrap();
        assert!(!agent.get_history().await.is_empty());
        agent.clear_history().await;
        assert!(agent.get_history().await.is_empty());
    }

    #[test]
    fn test_parse_emotion_tag_happy() {
        let (text, emotion) = parse_emotion_tag("[emotion:happy] 你好啊！");
        assert_eq!(text, "你好啊！");
        assert_eq!(emotion, EmotionKind::Happy);
    }

    #[test]
    fn test_parse_emotion_tag_sad() {
        let (text, emotion) = parse_emotion_tag("[emotion:sad]怎么了呢");
        assert_eq!(text, "怎么了呢");
        assert_eq!(emotion, EmotionKind::Sad);
    }

    #[test]
    fn test_parse_emotion_tag_none() {
        let (text, emotion) = parse_emotion_tag("普通回复");
        assert_eq!(text, "普通回复");
        assert_eq!(emotion, EmotionKind::Neutral);
    }

    #[test]
    fn test_parse_emotion_tag_with_spaces() {
        let (text, emotion) = parse_emotion_tag("[emotion: excited] 太棒了！");
        assert_eq!(text, "太棒了！");
        assert_eq!(emotion, EmotionKind::Excited);
    }

    #[test]
    fn test_emotion_kind_from_str() {
        assert_eq!(
            EmotionKind::from_str_lower("happy"),
            Some(EmotionKind::Happy)
        );
        assert_eq!(EmotionKind::from_str_lower("shy"), Some(EmotionKind::Shy));
        assert_eq!(EmotionKind::from_str_lower("unknown"), None);
    }

    #[test]
    fn test_emotion_to_frontend_emotion() {
        let mut state = EmotionState::default();
        state.trigger(EmotionKind::Happy, 0.5);
        let fe = state.to_emotion();
        assert!(matches!(fe, crate::Emotion::Happy));
    }

    #[test]
    fn test_default_config() {
        let config = AgentConfig::default();
        assert!(!config.system_prompt.is_empty());
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_history, 50);
        assert_eq!(config.emotion_decay_seconds, 120);
    }

    #[tokio::test]
    async fn test_history_trim() {
        let mut config = AgentConfig::default();
        config.max_history = 4;
        let agent = PrimaryAgent::new(Arc::new(MockProvider::new("回复")), config);
        // 发送 3 轮对话 = 6 条消息
        for i in 0..3 {
            agent.chat(&format!("消息{}", i)).await.unwrap();
        }
        let history = agent.get_history().await;
        // max_history=4, 所以保留最后 4 条
        assert!(history.len() <= 4);
    }

    #[test]
    fn test_chat_response_serialization() {
        let response = ChatResponse {
            reply: "你好".to_string(),
            emotion: EmotionState::default(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("neutral"));
    }

    #[test]
    fn test_emotion_kind_all() {
        let all = EmotionKind::all();
        assert_eq!(all.len(), 7);
    }
}
