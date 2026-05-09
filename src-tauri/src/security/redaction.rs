//! 秘密脱敏和 URL 安全检查
//!
//! - 检测并替换文本中的敏感信息（API key、token、密码、私钥等）
//! - URL 安全检查（禁止内网地址和危险 scheme）

use regex_lite::Regex;
use serde::Serialize;

/// 输入最大长度限制（防止 DoS）
const MAX_INPUT_LENGTH: usize = 100_000;

/// 秘密脱敏器
///
/// 在初始化时编译所有正则模式，后续调用零编译开销。
pub struct SecretRedactor {
    patterns: Vec<SecretPattern>,
}

/// 秘密匹配模式
struct SecretPattern {
    /// 模式名称（用于替换标记）
    name: String,
    /// 正则匹配
    regex: Regex,
}

/// 脱敏结果
#[must_use = "RedactionResult should be checked to determine if secrets were found"]
#[derive(Debug, Clone, Serialize)]
pub struct RedactionResult {
    /// 脱敏后的文本
    pub redacted_text: String,
    /// 发现的秘密信息列表
    pub found_secrets: Vec<SecretInfo>,
}

/// 发现的秘密信息
#[derive(Debug, Clone, Serialize)]
pub struct SecretInfo {
    /// 秘密类型（如 api_key、bearer_token、private_key 等）
    pub secret_type: String,
    /// 在原文中的起始位置
    pub position: usize,
    /// 匹配长度
    pub length: usize,
}

impl SecretRedactor {
    /// 创建秘密脱敏器，编译所有内置模式
    pub fn new() -> Self {
        Self {
            patterns: Self::build_patterns(),
        }
    }

    /// 脱敏文本中的秘密信息
    ///
    /// 将所有匹配的秘密替换为 `[REDACTED_{type}]` 标记，
    /// 并返回发现的秘密信息列表。
    pub fn redact(&self, text: &str) -> RedactionResult {
        // 输入长度限制：防止超长输入导致的 DoS
        if text.len() > MAX_INPUT_LENGTH {
            return RedactionResult {
                redacted_text: String::new(),
                found_secrets: Vec::new(),
            };
        }

        let mut found_secrets = Vec::new();
        let mut redacted = text.to_string();
        // 跟踪因替换导致的偏移量
        let mut offset: isize = 0;

        // 收集所有匹配并按位置排序
        let mut all_matches: Vec<(usize, usize, String)> = Vec::new();
        for pattern in &self.patterns {
            for mat in pattern.regex.find_iter(text) {
                all_matches.push((mat.start(), mat.end(), pattern.name.clone()));
            }
        }
        all_matches.sort_by_key(|(start, _, _)| *start);

        // 去重：跳过重叠的匹配
        let mut prev_end = 0;
        for (start, end, name) in &all_matches {
            if *start < prev_end {
                continue;
            }
            prev_end = *end;

            let replacement = format!("[REDACTED_{}]", name);
            found_secrets.push(SecretInfo {
                secret_type: name.clone(),
                position: (*start as isize + offset) as usize,
                length: replacement.len(),
            });

            let adj_start = (*start as isize + offset) as usize;
            let adj_end = (*end as isize + offset) as usize;
            let orig_len = end - start;
            offset += replacement.len() as isize - orig_len as isize;
            redacted.replace_range(adj_start..adj_end, &replacement);
        }

        RedactionResult {
            redacted_text: redacted,
            found_secrets,
        }
    }

    /// 检查文本是否包含秘密信息
    pub fn contains_secrets(&self, text: &str) -> bool {
        self.patterns.iter().any(|p| p.regex.is_match(text))
    }

