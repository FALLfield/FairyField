//! 工具执行器模块
//!
//! 管理 Agent 可调用的工具集，包括：
//! - 工具执行器（executor）
//! - Git 操作（git）
//! - 文件操作（file_ops）
//! - Shell 命令（shell）
//!
//! Phase 3 实现目标。

pub mod executor;
pub mod file_ops;
pub mod git;
pub mod shell;
