//! LLM Provider 模块
//!
//! 定义 LLM 提供商 trait 和各提供商实现。
//! 支持 OpenAI 兼容 API（OpenAI、Deepseek、Moonshot 等）和 Anthropic Claude API。

use crate::llm::{
    ChatConfig, LlmError, LlmProviderConfig, LlmResponse, Message, MessageRole, StreamChunk,
    ToolCallRequest, ToolDefinition,
};
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;

/// LLM 提供商 trait（dyn-compatible，使用 Pin<Box<dyn Future>>）
pub trait LlmProvider: Send + Sync {
    /// 发送聊天请求，返回完整回复
    fn chat(
        &self,
        messages: Vec<Message>,
        config: ChatConfig,
    ) -> Pin<Box<dyn Future<Output = Result<String, LlmError>> + Send + '_>>;

    /// 发送流式聊天请求，通过 channel 逐 token 发送
    fn chat_stream(
        &self,
        messages: Vec<Message>,
        config: ChatConfig,
        tx: mpsc::Sender<StreamChunk>,
    ) -> Pin<Box<dyn Future<Output = Result<(), LlmError>> + Send + '_>>;

    /// 提供商名称
    fn name(&self) -> &str;

    /// 发送带工具的聊天请求，返回 LlmResponse（文本或工具调用）
    ///
    /// 默认实现忽略工具列表，回退到普通 chat 并包装为 LlmResponse::Text。
    fn chat_with_tools(
        &self,
        messages: Vec<Message>,
        config: ChatConfig,
        _tools: Vec<ToolDefinition>,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse, LlmError>> + Send + '_>> {
        let fut = self.chat(messages, config);
        Box::pin(async move {
            let text = fut.await?;
            Ok(LlmResponse::Text(text))
        })
    }
}

/// 构建 OpenAI 格式消息 JSON 数组
///
/// 处理 System/User/Assistant/Tool 角色，以及 Assistant 消息中的 tool_calls。
fn serialize_messages_for_openai(messages: &[Message]) -> Vec<serde_json::Value> {
    messages
        .iter()
        .map(|m| match m.role {
            MessageRole::System | MessageRole::User | MessageRole::Assistant => {
                let role = match m.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::Tool => unreachable!(),
                };
                let mut obj = json!({ "role": role, "content": m.content });
                // Assistant 消息可能携带 tool_calls
                if let Some(calls) = &m.tool_calls {
                    let tool_calls_json: Vec<serde_json::Value> = calls
                        .iter()
                        .map(|tc| {
                            json!({
                                "id": tc.id,
                                "type": "function",
                                "function": {
                                    "name": tc.name,
                                    "arguments": tc.arguments,
                                }
                            })
                        })
                        .collect();
                    obj.as_object_mut()
                        .unwrap()
                        .insert("tool_calls".to_string(), json!(tool_calls_json));
                }
                obj
            }
            MessageRole::Tool => {
                // OpenAI tool result 格式
                json!({
                    "role": "tool",
                    "tool_call_id": m.tool_call_id,
                    "content": m.content,
                })
            }
        })
        .collect()
}

/// 构建 Claude 格式消息 JSON 数组
///
/// Tool 消息在 Claude API 中作为 user 消息的 tool_result content block。
/// Assistant 消息中的 tool_calls 变为 content 数组中的 tool_use blocks。
fn serialize_messages_for_claude(messages: &[Message]) -> Vec<serde_json::Value> {
    let mut result = Vec::new();
    let mut pending_tool_results: Vec<serde_json::Value> = Vec::new();

    for m in messages {
        match m.role {
            MessageRole::System => {
                // System 消息由 ClaudeProvider::build_body 单独处理，这里跳过
                continue;
            }
            MessageRole::User => {
                flush_tool_results(&mut result, &mut pending_tool_results);
                result.push(json!({ "role": "user", "content": m.content }));
            }
            MessageRole::Assistant => {
                flush_tool_results(&mut result, &mut pending_tool_results);
                if let Some(calls) = &m.tool_calls {
                    // Claude: assistant 消息包含 text + tool_use content blocks
                    let mut content_blocks: Vec<serde_json::Value> = Vec::new();
                    if !m.content.is_empty() {
                        content_blocks.push(json!({ "type": "text", "text": m.content }));
                    }
                    for tc in calls {
                        // Claude arguments 是 JSON 对象，不是字符串
                        let args: serde_json::Value =
                            serde_json::from_str(&tc.arguments).unwrap_or(json!({}));
                        content_blocks.push(json!({
                            "type": "tool_use",
                            "id": tc.id,
                            "name": tc.name,
                            "input": args,
                        }));
                    }
                    result.push(json!({ "role": "assistant", "content": content_blocks }));
                } else {
                    result.push(json!({ "role": "assistant", "content": m.content }));
                }
            }
            MessageRole::Tool => {
                // 收集 tool_result，等 flush 时作为 user 消息发送
                let mut tool_result = json!({
                    "type": "tool_result",
                    "tool_use_id": m.tool_call_id,
                    "content": m.content,
                });
                if let Some(id) = &m.tool_call_id {
                    tool_result
                        .as_object_mut()
                        .unwrap()
                        .insert("tool_use_id".to_string(), json!(id));
                }
                pending_tool_results.push(tool_result);
            }
        }
    }
    flush_tool_results(&mut result, &mut pending_tool_results);
    result
}

