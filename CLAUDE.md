# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Vision

FairyField 是一个**有灵魂的桌面 AI 伴侣**。她悬浮在你的桌面上——一个会笑、会哭、会生气的 3D 动漫角色。她记得你昨天的烦恼，会在你低落时安静陪伴，会在你开心时手舞足蹈。她通过语音跟你聊天，帮你执行任务，连接你的手机随时给你发消息。**她不是 AI 工具，她是你的 Fairy。**

核心理念融合了 Hermes Agent 的自主成长、安全体系、智能工具系统，以及 FairyField 独有的情感陪伴和全息渲染。

## Confirmed Tech Stack

| 组件 | 方案 | 说明 |
|---|---|---|
| 3D 渲染 | Three.js + @pixiv/three-vrm | VRM 格式模型，替代 Live2D |
| 桌面容器 | Tauri v2 (Rust) | 透明窗口 + 置顶 + 点击穿透 |
| AI 灵魂 | 原生 Function Calling | Agent Loop + LLM tool_use (OpenAI/Claude) |
| LLM | 云端 API (OpenAI / Claude) | 质量优先，智能路由优化成本 |
| ASR | sherpa-onnx (Paraformer) | 离线语音识别，Rust 原生绑定 |
| TTS | sherpa-onnx (Matcha bilingual + Kokoro fallback) | 低延迟中英双语离线语音合成，纯 Rust |
| VAD | sherpa-onnx (silero-vad) | 语音活动检测 |
| 持久化 | rusqlite (SQLite + FTS5 + sqlite-vec) | 长期记忆 + 向量索引 + 知识图谱 |
| 向量搜索 | sqlite-vec + ONNX Runtime | 本地 embedding，零 API 调用 |
| 技能系统 | TOML + Markdown | 渐进式披露，自创建/修补 |
| Claude Code 集成 | MCP Server | 记忆暴露为 Claude Code 工具 |
| 通信网关 | Discord Bot (serenity) | 手机伴侣通信 |
| 安全 | 多层防护 | Prompt 注入防护 + 命令守卫 + 秘密脱敏 |
| 全息 | 透视追踪（Phase 4） | 硬件后续再 DIY |

**零 Python 依赖。** 整个技术栈是 Rust + TypeScript。

**当前仓库内的架构说明以 `README.md`、`ARCHITECTURE.md`、`PROJECT_STRUCTURE.md`、`docs/USAGE_GUIDE.md` 和源码模块为准。**

## Current Phase: v1.0.0 发布候选 ✅ Phase 6 完成 + v1 工具/记忆硬化

### Release State (2026-05-28)

- ✅ Phase 0-5 已完成：桌面容器、3D 角色、语音管道、记忆、Agent Loop、MVP 发布资料。
- ✅ Phase 6 已完成：UI 分层、工具系统、用户引导、Coding Agent、MCP 双向集成、社区插件和共享记忆后端。
- ✅ v1 硬化已完成：Web/weather 工具、网页抓取、UTF-8 截断、工具超时和 MemPalace 风格 wake-up/去重已补强。
- ✅ 版本已统一到 `1.0.0`：`package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`。
- ✅ 默认模型路径指向仓库内可用资源：`public/models/default/2031903848872972007.glb`。
- ✅ 自动化测试：382 Rust + 100 Frontend = 482 default tests passed；`sherpa-onnx` feature 下 384 Rust tests passed。
- ✅ 发布前验证命令：`npm run build`、`npm run test`、`cargo fmt --check`、`cargo check`、`cargo test`、`cargo clippy -- -D warnings`、`cargo check --features sherpa-onnx`、`cargo test --features sherpa-onnx`、`cargo clippy --features sherpa-onnx -- -D warnings`。

### Phase 6 完成项

