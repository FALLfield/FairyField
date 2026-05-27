//! Prompt Caching 优化
//!
//! 实现 Anthropic Prompt Caching 策略：system_and_3。
//! 缓存系统 prompt + 最近 3 轮对话，减少重复 token 计费。
//! 参考 Hermes agent/prompt_caching.py。

use serde::{Deserialize, Serialize};

/// 缓存策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheStrategy {
    /// 缓存 system prompt + 最近 3 轮对话
    SystemAnd3,
    /// 缓存全部（适用于固定 prompt 场景）
    All,
    /// 不缓存
    None,
}

impl Default for CacheStrategy {
    fn default() -> Self {
        Self::SystemAnd3
    }
}

/// 缓存统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// 总请求数
    pub total_requests: u64,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 估计节省的 token 数
    pub tokens_saved: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_requests as f64
        }
    }
}

/// Prompt Cache 管理器
pub struct PromptCache {
    strategy: CacheStrategy,
    stats: CacheStats,
    /// 上次缓存的消息摘要（用于检测变化）
    last_cache_hash: Option<u64>,
}

impl PromptCache {
    pub fn new(strategy: CacheStrategy) -> Self {
        Self {
            strategy,
            stats: CacheStats::default(),
            last_cache_hash: None,
        }
    }

    pub fn with_default_strategy() -> Self {
        Self::new(CacheStrategy::SystemAnd3)
    }

    /// 标记消息的缓存控制标记（Anthropic 格式）
    ///
    /// 返回标记了 cache_control 的消息列表和缓存统计更新。
    pub fn apply_cache_control(&mut self, messages: &mut [serde_json::Value]) -> bool {
        if matches!(self.strategy, CacheStrategy::None) || messages.is_empty() {
            self.stats.total_requests += 1;
            return false;
        }

        let cache_hash = self.compute_hash(messages);

        // 检查是否命中缓存
        let hit = self.last_cache_hash == Some(cache_hash);
        if hit {
            self.stats.cache_hits += 1;
            let estimated_tokens = self.estimate_tokens(messages);
            self.stats.tokens_saved += estimated_tokens;
        }

        self.last_cache_hash = Some(cache_hash);
        self.stats.total_requests += 1;

        match self.strategy {
            CacheStrategy::SystemAnd3 => self.mark_system_and_3(messages),
            CacheStrategy::All => self.mark_all(messages),
            CacheStrategy::None => false,
        }
    }

    /// System + 3 策略：在 system 消息和第 3 轮对话末尾添加 ephemeral 缓存标记
    fn mark_system_and_3(&self, messages: &mut [serde_json::Value]) -> bool {
        let mut marked = false;

        // 标记 system 消息
        for msg in messages.iter_mut() {
            if msg.get("role").and_then(|v| v.as_str()) == Some("system") {
                if let Some(obj) = msg.as_object_mut() {
                    obj.insert(
                        "cache_control".into(),
                        serde_json::json!({ "type": "ephemeral" }),
                    );
                    marked = true;
                    break;
                }
            }
        }

        // 找到倒数第 3 轮用户消息（user + assistant = 1 轮）
        let mut user_assistant_pairs = 0;
        for i in (0..messages.len()).rev() {
            let role = messages[i].get("role").and_then(|v| v.as_str());
            if role == Some("user") {
                user_assistant_pairs += 1;
                if user_assistant_pairs == 3 {
                    if let Some(obj) = messages[i].as_object_mut() {
                        obj.insert(
                            "cache_control".into(),
                            serde_json::json!({ "type": "ephemeral" }),
                        );
                        marked = true;
                    }
                    break;
                }
            }
        }

        marked
    }

    /// All 策略：标记最后一条消息
    fn mark_all(&self, messages: &mut [serde_json::Value]) -> bool {
        if let Some(last) = messages.last_mut() {
            if let Some(obj) = last.as_object_mut() {
                obj.insert(
                    "cache_control".into(),
                    serde_json::json!({ "type": "ephemeral" }),
                );
                return true;
            }
        }
        false
    }

    /// 计算消息列表的简单哈希
    fn compute_hash(&self, messages: &[serde_json::Value]) -> u64 {
        let mut hash: u64 = 0;
        for msg in messages {
            if let Some(content) = msg.get("content").and_then(|v| v.as_str()) {
                for byte in content.bytes() {
                    hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
                }
            }
        }
        hash
    }

    /// 粗略估计 token 数（中文 ~1.5 字/token，英文 ~4 字符/token）
    fn estimate_tokens(&self, messages: &[serde_json::Value]) -> u64 {
        let mut chars = 0u64;
        for msg in messages {
            if let Some(content) = msg.get("content").and_then(|v| v.as_str()) {
                chars += content.len() as u64;
            }
        }
        chars / 3
    }

    /// 获取缓存统计
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// 重置缓存状态（切换对话时使用）
    pub fn reset(&mut self) {
        self.last_cache_hash = None;
    }
}