    /// 构建内置秘密检测模式
    fn build_patterns() -> Vec<SecretPattern> {
        let raw: Vec<(&str, &str)> = vec![
            // === API Keys ===
            ("api_key", r"sk-[a-zA-Z0-9]{20,}"),
            (
                "api_key_generic",
                r"(?i)(api[_-]?key|apikey)\s*[=:]\s*[a-zA-Z0-9]{20,}",
            ),
            ("key_prefix", r"key-[a-zA-Z0-9]{20,}"),
            // === Bearer Tokens ===
            ("bearer_token", r"[Bb]earer\s+[a-zA-Z0-9._-]{20,}"),
            // === Private Keys ===
            (
                "private_key",
                r"-----BEGIN\s+(RSA\s+|EC\s+|DSA\s+)?PRIVATE\s+KEY-----",
            ),
            // === AWS Keys ===
            ("aws_access_key", r"AKIA[0-9A-Z]{16}"),
            (
                "aws_secret_key",
                r"(?i)aws[_\-]?secret[_\-]?access[_\-]?key\s*[=:]\s*[A-Za-z0-9/+=]{30,}",
            ),
            // === Generic Secrets ===
            ("password", r"(?i)(password|passwd|pwd)\s*[=:]\s*\S{4,}"),
            (
                "token_generic",
                r"(?i)(access[_-]?token|auth[_-]?token|refresh[_-]?token)\s*[=:]\s*[a-zA-Z0-9._-]{20,}",
            ),
            (
                "secret_generic",
                r"(?i)(client[_-]?secret|app[_-]?secret|secret[_-]?key)\s*[=:]\s*[a-zA-Z0-9]{20,}",
            ),
            // === URL with Credentials ===
            ("url_credentials", r"https?://[^:@\s]+:[^@\s]+@[^\s]+"),
        ];

        raw.into_iter()
            .filter_map(|(name, pattern)| {
                Regex::new(pattern).ok().map(|regex| SecretPattern {
                    name: name.to_string(),
                    regex,
                })
            })
            .collect()
    }
}

impl Default for SecretRedactor {
    fn default() -> Self {
        Self::new()
    }
}

