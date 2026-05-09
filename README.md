# FairyField ✨

> **有灵魂的桌面 AI 伴侣** — 她悬浮在你的桌面上，会笑、会哭、会生气。记得你的烦恼，用真实的声音跟你聊天。她不是 AI 工具，她是你的 Natasha。

[![Rust](https://img.shields.io/badge/Rust-1.85+-orange?logo=rust)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/Vue-3.5-green?logo=vue.js)](https://vuejs.org)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri)](https://tauri.app)
[![License](https://img.shields.io/badge/License-MIT-yellow)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-341%20passed-brightgreen)]()

## 预览

> *截图即将添加 — 请查看 `docs/screenshots/PLACEHOLDER.md` 了解如何贡献截图*

## 功能特性

- 🎭 **3D 动漫角色** — VRM 模型渲染，情绪驱动表情、口型同步、眼神跟随
- 🎤 **离线语音对话** — 真实 STT（Paraformer）+ TTS（Kokoro）+ VAD（Silero），零网络调用
- 🧠 **AI 灵魂** — Claude / GPT 驱动的情感陪伴对话，带自主工具调用
- 💬 **语音输入** — 点击麦克风按钮说话，自动转文字发送
- 📝 **长期记忆** — 4 层记忆堆栈 + 时序知识图谱，记住你说过的一切
- 🔒 **本地优先** — 语音完全离线，记忆存储在本地 SQLite
- 🪟 **桌面覆盖层** — 透明、置顶的 Tauri 窗口，点击穿透桌面
- 🛡️ **安全防护** — Prompt 注入检测 + 命令守卫 + 密钥脱敏

## 快速开始

### 环境要求

- Rust 1.85+
- Node.js 20+
- macOS 14+（Windows/Linux 可运行但部分语音功能受限）

### 安装

```bash
git clone https://github.com/fallfield/FairyField.git
cd FairyField/FairyField
npm install
```

### 配置 API Key

```bash
export OPENAI_API_KEY="sk-your-key-here"
# 或
export ANTHROPIC_API_KEY="sk-ant-your-key-here"
```

### 下载语音模型（可选）

```bash
bash scripts/download-models.sh   # ~550MB
```

语音模型是可选的 — 不下载时使用 macOS 系统 TTS，ASR/VAD 使用模拟引擎。

### 启动

```bash
npm run tauri dev
```

启用真实语音引擎：
```bash
cd src-tauri && cargo build --features sherpa-onnx && cd ..
```

## 使用方式

| 操作 | 方式 |
|------|------|
| 文字聊天 | 底部输入框输入，Enter 发送 |
| 语音输入 | 点击麦克风按钮说话 |
| 查看历史 | 点击聊天气泡区域展开 |
| 切换 LLM | `Ctrl+Shift+D` → 控制面板 |
| 移动窗口 | 拖拽角色区域 |

## 架构

```
┌─────────────────────────────────────────┐
│              Tauri 透明窗口               │
│  ┌─────────────┐  ┌───────────────────┐  │
│  │ 3D 角色      │  │ 聊天面板           │  │
│  │ Three.js     │  │ Vue 3 + 气泡 UI   │  │
│  │ + VRM 模型   │  │ + 语音输入按钮     │  │
│  └─────────────┘  └───────────────────┘  │
├─────────────────────────────────────────┤
│            Tauri IPC 通信层              │
├─────────────────────────────────────────┤
│  Rust 后端                              │
│  ├── agent/     AI 灵魂 + Agent Loop     │
│  ├── voice/     语音管道 (ASR/TTS/VAD)   │
│  ├── memory/    4 层记忆 + 知识图谱      │
│  ├── tools/     工具系统 (自注册)        │
│  ├── security/  注入防护 + 命令守卫      │
│  ├── gateway/   Discord Webhook 通知     │
│  └── growth/    自主成长引擎             │
└─────────────────────────────────────────┘
```

## 项目状态

| Phase | 内容 | 状态 |
|-------|------|------|
| Phase 0 | 架构重建 | ✅ |
| Phase 1 | 核心交互 | ✅ |
| Phase 2 | 语音管道 + 灵魂骨架 | ✅ |
| Phase 3 | 智能系统 | ✅ |
| Phase 4 | 成长 + 通信 | ✅ |
| Phase 4.5 | 收尾完善 | ✅ |
| Phase 5 | MVP 发布 | 🔄 |

**总测试数**: 248 Rust + 93 Frontend = 341 通过

## 技术栈

| 组件 | 方案 |
|------|------|
| 3D 渲染 | Three.js + @pixiv/three-vrm |
| 桌面容器 | Tauri v2 (Rust) |
| AI 对话 | OpenAI / Claude API + Function Calling |
| 语音识别 | sherpa-onnx Paraformer（离线） |
| 语音合成 | sherpa-onnx Kokoro（离线） |
| 语音检测 | sherpa-onnx silero-vad（离线） |
| 长期记忆 | SQLite + FTS5 + sqlite-vec |
| 知识图谱 | 时序三元组 (valid_from/to) |
| 安全防护 | Prompt 注入检测 + 命令守卫 |

## 文档

- [用户指南](docs/USAGE_GUIDE.md)
- [Discord 配置](docs/DISCORD_SETUP.md)
- [开发约定](CLAUDE.md)
- [技术方案](PROPOSAL.md)
- [贡献指南](CONTRIBUTING.md)

## License

MIT © 2024-2026 FairyField Contributors
