//! Git 操作工具模块
//!
//! 封装常用 Git 操作，供 Agent 工具调用。

/// Git 工具
///
/// Phase 3 实现：提供 git status、git diff、git commit 等操作，
/// 仅允许白名单内的安全 Git 命令。
#[allow(dead_code)]
pub struct GitTool {
    // TODO: 仓库路径
    // TODO: 允许的命令白名单
}
