# FairyField 用户指南

> FairyField 是一个有灵魂的桌面 AI 伴侣 — 她悬浮在你的桌面上，记得你的烦恼，会用真实的声音跟你聊天。

## 快速开始

### 环境要求

- **Rust** 1.85+
- **Node.js** 20+
- **macOS** 14+（Windows/Linux 实验性支持）
- ~2GB 磁盘空间（含模型）

### 安装

```bash
# 克隆项目
git clone https://github.com/fallfield/FairyField.git
cd FairyField/FairyField

# 安装前端依赖
npm install

# 下载语音模型（可选；默认安装低延迟 Matcha 双语模型）
bash scripts/download-models.sh

# 配置 LLM API
export OPENAI_API_KEY="sk-your-key-here"
# 或
export ANTHROPIC_API_KEY="sk-ant-your-key-here"
```

### 启动

```bash
# 开发模式
npm run tauri dev

# 生产构建
npm run tauri build
```

## 配置

配置文件位于 `~/.fairyfield/config.json`（首次启动自动生成）。

### LLM 提供商

支持多个 LLM 提供商，可在左上角折叠的 ControlPanel 中展开切换：

```json
{
  "llm": {
    "active_provider": "OpenAI GPT-4o",
    "providers": [
      {
        "name": "OpenAI GPT-4o",
        "provider_type": "openai",
        "api_endpoint": "https://api.openai.com/v1",
        "model": "gpt-4o"
      }
    ]
  }
}
```

API Key 也可以通过环境变量设置：
- `OPENAI_API_KEY` — OpenAI
- `ANTHROPIC_API_KEY` — Anthropic Claude
- `FAIRYFIELD_API_KEY` — 全局 API Key（覆盖所有提供商）

### 语音引擎

FairyField 支持离线语音处理：

| 引擎 | 类型 | 状态 | 需要模型 |
|------|------|------|----------|
| Matcha bilingual | TTS（文字→语音） | ✅ 默认 | 中文 `matcha-icefall-zh-baker` + 英文 `matcha-icefall-en_US-ljspeech` + `vocos-22khz-univ.onnx` |
| Kokoro | TTS（文字→语音） | ✅ 兜底 | `model.onnx` + `voices.bin` + lexicon/rule FST |
| Paraformer | ASR（语音→文字） | ✅ 支持 | `sherpa-onnx-paraformer-zh` 目录 |
| Silero VAD | 语音活动检测 | ✅ 支持 | `silero-vad.onnx` |
| MacSayTts | TTS（仅 macOS） | ✅ 内置 | 无需模型 |

启用真实语音引擎：
```bash
cd src-tauri
cargo build --features sherpa-onnx
```

不启用 `sherpa-onnx` feature 时，TTS 回退到 macOS `say` 命令，ASR/VAD 使用 Mock 引擎。启用后默认优先 Matcha 双语低延迟 TTS；缺少 Matcha 模型时回退到 Kokoro，再回退到平台默认。

更多本地语音方案、延迟原因和模型选择见 `docs/LOCAL_TTS.md`。

### 角色模型

将 VRM 格式的 3D 模型放到 `public/models/default/`：
```
public/models/default/
├── 2031903848872972007.glb  # 默认模型文件（可替换为 .vrm 或 .glb）
└── textures/            # 纹理（可选）
```

推荐从 [VRoid Hub](https://hub.vroid.com/) 下载免费模型测试。

## 使用技巧

### 聊天

- 在底部输入框输入文字，按 `Enter` 发送
- `Shift+Enter` 换行
- 点击气泡区域展开历史记录
- 点击麦克风按钮使用语音输入

### 开发者模式

- `Ctrl+Shift+D` — 切换开发者面板
- 可查看记忆库、工具调用日志和 Agent 状态
- 三次点击角色头像也可切换

### 多 Agent 开发闭环

FairyField 提供 ManagerAgent、CodingAgent、TestingAgent、GoalAgent 四段式开发闭环。它会先规划任务和文件范围，再通过已配置的 coding CLI 执行真实改动，随后运行 allowlist 测试命令，最后由 GoalAgent 对照需求和证据判断是否完成。详情见 `docs/AGENT_LOOP.md`。

### 情绪系统

Fairy 会根据对话内容自动调整情绪：
- 😊 快乐 — 活泼的语气，黄色气泡
- 😢 悲伤 — 温柔的语气，蓝色气泡
- 😠 生气 — 红色气泡
- 😲 惊讶 — 紫色气泡
- 😐 平静 — 默认状态

情绪会在 2 分钟内自然衰减回平静。

## 架构概览

```
桌面窗口（Tauri 透明窗口）
├── 3D 角色（Three.js + VRM）     ← 会笑会动的动漫角色
├── 聊天面板（Vue 3）              ← 对话气泡 + 语音输入
└── 开发者面板（可隐藏）            ← 调试控制

Tauri IPC 通信层
├── AI 灵魂（Agent Loop）          ← LLM 对话 + 工具调用
├── 语音管道（sherpa-onnx）       ← ASR + TTS + VAD
├── 记忆系统（SQLite + FTS5）     ← 4 层记忆 + 知识图谱
├── 安全体系                        ← 注入防护 + 命令守卫
└── 通信网关（Discord Webhook）    ← 手机伴侣通知
```

## 故障排除

### 窗口是黑色的
确认 VRM 模型文件在 `public/models/default/` 目录下且格式正确。

### 麦克风不工作
1. 确认 macOS 系统设置中已授权终端/IDE 麦克风权限
2. 检查是否安装了语音模型：`ls ~/.fairyfield/models/`
3. 未安装模型时 ASR 会返回固定文本，不影响文本聊天

### LLM 无响应
1. 检查 API Key 环境变量是否设置：`echo $OPENAI_API_KEY`
2. 确认网络可访问 API 端点
3. 查看终端日志（启动 Tauri dev 时可见）

### Web 搜索或网页抓取无结果
1. 天气问题建议直接写城市 + 天气，例如 `澳门天气`
2. `web_fetch` 只允许公开 `http://`/`https://` 页面；localhost、内网 IP 和 `.local` 域名会被拦截
3. 大网页会自动截断，避免工具调用长时间占用 Agent Loop
4. 如果网络源不可达，Fairy 会返回明确的失败原因和备用搜索链接

### 构建失败
```bash
# 清理构建缓存
cd src-tauri && cargo clean
cd .. && rm -rf node_modules dist
npm install
npm run tauri build
```

## 项目状态

- Phase 0-4：✅ 已完成
- Phase 4.5：✅ 已完成
- Phase 5：✅ 已完成
- Phase 6：✅ 已完成（含 v1 Web/工具/记忆硬化）
- 全息模式：⏳ 延后

## 获取帮助

- GitHub Issues: [github.com/fallfield/FairyField/issues](https://github.com/fallfield/FairyField/issues)
- 查看 `CLAUDE.md` 了解开发约定
