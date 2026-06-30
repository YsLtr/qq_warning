# QQ Warning 项目说明

## 项目概述

这是一个使用 Rust 编写的 QQ 机器人命令行工具，支持发送消息和通过 WebSocket 接收消息。

## 项目结构

```
qq_warning/
├── src/
│   ├── main.rs          # 主程序入口，命令行解析
│   ├── config.rs        # 配置文件加载和验证
│   ├── qqbot.rs         # QQ Bot API 客户端（发送消息、Token 管理）
│   └── websocket.rs     # WebSocket 服务（长连接、接收消息）
├── .github/
│   └── workflows/
│       └── release.yml  # GitHub Actions 构建配置
├── Cargo.toml           # Rust 项目配置
├── config.toml          # 机器人配置模板
├── README.md            # 项目文档
└── LICENSE              # MIT 许可证

## 核心功能

### 1. 消息发送 (qqbot.rs)

- **认证**: 通过 App ID 和 Client Secret 获取 Access Token
- **Token 缓存**: 自动缓存和刷新 Token（提前 60 秒刷新）
- **速率限制**: 内置消息发送速率控制
- **API 支持**:
  - C2C 消息 (私聊): `/v2/users/{user_openid}/messages`
  - 群消息: `/v2/groups/{group_openid}/messages`

### 2. WebSocket 后台服务 (websocket.rs)

- **Gateway 连接**: 自动获取 WebSocket Gateway URL
- **认证流程**: 
  1. 连接 WebSocket
  2. 接收 Hello 消息
  3. 发送 Identify 认证（intents: C2C + Group + Guild）
- **心跳机制**: 按服务器指定间隔的 80% 自动发送心跳
- **事件处理**: 接收和显示各类消息事件
- **自动重连**: 连接断开后 5 秒自动重连

### 3. 命令行界面 (main.rs)

使用 Clap 实现的命令行界面：

```bash
# 测试连接
qq_warning test

# 发送私聊消息
qq_warning send-user <user_openid> "消息内容"

# 发送群聊消息
qq_warning send-group <group_openid> "消息内容"

# 启动后台服务
qq_warning daemon

# 使用自定义配置
qq_warning -c custom_config.toml test
```

## 配置文件

`config.toml` 格式：

```toml
[bot]
app_id = "你的机器人 App ID"
client_secret = "你的机器人 Client Secret"

[api]
base_url = "https://api.sgroup.qq.com"
auth_url = "https://bots.qq.com/app/getAppAccessToken"

[rate_limit]
min_interval_secs = 1
```

## GitHub Actions 构建

`.github/workflows/release.yml` 配置了自动构建流程：

### 触发方式

- 推送 tag (v*)
- 手动触发 (workflow_dispatch)

### 构建目标

- Linux x86_64
- Linux ARM64
- macOS x86_64 
- macOS ARM64 (Apple Silicon)
- Windows x86_64

### 产物格式

- Unix: `.tar.gz` (包含二进制、config.toml、README.md)
- Windows: `.zip` (包含二进制、config.toml、README.md)

### 使用方式

```bash
# 创建 tag 并推送
git tag v0.1.0
git push origin v0.1.0

# GitHub Actions 自动构建并创建 Release
```

## 依赖库

| 库 | 版本 | 用途 |
|---|---|---|
| tokio | 1.x | 异步运行时 |
| reqwest | 0.12 | HTTP 客户端 |
| tokio-tungstenite | 0.24 | WebSocket 客户端 |
| serde/serde_json | 1.0 | 序列化/反序列化 |
| clap | 4.x | 命令行参数解析 |
| anyhow | 1.0 | 错误处理 |
| chrono | 0.4 | 时间处理 |
| toml | 0.8 | TOML 配置解析 |

## QQ Bot API 参考

### 认证

```
POST https://bots.qq.com/app/getAppAccessToken
Content-Type: application/json

{
  "appId": "...",
  "clientSecret": "..."
}
```

### 发送消息

```
POST https://api.sgroup.qq.com/v2/users/{user_openid}/messages
Authorization: QQBot {access_token}
Content-Type: application/json

{
  "msg_type": 0,
  "content": "消息内容",
  "msg_seq": 12345
}
```

### WebSocket Gateway

```
GET https://api.sgroup.qq.com/gateway
Authorization: QQBot {access_token}

Response:
{
  "url": "wss://..."
}
```

### WebSocket Opcodes

- 0: Dispatch (事件)
- 1: Heartbeat (心跳)
- 2: Identify (认证)
- 7: Reconnect (重连)
- 9: Invalid Session (会话无效)
- 10: Hello (握手)
- 11: Heartbeat ACK (心跳确认)

## 下一步开发建议

1. **消息回复功能**: 在 daemon 模式下自动回复消息
2. **插件系统**: 支持加载自定义消息处理插件
3. **日志记录**: 将消息和事件记录到文件
4. **富文本消息**: 支持 Markdown、图片等
5. **数据库集成**: 存储消息历史和用户数据
6. **Web UI**: 提供 Web 界面管理机器人

## 安全注意事项

1. 不要将 `config.toml` 提交到 Git（已在 .gitignore 中）
2. 生产环境使用 `config.local.toml`
3. 妥善保管 App ID 和 Client Secret
4. 遵守 QQ 开放平台的使用规范和速率限制

## 参考资料

- [QQ 开放平台](https://q.qq.com/)
- [QQ 机器人开发文档](https://bot.q.qq.com/wiki/)
- 参考项目:
  - https://github.com/YsLtr/hermes-agent-rs
  - https://github.com/NousResearch/hermes-agent/
