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
| TTS | sherpa-onnx (Kokoro) | 离线语音合成，纯 Rust |
| VAD | sherpa-onnx (silero-vad) | 语音活动检测 |
| 持久化 | rusqlite (SQLite + FTS5 + sqlite-vec) | 长期记忆 + 向量索引 + 知识图谱 |
| 向量搜索 | sqlite-vec + ONNX Runtime | 本地 embedding，零 API 调用 |
| 技能系统 | TOML + Markdown | 渐进式披露，自创建/修补 |
| Claude Code 集成 | MCP Server | 记忆暴露为 Claude Code 工具 |
| 通信网关 | Discord Bot (serenity) | 手机伴侣通信 |
| 安全 | 多层防护 | Prompt 注入防护 + 命令守卫 + 秘密脱敏 |
| 全息 | 透视追踪（Phase 4） | 硬件后续再 DIY |

**零 Python 依赖。** 整个技术栈是 Rust + TypeScript。

**完整技术方案和架构设计见 `PROPOSAL.md`。**

## Current Phase: Phase 4.5 完成 → Phase 5 部分完成（待人工验证）

### Phase 3 全部完成 (2026-04-25)

**Wave 1 ✅ 已完成（2026-04-23）：** 工具系统、记忆存储、安全防护、子 Agent 委派（172/172 测试通过）

**Wave 2 ✅ 已完成（2026-04-25）：**
- ✅ 前端记忆/工具/安全 IPC 替换 mock 为真实调用
- ✅ 知识图谱 IPC 命令（kg_add_fact, kg_invalidate, kg_query_entity, kg_query_relation, kg_search）
- ✅ 向量搜索 IPC 命令（memory_vector_search）
- ✅ Gateway 基础模块（Discord Bot stub + Cron 调度器 + 命令）
- ✅ Growth 基础模块（成长引擎 + 技能管理）
- ✅ LLM 智能路由（routing.rs）

**Wave 3 ✅ 已完成（2026-04-25）：**
- ✅ 知识图谱（时序三元组 + valid_from/to + 失效标记）
- ✅ 向量搜索（sqlite-vec + ONNX embedding）
- ✅ 234/234 Rust 测试通过，93/93 前端测试通过，frontend build 通过

**Agent Loop ✅ 已完成（2026-04-25）：** 自主工具调用 + 记忆集成
- ✅ LLM Function Calling（OpenAI tool_use / Claude function_calling）
- ✅ Memory Tool Integration（MemorySearchTool/MemorySaveTool 真实记忆层）
- ✅ Agent Loop（ReAct 循环 max 5 rounds + 记忆注入 + 工具执行）
- ✅ Frontend DevMode（Ctrl+Shift+D 切换开发面板，AgentStatusBadge + AgentLogPanel）

### Phase 4 完成 (2026-04-28, 244 Rust + 93 Frontend 测试通过)
- ✅ Growth Engine（growth/engine.rs + skill.rs + growth/commands.rs，9 IPC 命令）
- ✅ Gateway（gateway/discord.rs + cron.rs + commands.rs，Webhook + CronScheduler）
- ✅ LLM Routing（llm/routing.rs，复杂度分类 Fast/Balanced/Powerful）
- ✅ MCP Server（mcp/server.rs，4 工具：memory_search/wake_up/recall_wing/get_user_profile）
- ✅ Prompt Caching（llm/caching.rs，System+3 策略，缓存命中统计）
- ✅ 端到端集成验证（244 Rust 测试 + 93 前端测试 + build 通过）

### Phase 4.5 收尾 ✅ 已完成（2026-05-07，248 Rust + 93 Frontend）
1. ✅ **前端流式回复** — useAgent.ts 改用 agentChatStream，监听 agent:stream-token/done 事件
2. ✅ **System prompt 增强** — SOUL.md 新增"工具使用"章节，定义何时使用工具/记忆搜索
3. ✅ **中文 FTS5** — 集成 jieba-rs 0.7，MemoryStore 自动分词中文查询和索引
4. ✅ **前端代码拆分** — vite.config.ts manualChunks: vendor(70KB)/three(520KB)/three-vrm(146KB)/index(86KB)
5. ✅ **插件系统** — PluginLoader 实现（JSON 动态加载，4 测试），plugins/example.json 示例
6. ✅ **Chat UI 增强** — ChatBubble 情绪颜色 + 打字动画 + 无障碍
7. ✅ **Kokoro TTS stub** — KokoroTts 结构体完整，需模型文件；MacSayTts 仍为默认
8. ✅ **ASR/VAD stub** — SherpaOnnxAsr/SileroVad 定义，含降级回退和模型下载指引