/// 将累积的 tool_result 刷出为一条 user 消息
fn flush_tool_results(
    result: &mut Vec<serde_json::Value>,
    pending: &mut Vec<serde_json::Value>,
) {
    if pending.is_empty() {
        return;
    }
    let blocks: Vec<serde_json::Value> = pending.drain(..).collect();
    result.push(json!({ "role": "user", "content": blocks }));
}

/// 构建消息 JSON 数组（OpenAI 兼容格式）
fn serialize_messages(messages: &[Message]) -> Vec<serde_json::Value> {
    serialize_messages_for_openai(messages)
}

// ========== Mock Provider ==========

/// Mock LLM 提供商（用于测试）
pub struct MockProvider {
    reply: String,
    messages_received: std::sync::Mutex<Vec<Message>>,
}

impl MockProvider {
    pub fn new(reply: impl Into<String>) -> Self {
        Self {
            reply: reply.into(),
            messages_received: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn messages_received(&self) -> Vec<Message> {
        self.messages_received.lock().unwrap().clone()
    }
}

impl LlmProvider for MockProvider {
    fn chat(
        &self,
        messages: Vec<Message>,
        _config: ChatConfig,
    ) -> Pin<Box<dyn Future<Output = Result<String, LlmError>> + Send + '_>> {
        self.messages_received.lock().unwrap().extend(messages);
        let reply = self.reply.clone();
        Box::pin(async move { Ok(reply) })
    }

    fn chat_stream(
        &self,
        messages: Vec<Message>,
        _config: ChatConfig,
        tx: mpsc::Sender<StreamChunk>,
    ) -> Pin<Box<dyn Future<Output = Result<(), LlmError>> + Send + '_>> {
        self.messages_received.lock().unwrap().extend(messages);
        let reply = self.reply.clone();
        Box::pin(async move {
            for ch in reply.chars() {
                if tx
                    .send(StreamChunk {
                        text: ch.to_string(),
                        done: false,
                    })
                    .await
                    .is_err()
                {
                    break;
                }
            }
            let _ = tx
                .send(StreamChunk {
                    text: String::new(),
                    done: true,
                })
                .await;
            Ok(())
        })
    }

    fn name(&self) -> &str {
        "mock"
    }
}

// ========== OpenAI Compatible Provider ==========

/// OpenAI API 响应结构
#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessageContent,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessageContent {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<OpenAiToolCall>>,
}

/// OpenAI 流式数据块
#[derive(Debug, Deserialize)]
struct OpenAiStreamDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
}

/// OpenAI 错误响应
#[derive(Debug, Deserialize)]
struct OpenAiErrorResponse {
    error: OpenAiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct OpenAiErrorDetail {
    message: String,
}

/// OpenAI 工具调用结构
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAiToolCall {
    id: String,
    r#type: String,
    function: OpenAiFunction,
}

#[derive(Debug, Deserialize)]
struct OpenAiFunction {
    name: String,
    arguments: String,
}

/// OpenAI 兼容提供商（支持 OpenAI、Deepseek、Moonshot 等）
pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(config: LlmProviderConfig) -> Result<Self, LlmError> {
        if config.api_key.is_empty() {
            return Err(LlmError("OpenAI API Key 不能为空".to_string()));
        }
        Ok(Self {
            client: Client::new(),
            api_key: config.api_key,
            base_url: config.api_endpoint.trim_end_matches('/').to_string(),
            model: config.model,
        })
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    fn build_body(
        &self,
        messages: &[Message],
        config: &ChatConfig,
        stream: bool,
    ) -> serde_json::Value {
        json!({
            "model": self.model,
            "messages": serialize_messages(messages),
            "stream": stream,
        })
    }

    fn parse_error(status: reqwest::StatusCode, body: &str) -> LlmError {
        if let Ok(err_resp) = serde_json::from_str::<OpenAiErrorResponse>(body) {
            match status.as_u16() {
                429 => LlmError(format!("API 限流: {}", err_resp.error.message)),
                401 => LlmError(format!("认证失败: {}", err_resp.error.message)),
                403 => LlmError(format!("权限不足: {}", err_resp.error.message)),
                500..=599 => LlmError(format!("服务器错误: {}", err_resp.error.message)),
                _ => LlmError(format!("API 错误 ({}): {}", status, err_resp.error.message)),
            }
        } else {
            LlmError(format!("API 错误 ({}): {}", status, body))
        }
    }

    /// 构建带工具的请求体（OpenAI function calling 格式）
    fn build_body_with_tools(
        &self,
        messages: &[Message],
        config: &ChatConfig,
        stream: bool,
        tools: &[ToolDefinition],
    ) -> serde_json::Value {
        let mut body = self.build_body(messages, config, stream);
        if !tools.is_empty() {
            let tools_json: Vec<serde_json::Value> = tools
                .iter()
                .map(|t| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters,
                        }
                    })
                })
                .collect();
            body.as_object_mut()
                .unwrap()
                .insert("tools".to_string(), json!(tools_json));
        }
        body
    }
}

