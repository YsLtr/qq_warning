# QQ Warning - QQ Bot 完整命令行工具

一个功能完整的 QQ 机器人命令行工具，基于 QQ 官方 Bot API，使用 Rust 开发，方便接入脚本操作。

## 功能特性

### 消息功能
- ✅ 支持发送私聊消息 (C2C)
- ✅ 支持发送群聊消息
- ✅ 支持 Markdown 格式消息
- ✅ 支持发送图片
- ✅ 支持消息撤回（可隐藏撤回提示）
- ✅ 支持带按钮的交互消息
- ✅ 支持 Ark 模板消息

### 频道管理
- ✅ 禁言用户（指定时长）
- ✅ 全员禁言
- ✅ 精华消息管理（添加/删除/查看）
- ✅ 公告管理（创建/删除）
- ✅ 表情反应（添加/删除）

### 成员与权限
- ✅ 获取频道成员信息
- ✅ 获取频道身份组列表
- ✅ 添加频道身份组成员

### 实时通信
- ✅ WebSocket 后台服务（保持长连接，实时接收消息）
- ✅ 自动管理 Access Token（缓存和刷新）
- ✅ 内置速率限制保护
- ✅ 自动重连机制
- ✅ 自动下载媒体文件
- ✅ 桌面通知（Linux/macOS）

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

#### 发送消息

**发送私聊消息**
```bash
qq_warning send-user <user_openid> "你好，这是一条测试消息"
```

**发送 Markdown 消息**
```bash
qq_warning send-user <user_openid> "# 标题\n**粗体文字**" --markdown
```

**发送图片**
```bash
qq_warning send-user <user_openid> "看这张图" --image "https://example.com/image.jpg"
```

**发送群聊消息**
```bash
qq_warning send-group <group_openid> "群公告：系统维护通知"
```

#### 撤回消息

**撤回私聊消息**
```bash
qq_warning recall user <user_openid> <message_id>
```

**撤回群消息（隐藏撤回提示）**
```bash
qq_warning recall group <group_openid> <message_id> --hidetip
```

#### 频道管理

**禁言用户 60 秒**
```bash
qq_warning guild mute <guild_id> <user_id> --seconds 60
```

**全员禁言 1 小时**
```bash
qq_warning guild mute-all <guild_id> --seconds 3600
```

**添加精华消息**
```bash
qq_warning guild pin-add <channel_id> <message_id>
```

**查看精华消息列表**
```bash
qq_warning guild pin-list <channel_id>
```

**创建公告**
```bash
qq_warning guild announce-create <guild_id> <channel_id> <message_id>
```

**添加表情反应**
```bash
qq_warning guild reaction-add <channel_id> <message_id> 1 "123"
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
- 自动下载图片和文件
- 发送桌面通知
- 断线后自动重连

#### 使用自定义配置文件

```bash
qq_warning -c /path/to/config.toml test
```

## 命令参考

```
qq_warning [OPTIONS] <COMMAND>

Commands:
  send-user   发送消息到私聊（支持文本、Markdown、图片）
  send-group  发送消息到群聊
  recall      撤回消息
  guild       频道管理
    mute            禁言用户
    mute-all        全员禁言
    pin-add         添加精华消息
    pin-delete      删除精华消息
    pin-list        查看精华消息列表
    announce-create 创建公告
    announce-delete 删除公告
    reaction-add    添加表情反应
  test        测试连接
  daemon      启动 WebSocket 服务（保持连接，接收消息）

Options:
  -c, --config <CONFIG>  配置文件路径 [default: config.toml]
  -h, --help            显示帮助信息
```

## 脚本集成示例

### Bash 脚本监控

```bash
#!/bin/bash
# 监控服务状态，异常时发送 QQ 通知

SERVICE="nginx"
USER_ID="your_user_openid"

if ! systemctl is-active --quiet $SERVICE; then
    qq_warning send-user "$USER_ID" "⚠️ 警告：服务 $SERVICE 已停止！"
fi
```

### Python 集成

```python
import subprocess
import json

def send_qq_notification(user_id: str, message: str, markdown: bool = False):
    """发送 QQ 通知"""
    cmd = ["qq_warning", "send-user", user_id, message]
    if markdown:
        cmd.append("--markdown")
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    return result.returncode == 0

# 使用示例
send_qq_notification(
    "user_openid",
    "# 任务完成\n- 耗时：2.5秒\n- 状态：✅ 成功",
    markdown=True
)
```

### Cron 定时任务

```cron
# 每天早上 8 点发送问候
0 8 * * * /usr/local/bin/qq_warning send-group "group_openid" "早上好！☀️"

# 每小时检查磁盘空间
0 * * * * /home/user/scripts/disk_check.sh
```

### Systemd 服务

创建 `/etc/systemd/system/qqbot.service`：

```ini
[Unit]
Description=QQ Bot Daemon
After=network.target

[Service]
Type=simple
User=your_user
WorkingDirectory=/home/your_user/qq_warning
ExecStart=/usr/local/bin/qq_warning daemon
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

启动服务：
```bash
sudo systemctl enable qqbot
sudo systemctl start qqbot
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
