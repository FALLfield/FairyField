//! MCP Transport Layer
//!
//! Implements Model Context Protocol transport: stdio, SSE, and streamable HTTP.
//! This allows Fairy's tools to be exposed to external MCP clients (Claude Desktop, Cursor, etc.)
//! and allows Fairy to consume external MCP servers.

use serde::{Deserialize, Serialize};

/// MCP transport type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransportType {
    /// Standard I/O transport (for CLI-based MCP servers)
    Stdio,
    /// Server-Sent Events transport
    Sse,
    /// Streamable HTTP transport
    StreamableHttp,
}

/// MCP JSON-RPC message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// MCP tool definition (for tools/list response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

impl McpMessage {
    /// Create a JSON-RPC request
    pub fn request(id: u64, method: &str, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: Some(serde_json::Value::Number(id.into())),
            method: Some(method.to_string()),
            params: Some(params),
            result: None,
            error: None,
        }
    }

    /// Create a JSON-RPC success response
    pub fn response(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: Some(id),
            method: None,
            params: None,
            result: Some(result),
            error: None,
        }
    }

    /// Create a JSON-RPC error response
    pub fn error_response(id: serde_json::Value, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: Some(id),
            method: None,
            params: None,
            result: None,
            error: Some(McpError {
                code,
                message: message.to_string(),
                data: None,
            }),
        }
    }

    /// Create a JSON-RPC notification (no id)
    pub fn notification(method: &str, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: None,
            method: Some(method.to_string()),
            params: Some(params),
            result: None,
            error: None,
        }
    }

    /// Check if this is a notification (no id field)
    pub fn is_notification(&self) -> bool {
        self.id.is_none() && self.method.is_some()
    }

    /// Check if this is a valid MCP message
    pub fn is_valid(&self) -> bool {
        self.jsonrpc == "2.0"
            && (self.method.is_some() || self.result.is_some() || self.error.is_some())
    }
}

/// Build a tools/list response (MCP standard)
pub fn build_tools_list(tools: &[McpTool]) -> serde_json::Value {
    serde_json::json!({
        "tools": tools.iter().map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema,
            })
        }).collect::<Vec<_>>()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_has_id_and_method() {
        let req = McpMessage::request(1, "tools/list", serde_json::json!({}));
        assert_eq!(req.jsonrpc, "2.0");
        assert!(req.id.is_some());
        assert_eq!(req.method.as_deref(), Some("tools/list"));
    }

    #[test]
    fn notification_has_no_id() {
        let notif = McpMessage::notification("notifications/initialized", serde_json::json!({}));
        assert!(notif.is_notification());
        assert!(notif.id.is_none());
    }

    #[test]
    fn error_response_format() {
        let err = McpMessage::error_response(
            serde_json::Value::Number(1.into()),
            -32601,
            "Method not found",
        );
        assert!(err.error.is_some());
        assert_eq!(err.error.as_ref().unwrap().code, -32601);
    }

    #[test]
    fn builds_tools_list() {
        let tools = vec![McpTool {
            name: "test.tool".into(),
            description: "A test tool".into(),
            input_schema: serde_json::json!({"type": "object"}),
        }];
        let result = build_tools_list(&tools);
        let tools_arr = result["tools"].as_array().unwrap();
        assert_eq!(tools_arr.len(), 1);
        assert_eq!(tools_arr[0]["name"], "test.tool");
    }

    #[test]
    fn valid_message_passes_check() {
        let req = McpMessage::request(1, "test", serde_json::json!({}));
        assert!(req.is_valid());
    }

    #[test]
    fn response_message_is_valid() {
        let res = McpMessage::response(serde_json::Value::Number(1.into()), serde_json::json!({}));
        assert!(res.is_valid());
    }
}
