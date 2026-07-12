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
| ASR | Feature build: sherpa-onnx Paraformer | 普通 build 为 empty Mock；当前固定三秒录音 |
| TTS | Feature build: Matcha/Kokoro；普通 macOS build: say | 流式首句、barge-in 和多语质量未完成 |
| VAD | sherpa-onnx Silero 模块 | 引擎可初始化，但未接 microphone capture |
| 持久化 | rusqlite (SQLite + FTS5) | drawer 记忆 + 独立知识图谱 CRUD |
| 向量搜索 | 占位实现 | 当前为进程内伪向量，未接 ONNX/sqlite-vec |
| 技能系统 | SQLite CRUD | 尚未接入正常 Agent 自我改进闭环 |
| Coding 集成 | Tauri IPC facade + CLI bridge | 尚无外部 MCP 协议 server；尚不支持 Codex CLI |
| 通信网关 | Discord webhook + Cron 基础模块 | 完整 Bot/平台通信仍待实现 |
| 安全 | 模块存在，端到端未闭环 | 注入检测、命令守卫、脱敏存在；Git mutation/output 已收紧，共享审批、file 边界、记忆隔离待修 |
| 全息 | 透视追踪（Phase 4） | 硬件后续再 DIY |

**零 Python 依赖。** 整个技术栈是 Rust + TypeScript。

**当前仓库内的架构说明以 `README.md`、`ARCHITECTURE.md`、`PROJECT_STRUCTURE.md`、`docs/USAGE_GUIDE.md` 和源码模块为准。**

## Current Phase: v1.0.0 源码级发布候选，Phase 6 结构完成，产品闭环未完成

> 真实状态以 `docs/REALITY_CHECK.md` 和源码为准。Phase 6 的目录、类型和基础设施已经落地，但不能据此推断全部第三方工具、外部 MCP、语义向量、语音闭环、个性化或开发闭环已经达到产品可用状态。

### Release State (2026-05-28)

- ✅ Phase 0-5 的目录、核心类型、桌面容器、3D 角色、基础 Agent/记忆/语音模块和 MVP 文档已落地；真实端到端验收仍有缺口。
- ✅ Phase 6 结构已落地：UI 分层、21 个运行时工具定义、用户引导、Coding CLI bridge、IPC memory facade、社区插件 metadata 和共享记忆类型。
- ✅ v1 硬化已完成：Web/weather 工具、网页抓取、UTF-8 截断、工具超时和 MemPalace 风格 wake-up/去重已补强。
- ✅ v1 closure hardening 已完成：真实麦克风 ASR 捕获/重采样、TTS 开发文本清洗、Tools IPC/MCP 共享 Toolset、终端直执行安全收紧、Development Loop Git 证据门。
- ✅ 版本已统一到 `1.0.0`：`package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`。
- ✅ 默认模型路径指向仓库内可用资源：`public/models/default/2031903848872972007.glb`。
- ✅ 自动化测试：425 Rust + 102 Frontend = 527 default tests passed；`sherpa-onnx` feature 下 427 Rust tests passed。
- ✅ 发布前验证命令：`npm run build`、`npm run test`、`cargo fmt --check`、`cargo check`、`cargo test`、`cargo clippy -- -D warnings`、`cargo check --features sherpa-onnx`、`cargo test --features sherpa-onnx`、`cargo clippy --features sherpa-onnx -- -D warnings`。

### Phase 6 完成项

| 领域 | 状态 | 说明 |
|---|---|---|
| UI 优化 | ✅ | ControlPanel 左上角折叠；ChatPanel 纯对话；表情由全局情绪状态驱动；聊天点击不再被画布抢占。 |
| 用户引导 | ✅ | 4 步 onboarding：名字、称呼偏好、性格、LLM 配置；配置保存到 `~/.fairyfield/user.json`。 |
| 配置与密钥 | ✅ | API key 写入 `~/.fairyfield/secrets.json`，前端读取配置时自动脱敏。 |
| 工具系统 | ⚠️ Partial | 21 个工具注册；核心工具有真实逻辑，多个第三方工具、Browser、Cron、Composio 和社区插件仍为 scaffold。 |
| Coding Agent | ⚠️ Partial | Claude/KiloCode/OpenCode 子进程 bridge 存在；交接时本机已有 Codex 但 bridge 未支持，且没有 UI、worktree、取消/恢复和真实 Goal 语义审计。 |
| MCP | ⚠️ IPC only | 暴露 Tauri commands 和 schema，但没有 stdio/SSE/HTTP MCP handshake 或外部 client 连接。 |
| 共享记忆 | ⚠️ Partial | `AgentMemoryBackend` 有单元测试，但没有生产调用方；向量 index 和外部 MCP 未接通。 |
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
- ✅ `voice_start_asr` 使用真实麦克风样本，自动 downmix/resample，静音或过短录音返回明确错误。
- ✅ `fairy_execute_tool`、`tools_execute` 和 Agent Loop 共享 Toolset 安全路径。
- ✅ Terminal 工具不再通过 `sh -c`，并拒绝 Python/Node/npm/pnpm/Cargo 等通用执行器入口。
- ✅ Development Loop 不再信任 agent 自述；GoalAgent 要求 Git changed-file evidence，越界修改会失败。
- ✅ PrimaryAgent 不再把 web/weather/Git 观察直接当作最终答案；12 轮 Hermes-style loop 支持错误恢复、重复调用限制、预算提示和 tool-free finalization。
- ✅ 多步回归覆盖 web search → file write → file read verification → final response；Git 工具具有 function-calling schema，并拒绝 mutation、`--output`、`--no-index`、external diff/textconv 及其缩写形式。

