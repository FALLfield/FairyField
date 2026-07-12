# Discord 网关配置指南

FairyField 支持通过 Discord 网关实现手机伴侣通信。本指南说明如何配置。

## 当前状态

Webhook HTTP client code exists, but the running app initializes GatewayState with no DiscordGateway and never hydrates it from AppConfig. The documented webhook_url field is not part of the current GatewayConfig. Therefore chat-to-Discord notification is not currently configurable through this guide. Full Bot mode is also not implemented.

## Webhook 模式（目标配置，尚未接入启动流程）

### 1. 创建 Discord Webhook

1. 在你的 Discord 服务器中，进入**服务器设置 → 集成 → Webhooks**
2. 点击**新建 Webhook**
3. 命名为 `FairyField`
4. 选择发送到的频道（建议创建专用 `#fairy` 频道）
5. 点击 **复制 Webhook URL**

### 2. 配置 FairyField

The following is the target configuration shape. Adding it to `~/.fairyfield/config.json` does not work until GatewayConfig and startup wiring are implemented:

```json
{
  "gateway": {
    "webhook_url": "https://discord.com/api/webhooks/xxxxx/yyyyy",
    "cron_enabled": true
  }
}
```

### 3. 测试

Do not expect this test to pass yet. First wire GatewayState from configuration, add a safe connection test, and expose an enable/disable control.

## Bot 模式（未来版本，需 Bot Token）

Bot 模式支持双向通信 — 你可以从手机 Discord 客户端给 Fairy 发消息。

### 准备工作（等 Bot Token 就绪后）

1. 访问 [Discord Developer Portal](https://discord.com/developers/applications)
2. 创建新应用，命名为 `FairyField`
3. 进入 **Bot** 页面，点击 **Add Bot**
4. 复制 **Bot Token**
5. 在 **OAuth2 → URL Generator** 中：
   - 选择 `bot` scope
   - 选择权限：`Send Messages`, `Read Message History`, `Use Slash Commands`
   - 用生成的 URL 邀请 Bot 到服务器
6. 配置 FairyField：

```json
{
  "gateway": {
    "discord_token": "YOUR_BOT_TOKEN_HERE",
    "discord_channel_id": "YOUR_CHANNEL_ID"
  }
}
```

### 源码位置

- Webhook 实现: `src-tauri/src/gateway/discord.rs`
- 定时任务: `src-tauri/src/gateway/cron.rs`
- IPC 命令: `src-tauri/src/gateway/commands.rs`

## 常见问题

**Q: 消息发不出去？**
A: 当前首先会遇到 GatewayState 未初始化的问题。完成启动接线后，再检查 Webhook URL 和频道状态。

**Q: 手机现在能收到通知吗？**
A: 不能通过当前 FairyField 启动流程保证。Webhook client 单元代码存在，但运行时网关没有配置入口。

**Q: 如何让 Fairy 主动发消息？**
A: 目前只存在内存 Cron 数据结构和 IPC CRUD；没有持久调度循环连接到 Discord 发送。该功能仍待实现。
