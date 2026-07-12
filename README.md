# FairyField ✨

> **项目愿景：有灵魂的桌面 AI 伴侣。**
> 目标是让她悬浮在桌面上，用表情、语音、记忆和工具长期陪伴用户。
> 当前仓库仍是源码级发布候选，真实完成度见下方状态说明。

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.88+-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Vue-3.5-green?logo=vue.js" alt="Vue">
  <img src="https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri" alt="Tauri">
  <img src="https://img.shields.io/badge/License-MIT-yellow" alt="License">
  <img src="https://img.shields.io/badge/Tests-527%20passed-brightgreen" alt="Tests">
</p>

## 为什么是 FairyField？

在 AI 工具泛滥的时代，FairyField 走了另一条路——**她不是工具，是伴侣**。

- ❌ 不是又一个 ChatGPT 套壳
- ❌ 不是冷冰冰的命令行机器人
- ✅ 是一个有情绪的 3D 角色，住在你的桌面
- ✅ 可选本地语音——启用 sherpa-onnx 并安装模型后，语音可离线处理
- ✅ 本地长期记忆——SQLite + FTS 搜索已实现，语义向量仍在开发

## 功能

| 模块 | 说明 |
|------|------|
| 🎭 **3D 角色** | VRM 模型 + 情绪驱动表情 + 眼神 + 随机 idle；音频驱动口型尚未接通 |
| 🎤 **本地语音路径** | Feature build 可用 Paraformer + Matcha/Kokoro；默认 macOS build 使用系统 TTS，ASR/VAD 尚不可用 |
| 🧠 **AI 灵魂** | Claude / GPT 驱动的 12 轮 Hermes-style 工具循环；多步 search/write/read 已回归覆盖，语义 Goal 验证和 Soul 个性化仍未闭环 |
| 💬 **语音输入** | Feature build 有固定三秒离线转写路径；VAD、stop、partial transcript 和默认 build 支持未完成 |
| 📝 **长期记忆** | SQLite + FTS5 + wake-up 记忆层；知识图谱为独立 CRUD，向量语义搜索仍是占位实现 |
| 🧰 **工具系统** | 运行时注册 21 个工具；核心 Web/文件/Git/记忆路径可执行，部分第三方、浏览器、Cron、Composio 和社区插件仍是 scaffold |
| 🔒 **隐私优先** | 记忆保存在本地 SQLite；密钥与 OAuth token 的平台级安全存储仍待完成 |
| 🪟 **桌面覆盖** | 透明置顶窗口，点击穿透，不干扰工作 |
| 🛡️ **安全模块** | 注入检测、命令守卫、脱敏代码存在；Git mutation/output 已收紧，共享审批、file 边界和记忆隔离仍有高优先级 bug |

## 快速开始

### 前提

- Rust 1.88+
- Node.js 20+
- macOS 14+（Windows/Linux 部分语音功能受限）

### 1. 克隆并安装

```bash
git clone https://github.com/FALLfield/FairyField.git
cd FairyField
npm install
```

### 2. 配置 LLM

```bash
export OPENAI_API_KEY="sk-your-key"
# 或 Anthropic Claude
export ANTHROPIC_API_KEY="sk-ant-your-key"
```

### 3. （可选）下载离线语音模型

```bash
bash scripts/download-models.sh
```

默认下载 Matcha 中文女声 + 英文女声 + Paraformer + Silero VAD。真实本地语音仍需使用 sherpa-onnx feature 启动；普通 macOS 启动使用系统 TTS，Mock ASR 不会产生可用转写。

### 4. 启动

```bash
npm run tauri dev
```

> 启用真实离线语音引擎（开发运行）：`npm run tauri -- dev --features sherpa-onnx`

## 使用

| 操作 | 方式 |
|------|------|
| 发送消息 | 输入文字 → Enter |
| 语音输入 | 点击 🎤 → 说话 → 自动发送 |
| 查看历史 | 点击聊天气泡区域 |
| 切换 LLM | 左上角 ControlPanel 展开后选择 |
| 开发者面板 | `Ctrl+Shift+D` |
| 移动窗口 | 拖拽角色 |

## 架构

