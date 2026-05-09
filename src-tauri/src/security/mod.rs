//! 安全模块
//!
//! 提供多层安全防护：
//! - Prompt Injection 检测和防护
//! - 命令守卫（dangerous/caution/safe 三级）
//! - 秘密脱敏（API key、token 等敏感信息）
//! - URL 安全检查（禁止内网地址和危险 scheme）

// TODO (Wave 2): Wire security checks into tool execution pipeline:
// - CommandGuard::check_command() must be called before ShellTool execution
// - InjectionDetector::check() must be called on all LLM-generated tool inputs
// - SecretRedactor::redact() must be called on all tool outputs before returning to LLM
// - is_url_safe() must be called in WebFetchTool.validate_input()

// TODO (Wave 2): Unify shell command whitelist — tools/shell.rs::ALLOWED_COMMANDS
// and security/guard.rs::CommandGuard must share a single source of truth.
// CommandGuard should be the authoritative list; shell.rs should delegate to it.

pub mod commands;
pub mod guard;
pub mod injection;
pub mod redaction;

pub use guard::CommandGuard;
pub use injection::InjectionDetector;
pub use redaction::SecretRedactor;
