# QQ Warning - QQ Bot 消息发送工具

一个简单的命令行工具，用于通过 QQ 机器人官方 API 发送消息和接收消息。

## 功能特性

- ✅ 支持发送私聊消息 (C2C)
- ✅ 支持发送群聊消息
- ✅ WebSocket 后台服务（保持长连接，实时接收消息）
- ✅ 自动管理 Access Token（缓存和刷新）
- ✅ 内置速率限制保护
- ✅ 自动重连机制
- ✅ 简洁的配置文件

## 快速开始

### 1. 获取机器人凭证

访问 [QQ 开放平台](https://q.qq.com/) 创建机器人，获取：
- `App ID`
- `Client Secret`

### 2. 配置

编辑 `config.toml` 文件，填入你的机器人凭证：

```toml
[bot]
app_id = "your_app_id_here"
client_secret = "your_client_secret_here"

[api]
base_url = "https://api.sgroup.qq.com"
auth_url = "https://bots.qq.com/app/getAppAccessToken"

[rate_limit]
min_interval_secs = 1
```

### 3. 使用

#### 测试连接

```bash
qq_warning test
```

#### 发送私聊消息

```bash
qq_warning send-user <user_openid> "你好，这是一条测试消息"
```

#### 发送群聊消息

```bash
qq_warning send-group <group_openid> "群公告：系统维护通知"
```

#### 启动 WebSocket 后台服务

启动后台服务，保持与 QQ 服务器的 WebSocket 长连接，实时接收消息：

```bash
qq_warning daemon
```

服务会自动：
- 连接到 QQ WebSocket Gateway
- 发送心跳保持连接
- 接收并显示所有消息（私聊、群聊、频道）
- 断线后自动重连

#### 使用自定义配置文件

```bash
qq_warning -c /path/to/config.toml test
```

## 命令参考

```
qq_warning [OPTIONS] <COMMAND>

Commands:
  send-user   发送消息到私聊
  send-group  发送消息到群聊
  test        测试连接
  daemon      启动 WebSocket 服务（保持连接，接收消息）

Options:
  -c, --config <CONFIG>  配置文件路径 [default: config.toml]
  -h, --help            显示帮助信息
```

## 编译

### 本地编译

```bash
cargo build --release
```

生成的二进制文件位于 `target/release/qq_warning`

### 交叉编译

项目配置了 GitHub Actions，推送到仓库后自动构建以下平台的二进制文件：
- Linux x86_64
- Linux ARM64
- macOS x86_64
- macOS ARM64 (Apple Silicon)
- Windows x86_64

## 技术栈

- Rust 2021
- Tokio (异步运行时)
- Reqwest (HTTP 客户端)
- Tokio-Tungstenite (WebSocket 客户端)
- Clap (命令行参数解析)
- Serde (序列化/反序列化)

## WebSocket 后台服务

`daemon` 命令启动一个后台服务，通过 WebSocket 与 QQ 服务器保持长连接：

### 工作流程

1. 获取 Gateway URL
2. 建立 WebSocket 连接
3. 接收 Hello 消息，获取心跳间隔
4. 发送 Identify 认证
5. 启动心跳任务（每隔心跳间隔的 80% 发送一次）
6. 接收并处理各类事件

### 支持的事件

- `READY` - 机器人就绪
- `MESSAGE_CREATE` - 频道消息
- `C2C_MESSAGE_CREATE` - 私聊消息
- `GROUP_AT_MESSAGE_CREATE` - 群聊 @ 消息

### 重连机制

- 连接断开后自动等待 5 秒重连
- 会话失效时清除状态重新认证
- 收到服务器重连请求时立即重连

## 速率限制

工具内置了速率限制保护，默认每条消息间隔 1 秒。可以在配置文件中调整 `min_interval_secs` 参数。

## 注意事项

1. 配置文件中的凭证请妥善保管，不要泄露
2. 遵守 QQ 开放平台的速率限制规则
3. 消息内容不超过 4000 字符
4. 需要先将机器人添加到群聊才能发送群消息

## 参考资料

- [QQ 开放平台文档](https://bot.q.qq.com/wiki/)
- [QQ 机器人开发指南](https://bot.q.qq.com/wiki/develop/api/)

## License

MIT