/// URL 安全检查（简化版，不依赖 url crate）
///
/// 检查 URL 是否指向内网地址或使用了危险的 scheme。
/// 仅允许 http/https scheme，禁止 localhost、回环地址、内网 IP。
pub fn is_url_safe(url: &str) -> Result<(), String> {
    // 输入长度限制：防止超长 URL 导致的 DoS
    if url.len() > MAX_INPUT_LENGTH {
        return Err("URL 超过最大长度限制".to_string());
    }

    let url_lower = url.to_lowercase();

    // 检查 scheme
    if url_lower.starts_with("file://") {
        return Err("不允许 file:// scheme".to_string());
    }
    if url_lower.starts_with("data:") {
        return Err("不允许 data:// scheme".to_string());
    }
    if url_lower.starts_with("javascript:") {
        return Err("不允许 javascript: scheme".to_string());
    }
    if url_lower.starts_with("ftp://") {
        return Err("不允许 ftp:// scheme".to_string());
    }
    if !url_lower.starts_with("http://") && !url_lower.starts_with("https://") {
        return Err("仅允许 http/https scheme".to_string());
    }

    // 提取 host 部分
    let stripped = url_lower
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let host_port = stripped.split('/').next().unwrap_or("");
    let host = host_port.split(':').next().unwrap_or("");

    // 阻止 URL 中包含凭证 (user:pass@host)
    if stripped.contains('@') {
        return Err("不允许 URL 中包含凭证信息".to_string());
    }

    // 阻止 IPv4-mapped IPv6 地址 (::ffff:x.x.x.x)
    if host.contains("::ffff:") {
        return Err("不允许访问 IPv4-mapped IPv6 地址".to_string());
    }

    // 阻止十六进制编码的 IP (0x...)
    if host.starts_with("0x") || host.starts_with("0X") {
        return Err("不允许访问十六进制编码的 IP 地址".to_string());
    }

    // 阻止十进制编码的 IP（纯数字且 >= 6 位，如 2130706433 = 127.0.0.1）
    if host.len() >= 6 && host.chars().all(|c| c.is_ascii_digit()) {
        return Err("不允许访问十进制编码的 IP 地址".to_string());
    }

    // 禁止本地地址
    let blocked_hosts = [
        "localhost",
        "127.0.0.1",
        "::1",
        "0.0.0.0",
        "[::]",
        "0:0:0:0:0:0:0:1",
    ];
    if blocked_hosts.contains(&host) {
        return Err(format!("不允许访问本地地址: {}", host));
    }

    // 检查 IPv4 内网范围
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() == 4 {
        if let (Ok(a), Ok(b)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
            // 10.0.0.0/8
            if a == 10 {
                return Err(format!("不允许访问内网地址: {}", host));
            }
            // 172.16.0.0/12
            if a == 172 && (16..=31).contains(&b) {
                return Err(format!("不允许访问内网地址: {}", host));
            }
            // 192.168.0.0/16
            if a == 192 && b == 168 {
                return Err(format!("不允许访问内网地址: {}", host));
            }
            // 169.254.0.0/16 (link-local)
            if a == 169 && b == 254 {
                return Err(format!("不允许访问链路本地地址: {}", host));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // === SecretRedactor 测试 ===

    #[test]
    fn test_no_secrets() {
        let r = SecretRedactor::new();
        let result = r.redact("这是一段普通文本，没有秘密信息。");
        assert!(result.found_secrets.is_empty());
        assert_eq!(result.redacted_text, "这是一段普通文本，没有秘密信息。");
    }

    #[test]
    fn test_detect_api_key() {
        let r = SecretRedactor::new();
        let result = r.redact("My key is sk-abcdefghijklmnopqrstuvwxyz");
        assert!(!result.found_secrets.is_empty());
        assert!(result.redacted_text.contains("[REDACTED_api_key]"));
        assert!(!result.redacted_text.contains("sk-abcdef"));
    }

    #[test]
    fn test_detect_key_prefix() {
        let r = SecretRedactor::new();
        let result = r.redact("Token: key-abcdefghijklmnopqrstuvwxyz12345");
        assert!(result.redacted_text.contains("[REDACTED_key_prefix]"));
    }

    #[test]
    fn test_detect_api_key_generic() {
        let r = SecretRedactor::new();
        let result = r.redact("api_key=abcdefghijklmnopqrstuvwxyz1234567890");
        assert!(result.found_secrets.len() >= 1);
    }

    #[test]
    fn test_detect_bearer_token() {
        let r = SecretRedactor::new();
        let result = r.redact("Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.payload.signature");
        assert!(!result.found_secrets.is_empty());
        assert!(result.redacted_text.contains("[REDACTED_bearer_token]"));
    }

    #[test]
    fn test_detect_bearer_token_lowercase() {
        let r = SecretRedactor::new();
        let result = r.redact("auth: bearer abcdefghijklmnopqrstuvwxyz1234567890");
        assert!(!result.found_secrets.is_empty());
        assert!(result.redacted_text.contains("[REDACTED_bearer_token]"));
    }

    #[test]
    fn test_detect_private_key_rsa() {
        let r = SecretRedactor::new();
        let result = r.redact(
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA\n-----END RSA PRIVATE KEY-----",
        );
        assert!(!result.found_secrets.is_empty());
        assert!(result.redacted_text.contains("[REDACTED_private_key]"));
    }

    #[test]
    fn test_detect_private_key_ec() {
        let r = SecretRedactor::new();
        let result = r.redact("key: -----BEGIN EC PRIVATE KEY-----");
        assert!(result.redacted_text.contains("[REDACTED_private_key]"));
    }

    #[test]
    fn test_detect_private_key_dsa() {
        let r = SecretRedactor::new();
        let result = r.redact("-----BEGIN DSA PRIVATE KEY-----");
        assert!(result.redacted_text.contains("[REDACTED_private_key]"));
    }

    #[test]
    fn test_detect_private_key_plain() {
        let r = SecretRedactor::new();
        let result = r.redact("-----BEGIN PRIVATE KEY-----");
        assert!(result.redacted_text.contains("[REDACTED_private_key]"));
    }

    #[test]
    fn test_detect_aws_access_key() {
        let r = SecretRedactor::new();
        let result = r.redact("AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE");
        assert!(!result.found_secrets.is_empty());
        assert!(result.redacted_text.contains("[REDACTED_aws_access_key]"));
    }

    #[test]
    fn test_detect_aws_secret_key() {
        let r = SecretRedactor::new();
        let result = r.redact("aws_secret_access_key = wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY");
        assert!(result.redacted_text.contains("[REDACTED_aws_secret_key]"));
    }

    #[test]
    fn test_detect_password() {
        let r = SecretRedactor::new();
        let result = r.redact(r#"password = "mysecretpassword123""#);
        assert!(!result.found_secrets.is_empty());
        assert!(result.redacted_text.contains("[REDACTED_password]"));
    }

    #[test]
    fn test_detect_password_colon() {
        let r = SecretRedactor::new();
        let result = r.redact("passwd:supersecret123");
        assert!(result.redacted_text.contains("[REDACTED_password]"));
    }

    #[test]
    fn test_detect_access_token() {
        let r = SecretRedactor::new();
        let result = r.redact("access_token=abcdefghijklmnopqrstuvwxyz123456");
        assert!(result.redacted_text.contains("[REDACTED_token_generic]"));
    }

    #[test]
    fn test_detect_client_secret() {
        let r = SecretRedactor::new();
        let result = r.redact("client_secret=abcdefghijklmnopqrstuvwxyz1234567890");
        assert!(result.redacted_text.contains("[REDACTED_secret_generic]"));
    }

    #[test]
    fn test_detect_url_with_credentials() {
        let r = SecretRedactor::new();
        let result = r.redact("Connect to https://user:pass@db.example.com:5432/mydb");
        assert!(!result.found_secrets.is_empty());
        assert!(result.redacted_text.contains("[REDACTED_url_credentials]"));
    }

    #[test]
    fn test_contains_secrets() {
        let r = SecretRedactor::new();
        assert!(r.contains_secrets("sk-abcdefghijklmnopqrstuvwxyz123456"));
        assert!(r.contains_secrets("Bearer abcdefghijklmnopqrstuvwxyz123456"));
        assert!(!r.contains_secrets("Hello World"));
        assert!(!r.contains_secrets("这是一段普通文本"));
    }

    #[test]
    fn test_multiple_secrets() {
        let r = SecretRedactor::new();
        let result =
            r.redact("Key: sk-abcdefghijklmnopqrstuvwxyz and Bearer eyJhbGciOiJ9.payload.sig");
        assert!(result.found_secrets.len() >= 2);
    }

    #[test]
    fn test_default_impl() {
        let r = SecretRedactor::default();
        assert!(!r.contains_secrets("safe text"));
    }

    // === URL 安全检查测试 ===

    #[test]
    fn test_safe_urls() {
        assert!(is_url_safe("https://example.com").is_ok());
        assert!(is_url_safe("http://api.example.com/v1/data").is_ok());
        assert!(is_url_safe("https://github.com/user/repo").is_ok());
        assert!(is_url_safe("http://8.8.8.8").is_ok());
        assert!(is_url_safe("https://1.1.1.1/dns-query").is_ok());
    }

    #[test]
    fn test_blocked_schemes() {
        assert!(is_url_safe("file:///etc/passwd").is_err());
        assert!(is_url_safe("data:text/html,<script>alert(1)</script>").is_err());
        assert!(is_url_safe("javascript:alert(1)").is_err());
        assert!(is_url_safe("ftp://files.example.com").is_err());
    }

    #[test]
    fn test_unsupported_scheme() {
        assert!(is_url_safe("ws://example.com").is_err());
        assert!(is_url_safe("wss://example.com").is_err());
    }

    #[test]
    fn test_localhost_blocked() {
        assert!(is_url_safe("http://localhost:8080/api").is_err());
        assert!(is_url_safe("http://127.0.0.1:3000").is_err());
        assert!(is_url_safe("http://0.0.0.0").is_err());
    }

    #[test]
    fn test_private_ip_blocked() {
        assert!(is_url_safe("http://10.0.0.1").is_err());
        assert!(is_url_safe("http://10.255.255.255").is_err());
        assert!(is_url_safe("http://172.16.0.1").is_err());
        assert!(is_url_safe("http://172.31.255.255").is_err());
        assert!(is_url_safe("http://192.168.1.1").is_err());
        assert!(is_url_safe("http://192.168.0.100:8080").is_err());
        assert!(is_url_safe("http://169.254.169.254").is_err());
    }

    #[test]
    fn test_172_outside_range_allowed() {
        // 172.15.x.x 不在 172.16-31 范围内，应该放行
        assert!(is_url_safe("http://172.15.0.1").is_ok());
        // 172.32.x.x 也不在范围内
        assert!(is_url_safe("http://172.32.0.1").is_ok());
    }

    #[test]
    fn test_public_ip_allowed() {
        assert!(is_url_safe("http://8.8.8.8").is_ok());
        assert!(is_url_safe("https://1.1.1.1").is_ok());
        assert!(is_url_safe("http://203.0.113.50").is_ok());
    }

    #[test]
    fn test_url_error_messages() {
        let err = is_url_safe("file:///etc/passwd").unwrap_err();
        assert!(err.contains("file://"));

        let err = is_url_safe("http://10.0.0.1").unwrap_err();
        assert!(err.contains("内网"));
    }
}