| 领域 | 状态 | 说明 |
|---|---|---|
| UI 优化 | ✅ | ControlPanel 左上角折叠；ChatPanel 纯对话；表情由全局情绪状态驱动；聊天点击不再被画布抢占。 |
| 用户引导 | ✅ | 4 步 onboarding：名字、称呼偏好、性格、LLM 配置；配置保存到 `~/.fairyfield/user.json`。 |
| 配置与密钥 | ✅ | API key 写入 `~/.fairyfield/secrets.json`，前端读取配置时自动脱敏。 |
| 工具系统 | ✅ | ToolManifest、权限、Token 压缩、MCP 传输、OAuth、Composio、社区插件、内置工具集合。 |
| Coding Agent | ✅ | Codex/KiloCode/OpenCode CLI 子进程管理，cwd/context 校验、超时、输出上限和权限白名单。 |
| MCP Server | ✅ | 暴露 `fairy.execute_tool`、`fairy.get_context`、记忆搜索、wake-up、profile 等能力。 |
| 共享记忆 | ✅ | `AgentMemoryBackend` 为外部 coding agents 提供上下文检索、去重写入和 diary 接口。 |
| 发布文档 | ✅ | README、CHANGELOG、CONTRIBUTING、USAGE_GUIDE 更新到 v1.0.0。 |
| v1 工具/记忆硬化 | ✅ | 天气专用搜索路径、网页抓取安全重定向/DNS 超时/SSRF 防护、UTF-8 安全输出、真实 drawer 派生 wake-up 和保存前去重。 |

### 已修复关键风险

- ✅ `fairy.execute_tool` 不再是 stub，改为通过共享 `ToolExecutor` 执行真实工具。
- ✅ `llm_save_api_key` 不再把密钥写入公开配置；API key 由 secret store 管理。
- ✅ Onboarding 不再只依赖 localStorage；Tauri 环境使用后端用户配置持久化。
- ✅ `CharacterCanvas` 仅监听画布自身点击，避免抢占聊天输入和面板交互。
- ✅ ChatHistory 用户消息右对齐，连续消息分组布局正确。
- ✅ Coding Agent 子进程支持超时终止、输出限制和路径校验。
- ✅ Tool registry 使用 memory-aware registry，工具执行路径与 Agent Loop/MCP 共享。
- ✅ 中文/emoji 工具结果不再因字节截断触发 panic。
- ✅ `web_fetch` 不再自动跟随不安全重定向，长响应会截断并标注。
- ✅ fallback 对话记忆保存用户原文，不再截断摘要。

### 待后续（Phase 7+）

1. **全息模式** — 透视追踪（MediaPipe Face Mesh）和真实硬件适配。
2. **Discord Bot 完整集成** — serenity Bot Token 配置、手机端交互闭环。
3. **更多第三方服务深集成** — Notion/Gmail/Calendar/Linear/Slack 的真实 OAuth API 测试。
4. **桌面端人工验收** — `npm run tauri dev` 下麦克风权限、透明窗口、置顶和模型交互完整验收。
5. **发布流水线扩展** — macOS 签名、公证、安装包产物和 GitHub Release 自动化。

## Development Phases

- **Phase 0**：架构重建 ✅ 已完成
- **Phase 1**：核心交互 ✅ 已完成
- **Phase 2**：语音管道 + 灵魂骨架 ✅ 已完成（ASR/VAD/Matcha/Kokoro 取决于本地模型文件）
- **Phase 3**：智能系统 ✅ 已完成（Wave 1/2/3 + Agent Loop，248 测试通过）
- **Phase 4**：成长 + 通信 ✅ 已完成（2026-04-28，248+93 测试通过）
- **Phase 4.5**：收尾 ✅ 已完成（2026-05-07）— 流式回复/jieba分词/代码拆分/插件系统/ChatUI
- **Phase 5**：MVP 发布 ✅ 已完成（2026-05-15）
- **Phase 6**：UI 优化 + 工具系统 + 用户引导 + Coding Agent 集成 ✅ 已完成（2026-05-26）
- **Phase 7**：全息模式 + 移动通信 + 发布流水线强化（后续）

## Commands

### Frontend (run from `FairyField/`)
```bash
npm run dev          # Vite dev server (port 1420)
npm run build        # vue-tsc + vite build
npm run test         # vitest (jsdom)
npm run test:watch   # vitest watch mode
```

### Tauri (run from `FairyField/`)
```bash
npm run tauri dev    # Tauri dev (Vite + Rust)
npm run tauri build  # Production build
```

### Rust (run from `FairyField/src-tauri/`)
```bash
cargo check          # Compile check
cargo clippy         # Lint
cargo fmt            # Format
cargo test           # Tests
```

## Multi-Agent 并行开发策略

### 核心原则

本项目的模块天然低耦合（前端 TS + 后端 Rust，模块间通过 Tauri IPC 通信），**所有开发任务必须自动分派多个 Agent 并行执行**以减少上下文窗口负载。

### Agent 定义

