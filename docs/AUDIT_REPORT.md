# FairyField 代码审计报告

> 2026-05-13 | 基线: 250 Rust + 93 Frontend = 343 tests | 审计范围: 全代码库 107 文件

---

## 总览

| 维度 | 评分 | 关键问题数 |
|------|------|-----------|
| 错误处理健壮性 | 🔴 D | 169 个裸 unwrap/expect |
| 安全性 | 🟡 B | 3 个中等风险 |
| 可扩展性 | 🟠 C | 6 个耦合点 |
| 代码整洁度 | 🟡 B | 8 处死代码/冗余 |
| 配置鲁棒性 | 🟡 B | 反序列化已修复，缺少验证 |
| 测试覆盖 | 🟡 B | 343 tests 但集成测试缺失 |
| Git 卫生 | 🟡 B | 18MB 残留历史，无 tags |
| 文档完整性 | 🟡 B | 缺少 CHANGELOG、API docs |

---

## 一、错误处理健壮性 🔴

### P0 — 169 个裸 unwrap/expect 在生产代码中

这是代码库最大的单一风险源。每次 `.unwrap()` 调用都是一个潜在的崩溃点。

| 文件 | 数量 | 风险 |
|------|------|------|
| `llm/provider.rs` | 24 | 🔴 LLM 调用失败时崩溃 |
| `memory/store.rs` | 19 | 🔴 数据库操作失败时崩溃 |
| `memory/knowledge_graph.rs` | 17 | 🔴 知识图谱查询崩溃 |
| `tools/memory_tool.rs` | 14 | 🟠 记忆工具崩溃 |
| `growth/skill.rs` | 12 | 🟠 技能系统崩溃 |
| `lib.rs` | 11 | 🔴 启动阶段崩溃 |
| `plugins/loader.rs` | 10 | 🟠 插件加载崩溃 |
| `growth/engine.rs` | 8 | 🟠 |
| `config/settings.rs` | 8 | 🟡 配置解析 |
| `agent/primary.rs` | 7 | 🔴 Agent 核心崩溃 |

**建议修复策略：**

```rust
// 当前（脆弱）
let db = MemoryStore::new(&path).unwrap();

// 建议（健壮）
let db = MemoryStore::new(&path)
    .context("无法打开记忆数据库")
    .map_err(|e| format!("启动失败: {e}"))?;
```

**优先级：**
1. `agent/primary.rs` — Agent 核心，任何崩溃用户直接看到
2. `lib.rs` — 启动路径，阻止应用启动
3. `memory/store.rs` — 数据持久化，数据丢失风险
4. `llm/provider.rs` — 网络调用，最容易失败

### P1 — 缺少 Structured Logging

当前只用 `eprintln!` / `println!`。无日志级别、无结构化字段、无法在生产中按级别过滤。

**建议：** 引入 `tracing` crate（tokio 生态标准）或 `log` + `env_logger`。

### P2 — 启动阶段 silent error swallowing

```rust
// lib.rs:364
let config = load_from_file().unwrap_or_else(|e| {
    eprintln!("...");  // 打印错误但继续用默认配置
    default_config()    // 可能不是用户想要的
});
```

用户配置解析失败→静默回退默认→API Key 丢失→用户困惑。

**建议：** 配置解析失败时应提示用户修复配置，而非静默回退。

---

## 二、安全性 🟡

### P0 — Shell 命令执行无沙箱

`tools/shell.rs` 允许执行系统命令。虽然有白名单 (`ALLOWED_COMMANDS`)，但白名单维护是持续的安全债务。

**建议：**
- 添加命令参数验证（防止 `echo $(rm -rf /)` 绕过）
- 添加超时机制（已有 partial，需完善）
- 添加环境变量隔离

### P1 — API Key 可以在进程列表中暴露

```rust
// llm/provider.rs — API key 通过环境变量和配置文件传递
// 如果通过命令行参数传递，会暴露在 ps aux 中
```

**当前状态：✅** 环境变量传递，相对安全。

### P2 — 文件操作路径验证

`tools/file_ops.rs` 有一个 TODO 注释：
```rust
// TODO(Phase 3 Wave 2): Full canonicalization check against a configured base directory.
```

**建议：** 实现完整的路径规范化和基础目录限制。

### P3 — `config/default.json` 包含 API Key 字段模板

即使值为空字符串，结构体本身提示了攻击向量（知道字段名 = 知道攻击面）。

**当前状态：✅** `#[serde(skip_serializing)]` 保护序列化路径。

---

## 三、可扩展性 🟠