### 当前关键缺口（交接必读）

- PrimaryAgent 的多步工具继续执行已修复，但完成判断仍依赖模型；没有独立语义 Goal verifier、request cancel、provider failover 或 tool-pair-aware context compaction。
- Security IPC 与 Toolset 使用不同 CommandGuard，审批无法作用于真实工具执行；file/symlink containment 和 memory/tool-output 隔离仍未闭环。
- 前端 abort 与 clear 只改本地状态，不能取消后端请求或清空后端 history。
- 正常聊天的流式输出是完整回复后的两字符模拟，不是 provider streaming。
- VAD 未接麦克风、PCM lip-sync 未接通，模拟嘴型存在只更新一次和不能完全闭合的风险。
- UserConfig 与 soul/SOUL.md 未注入 PrimaryAgent；onboarding 个性化只保存不生效。
- 记忆召回以 system role 注入，存在持久 prompt poisoning 风险；向量搜索生产 index 永远为空。
- 外部 MCP、Composio、community plugin execution 和多数第三方工具仍未接通。
- 详细证据、发布状态与修复顺序见 docs/REALITY_CHECK.md。

### 待后续（Phase 7+）

0. **安全与任务闭环** — 共享 CommandGuard、file 安全、记忆 prompt poisoning、真实 abort/clear、语义 Goal verifier。
1. **流式语音体验** — ASR partial、LLM token queue、首句 TTS、口型同步和延迟遥测。
2. **语音质量修复** — Matcha/Kokoro/Paraformer/Silero 模型健康检查、TTS 清洗测试和语音设置 UI。
3. **Hermes 级工具扩展** — 全工具 smoke harness、MCP 双向导入导出、权限 UI 和日常工具优先级。
4. **UI 精修** — compact/expanded chat、voice-call mode、模型状态和 motion-safe 设置。
5. **XR/全息路径** — 桌面视差、Pepper's ghost 原型、多视图实验和硬件 BOM。
6. **市场与 HCI 研究** — 详见 `docs/ROADMAP_PHASE7.md` 的市场定位、商业验证和研究方向。

## Development Phases

- **Phase 0**：架构重建 ✅ 已完成
- **Phase 1**：核心交互 ✅ 已完成
- **Phase 2**：语音管道 + 灵魂骨架 ⚠️ 模块存在；默认 ASR、VAD、流式语音和 PCM lip-sync 未完成
- **Phase 3**：智能系统 ⚠️ 基础 Agent Loop/FTS/安全模块存在；语义向量、KG 集成和端到端安全未完成
- **Phase 4**：成长 + 通信 ⚠️ CRUD/scaffold 存在；自主成长、Discord 启动接线和 scheduler loop 未完成
- **Phase 4.5**：前端收尾 ⚠️ UI/分词/插件类型存在；真正 provider streaming、abort/clear 和 plugin execution 未完成
- **Phase 5**：MVP 源码基线 ⚠️ 已建立，真实语音和发布安装包未验收
- **Phase 6**：UI + 工具 + 引导 + Coding bridge ⚠️ 结构完成，真实集成和 Codex loop 未完成
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
  Agent 2 (memory):     4-Layer Stack + SQLite + FTS5 + 持久向量索引 + 知识图谱 + Mining
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

## Architecture (current files, status annotated)