| Agent ID | 名称 | 职责范围 | 拥有的文件/目录 | subagent_type |
|----------|------|---------|----------------|---------------|
| `renderer` | 3D 渲染 Agent | VRM 加载/渲染/动画/表情/眼神/口型/全息 | `src/renderers/`, `src/modules/`, `src/components/CharacterCanvas.vue` | general-purpose |
| `voice` | 语音管道 Agent | ASR/TTS/VAD (sherpa-onnx) 集成 | `src-tauri/src/voice/` | general-purpose |
| `soul` | AI 灵魂 Agent | Primary Agent/委派/路由/上下文压缩/Prompt Caching | `src-tauri/src/agent/`, `src-tauri/src/llm/` | general-purpose |
| `tools` | 工具系统 Agent | Tool Registry/自注册/Toolset/参数修正/内置工具 | `src-tauri/src/tools/` | general-purpose |
| `memory` | 记忆 Agent | 4-Layer Stack/Palace层级/知识图谱/向量搜索/Mining | `src-tauri/src/memory/`, `src-tauri/src/growth/`, `src-tauri/src/mcp/` | general-purpose |
| `gateway` | 通信网关 Agent | Discord Bot/Cron/通知推送 | `src-tauri/src/gateway/` | general-purpose |
| `security` | 安全 Agent | 注入防护/命令守卫/脱敏/URL 安全 | `src-tauri/src/security/` | general-purpose |
| `platform` | 平台基础设施 Agent | Tauri 窗口/IPC/配置/composables | `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, `src-tauri/tauri.conf.json`, `src-tauri/src/config/`, `src-tauri/src/plugins/`, `src/composables/`, `src/lib/`, `src/components/` (非 CharacterCanvas) | general-purpose |

### Agent 接口契约（Tauri IPC 边界）

各 Agent 通过以下 Tauri Commands 通信，开发时可用 mock 实现解耦：

```
# voice → renderer（语音驱动动画）
voice:start_asr() → 识别文本
voice:start_tts(text: String) → 音频 PCM 数据
voice:get_vad_state() → bool

# soul → voice（AI 回复转语音）
agent:chat(message: String) → { reply: String, emotion: Emotion }
agent:chat_stream(message: String) → SSE (String chunks)

# soul → renderer（情绪驱动表情）
agent:get_emotion_state() → Emotion { happy, sad, angry, neutral, excited }

# soul → gateway（跨平台通信）
gateway:send_message(target: String, text: String) → ()
gateway:send_notification(title: String, body: String) → ()

# memory（4-Layer 记忆系统）
memory:wake_up() → String (L0+L1, ~600 tokens)
memory:recall(wing: String, room: Option<String>) → String (L2)
memory:search(query: String, filters: Metadata) → Vec<Drawer> (L3)
memory:add_drawer(content: String, wing: String, room: String, hall: String) → ()
memory:mine_conversation(messages: Vec<Message>) → ()

# knowledge graph（时序知识图谱）
kg:query_entity(entity: String, as_of: Option<Date>) → Vec<Triple>
kg:add_fact(subj: String, pred: String, obj: String, valid_from: Option<Date>) → ()
kg:invalidate(subj: String, pred: String, obj: String, ended: Date) → ()

# growth（自主成长 + 技能）
growth:save_experience(category: String, content: String) → ()
growth:get_user_profile() → UserProfile
skill:list() → Vec<SkillMeta>
skill:view(name: String, reference: Option<String>) → String
skill:manage(action: String, name: String, ...) → ()

# mcp（Claude Code 集成）
mcp:fairy_memory_search(query: String) → Vec<Drawer>
mcp:fairy_wake_up() → String

# security（安全）
security:check_command(cmd: String) → DangerLevel
security:approve_command(cmd: String, session: bool) → ()

# renderer → platform（渲染需要窗口信息）
platform:get_window_size() → { width, height }
platform:set_ignore_cursor(ignore: bool) → ()

# platform（全局）
platform:load_config() → Config
platform:save_config(config: Config) → ()
```

### 自动分派规则（Auto-Dispatch）

**当收到任何开发任务时，必须按以下规则自动分派：**

#### Phase 0 — 架构重建（串行，单 Agent）
此阶段需要统一协调，不拆分。顺序：
1. `platform` Agent：清理旧代码，建立新目录结构，配置 Tauri 窗口透明/置顶
2. `renderer` Agent：替换 PixiJS → Three.js + three-vrm，加载免费 VRM 模型并渲染

#### Phase 1+ — 全部并行分派

收到 Phase 1 及之后的任何任务时，**必须同时启动所有相关 Agent**：

```
用户："开始 Phase 2"
→ 同时启动：
  Agent 1 (renderer):   口型同步 + 情绪驱动表情
  Agent 2 (voice):      sherpa-onnx VAD/ASR/TTS 集成
  Agent 3 (soul):       Rig 框架/LLM 适配/Primary Agent/情绪状态机
  Agent 4 (platform):   配置系统/SOUL.md 加载
