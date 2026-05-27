//! Browser control tool with conservative URL validation.

use crate::tools::executor::{Tool, ToolError};
use crate::tools::manifest::{ToolCategory, ToolManifest, ToolPermission};
use serde::Deserialize;
use std::net::{IpAddr, ToSocketAddrs};

#[derive(Debug, Deserialize)]
struct BrowserInput {
    action: String,
    url: String,
}

pub struct BrowserTool;

impl BrowserTool {
    pub fn new() -> Self {
        Self
    }

    pub fn manifest() -> ToolManifest {
        ToolManifest {
            id: "browser".into(),
            name: "Browser".into(),
            description: "Validate and queue safe browser URL open operations.".into(),
            category: ToolCategory::Browser,
            parameters: Self.parameters_schema(),
            permissions: vec![
                ToolPermission::BrowserControl,
                ToolPermission::NetworkAccess,
            ],
            rate_limit: Some(5),
            timeout_ms: 10000,
            enabled_by_default: true,
            oauth: None,
            version: Some("1.0.0".into()),
            author: Some("FairyField".into()),
            long_description: Some(
                "Only http/https public hosts are accepted; local and private hosts are rejected."
                    .into(),
            ),
        }
    }

    fn validate_url(url: &str) -> Result<(), String> {
        let lower = url.to_ascii_lowercase();
        if !lower.starts_with("https://") && !lower.starts_with("http://") {
            return Err("Only http and https URLs are allowed".into());
        }
        let host = extract_host(url).ok_or_else(|| "URL host is required".to_string())?;
        let host_lower = host.to_ascii_lowercase();
        if matches!(host_lower.as_str(), "localhost" | "0.0.0.0") || host_lower.ends_with(".local")
        {
            return Err("Local hosts are not allowed".into());
        }
        if let Ok(ip) = host_lower.parse::<IpAddr>() {
            if is_private_or_local_ip(ip) {
                return Err("Private or local IP addresses are not allowed".into());
            }
        } else if let Ok(addrs) = (host_lower.as_str(), 80).to_socket_addrs() {
            for addr in addrs.take(4) {
                if is_private_or_local_ip(addr.ip()) {
                    return Err("Host resolves to a private or local address".into());
                }
            }
        }
        Ok(())
    }
}

fn extract_host(url: &str) -> Option<String> {
    let after_scheme = url.split_once("://")?.1;
    let host_port = after_scheme.split(['/', '?', '#']).next()?;
    let host = host_port.rsplit('@').next()?.split(':').next()?;
    if host.is_empty() {
        None
    } else {
        Some(host.trim_matches(['[', ']']).to_string())
    }
}

fn is_private_or_local_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private() || v4.is_loopback() || v4.is_link_local() || v4.is_unspecified()
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified() || v6.is_unique_local(),
    }
}

impl Tool for BrowserTool {
    fn name(&self) -> &str {
        "browser"
    }
    fn description(&self) -> &str {
        "Validate and queue safe browser URL open operations"
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type":"object",
            "properties":{
                "action":{"type":"string","enum":["open_url"],"description":"Browser operation"},
                "url":{"type":"string","description":"Public http/https URL"}
            },
            "required":["action","url"]
        })
    }
    fn validate_input(&self, input: &str) -> bool {
        serde_json::from_str::<BrowserInput>(input)
            .map(|p| p.action == "open_url" && Self::validate_url(&p.url).is_ok())
            .unwrap_or(false)
    }
    fn execute(
        &self,
        input: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>>
    {
        let parsed = serde_json::from_str::<BrowserInput>(input);
        Box::pin(async move {
            let params = parsed.map_err(|e| ToolError::ValidationFailed(e.to_string()))?;
            BrowserTool::validate_url(&params.url).map_err(ToolError::ValidationFailed)?;
            Ok(serde_json::json!({"status":"queued","action":"open_url","url":params.url,"platform_command":"open_external_url"}).to_string())
        })
    }
}

impl Default for BrowserTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_public_https() {
        assert!(
            BrowserTool.validate_input(r#"{"action":"open_url","url":"https://example.com/a"}"#)
        );
    }
    #[test]
    fn rejects_localhost() {
        assert!(
            !BrowserTool.validate_input(r#"{"action":"open_url","url":"http://localhost:1420"}"#)
        );
    }
    #[test]
    fn rejects_private_ip() {
        assert!(!BrowserTool.validate_input(r#"{"action":"open_url","url":"http://192.168.1.1"}"#));
    }
}