### 已修复 Bug (2026-04-27)
1. ✅ **agent:status 事件从未发出** — PrimaryAgent.app_handle 修复
2. ✅ **对话记忆挖掘空操作** — mine_conversation 完整流程实现
3. ✅ **strip_html_tags 泄漏 script/style** — 标签名跟踪修复

### 已修复 Bug (2026-05-09) — 人工测试反馈修复
1. ✅ **`[emotion:xxx]` 标签泄漏到聊天** — chat_stream 改为先 parse_emotion_tag 再流式发送 cleaned_reply
2. ✅ **ControlPanel LLM 提供商不显示** — 增加错误处理和重试按钮，provider section 始终可见
3. ✅ **SherpaOnnxAsr/SileroVad stub 缺失** — 新增带文档注释和降级回退的 stub 实现

### 人工测试结果 (2026-05-09)

| # | 测试项 | 状态 | 备注 |
|---|--------|------|------|
| 1 | 流式聊天回复 | ✅ 已修复 | emotion tag 泄漏和 thinking 持续 bug 已修复 |
| 2 | System Prompt 工具指引 | ✅ 通过 | SOUL.md 含完整工具使用章节 |
| 3 | 中文 FTS5 分词 | ✅ 通过 | jieba-rs 0.7 集成，编译和搜索正常 |
| 4 | 前端代码拆分 | ✅ 通过 | vendor/three/three-vrm/index 4 chunk 构建 |
| 5 | 插件系统 | ✅ 通过 | PluginLoader 4 测试通过 |
| 6 | Discord 网关 | ✅ 通过 | gateway 6 测试通过 |
| 7 | 语音管道 | ✅ 已补全 | MacSayTts+KokoroTts+SherpaOnnxAsr+SileroVad stubs 完整 |
| 8 | 情绪系统 | ✅ 通过 | 情绪切换 + 自然衰减正常 |
| 9 | LLM 提供商切换 | ✅ 已修复 | ControlPanel 错误处理增强，重试功能 |
| 10 | 记忆系统 | ✅ 通过 | 多轮对话后可回忆相关内容 |
| 11 | 安全系统 | ✅ 通过 | 注入检测 + 密钥脱敏正常 |
| -- | --- | -- | -- |
| A | `npm run build` | ✅ 通过 | 5 chunk 构建无错误 |
| B | `npm run test` | ✅ 通过 | 93 tests passed |
| C | `cargo check` | ✅ 通过 | 无 error（16 warnings） |
| D | `cargo test` | ✅ 通过 | 248 tests passed |

### Phase 5：MVP 发布（进行中）

**MVP 目标：** 真实语音引擎 + 语音输入 UI + 开源发布

**进行中 (2026-05-09)：**

| # | 任务 | 状态 | 涉及文件 |
|---|------|------|----------|
| 1 | 真实 Kokoro TTS 引擎 | 🔄 实现中 | `src-tauri/src/voice/tts.rs` — sherpa-onnx OfflineTts API |
| 2 | 真实 Paraformer ASR 引擎 | 🔄 实现中 | `src-tauri/src/voice/asr.rs` — OnlineRecognizer |
| 3 | 真实 Silero VAD 引擎 | 🔄 实现中 | `src-tauri/src/voice/vad.rs` — VoiceActivityDetector |
| 4 | AudioInput 麦克风捕获 | 🔄 实现中 | `src-tauri/src/voice/audio_input.rs` — cpal input stream |
| 5 | 模型下载脚本 | ✅ 已完成 | `scripts/download-models.sh` |
| 6 | Chat UI 重设计 + STT | 🔄 实现中 | ChatPanel 麦克风按钮 + 录音 UI + App.vue 接线 |
| 7 | Discord 使用指南 | ✅ 已完成 | `docs/DISCORD_SETUP.md` |
| 8 | 用户指南 | ✅ 已完成 | `docs/USAGE_GUIDE.md` |
| 9 | GitHub 开源发布 | 🔄 实现中 | README + LICENSE + CONTRIBUTING + .gitignore |
| 10 | macOS 麦克风权限 | 🔄 实现中 | `tauri.conf.json` Info.plist 配置 |