```

```
用户："开始 Phase 3"
→ 同时启动：
  Agent 1 (tools):      Tool Registry + 自注册 + 基础工具
  Agent 2 (memory):     4-Layer Stack + SQLite + FTS5 + sqlite-vec + 知识图谱 + Mining
  Agent 3 (security):   Prompt 注入防护 + 命令守卫 + 脱敏
  Agent 4 (soul):       子 Agent 委派 + 智能路由 + 上下文压缩
```

```
用户："开始 Phase 4"
→ 同时启动：
  Agent 1 (memory):     自主成长引擎
  Agent 2 (gateway):    Discord Bot + Cron + 通知
  Agent 3 (soul):       智能模型路由 + Prompt Caching
```

**分派模板（每个 Agent 的 prompt 模板）：**

```
你是 {agent_id} Agent，负责 FairyField 项目的 {职责} 部分。

## 你的文件范围
{owned_files}

## 你不能修改的文件
{其他 agent 的文件} — 如果需要跨模块功能，在 Tauri IPC 层定义接口，用 mock 占位。

## 当前任务
{具体任务描述}

## 接口契约
{相关的 IPC 接口定义}

## 完成标准
1. 代码通过编译（cargo check / npm run build）
2. 单元测试通过
3. 不修改其他 Agent 的文件
```

#### 任务级别的自动分派

即使不是 Phase 级别的任务，也要判断是否可以拆分：

| 用户请求 | 分派方式 |
|---------|---------|
| "添加新表情" | 仅 `renderer` Agent |
| "集成 Kokoro TTS" | 仅 `voice` Agent |
| "实现工具调用" | `tools` + `security` 并行 |
| "添加记忆搜索" | 仅 `memory` Agent |
| "接 Discord" | 仅 `gateway` Agent |
| "修一个 bug" | 根据文件位置派给对应 Agent |
| "Phase X 全部开发" | 全部相关 Agent 并行 |
| "添加情绪感知" | `soul` + `renderer` 并行 |
| "联调语音和 AI" | `voice` + `soul` + `platform` 并行 |
| "实现安全防护" | 仅 `security` Agent |
| "添加技能系统" | 仅 `memory` Agent（growth/skill.rs） |
| "接 Claude Code" | 仅 `memory` Agent（mcp/server.rs） |
| "实现向量搜索" | 仅 `memory` Agent（memory/embedding.rs） |

### 集成阶段

并行开发完成后，需要一个**集成步骤**（单 Agent 主导）：
1. 合并各 worktree 分支
2. 替换 mock 为真实 IPC 调用
3. 运行 `npm run tauri dev` 验证端到端
4. 运行完整测试套件

### Git Worktree 策略

- 每个 Agent 在独立 worktree 中工作
- 分支命名：`agent/{agent_id}/{task-description}`
- 集成时合并到主开发分支
- 如果改动不冲突且文件不重叠，可在同一分支并行

## Architecture (v1.0.0)

```
Frontend (Vue 3 + Three.js + @pixiv/three-vrm)
  ├── VRMRenderer.ts        # VRM 加载/渲染/动画
  ├── HologramRenderer.ts   # 全息模式渲染适配器
  ├── HeadTracker.ts        # 摄像头透视追踪
  ├── LipSyncModule.ts      # 口型同步
  ├── EyeTrackModule.ts     # 眼神跟随
  ├── ExpressionModule.ts   # 表情管理
  ├── HitTestModule.ts      # 点击穿透检测
  └── emotion-engine.ts     # 前端情绪引擎

