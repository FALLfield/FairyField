# 插件系统

FairyField 支持通过 JSON 插件扩展 AI Agent 的能力。

## 插件格式

每个插件是一个独立的 JSON 文件，放在 `plugins/` 目录下：

```json
{
  "name": "plugin-name",
  "version": "1.0.0",
  "description": "插件描述",
  "tools": [
    {
      "name": "tool_name",
      "description": "工具功能描述",
      "parameters": {
        "type": "object",
        "properties": {
          "param1": {
            "type": "string",
            "description": "参数说明"
          }
        },
        "required": ["param1"]
      },
      "handler": "builtin:command_name"
    }
  ]
}
```

## handler 类型

- `builtin:command_name` — 调用预注册的 Rust 命令
- `shell:command` — 执行白名单内的 shell 命令（受限）

## 加载顺序

插件按文件名字母序加载。同名工具后加载的覆盖先加载的。
