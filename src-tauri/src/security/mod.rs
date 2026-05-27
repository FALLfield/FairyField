//! 安全模块
//!
//! 提供多层安全防护：
//! - Prompt Injection 检测和防护
//! - 命令守卫（dangerous/caution/safe 三级）
//! - 秘密脱敏（API key、token 等敏感信息）
//! - URL 安全检查（禁止内网地址和危险 scheme）

// v1 wiring notes:
// - ShellTool enforces an execution whitelist and metacharacter blocking.
// - Toolset paths run prompt-injection checks before executing LLM-selected tools.
// - WebFetchTool performs scheme, localhost/private-IP, DNS, redirect, and size checks.
// - SecretRedactor remains available for logs and user-facing config redaction.
// Phase 7 should consolidate ShellTool's whitelist with CommandGuard's classification table.

pub mod commands;
pub mod guard;
pub mod injection;
pub mod redaction;

pub use guard::CommandGuard;
pub use injection::InjectionDetector;
pub use redaction::SecretRedactor;
