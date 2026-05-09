# Discord 网关配置指南

FairyField 支持通过 Discord 网关实现手机伴侣通信。本指南说明如何配置。

## 当前状态

目前 Discord 网关使用 **Webhook** 模式（已实现并测试通过）。完整的 Discord Bot 模式（serenity）需要 Bot Token，待后续版本实现。

## Webhook 模式（当前可用）

### 1. 创建 Discord Webhook

1. 在你的 Discord 服务器中，进入**服务器设置 → 集成 → Webhooks**
2. 点击**新建 Webhook**
3. 命名为 `FairyField`
4. 选择发送到的频道（建议创建专用 `#fairy` 频道）
5. 点击 **复制 Webhook URL**

### 2. 配置 FairyField

将 Webhook URL 添加到 `~/.fairyfield/config.json`：

```json
{
  "gateway": {
    "webhook_url": "https://discord.com/api/webhooks/xxxxx/yyyyy",
    "cron_enabled": true
  }
}
```

### 3. 测试

重启 FairyField，发送消息时观察 Discord 频道是否收到通知。

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
A: 检查 Webhook URL 是否正确，频道是否仍然存在。

**Q: 手机能收到通知吗？**
A: 是的，只要 Discord 手机版有通知权限，通过 Webhook 发送的消息会推送。

**Q: 如何让 Fairy 主动发消息？**
A: 在 `config.json` 中启用 `cron_enabled: true`，Fairy 会定时发送问候/提醒。