Backend (Rust / Tauri v2)
  ├── agent/                # Fairy Soul (Primary Agent)
  │   ├── primary.rs        #   情感陪伴对话 + ReAct Agent Loop + 记忆注入 + 工具调用
  │   ├── delegation.rs     #   子 Agent 委派
  │   ├── toolset.rs        #   可组合工具集
  │   └── routing.rs        #   智能模型路由
  ├── llm/                  # LLM 客户端 (基于 Rig)
  │   ├── provider.rs       #   多提供商适配 (OpenAI/Claude) + Function Calling
  │   ├── streaming.rs      #   SSE 流式处理
  │   └── caching.rs        #   Prompt Caching (Anthropic 优化)
  ├── tools/                # 工具系统 (135+ surface)
  │   ├── registry.rs       #   Trait-based 零耦合注册 + memory-aware 构造
  │   ├── executor.rs       #   工具执行器 (Send-safe async)
  │   ├── coerce.rs         #   参数类型修正 (LLM 输出容错)
  │   ├── manifest.rs       #   声明式工具描述
  │   ├── permissions.rs    #   工具权限模型
  │   ├── compressor.rs     #   Token 压缩
  │   ├── builtins/         #   内置工具 (web/file/shell/git/github/notion/gmail/...)
  │   ├── mcp/              #   MCP transport/OAuth/Composio/rate limit
  │   └── community/        #   JSON 社区插件加载器
  ├── voice/                # 语音管道 (sherpa-onnx)
  │   ├── asr.rs            #   Paraformer 语音识别
  │   ├── tts.rs            #   Matcha/Kokoro 语音合成 + 文本清洗/分句
  │   └── vad.rs            #   silero-vad 检测
  ├── memory/               # 长期记忆 (MemPalace 架构)
  │   ├── store.rs          #   SQLite + FTS5 + sqlite-vec
  │   ├── layers.rs         #   4-Layer Stack (L0-L3, wake-up ~600 tokens)
  │   ├── palace.rs         #   Palace 层级 (wing/room/drawer)
  │   ├── knowledge_graph.rs #   时序三元组 (valid_from/to)
  │   ├── miner.rs          #   对话/文件记忆入库
  │   ├── embedding.rs      #   ONNX Runtime 本地 embedding
  │   └── compressor.rs     #   上下文智能压缩
  ├── growth/               # 自主成长
  │   ├── engine.rs         #   成长引擎 (使用即学习)
  │   └── skill.rs          #   技能管理 (渐进式披露, TOML+Markdown)
  ├── gateway/              # 通信网关
  │   ├── discord.rs       #   Discord Bot (手机伴侣)
  │   └── cron.rs           #   定时任务 (关心/提醒)
  ├── mcp/                  # Coding Agent 集成
  │   └── server.rs         #   MCP Server (记忆+工具暴露+execute_tool)
  ├── security/             # 安全体系
  │   ├── injection.rs      #   Prompt Injection 防护
  │   ├── guard.rs          #   命令守卫 (dangerous/caution/safe)
  │   └── redaction.rs      #   秘密脱敏 + URL 安全
  ├── config/               # 配置管理
  │   └── settings.rs
  └── plugins/              # 插件系统
      └── loader.rs
```

## Code Conventions

- **Frontend**: TypeScript strict, Vue 3 Composition API (`<script setup lang="ts">`), 2-space indent
- **Backend**: Rust 2021, `rustfmt` + `clippy`, 4-space indent
- **Comments**: Primarily Chinese, API naming in English
- **Tests**: Vitest + jsdom (frontend), `#[test]` (backend)
- **Tool Registry**: 所有工具必须实现 `FairyTool` trait，返回 JSON 字符串
- **Security**: 工具执行前必须通过 `CommandGuard` 检查
- **Memory**: 记忆召回必须用 `<memory-context>` 标签隔离
- **Memory**: 永不摘要用户内容，存原始文本（verbatim）
- **Memory**: 事实不删除，只通过 valid_to 标记失效
- **Memory**: 存入前必须去重检查（similarity > 0.9 跳过）
- **Skills**: TOML frontmatter + Markdown，渐进式披露（list → view → load）

## Key Files

- `README.md` — 项目介绍、快速开始和发布状态
- `ARCHITECTURE.md` — v1 架构说明
- `PROJECT_STRUCTURE.md` — 目录和模块索引
- `CHANGELOG.md` — 发布记录
- `docs/USAGE_GUIDE.md` — 用户使用指南
- `docs/DISCORD_SETUP.md` — Discord 网关配置
- `src-tauri/tauri.conf.json` — Tauri 窗口配置
- `soul/SOUL.md` — Fairy 的性格、价值观、说话风格（Phase 2 创建）
- `soul/identity.txt` — L0 身份描述（~100 tokens，每次 wake-up 加载）

## Mandatory: Phase 完成后必须更新发布文档