/// 为 Anthropic API 请求体添加 cache_control 标记（无状态版本）
///
/// 将 system prompt 转为内容块数组并标记 ephemeral，
/// 并在倒数第 2 个 user 消息的内容上标记 ephemeral。
/// 这利用了 Anthropic 的 Prompt Caching 功能减少 token 计费。
pub fn apply_anthropic_cache(body: &mut serde_json::Value) -> bool {
    let mut applied = false;

    // 1. 将 system 字符串转为内容块数组并标记缓存
    if let Some(system_val) = body.get("system").cloned() {
        if let Some(text) = system_val.as_str() {
            if let Some(obj) = body.as_object_mut() {
                obj.insert(
                    "system".into(),
                    serde_json::json!([{
                        "type": "text",
                        "text": text,
                        "cache_control": { "type": "ephemeral" }
                    }]),
                );
                applied = true;
            }
        }
    }

    // 2. 在倒数第 2 个 user 消息的内容块上标记缓存
    if let Some(messages) = body.get_mut("messages").and_then(|m| m.as_array_mut()) {
        let user_indices: Vec<usize> = messages
            .iter()
            .enumerate()
            .filter(|(_, m)| m.get("role").and_then(|v| v.as_str()) == Some("user"))
            .map(|(i, _)| i)
            .collect();

        if user_indices.len() >= 2 {
            let idx = user_indices[user_indices.len() - 2];
            if let Some(msg) = messages.get_mut(idx) {
                // 将字符串 content 转为内容块数组
                if let Some(content) = msg
                    .get("content")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                {
                    if let Some(msg_obj) = msg.as_object_mut() {
                        msg_obj.insert(
                            "content".into(),
                            serde_json::json!([{
                                "type": "text",
                                "text": content,
                                "cache_control": { "type": "ephemeral" }
                            }]),
                        );
                        applied = true;
                    }
                }
            }
        }
    }

    applied
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_messages(count: usize) -> Vec<serde_json::Value> {
        let mut msgs = vec![serde_json::json!({
            "role": "system",
            "content": "你是 Fairy，一个温暖的 AI 伴侣。"
        })];
        for i in 0..count {
            msgs.push(serde_json::json!({
                "role": "user",
                "content": format!("用户消息 {}", i)
            }));
            msgs.push(serde_json::json!({
                "role": "assistant",
                "content": format!("Fairy 回复 {}", i)
            }));
        }
        msgs
    }

    #[test]
    fn system_and_3_marks_system() {
        let mut cache = PromptCache::new(CacheStrategy::SystemAnd3);
        let mut msgs = make_messages(4);
        cache.apply_cache_control(&mut msgs);

        let system_msg = msgs.first().unwrap();
        assert!(system_msg.get("cache_control").is_some());
    }

    #[test]
    fn system_and_3_marks_third_user() {
        let mut cache = PromptCache::new(CacheStrategy::SystemAnd3);
        let mut msgs = make_messages(5);

        cache.apply_cache_control(&mut msgs);

        // 找到第 3 个用户消息
        let user_msgs: Vec<_> = msgs
            .iter()
            .filter(|m| m.get("role").and_then(|v| v.as_str()) == Some("user"))
            .collect();
        // 倒数第 3 个用户消息应该被标记
        let third_from_end = &user_msgs[user_msgs.len() - 3];
        assert!(third_from_end.get("cache_control").is_some());
    }

    #[test]
    fn no_strategy_does_not_mark() {
        let mut cache = PromptCache::new(CacheStrategy::None);
        let mut msgs = make_messages(3);
        let marked = cache.apply_cache_control(&mut msgs);
        assert!(!marked);
        for msg in &msgs {
            assert!(msg.get("cache_control").is_none());
        }
    }

    #[test]
    fn cache_hit_detection() {
        let mut cache = PromptCache::new(CacheStrategy::SystemAnd3);
        let mut msgs = make_messages(3);

        cache.apply_cache_control(&mut msgs);
        assert_eq!(cache.stats().total_requests, 1);
        assert_eq!(cache.stats().cache_hits, 0);

        // Same messages again
        cache.apply_cache_control(&mut msgs);
        assert_eq!(cache.stats().total_requests, 2);
        assert_eq!(cache.stats().cache_hits, 1);
        assert!(cache.stats().tokens_saved > 0);
    }

    #[test]
    fn reset_clears_cache() {
        let mut cache = PromptCache::new(CacheStrategy::SystemAnd3);
        let mut msgs = make_messages(3);

        cache.apply_cache_control(&mut msgs);
        cache.reset();

        let mut msgs2 = make_messages(3);
        cache.apply_cache_control(&mut msgs2);
        assert_eq!(cache.stats().cache_hits, 0);
    }

    #[test]
    fn stats_hit_rate() {
        let mut cache = PromptCache::new(CacheStrategy::SystemAnd3);
        let mut msgs = make_messages(2);

        cache.apply_cache_control(&mut msgs);
        cache.apply_cache_control(&mut msgs);
        cache.apply_cache_control(&mut msgs);

        assert_eq!(cache.stats().hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn apply_anthropic_cache_marks_system() {
        let mut body = serde_json::json!({
            "model": "claude-sonnet-4-6",
            "system": "你是 Fairy",
            "messages": [
                {"role": "user", "content": "你好"},
                {"role": "assistant", "content": "你好呀"},
                {"role": "user", "content": "今天天气"},
                {"role": "assistant", "content": "晴天"},
                {"role": "user", "content": "再见"},
            ]
        });

        let applied = apply_anthropic_cache(&mut body);
        assert!(applied);

        // system 应该变成数组格式并带 cache_control
        let system = body.get("system").unwrap().as_array().unwrap();
        assert_eq!(system.len(), 1);
        assert!(system[0].get("cache_control").is_some());
    }

    #[test]
    fn apply_anthropic_cache_marks_second_last_user() {
        let mut body = serde_json::json!({
            "model": "claude-sonnet-4-6",
            "system": "system prompt",
            "messages": [
                {"role": "user", "content": "msg1"},
                {"role": "assistant", "content": "reply1"},
                {"role": "user", "content": "msg2"},
                {"role": "assistant", "content": "reply2"},
                {"role": "user", "content": "msg3"},
            ]
        });

        apply_anthropic_cache(&mut body);

        let messages = body.get("messages").unwrap().as_array().unwrap();
        // msg2 (index 2) should be marked as it's the 2nd-to-last user message
        let msg2 = &messages[2];
        let content = msg2.get("content").unwrap().as_array().unwrap();
        assert!(content[0].get("cache_control").is_some());
    }
}