impl LlmProvider for OpenAiProvider {
    fn chat(
        &self,
        messages: Vec<Message>,
        config: ChatConfig,
    ) -> Pin<Box<dyn Future<Output = Result<String, LlmError>> + Send + '_>> {
        let body = self.build_body(&messages, &config, false);
        let url = self.chat_url();
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        Box::pin(async move {
            let response = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| LlmError(format!("网络错误: {}", e)))?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .map_err(|e| LlmError(format!("读取响应失败: {}", e)))?;

            if !status.is_success() {
                return Err(Self::parse_error(status, &response_text));
            }

            eprintln!(
                "[DEBUG] LLM raw response: {}",
                &response_text[..response_text.len().min(500)]
            );
            let parsed: OpenAiResponse = serde_json::from_str(&response_text)
                .map_err(|e| LlmError(format!("解析响应失败: {} - {}", e, response_text)))?;

            let choice = parsed
                .choices
                .first()
                .ok_or_else(|| LlmError("响应中无 choices".to_string()))?;
            let content = choice.message.content.clone().filter(|s| !s.is_empty());
            Ok(content.unwrap_or_else(|| "[思考中...]".to_string()))
        })
    }

    fn chat_stream(
        &self,
        messages: Vec<Message>,
        config: ChatConfig,
        tx: mpsc::Sender<StreamChunk>,
    ) -> Pin<Box<dyn Future<Output = Result<(), LlmError>> + Send + '_>> {
        let body = self.build_body(&messages, &config, true);
        let url = self.chat_url();
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        Box::pin(async move {
            let response = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| LlmError(format!("网络错误: {}", e)))?;

            let status = response.status();
            if !status.is_success() {
                let error_text = response
                    .text()
                    .await
                    .map_err(|e| LlmError(format!("读取错误响应失败: {}", e)))?;
                return Err(Self::parse_error(status, &error_text));
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result.map_err(|e| LlmError(format!("流读取错误: {}", e)))?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim_end_matches('\r').to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if let Some(data) = line.strip_prefix("data: ") {
                        if data.trim() == "[DONE]" {
                            let _ = tx
                                .send(StreamChunk {
                                    text: String::new(),
                                    done: true,
                                })
                                .await;
                            return Ok(());
                        }

                        if let Ok(parsed) = serde_json::from_str::<OpenAiStreamChunk>(data) {
                            if let Some(choice) = parsed.choices.first() {
                                if let Some(content) = &choice.delta.content {
                                    let _ = tx
                                        .send(StreamChunk {
                                            text: content.clone(),
                                            done: false,
                                        })
                                        .await;
                                }
                                if choice.finish_reason.as_deref() == Some("stop") {
                                    let _ = tx
                                        .send(StreamChunk {
                                            text: String::new(),
                                            done: true,
                                        })
                                        .await;
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }

            let _ = tx
                .send(StreamChunk {
                    text: String::new(),
                    done: true,
                })
                .await;
            Ok(())
        })
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn chat_with_tools(
        &self,
        messages: Vec<Message>,
        config: ChatConfig,
        tools: Vec<ToolDefinition>,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse, LlmError>> + Send + '_>> {
        let body = self.build_body_with_tools(&messages, &config, false, &tools);
        let url = self.chat_url();
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        Box::pin(async move {
            let response = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| LlmError(format!("网络错误: {}", e)))?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .map_err(|e| LlmError(format!("读取响应失败: {}", e)))?;

            if !status.is_success() {
                return Err(OpenAiProvider::parse_error(status, &response_text));
            }

            let parsed: OpenAiResponse = serde_json::from_str(&response_text)
                .map_err(|e| LlmError(format!("解析响应失败: {} - {}", e, response_text)))?;

            let choice = parsed
                .choices
                .first()
                .ok_or_else(|| LlmError("响应中无 choices".to_string()))?;

            // 检查是否有工具调用
            if let Some(tool_calls) = &choice.message.tool_calls {
                let calls: Vec<ToolCallRequest> = tool_calls
                    .iter()
                    .map(|tc| ToolCallRequest {
                        id: tc.id.clone(),
                        name: tc.function.name.clone(),
                        arguments: tc.function.arguments.clone(),
                    })
                    .collect();
                let text = choice
                    .message
                    .content
                    .clone()
                    .filter(|s| !s.is_empty());
                Ok(LlmResponse::ToolCalls { calls, text })
            } else {
                let content = choice
                    .message
                    .content
                    .clone()
                    .filter(|s| !s.is_empty());
                Ok(LlmResponse::Text(content.unwrap_or_else(|| "[思考中...]".to_string())))
            }
        })
    }
}

// ========== Anthropic Claude Provider ==========

/// Anthropic API 响应结构
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContentBlock {
    r#type: Option<String>,
    text: Option<String>,
    id: Option<String>,
    name: Option<String>,
    input: Option<serde_json::Value>,
}

/// Anthropic 流式事件
#[derive(Debug, Deserialize)]
struct ClaudeStreamEvent {
    r#type: String,
    delta: Option<ClaudeDelta>,
}

#[derive(Debug, Deserialize)]
struct ClaudeDelta {
    text: Option<String>,
    stop_reason: Option<String>,
}

/// Anthropic 错误响应
#[derive(Debug, Deserialize)]
struct ClaudeErrorResponse {
    error: ClaudeErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ClaudeErrorDetail {
    message: String,
}

/// Anthropic Claude 提供商
pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl ClaudeProvider {
    const API_URL: &'static str = "https://api.anthropic.com/v1/messages";

    pub fn new(config: LlmProviderConfig) -> Result<Self, LlmError> {
        if config.api_key.is_empty() {
            return Err(LlmError("Anthropic API Key 不能为空".to_string()));
        }
        Ok(Self {
            client: Client::new(),
            api_key: config.api_key,
            model: config.model,
        })
    }

    /// 构建请求体（Anthropic 将 system prompt 与 messages 分离）
    fn build_body(
        &self,
        messages: &[Message],
        config: &ChatConfig,
        stream: bool,
    ) -> serde_json::Value {
        let system_text: Vec<String> = messages
            .iter()
            .filter(|m| m.role == MessageRole::System)
            .map(|m| m.content.clone())
            .collect();

        let non_system = serialize_messages_for_claude(messages);

        let mut body = json!({
            "model": self.model,
            "messages": non_system,
            "temperature": config.temperature,
            "max_tokens": config.max_tokens,
            "top_p": config.top_p,
            "stream": stream,
        });

        if !system_text.is_empty() {
            body.as_object_mut()
                .unwrap()
                .insert("system".to_string(), json!(system_text.join("\n\n")));
        }

        // 应用 Prompt Caching（system + 倒数第 2 个 user 消息标记 ephemeral）
        super::caching::apply_anthropic_cache(&mut body);

        body
    }

    /// 构建带工具的请求体（Claude tool_use 格式）
    fn build_body_with_tools(
        &self,
        messages: &[Message],
        config: &ChatConfig,
        stream: bool,
        tools: &[ToolDefinition],
    ) -> serde_json::Value {
        let mut body = self.build_body(messages, config, stream);
        if !tools.is_empty() {
            let tools_json: Vec<serde_json::Value> = tools
                .iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "input_schema": t.parameters,
                    })
                })
                .collect();
            body.as_object_mut()
                .unwrap()
                .insert("tools".to_string(), json!(tools_json));
        }
        body
    }

    fn parse_error(status: reqwest::StatusCode, body: &str) -> LlmError {
        if let Ok(err_resp) = serde_json::from_str::<ClaudeErrorResponse>(body) {
            match status.as_u16() {
                429 => LlmError(format!("API 限流: {}", err_resp.error.message)),
                401 => LlmError(format!("认证失败: {}", err_resp.error.message)),
                403 => LlmError(format!("权限不足: {}", err_resp.error.message)),
                500..=599 => LlmError(format!("服务器错误: {}", err_resp.error.message)),
                _ => LlmError(format!("API 错误 ({}): {}", status, err_resp.error.message)),
            }
        } else {
            LlmError(format!("API 错误 ({}): {}", status, body))
        }
    }
}