```
Frontend (Vue 3 + Three.js + @pixiv/three-vrm)
  ├── VRMRenderer.ts        # VRM 加载/渲染/动画
  ├── HologramRenderer.ts   # 占位开关，无全息渲染
  ├── HeadTracker.ts        # 实验模块，未接入当前 UI
  ├── LipSyncModule.ts      # 模块存在；生产 PCM 未接通
  ├── EyeTrackModule.ts     # 眼神跟随
  ├── ExpressionModule.ts   # 表情管理
  ├── HitTestModule.ts      # 点击穿透检测
  └── emotion-engine.ts     # 前端情绪引擎

Backend (Rust / Tauri v2)
  ├── agent/                # Fairy Soul (Primary Agent)
  │   ├── primary.rs        #   情感陪伴对话 + ReAct Agent Loop + 记忆注入 + 工具调用
  │   ├── delegation.rs     #   Mock/占位委派，未接 PrimaryAgent
  │   ├── toolset.rs        #   可组合工具集
  │   └── development_loop.rs # 确定性开发 pipeline，非默认 chat path
  ├── llm/                  # 原生 HTTP LLM client
  │   ├── provider.rs       #   多提供商适配 (OpenAI/Claude) + Function Calling
  │   ├── streaming.rs      #   SSE parser；PrimaryAgent 当前未使用真实 streaming
  │   ├── routing.rs        #   分类模块；未接 PrimaryAgent
  │   └── caching.rs        #   caching helper；未接正常 chat path
  ├── tools/                # 21 个运行时工具 + 未来扩展 scaffold
  │   ├── registry.rs       #   Trait-based 零耦合注册 + memory-aware 构造
  │   ├── executor.rs       #   工具执行器 (Send-safe async)
  │   ├── coerce.rs         #   参数类型修正 (LLM 输出容错)
  │   ├── manifest.rs       #   声明式工具描述
  │   ├── permissions.rs    #   工具权限模型
  │   ├── compressor.rs     #   Token 压缩
  │   ├── builtins/         #   内置工具 (web/file/shell/git/github/notion/gmail/...)
  │   ├── mcp/              #   MCP/OAuth/Composio 数据结构与 scaffold
  │   └── community/        #   JSON 社区插件加载器
  ├── voice/                # Feature-gated voice + normal-build fallback
  │   ├── asr.rs            #   Paraformer feature path；普通 build empty Mock
  │   ├── tts.rs            #   Matcha/Kokoro feature path + macOS say fallback
  │   └── vad.rs            #   Silero module；未接 microphone capture
  ├── memory/               # MemPalace-inspired storage，非完整 parity
  │   ├── store.rs          #   SQLite + FTS5
  │   ├── layers.rs         #   4-Layer Stack (L0-L3, wake-up ~600 tokens)
  │   ├── palace.rs         #   Palace 层级 (wing/room/drawer)
  │   ├── knowledge_graph.rs #   时序三元组 (valid_from/to)
  │   ├── miner.rs          #   keyword-based 对话入库
  │   ├── embedding.rs      #   当前为非持久伪向量占位
  │   └── agentmemory_backend.rs # unit-tested facade，未接生产 consumer
  ├── growth/               # 手动 SQLite CRUD，未接自主成长
  │   ├── engine.rs         #   experience CRUD
  │   └── skill.rs          #   SQLite skill CRUD，非 TOML/Markdown runtime
  ├── gateway/              # 未接启动流程的通信 scaffold
  │   ├── discord.rs        #   Webhook client；GatewayState 默认 None
  │   └── cron.rs           #   内存 CRUD；无后台 dispatch loop
  ├── mcp/                  # Coding Agent 集成
  │   └── server.rs         #   Tauri IPC memory/tool facade，非外部 MCP server
  ├── security/             # 安全体系
  │   ├── injection.rs      #   Prompt Injection 防护
  │   ├── guard.rs          #   命令守卫 (dangerous/caution/safe)
  │   └── redaction.rs      #   秘密脱敏 + URL 安全
  ├── config/               # 配置管理
  │   └── settings.rs
  └── plugins/              # JSON loader 类型，未接运行时 registry
      └── loader.rs
```

## Code Conventions

- **Frontend**: TypeScript strict, Vue 3 Composition API (`<script setup lang="ts">`), 2-space indent
- **Backend**: Rust 2021, `rustfmt` + `clippy`, 4-space indent
- **Comments**: Primarily Chinese, API naming in English
- **Tests**: Vitest + jsdom (frontend), `#[test]` (backend)
- **Tool Registry**: 运行时工具实现 `Tool` trait并返回 JSON 字符串
- **Security target**: 所有执行路径必须共享同一个 `CommandGuard`；当前 IPC/Toolset 状态分离，必须先修
- **Memory target**: 召回内容必须作为带来源的 untrusted data 隔离；当前 system-role 注入违反此目标
- **Memory storage**: drawer 保存用户原文；展示层可以截断，但不能悄悄改写事实
- **Knowledge graph**: 事实通过 valid_to 失效；当前未接正常 chat mining/recall
- **Dedup current**: 同 wing/room 的 normalized exact match 去重；语义 threshold 未实现
- **Skills target**: 未来采用 TOML frontmatter + Markdown；当前 runtime 是 SQLite CRUD

## Key Files

- `README.md` — 项目介绍、快速开始和发布状态
- `ARCHITECTURE.md` — v1 架构说明
- `PROJECT_STRUCTURE.md` — 目录和模块索引
- `CHANGELOG.md` — 发布记录
- `docs/REALITY_CHECK.md` — v1 真实状态、已知 bug、未完成缺口和下一轮交接
- `docs/USAGE_GUIDE.md` — 用户使用指南
- `docs/AGENT_LOOP.md` — 多 Agent 开发闭环：Manager/Coding/Testing/Goal Agent 协作与安全门
- `docs/ROADMAP_PHASE7.md` — Phase 7 路线图：流式语音、工具、UI、XR、市场、HCI 研究
- `docs/DISCORD_SETUP.md` — Discord 网关配置
- `src-tauri/tauri.conf.json` — Tauri 窗口配置
- `soul/SOUL.md` — 目标性格文档；当前 PrimaryAgent 未加载
- `soul/identity.txt` — 目标 L0 资产；当前 wake-up 未从文件加载

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
