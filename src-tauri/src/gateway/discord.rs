//! Discord Webhook 通信
//!
//! 使用 Webhook（而非 Bot）发送消息，更轻量。

use serde::{Deserialize, Serialize};

/// Discord 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub webhook_url: String,
}

/// Discord 网关
pub struct DiscordGateway {
    config: DiscordConfig,
    client: reqwest::Client,
}

impl DiscordGateway {
    pub fn new(config: DiscordConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// 发送消息到 Discord 频道
    pub async fn send_message(&self, content: &str) -> Result<(), String> {
        if self.config.webhook_url.is_empty() {
            return Err("Discord Webhook URL 未配置".into());
        }
        self.client
            .post(&self.config.webhook_url)
            .json(&serde_json::json!({ "content": content }))
            .send()
            .await
            .map_err(|e| format!("Discord 发送失败: {}", e))?;
        Ok(())
    }

    /// 发送嵌入消息（标题 + 正文）
    pub async fn send_notification(&self, title: &str, body: &str) -> Result<(), String> {
        if self.config.webhook_url.is_empty() {
            return Err("Discord Webhook URL 未配置".into());
        }
        self.client
            .post(&self.config.webhook_url)
            .json(&serde_json::json!({
                "embeds": [{
                    "title": title,
                    "description": body,
                    "color": 0x00d4aa
                }]
            }))
            .send()
            .await
            .map_err(|e| format!("Discord 发送失败: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_gateway() {
        let gw = DiscordGateway::new(DiscordConfig {
            webhook_url: "https://discord.com/api/webhooks/test".into(),
        });
        assert_eq!(gw.config.webhook_url, "https://discord.com/api/webhooks/test");
    }

    #[tokio::test]
    async fn send_message_empty_url() {
        let gw = DiscordGateway::new(DiscordConfig {
            webhook_url: String::new(),
        });
        let result = gw.send_message("test").await;
        assert!(result.is_err());
    }
}