impl LlmProvider for ClaudeProvider {
    fn chat(
        &self,
        messages: Vec<Message>,
        config: ChatConfig,
    ) -> Pin<Box<dyn Future<Output = Result<String, LlmError>> + Send + '_>> {
        let body = self.build_body(&messages, &config, false);
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        Box::pin(async move {
            let response = client
                .post(Self::API_URL)
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| LlmError(format!("网络错误: {}", e)))?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .map_err(|e| LlmError(format!("读取响应失败: {}", e)))?;

            if !status.is_success() {
                return Err(Self::parse_error(status, &response_text));
            }

            let parsed: ClaudeResponse = serde_json::from_str(&response_text)
                .map_err(|e| LlmError(format!("解析响应失败: {} - {}", e, response_text)))?;

            let text_parts: Vec<String> = parsed
                .content
                .iter()
                .filter_map(|block| block.text.clone())
                .collect();

            if text_parts.is_empty() {
                return Err(LlmError("响应中无文本内容".to_string()));
            }

            Ok(text_parts.join(""))
        })
    }

    fn chat_stream(
        &self,
        messages: Vec<Message>,
        config: ChatConfig,
        tx: mpsc::Sender<StreamChunk>,
    ) -> Pin<Box<dyn Future<Output = Result<(), LlmError>> + Send + '_>> {
        let body = self.build_body(&messages, &config, true);
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        Box::pin(async move {
            let response = client
                .post(Self::API_URL)
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| LlmError(format!("网络错误: {}", e)))?;

            let status = response.status();
            if !status.is_success() {
                let error_text = response
                    .text()
                    .await
                    .map_err(|e| LlmError(format!("读取错误响应失败: {}", e)))?;
                return Err(Self::parse_error(status, &error_text));
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result.map_err(|e| LlmError(format!("流读取错误: {}", e)))?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim_end_matches('\r').to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(event) = serde_json::from_str::<ClaudeStreamEvent>(data) {
                            match event.r#type.as_str() {
                                "content_block_delta" => {
                                    if let Some(delta) = event.delta {
                                        if let Some(text) = delta.text {
                                            let _ =
                                                tx.send(StreamChunk { text, done: false }).await;
                                        }
                                    }
                                }
                                "message_delta" => {
                                    if let Some(delta) = &event.delta {
                                        if delta.stop_reason.as_deref() == Some("end_turn") {
                                            let _ = tx
                                                .send(StreamChunk {
                                                    text: String::new(),
                                                    done: true,
                                                })
                                                .await;
                                            return Ok(());
                                        }
                                    }
                                }
                                "message_stop" => {
                                    let _ = tx
                                        .send(StreamChunk {
                                            text: String::new(),
                                            done: true,
                                        })
                                        .await;
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            let _ = tx
                .send(StreamChunk {
                    text: String::new(),
                    done: true,
                })
                .await;
            Ok(())
        })
    }

    fn name(&self) -> &str {
        "claude"
    }

    fn chat_with_tools(
        &self,
        messages: Vec<Message>,
        config: ChatConfig,
        tools: Vec<ToolDefinition>,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse, LlmError>> + Send + '_>> {
        let body = self.build_body_with_tools(&messages, &config, false, &tools);
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        Box::pin(async move {
            let response = client
                .post(Self::API_URL)
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| LlmError(format!("网络错误: {}", e)))?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .map_err(|e| LlmError(format!("读取响应失败: {}", e)))?;

            if !status.is_success() {
                return Err(ClaudeProvider::parse_error(status, &response_text));
            }

            let parsed: ClaudeResponse = serde_json::from_str(&response_text)
                .map_err(|e| LlmError(format!("解析响应失败: {} - {}", e, response_text)))?;

            // 分离文本块和 tool_use 块
            let mut text_parts: Vec<String> = Vec::new();
            let mut tool_calls: Vec<ToolCallRequest> = Vec::new();

            for block in &parsed.content {
                match block.r#type.as_deref() {
                    Some("tool_use") => {
                        if let (Some(id), Some(name)) = (&block.id, &block.name) {
                            let args = block
                                .input
                                .as_ref()
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "{}".to_string());
                            tool_calls.push(ToolCallRequest {
                                id: id.clone(),
                                name: name.clone(),
                                arguments: args,
                            });
                        }
                    }
                    _ => {
                        if let Some(text) = &block.text {
                            if !text.is_empty() {
                                text_parts.push(text.clone());
                            }
                        }
                    }
                }
            }

            if !tool_calls.is_empty() {
                let text = if text_parts.is_empty() {
                    None
                } else {
                    Some(text_parts.join(""))
                };
                Ok(LlmResponse::ToolCalls { calls: tool_calls, text })
            } else if text_parts.is_empty() {
                Err(LlmError("响应中无文本内容".to_string()))
            } else {
                Ok(LlmResponse::Text(text_parts.join("")))
            }
        })
    }
}