### P0 — 模块通过具体类型耦合，非 Trait

```rust
// 当前：PrimaryAgent 直接依赖具体类型
pub struct PrimaryAgent {
    provider: Arc<dyn LlmProvider>,  // ✅ trait object
    memory: Arc<Mutex<MemoryLayers>>, // ❌ 具体类型
    miner: Arc<Mutex<ConversationMiner>>, // ❌ 具体类型
}
```

`LlmProvider` 使用了 trait，是好的。但 `MemoryLayers`、`ConversationMiner` 是具体类型，无法替换实现（如切换到 Redis 或远程存储）。

**建议：** 为 MemoryLayers 定义 `MemoryBackend` trait。

### P1 — 5 个 Trait 定义，但大量代码绕过它们

```
src/tools/executor.rs:41:pub trait Tool: Send + Sync {
src/llm/provider.rs:20:pub trait LlmProvider: Send + Sync {
src/voice/tts.rs:30:pub trait TtsEngine: Send + Sync {
src/voice/asr.rs:26:pub trait AsrEngine: Send + Sync {
src/voice/vad.rs:24:pub trait VadEngine: Send + Sync {
```

语音管道已经正确地通过 trait 抽象（TtsEngine/AsrEngine/VadEngine），这是好的设计。但工具系统虽有 `Tool` trait，大量工具代码绕过它用具体类型。

### P2 — 硬编码的模型路径

```rust
// voice/tts.rs:343 — 硬编码路径
let home = std::env::var("HOME")...
PathBuf::from(home).join(".fairyfield/models/paraformer")
```

跨平台不支持 Windows。使用 `dirs` crate 应该用 `dirs::data_dir()` 而非 `$HOME`。

### P3 — sherpa-onnx feature gate 不一致

`create_tts_engine()` 和 `create_asr_engine()` 内部有条件编译块，但 helper 函数（`dirs_next_or_home` 等）已用 `#[cfg(feature = "sherpa-onnx")]` 保护。结构体 `SherpaOnnxAsr` 和 `SileroVad` 使用 `#[cfg(feature)]` 做 A/B 字段，增加了维护复杂度。

**建议：** 统一 feature gate 策略 — 要么全部用 cfg，要么全部用 trait + factory。

---

## 四、代码整洁度 🟡

### P0 — MemoryStore::new() 使用 unwrap

```rust
// memory/store.rs — 数据库创建失败直接 panic
MemoryStore::new(&db_path_str).expect("Failed to create MemoryStore")
```

在 `lib.rs` 中 4 处调用 `MemoryStore::new().expect()`。数据库文件权限问题或磁盘满会直接崩溃。

### P1 — 重复的 `$HOME` 查找模式

5 处代码各自独立查找 `$HOME` 环境变量：
- `voice/tts.rs` x2
- `voice/asr.rs` x1
- `voice/vad.rs` x1
- `config/settings.rs` x2

应抽取为一个 `config_dir()` 或 `models_dir()` 函数。

### P2 — 2.5GB `src-tauri/` 目录

大部分来自 `target/` 构建缓存。`cargo clean` 已清理。建议在 `.gitignore` 已有 `target/` 排除。

### P3 — 未使用的 clippy 建议

`cargo clippy` 有 27 个警告（多为 style）。`cargo fix --lib` 可自动修复 9 个。

---

## 五、配置鲁棒性 🟡

### P0 — ✅ 已修复：serde 字段默认值

已在 `f32aad8` commit 中修复。所有 config 字段现在都有 `#[serde(default)]`。

### P1 — 缺少配置版本迁移

如果配置 schema 在未来版本中改变（如重命名字段），旧用户配置将静默失败（回退默认）。需要：
- 添加 `version` 字段到 config
- 实现迁移函数 `migrate_config(v1) -> v2`

### P2 — 缺少配置验证

配置加载后不验证：
- `api_endpoint` 是否为合法 URL
- `model` 名称是否在已知列表中
- `temperature` 是否在 [0, 2] 范围

---

## 六、测试覆盖 🟡

### 优势
- 250 Rust + 93 Frontend = 343 tests ✅
- TypeScript strict mode enabled ✅
- 单元测试覆盖核心模块 ✅

### 差距
- **无集成测试** — `tests/` 目录不存在
- **无 Tauri IPC 端到端测试** — 无法验证前后端通信
- **无性能测试/基准测试**
- **Mock 使用广泛** — LLM provider、ASR、VAD 都是 mock，真实场景未测
- **无错误路径测试** — 测试只覆盖 happy path