```
┌──────────────────────────────────┐
│         Tauri 透明窗口             │
│  ┌──────────┐  ┌───────────────┐  │
│  │ 3D 角色   │  │ 聊天面板       │  │
│  │ Three.js  │  │ Vue 3 气泡 UI  │  │
│  │ + VRM     │  │ + 语音输入     │  │
│  └──────────┘  └───────────────┘  │
├──────────────────────────────────┤
│         Tauri IPC 通信层          │
├──────────────────────────────────┤
│  Rust 后端                       │
│  ├── agent/    AI 灵魂           │
│  ├── voice/    语音 ASR/TTS/VAD  │
│  ├── memory/   记忆 + 知识图谱   │
│  ├── tools/    工具系统          │
│  ├── security/ 安全防护          │
│  ├── gateway/  通信 scaffold     │
│  └── growth/   成长 CRUD         │
└──────────────────────────────────┘
```

## 技术栈

| 层 | 技术 |
|----|------|
| 3D 渲染 | Three.js + @pixiv/three-vrm |
| 桌面框架 | Tauri 2.0 (Rust) |
| 前端 | Vue 3 + TypeScript |
| AI 对话 | OpenAI / Anthropic API |
| 语音识别 | Feature build: sherpa-onnx Paraformer；普通 build: empty Mock |
| 语音合成 | Feature build: Matcha bilingual + Kokoro；普通 macOS build: say |
| 语音检测 | sherpa-onnx Silero 模块存在，麦克风接线待完成 |
| 存储 | SQLite + FTS5；持久向量索引待实现 |
| 知识图谱 | 时序三元组 |
| 安全 | Prompt 注入检测 + 命令守卫 + 密钥脱敏 |

## 开发

```bash
npm run dev          # Vite 开发服务器
npm run build        # 生产构建
npm run test         # 前端测试

cd src-tauri
cargo check          # 编译检查
cargo test           # Rust 测试
cargo clippy         # Lint
```

语音调试重点：启用 `sherpa-onnx` 后，麦克风输入会从系统默认设备捕获真实音频，自动下混到 mono、重采样到 16 kHz，并在静音/过短时返回清晰错误，而不是把空音频送进 ASR。

## 项目状态

| Phase | 内容 | 状态 |
|-------|------|------|
| 0 – 4.5 | 架构 → 核心 → 语音 → 智能基础设施 | ✅ 代码结构完成 |
| 5 | MVP 源码基线（语音引擎 + 输入 UI） | ⚠️ 默认 build 与真实语音验收未闭环 |
| 6 | UI · 工具 · 用户引导 · Coding bridge · 工具/记忆硬化 | ⚠️ 结构完成，产品集成未完成 |
| 未来 | 全息投影 · 数据采集 · 更多通信平台 | ⏳ |

> Status note: this is a source-level release candidate, not a finished public desktop product. Multi-step completion, shared security state, real streaming/abort, default voice, lip sync, semantic memory, external MCP, third-party integrations, CI, and signed packaging still have open blockers. See [V1 真实状态](docs/REALITY_CHECK.md) before publishing or handing work to another model.

## 文档

| 文档 | 说明 |
|------|------|
| [用户指南](docs/USAGE_GUIDE.md) | 安装、配置、使用 |
| [V1 真实状态](docs/REALITY_CHECK.md) | 已验证功能、已知 bug、未完成缺口和下一轮交接 |
| [本地语音策略](docs/LOCAL_TTS.md) | Matcha/Kokoro 选择、延迟设计、模型安装 |
| [多 Agent 开发闭环](docs/AGENT_LOOP.md) | Manager/Coding/Testing/Goal Agent 协作与安全门 |
| [Phase 7 路线图](docs/ROADMAP_PHASE7.md) | 语音流式化、工具扩展、UI、XR、市场和 HCI 研究方向 |
| [架构说明](ARCHITECTURE.md) | v1 前端、后端、工具、记忆和语音架构 |
| [项目结构](PROJECT_STRUCTURE.md) | 仓库目录职责 |
| [人工测试清单](TEST_MANUAL.md) | v1 发布前手动验收 |
| [Discord 配置](docs/DISCORD_SETUP.md) | 手机伴侣通知 |
| [开发约定](CLAUDE.md) | 多 Agent 开发策略 |
| [贡献指南](CONTRIBUTING.md) | 如何参与 |

## License

MIT © 2024-2026 FairyField Contributors