### 待后续（Phase 5+，按优先级）
1. **全息模式** — 透视追踪（MediaPipe Face Mesh），当前延后
2. **Discord Bot 完整集成** — 替换 Webhook stub 为 serenity Bot（需 Bot Token）
3. **Three.js 懒加载** — 动态 import 降低首屏加载
4. **npm run tauri dev 端到端** — 完整桌面验证
5. **数据采集模块** — 交互日志、延迟测量
6. **更多通信平台** — 微信 Server酱、邮件 SMTP

### 下一步指令
1. 等待 voice engine + Chat UI agent 完成实现
2. `cargo check && cargo test && npm run build && npm run test` — 全部验证
3. `npm run tauri dev` — 完整端到端启动验证
4. git commit 所有变更并推送 GitHub

## Development Phases

- **Phase 0**：架构重建 ✅ 已完成
- **Phase 1**：核心交互 ✅ 已完成
- **Phase 2**：语音管道 + 灵魂骨架 ✅ 基本完成（ASR/VAD/Kokoro 待模型文件）
- **Phase 3**：智能系统 ✅ 已完成（Wave 1/2/3 + Agent Loop，248 测试通过）
- **Phase 4**：成长 + 通信 ✅ 已完成（2026-04-28，248+93 测试通过）
- **Phase 4.5**：收尾 ✅ 已完成（2026-05-07）— 流式回复/jieba分词/代码拆分/插件系统/ChatUI
- **Phase 5**：完善 + 研究准备 — 全息模式延后，其余项基本完成

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

## Architecture (Target — See PROPOSAL.md §8 for details)

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
  ├── tools/                # 工具系统 (自注册)
  │   ├── registry.rs       #   Trait-based 零耦合注册 + memory-aware 构造
  │   ├── executor.rs       #   工具执行器 (Send-safe async)
  │   ├── coerce.rs         #   参数类型修正 (LLM 输出容错)
  │   ├── web.rs            #   web_search, web_fetch (⚠️ MOCK)
  │   ├── file_ops.rs       #   read_file, write_file, search_files
  │   ├── terminal.rs       #   终端命令 (白名单)
  │   ├── git.rs            #   Git 操作
  │   ├── memory_tool.rs    #   记忆检索/写入 (真实 MemoryLayers)
  │   └── communication.rs  #   跨平台消息
  ├── voice/                # 语音管道 (sherpa-onnx)
  │   ├── asr.rs            #   Paraformer 语音识别
  │   ├── tts.rs            #   Kokoro 语音合成
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
  ├── mcp/                  # Claude Code 集成
  │   └── server.rs         #   MCP Server (记忆+工具暴露)
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

- `PROPOSAL.md` — 技术方案（选型、架构、开发计划、参考代码库索引）
- `HERMES_TECHNICAL_ANALYSIS.md` — Hermes Agent 技术分析报告
- `MEMORY_TECHNICAL_ANALYSIS.md` — MemPalace 长期记忆系统技术拆解
- `.kiro/specs/desktop-anime-agent/requirements.md` — 需求规格（15 条需求，中文）
- `.kiro/specs/desktop-anime-agent/design.md` — 原设计文档（中文）
- `src-tauri/tauri.conf.json` — Tauri 窗口配置
- `soul/SOUL.md` — Fairy 的性格、价值观、说话风格（Phase 2 创建）
- `soul/identity.txt` — L0 身份描述（~100 tokens，每次 wake-up 加载）

## Mandatory: Phase 完成后必须更新 PROPOSAL.md

每当一个 Phase 的开发工作完成并验证通过后，**必须**立即更新 PROPOSAL.md 中 §9 对应的 Phase 章节：
- 将 `- [ ]` 改为 `- [x]`
- 在 Phase 标题后添加 `✅ 已完成`
- 更新 `Current Phase` 为下一个 Phase
- 更新 memory/phase0_status.md

**Why:** 用户多次指出此步骤被遗漏。这是非可选的工作流要求。

## Mandatory: 实现任何功能前，先查参考项目

**在实现任何新功能之前**（不只是遇到问题时），**必须先检查** Hermes Agent 和 MemPalace 是否已有类似实现。如果有，参考其设计模式再动手写代码。

**检查流程：**
1. 查 `HERMES_TECHNICAL_ANALYSIS.md` 或 `MEMORY_TECHNICAL_ANALYSIS.md` 的相关章节
2. 查 `graphify-out/GRAPH_REPORT.md` 的 god nodes 和 community 结构
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