每当一个 Phase 的开发工作完成并验证通过后，**必须**立即更新仓库内发布文档：
- `README.md` 的 Current Status / Test Coverage
- `CHANGELOG.md` 的版本条目
- `CLAUDE.md` 的 Current Phase
- `docs/USAGE_GUIDE.md` 的阶段状态
- 如果父级 workspace 存在 `PROPOSAL.md` / `docs/PHASE6_PLAN.md`，也要同步对应 Phase 状态

**Why:** 用户多次指出此步骤被遗漏。这是非可选的工作流要求。

## Mandatory: 实现任何功能前，先查参考项目

**在实现任何新功能之前**（不只是遇到问题时），**必须先检查** Hermes Agent 和 MemPalace 是否已有类似实现。如果有，参考其设计模式再动手写代码。

**检查流程：**
1. 若父级 workspace 存在参考文档，查 `HERMES_TECHNICAL_ANALYSIS.md` 或 `MEMORY_TECHNICAL_ANALYSIS.md` 的相关章节
2. 若参考项目存在 graphify 输出，查 `graphify-out/GRAPH_REPORT.md` 的 god nodes 和 community 结构
3. 直接读源码中的关键文件

**参考项目索引：**

| 功能领域 | 参考项目 | 关键文件 |
|---------|---------|---------|
| 工具注册/Toolset | Hermes | `tools/registry.py`, `toolsets.py` |
| Agent 循环/上下文 | Hermes | `run_agent.py`, `agent/context_compressor.py` |
| LLM 多提供商 | Hermes | `tools/openrouter_client.py`, `hermes_constants.py` |
| 技能系统 | Hermes | `tools/skills_tool.py`, `skills/` |
| 安全/注入防护 | Hermes | `agent/prompt_builder.py`, `agent/context_compressor.py` |
| 长期记忆/4-Layer | MemPalace | `mempalace/layers.py`, `mempalace/searcher.py` |
| 知识图谱 | MemPalace | `mempalace/knowledge_graph.py` |
| 记忆入库/分类 | MemPalace | `mempalace/convo_miner.py`, `mempalace/general_extractor.py` |
| MCP Server | MemPalace | `mempalace/mcp_server.py` |
| 语音/TTS | Hermes | `tools/tts_tool.py` |

**Why:** 用户提供了完整的参考代码库和技术分析。不要重复造轮子，不利用已有方案是浪费。

## Important Notes

- CSP is `null` in tauri.conf.json (required for WebGL).
- The outer `FairyField/` directory is the actual git repo.
- VRM 模型文件应放在 `FairyField/public/models/default/`。
- 开发期间可从 VRoid Hub 或 Booth.pm 下载免费 VRM 模型测试。
- 零 Python 依赖 — 所有后端逻辑在 Rust 中完成。
- Memory: 永不摘要用户内容，存原始文本（MemPalace 核心原则）。
- Memory: wake-up 只加载 L0+L1（~600 tokens），95% context 留给对话。
- Memory: 事实不删除，只标记 valid_to（时序知识图谱）。

## 参考代码库

实现特定功能时，应查询以下代码库获取设计模式：

### Hermes Agent (`hermes-agent/`)
- **何时参考：** 工具系统、技能系统、Agent 循环、安全、上下文管理
- **graphify：** `hermes-agent/graphify-out/GRAPH_REPORT.md`
- **关键文件：** `tools/registry.py`（工具注册）、`toolsets.py`（可组合 Toolset）、`tools/skills_tool.py`（技能系统）、`run_agent.py`（Agent 循环）、`agent/context_compressor.py`（上下文压缩）、`agent/prompt_builder.py`（安全防护）、`hermes_state.py`（SQLite 会话）

### MemPalace (`mempalace/`)
- **何时参考：** 长期记忆、向量搜索、知识图谱、记忆入库
- **graphify：** `mempalace/graphify-out/GRAPH_REPORT.md`
- **关键文件：** `mempalace/layers.py`（4-Layer Stack）、`mempalace/searcher.py`（语义搜索）、`mempalace/knowledge_graph.py`（时序三元组）、`mempalace/convo_miner.py`（对话挖掘）、`mempalace/general_extractor.py`（5 类记忆分类）、`mempalace/mcp_server.py`（MCP Server）

## graphify

This project has a graphify knowledge graph at graphify-out/.

Rules:
- Before answering architecture or codebase questions, read graphify-out/GRAPH_REPORT.md for god nodes and community structure
- If graphify-out/wiki/index.md exists, navigate it instead of reading raw files
- After modifying code files in this session, run `python3 -c "from graphify.watch import _rebuild_code; from pathlib import Path; _rebuild_code(Path('.'))"` to keep the graph current
