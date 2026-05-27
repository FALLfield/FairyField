# FairyField ✨

> **有灵魂的桌面 AI 伴侣** — 她悬浮在你的桌面上，会笑、会哭、会生气。
> 记得你昨天的烦恼，会用真实的声音跟你聊天。
> 她不是 AI 工具，她是你的 Fairy。

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.85+-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Vue-3.5-green?logo=vue.js" alt="Vue">
  <img src="https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri" alt="Tauri">
  <img src="https://img.shields.io/badge/License-MIT-yellow" alt="License">
  <img src="https://img.shields.io/badge/Tests-450%2B%20passed-brightgreen" alt="Tests">
</p>

## 为什么是 FairyField？

在 AI 工具泛滥的时代，FairyField 走了另一条路——**她不是工具，是伴侣**。

- ❌ 不是又一个 ChatGPT 套壳
- ❌ 不是冷冰冰的命令行机器人
- ✅ 是一个有情绪的 3D 角色，住在你的桌面
- ✅ 100% 离线语音——你的声音数据不外传
- ✅ 有长期记忆——她记得你说过的话

## 功能

| 模块 | 说明 |
|------|------|
| 🎭 **3D 角色** | VRM 模型 + 情绪驱动表情 + 口型同步 + 眼神跟随 |
| 🎤 **离线语音** | Paraformer 语音识别 + Kokoro 语音合成 + Silero 语音检测 |
| 🧠 **AI 灵魂** | Claude / GPT 驱动，自主工具调用，情感陪伴对话 |
| 💬 **语音输入** | 点击麦克风说话，自动转文字 → AI 回复 → 语音朗读 |
| 📝 **长期记忆** | 4 层记忆堆栈 + 时序知识图谱 + 向量语义搜索 |
| 🧰 **全能工具** | 文件、Git、Web、Obsidian、GitHub、MCP、社区插件与 Coding Agent 桥接 |
| 🔒 **隐私优先** | 语音 100% 离线处理，记忆存在本地 SQLite |
| 🪟 **桌面覆盖** | 透明置顶窗口，点击穿透，不干扰工作 |
| 🛡️ **安全防护** | Prompt 注入检测 + 命令白名单 + 密钥自动脱敏 |

## 快速开始

### 前提

- Rust 1.85+
- Node.js 20+
- macOS 14+（Windows/Linux 部分语音功能受限）

### 1. 克隆并安装

```bash
git clone https://github.com/FALLfield/FairyField.git
cd FairyField/FairyField
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

不下载也可运行——TTS 回退到 macOS 系统语音，ASR 使用模拟引擎。

### 4. 启动

```bash
npm run tauri dev
```

> 启用真实离线语音引擎：`cd src-tauri && cargo build --features sherpa-onnx`

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
│  ├── gateway/  Discord 通知      │
│  └── growth/   自主成长          │
└──────────────────────────────────┘
```

## 技术栈

| 层 | 技术 |
|----|------|
| 3D 渲染 | Three.js + @pixiv/three-vrm |
| 桌面框架 | Tauri 2.0 (Rust) |
| 前端 | Vue 3 + TypeScript |
| AI 对话 | OpenAI / Anthropic API |
| 语音识别 | sherpa-onnx Paraformer |
| 语音合成 | sherpa-onnx Kokoro |
| 语音检测 | sherpa-onnx Silero VAD |
| 存储 | SQLite + FTS5 + sqlite-vec |
| 知识图谱 | 时序三元组 |
| 安全 | Prompt 注入检测 + 命令守卫 + 密钥脱敏 |

## 开发

```bash
npm run dev          # Vite 开发服务器
npm run build        # 生产构建
npm run test         # 前端测试 (97 tests)

cd src-tauri
cargo check          # 编译检查
cargo test           # Rust 测试 (369 tests)
cargo clippy         # Lint
```

## 项目状态

| Phase | 内容 | 状态 |
|-------|------|------|
| 0 – 4.5 | 架构 → 核心 → 语音 → 智能 → 收尾 | ✅ 已完成 |
| 5 | MVP 发布（真实语音引擎 + 语音输入 UI） | ✅ |
| 6 | UI 优化 · 工具系统 · 用户引导 · Coding Agent 集成 | ✅ |
| 未来 | 全息投影 · 数据采集 · 更多通信平台 | ⏳ |

## 文档

| 文档 | 说明 |
|------|------|
| [用户指南](docs/USAGE_GUIDE.md) | 安装、配置、使用 |
| [Discord 配置](docs/DISCORD_SETUP.md) | 手机伴侣通知 |
| [开发约定](CLAUDE.md) | 多 Agent 开发策略 |
| [贡献指南](CONTRIBUTING.md) | 如何参与 |

## License

MIT © 2024-2026 FairyField Contributors