### P0 — 缺少关键模块的测试
- `agent/primary.rs` — Agent 循环（250 tests 中有覆盖，但主要是单元测试）
- `gateway/discord.rs` — Discord webhook（有 6 测试，✅）
- `llm/routing.rs` — 智能路由
- `config/settings.rs` — 配置迁移

---

## 七、Git 卫生 🟡

### P0 — 18MB .glb 模型在 git 历史中

```
$ git rev-list --objects --all | git cat-file --batch-check | sort -k3nr | head -3
18444064 public/models/default/2031903848872972007.glb
1589248 src-tauri/ruvector.db
126471 package-lock.json
```

即使 `HEAD` 已删除这些文件，它们仍在历史中永久占用空间。每次 clone 都会下载 18MB+。

**修复：** 使用 `git filter-branch` 或 `BFG Repo-Cleaner` 从历史中清除。

### P1 — 无 git tags

```
$ git tag  # 无输出
```

无法回退到特定版本。建议对每个 release 打 tag：
```bash
git tag v0.1.0
git tag v0.2.0
```

### P2 — 无 CHANGELOG

建议在根目录创建 `CHANGELOG.md`，按版本记录变更。

### P3 — 无 CI 配置

没有 `.github/workflows/`，无自动测试/构建。

---

## 八、Graphify 知识图谱

### 当前状态
- 上次构建：2026-04-10（一个月前）
- 458 nodes · 539 edges · 84 communities
- 97% EXTRACTED · 3% INFERRED
- God nodes: VRMRenderer(17), HeadTracker(17), ExpressionModule(17)

### 问题
- 图谱已过期 — 引用了已删除的文件（ARCHITECTURE.md, PROJECT_STRUCTURE.md, PixiJS）
- `graphify build` 命令不存在（通过 git hooks 自动更新）
- Git hooks 未安装

**修复：**
```bash
cd /Users/fallfield/Desktop/Projects/FairyField/FairyField
graphify hook install
# 然后修改任意文件并提交，hook 会自动重建图谱
```

---

## 九、修复优先级路线图

### 立即修复（本周）
| # | 问题 | 文件 | 影响 |
|---|------|------|------|
| 1 | 替换 agent/primary.rs 中 unwrap | `primary.rs` (7处) | 防止 Agent 崩溃 |
| 2 | 替换 lib.rs 中 unwrap | `lib.rs` (11处) | 防止启动崩溃 |
| 3 | 替换 memory/store.rs 中 unwrap | `store.rs` (19处) | 防止数据丢失 |
| 4 | 抽取 `models_dir()` 函数 | voice/ + config/ | 消除重复 |
| 5 | Git 历史清理 (BFG) | 整个仓库 | 减小 clone 体积 |
| 6 | 添加 git tag v0.1.0 | — | 版本回退 |
| 7 | 安装 graphify hooks | — | 知识图谱更新 |

### 短期修复（本月）
| # | 问题 | 影响 |
|---|------|------|
| 8 | 引入 `tracing` structured logging | 可观测性 |
| 9 | 定义 `MemoryBackend` trait | 可扩展性 |
| 10 | 配置启动验证 | 用户友好 |
| 11 | 配置版本迁移框架 | 向后兼容 |
| 12 | 创建 CHANGELOG.md | 可维护性 |
| 13 | 实现文件操作路径规范 | 安全性 |

### 中期改进
| # | 问题 | 影响 |
|---|------|------|
| 14 | 添加集成测试（Tauri IPC） | 质量保证 |
| 15 | 添加 CI/CD (GitHub Actions) | 自动化测试 |
| 16 | 统一 feature gate 策略 | 可维护性 |
| 17 | Shell 命令参数沙箱 | 安全性 |
| 18 | 配置热重载 | 用户体验 |

---

## 十、Graphify 知识图谱摘要

```
458 nodes · 539 edges · 84 communities

核心抽象 (God Nodes):
  VRMRenderer(17) · HeadTracker(17) · ExpressionModule(17)
  EyeTrackModule(12) · LipSyncModule(12) · FileOpsTool(10)
  ShellTool(10) · GitTool(8) · PrimaryAgent(8) · ToolExecutor(7)

关键社区:
  AI Agent Core (13 nodes) · LLM Providers (9 nodes)
  Voice Pipeline (3 engines) · Tool Registry (5 tools)
  VRM Rendering Pipeline (4 modules)
```

*(图谱需要重建以反映当前代码状态 — run `graphify hook install`)*

---

> 审计完成时间: 2026-05-13 | 下次审计: 修复 P0 项目后