// ========== 工厂函数 ==========

/// 根据 LlmProviderConfig 创建对应的 LLM Provider
///
/// 支持的提供商类型:
/// - `openai` — OpenAI 及所有 OpenAI 兼容接口（Deepseek、Moonshot 等）
/// - `claude` — Anthropic Claude API
/// - `glm` — 智谱 AI（GLM-4 等），使用 OpenAI 兼容接口
pub fn create_provider(config: LlmProviderConfig) -> Result<Arc<dyn LlmProvider>, LlmError> {
    match config.provider.as_str() {
        "openai" | "glm" => Ok(Arc::new(OpenAiProvider::new(config)?)),
        "claude" => Ok(Arc::new(ClaudeProvider::new(config)?)),
        other => Err(LlmError(format!(
            "不支持的提供商: {}。支持: openai, claude, glm",
            other
        ))),
    }
}

/// 根据 ProviderPreset（来自 AppConfig）创建对应的 LLM Provider
///
/// 将 ProviderPreset 转换为 LlmProviderConfig 后调用 create_provider。
pub fn create_provider_from_preset(
    preset: &crate::config::ProviderPreset,
) -> Result<Arc<dyn LlmProvider>, LlmError> {
    let config = LlmProviderConfig {
        provider: preset.provider_type.clone(),
        api_key: preset.api_key.clone(),
        api_endpoint: preset.api_endpoint.clone(),
        model: preset.model.clone(),
    };
    create_provider(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> i64 {
        1700000000000
    }

    /// 测试辅助：快速构建 Message（无工具调用）
    fn msg(role: MessageRole, content: &str) -> Message {
        Message {
            role,
            content: content.to_string(),
            timestamp: now(),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    fn test_config() -> LlmProviderConfig {
        LlmProviderConfig {
            provider: "openai".to_string(),
            api_key: "sk-test-key".to_string(),
            api_endpoint: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o".to_string(),
        }
    }

    #[tokio::test]
    async fn test_mock_provider_reply() {
        let provider = MockProvider::new("你好！");
        let messages = vec![msg(MessageRole::User, "嗨")];
        let result = provider.chat(messages, ChatConfig::default()).await;
        assert_eq!(result.unwrap(), "你好！");
    }

    #[tokio::test]
    async fn test_mock_provider_records_messages() {
        let provider = MockProvider::new("回复");
        let messages = vec![msg(MessageRole::User, "你好")];
        let _ = provider.chat(messages, ChatConfig::default()).await;
        let received = provider.messages_received();
        assert_eq!(received.len(), 1);
    }

    #[tokio::test]
    async fn test_mock_stream() {
        let provider = MockProvider::new("hello");
        let messages = vec![msg(MessageRole::User, "hi")];
        let (tx, mut rx) = mpsc::channel(64);
        provider
            .chat_stream(messages, ChatConfig::default(), tx)
            .await
            .unwrap();

        let mut collected = String::new();
        while let Some(chunk) = rx.recv().await {
            if chunk.done {
                break;
            }
            collected.push_str(&chunk.text);
        }
        assert_eq!(collected, "hello");
    }

    #[test]
    fn test_openai_provider_new() {
        let provider = OpenAiProvider::new(test_config());
        assert!(provider.is_ok());
    }

    #[test]
    fn test_openai_provider_empty_key() {
        let config = LlmProviderConfig {
            api_key: String::new(),
            ..test_config()
        };
        let result = OpenAiProvider::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_claude_provider_new() {
        let config = LlmProviderConfig {
            provider: "claude".to_string(),
            api_key: "sk-ant-test".to_string(),
            api_endpoint: "https://api.anthropic.com".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
        };
        let provider = ClaudeProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_claude_provider_empty_key() {
        let config = LlmProviderConfig {
            provider: "claude".to_string(),
            api_key: String::new(),
            api_endpoint: "https://api.anthropic.com".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
        };
        let result = ClaudeProvider::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_provider_openai() {
        let provider = create_provider(test_config());
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "openai");
    }

    #[test]
    fn test_create_provider_claude() {
        let config = LlmProviderConfig {
            provider: "claude".to_string(),
            api_key: "sk-ant-test".to_string(),
            api_endpoint: "https://api.anthropic.com".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
        };
        let provider = create_provider(config);
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "claude");
    }

    #[test]
    fn test_create_provider_unsupported() {
        let config = LlmProviderConfig {
            provider: "gemini".to_string(),
            ..test_config()
        };
        let result = create_provider(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_provider_glm() {
        let config = LlmProviderConfig {
            provider: "glm".to_string(),
            api_key: "test-glm-key".to_string(),
            api_endpoint: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            model: "glm-4".to_string(),
        };
        let provider = create_provider(config);
        assert!(provider.is_ok());
        // GLM 使用 OpenAI 兼容接口，所以 name 是 "openai"
        assert_eq!(provider.unwrap().name(), "openai");
    }

    #[test]
    fn test_create_provider_from_preset() {
        use crate::config::ProviderPreset;
        let preset = ProviderPreset {
            name: "智谱 GLM-4".to_string(),
            provider_type: "glm".to_string(),
            api_endpoint: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            model: "glm-4".to_string(),
            api_key: "test-glm-key".to_string(),
        };
        let provider = create_provider_from_preset(&preset);
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "openai");
    }

    #[test]
    fn test_create_provider_from_preset_claude() {
        use crate::config::ProviderPreset;
        let preset = ProviderPreset {
            name: "Claude Sonnet".to_string(),
            provider_type: "claude".to_string(),
            api_endpoint: "https://api.anthropic.com".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            api_key: "sk-ant-test".to_string(),
        };
        let provider = create_provider_from_preset(&preset);
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "claude");
    }

    #[test]
    fn test_serialize_messages() {
        let messages = vec![
            msg(MessageRole::System, "system prompt"),
            msg(MessageRole::User, "hello"),
        ];
        let serialized = serialize_messages(&messages);
        assert_eq!(serialized.len(), 2);
        assert_eq!(serialized[0]["role"], "system");
        assert_eq!(serialized[1]["role"], "user");
    }

    #[test]
    fn test_openai_build_body() {
        let provider = OpenAiProvider::new(test_config()).unwrap();
        let messages = vec![msg(MessageRole::User, "test")];
        let body = provider.build_body(&messages, &ChatConfig::default(), false);
        assert_eq!(body["model"], "gpt-4o");
        assert_eq!(body["stream"], false);
    }

    #[test]
    fn test_claude_build_body_separates_system() {
        let config = LlmProviderConfig {
            provider: "claude".to_string(),
            api_key: "sk-ant-test".to_string(),
            api_endpoint: "https://api.anthropic.com".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
        };
        let provider = ClaudeProvider::new(config).unwrap();
        let messages = vec![
            msg(MessageRole::System, "system prompt"),
            msg(MessageRole::User, "hello"),
        ];
        let body = provider.build_body(&messages, &ChatConfig::default(), false);
        // Prompt Caching 将 system 字符串转为内容块数组
        let system = body["system"].as_array().unwrap();
        assert_eq!(system[0]["text"], "system prompt");
        assert!(system[0].get("cache_control").is_some());
        let msg_array = body["messages"].as_array().unwrap();
        assert_eq!(msg_array.len(), 1);
        assert_eq!(msg_array[0]["role"], "user");
    }

    #[test]
    fn test_openai_parse_error() {
        let body =
            r#"{"error":{"message":"Rate limit exceeded","type":"rate_limit_error","code":null}}"#;
        let err = OpenAiProvider::parse_error(reqwest::StatusCode::TOO_MANY_REQUESTS, body);
        assert!(err.0.contains("限流"));
    }

    #[test]
    fn test_claude_parse_error() {
        let body = r#"{"error":{"message":"invalid x-api-key","type":"authentication_error"}}"#;
        let err = ClaudeProvider::parse_error(reqwest::StatusCode::UNAUTHORIZED, body);
        assert!(err.0.contains("认证失败"));
    }

    // ===== 工具调用测试 =====

    #[test]
    fn test_tool_definition_serialization_openai() {
        let tool = crate::llm::ToolDefinition {
            name: "get_weather".to_string(),
            description: "Get current weather".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string" }
                },
                "required": ["location"]
            }),
        };
        let provider = OpenAiProvider::new(test_config()).unwrap();
        let body = provider.build_body_with_tools(
            &[msg(MessageRole::User, "What's the weather?")],
            &ChatConfig::default(),
            false,
            &[tool],
        );
        let tools = body["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["type"], "function");
        assert_eq!(tools[0]["function"]["name"], "get_weather");
        assert_eq!(tools[0]["function"]["description"], "Get current weather");
    }

    #[test]
    fn test_tool_definition_serialization_claude() {
        let tool = crate::llm::ToolDefinition {
            name: "get_weather".to_string(),
            description: "Get current weather".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string" }
                },
                "required": ["location"]
            }),
        };
        let config = LlmProviderConfig {
            provider: "claude".to_string(),
            api_key: "sk-ant-test".to_string(),
            api_endpoint: "https://api.anthropic.com".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
        };
        let provider = ClaudeProvider::new(config).unwrap();
        let body = provider.build_body_with_tools(
            &[msg(MessageRole::User, "What's the weather?")],
            &ChatConfig::default(),
            false,
            &[tool],
        );
        let tools = body["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "get_weather");
        assert_eq!(tools[0]["description"], "Get current weather");
        // Claude uses input_schema, not parameters
        assert!(tools[0].get("input_schema").is_some());
    }

    #[test]
    fn test_llm_response_text_variant() {
        let resp = crate::llm::LlmResponse::Text("Hello!".to_string());
        match resp {
            crate::llm::LlmResponse::Text(t) => assert_eq!(t, "Hello!"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_llm_response_tool_calls_variant() {
        let calls = vec![crate::llm::ToolCallRequest {
            id: "call_123".to_string(),
            name: "search".to_string(),
            arguments: r#"{"query":"test"}"#.to_string(),
        }];
        let resp = crate::llm::LlmResponse::ToolCalls {
            calls,
            text: Some("Let me search".to_string()),
        };
        match resp {
            crate::llm::LlmResponse::ToolCalls { calls, text } => {
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0].id, "call_123");
                assert_eq!(calls[0].name, "search");
                assert_eq!(text, Some("Let me search".to_string()));
            }
            _ => panic!("Expected ToolCalls variant"),
        }
    }

    #[test]
    fn test_message_with_tool_role() {
        let m = Message {
            role: MessageRole::Tool,
            content: "result data".to_string(),
            timestamp: now(),
            tool_call_id: Some("call_123".to_string()),
            tool_calls: None,
        };
        assert_eq!(m.role, MessageRole::Tool);
        assert_eq!(m.tool_call_id, Some("call_123".to_string()));
        assert!(m.tool_calls.is_none());
    }

    #[test]
    fn test_message_with_tool_calls() {
        let m = Message {
            role: MessageRole::Assistant,
            content: "Let me look that up".to_string(),
            timestamp: now(),
            tool_call_id: None,
            tool_calls: Some(vec![crate::llm::ToolCallRequest {
                id: "call_abc".to_string(),
                name: "web_search".to_string(),
                arguments: r#"{"q":"rust lang"}"#.to_string(),
            }]),
        };
        assert_eq!(m.role, MessageRole::Assistant);
        let calls = m.tool_calls.unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "web_search");
    }

    #[test]
    fn test_serialize_tool_message_openai() {
        let messages = vec![
            msg(MessageRole::User, "Search for cats"),
            Message {
                role: MessageRole::Assistant,
                content: "".to_string(),
                timestamp: now(),
                tool_call_id: None,
                tool_calls: Some(vec![crate::llm::ToolCallRequest {
                    id: "call_1".to_string(),
                    name: "search".to_string(),
                    arguments: r#"{"q":"cats"}"#.to_string(),
                }]),
            },
            Message {
                role: MessageRole::Tool,
                content: "found 3 results".to_string(),
                timestamp: now(),
                tool_call_id: Some("call_1".to_string()),
                tool_calls: None,
            },
        ];
        let serialized = serialize_messages_for_openai(&messages);
        assert_eq!(serialized.len(), 3);

        // Assistant message with tool_calls
        assert_eq!(serialized[1]["role"], "assistant");
        let tc = serialized[1]["tool_calls"].as_array().unwrap();
        assert_eq!(tc.len(), 1);
        assert_eq!(tc[0]["id"], "call_1");
        assert_eq!(tc[0]["function"]["name"], "search");

        // Tool result message
        assert_eq!(serialized[2]["role"], "tool");
        assert_eq!(serialized[2]["tool_call_id"], "call_1");
        assert_eq!(serialized[2]["content"], "found 3 results");
    }

    #[test]
    fn test_serialize_tool_message_claude() {
        let messages = vec![
            msg(MessageRole::User, "Search for cats"),
            Message {
                role: MessageRole::Assistant,
                content: "Let me search".to_string(),
                timestamp: now(),
                tool_call_id: None,
                tool_calls: Some(vec![crate::llm::ToolCallRequest {
                    id: "toolu_1".to_string(),
                    name: "search".to_string(),
                    arguments: r#"{"q":"cats"}"#.to_string(),
                }]),
            },
            Message {
                role: MessageRole::Tool,
                content: "found 3 results".to_string(),
                timestamp: now(),
                tool_call_id: Some("toolu_1".to_string()),
                tool_calls: None,
            },
        ];
        let serialized = serialize_messages_for_claude(&messages);
        assert_eq!(serialized.len(), 3);

        // User message
        assert_eq!(serialized[0]["role"], "user");
        assert_eq!(serialized[0]["content"], "Search for cats");

        // Assistant message with tool_use content blocks
        assert_eq!(serialized[1]["role"], "assistant");
        let content = serialized[1]["content"].as_array().unwrap();
        assert_eq!(content.len(), 2); // text + tool_use
        assert_eq!(content[0]["type"], "text");
        assert_eq!(content[1]["type"], "tool_use");
        assert_eq!(content[1]["name"], "search");

        // Tool result as user message with tool_result content block
        assert_eq!(serialized[2]["role"], "user");
        let result_blocks = serialized[2]["content"].as_array().unwrap();
        assert_eq!(result_blocks.len(), 1);
        assert_eq!(result_blocks[0]["type"], "tool_result");
        assert_eq!(result_blocks[0]["tool_use_id"], "toolu_1");
    }
}
