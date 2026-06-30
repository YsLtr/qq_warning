# QQ Bot CLI 工具

一个功能完整的 QQ 机器人命令行工具，基于 QQ 官方 Bot API，使用 Rust 开发。

## 功能特性

### 消息发送
- ✅ 发送文本消息（C2C 私聊 / 群聊）
- ✅ 发送 Markdown 格式消息
- ✅ 发送图片消息
- ✅ 发送带按钮的交互消息（键盘组件）
- ✅ 发送 Ark 模板消息
- ✅ 消息撤回（支持隐藏撤回提示）
- ✅ "正在输入"提示

### 频道管理
- ✅ 禁言用户（指定时长）
- ✅ 全员禁言
- ✅ 添加/删除精华消息
- ✅ 查看精华消息列表
- ✅ 创建/删除公告
- ✅ 添加/删除表情反应

### 成员与权限
- ✅ 获取频道成员信息
- ✅ 获取频道身份组列表
- ✅ 添加频道身份组成员

### 日程管理
- ✅ 获取日程列表
- ✅ 创建日程
- ✅ 删除日程

### 实时通信
- ✅ WebSocket 连接（Daemon 模式）
- ✅ 接收实时消息推送
- ✅ 自动心跳保活
- ✅ 断线重连
- ✅ 自动下载媒体文件
- ✅ 桌面通知（支持 Linux/macOS）

## 安装

### 从源码编译

```bash
# 克隆项目
git clone <your-repo>
cd qq_warning

# 编译发布版本
cargo build --release

# 二进制文件位于
./target/release/qq_warning
```

### 配置文件

复制 `config.toml` 并填写你的 Bot 凭证：

```toml
[bot]
app_id = "your_app_id_here"
client_secret = "your_client_secret_here"

[api]
base_url = "https://api.sgroup.qq.com"
auth_url = "https://bots.qq.com/app/getAppAccessToken"

[rate_limit]
min_interval_secs = 1

[notifications]
enabled = true
sound = true
show_preview = true

[logging]
level = "info"
file = "qq_warning.log"

[features]
auto_download_media = true
media_dir = "./media"
```

## 使用方法

### 基础命令

#### 测试连接
```bash
qq_warning test
```

#### 发送消息

**发送文本消息到用户**
```bash
qq_warning send-user <USER_OPENID> "Hello, World!"
```

**发送 Markdown 消息**
```bash
qq_warning send-user <USER_OPENID> "# 标题\n- 列表项" --markdown
```

**发送图片**
```bash
qq_warning send-user <USER_OPENID> "看这张图" --image "https://example.com/image.jpg"
```

**发送群消息**
```bash
qq_warning send-group <GROUP_OPENID> "大家好！"
```

#### 撤回消息

**撤回用户消息**
```bash
qq_warning recall user <USER_OPENID> <MESSAGE_ID>
```

**撤回群消息（隐藏提示）**
```bash
qq_warning recall group <GROUP_OPENID> <MESSAGE_ID> --hidetip
```

### 频道管理

#### 禁言操作

**禁言单个用户（60秒）**
```bash
qq_warning guild mute <GUILD_ID> <USER_ID> --seconds 60
```

**全员禁言（3600秒 = 1小时）**
```bash
qq_warning guild mute-all <GUILD_ID> --seconds 3600
```

#### 精华消息

**添加精华消息**
```bash
qq_warning guild pin-add <CHANNEL_ID> <MESSAGE_ID>
```

**删除精华消息**
```bash
qq_warning guild pin-delete <CHANNEL_ID> <MESSAGE_ID>
```

**查看精华消息列表**
```bash
qq_warning guild pin-list <CHANNEL_ID>
```

#### 公告管理

**创建公告**
```bash
qq_warning guild announce-create <GUILD_ID> <CHANNEL_ID> <MESSAGE_ID>
```

**删除公告**
```bash
qq_warning guild announce-delete <GUILD_ID> <MESSAGE_ID>
```

#### 表情反应

**添加表情反应**
```bash
# emoji_type: 1=系统表情 2=emoji表情
qq_warning guild reaction-add <CHANNEL_ID> <MESSAGE_ID> 1 "123"
```

### 后台服务模式

启动 WebSocket 服务，保持在线并接收实时消息：

```bash
qq_warning daemon
```

在 Daemon 模式下：
- 自动接收所有消息推送（私聊、群聊、频道）
- 自动下载图片和文件（可配置）
- 发送桌面通知（可配置）
- 自动重连
- 日志输出到控制台和文件

## 脚本集成示例

### Bash 脚本

```bash
#!/bin/bash

# 监控脚本，出现错误时发送通知
if ! systemctl is-active --quiet myservice; then
    qq_warning send-user "USER_OPENID" "⚠️ 服务 myservice 已停止！"
fi
```

### Cron 定时任务

```cron
# 每天早上 8 点发送问候
0 8 * * * /usr/local/bin/qq_warning send-group "GROUP_OPENID" "早上好！新的一天开始了 ☀️"

# 每小时检查系统状态
0 * * * * /home/user/scripts/check_system.sh
```

### Python 集成

```python
import subprocess

def send_qq_message(user_id: str, message: str):
    subprocess.run([
        "qq_warning",
        "send-user",
        user_id,
        message
    ])

# 使用
send_qq_message("USER_OPENID", "Python 脚本执行完成！")
```

## 高级功能

### 自定义日志级别

```bash
qq_warning --config custom_config.toml test
```

修改配置文件中的日志级别：
```toml
[logging]
level = "debug"  # trace, debug, info, warn, error
```

### 速率限制

工具内置速率限制，防止触发 API 频率限制：

```toml
[rate_limit]
min_interval_secs = 1  # 每条消息之间最少间隔 1 秒
```

## API 覆盖

本工具实现了以下 QQ Bot API：

### 消息 API
- `POST /v2/users/{user_openid}/messages` - 发送私聊消息
- `POST /v2/groups/{group_openid}/messages` - 发送群消息
- `DELETE /v2/users/{user_openid}/messages/{message_id}` - 撤回私聊消息
- `DELETE /v2/groups/{group_openid}/messages/{message_id}` - 撤回群消息

### 频道管理 API
- `PATCH /guilds/{guild_id}/members/{user_id}/mute` - 禁言成员
- `PATCH /guilds/{guild_id}/mute` - 全员禁言
- `GET /guilds/{guild_id}/members/{user_id}` - 获取成员信息
- `GET /guilds/{guild_id}/roles` - 获取身份组列表
- `PUT /guilds/{guild_id}/members/{user_id}/roles/{role_id}` - 添加身份组

### 消息管理 API
- `PUT /channels/{channel_id}/pins/{message_id}` - 添加精华消息
- `DELETE /channels/{channel_id}/pins/{message_id}` - 删除精华消息
- `GET /channels/{channel_id}/pins` - 获取精华消息
- `POST /guilds/{guild_id}/announces` - 创建公告
- `DELETE /guilds/{guild_id}/announces/{message_id}` - 删除公告

### 表情反应 API
- `PUT /channels/{channel_id}/messages/{message_id}/reactions/{type}/{id}` - 添加表情
- `DELETE /channels/{channel_id}/messages/{message_id}/reactions/{type}/{id}` - 删除表情

### 日程 API
- `GET /channels/{channel_id}/schedules` - 获取日程列表
- `POST /channels/{channel_id}/schedules` - 创建日程
- `DELETE /channels/{channel_id}/schedules/{schedule_id}` - 删除日程

### WebSocket API
- Gateway 连接
- 心跳保活
- 事件订阅（消息、成员变更等）

## 项目结构

```
src/
├── main.rs         # 命令行入口
├── api.rs          # API 底层封装
├── qqbot.rs        # Bot 高级封装
├── types.rs        # 类型定义
├── websocket.rs    # WebSocket 服务
├── config.rs       # 配置管理
└── utils.rs        # 工具函数
```

## 技术栈

- **语言**: Rust 2021
- **HTTP 客户端**: reqwest
- **WebSocket**: tokio-tungstenite
- **异步运行时**: tokio
- **命令行**: clap
- **序列化**: serde + serde_json
- **日志**: tracing + tracing-subscriber
- **通知**: notify-rust

## 参考资料

- [QQ 官方 Bot API 文档](https://bot.q.qq.com/wiki/)
- [zhinjs/qq-official-bot](https://github.com/zhinjs/qq-official-bot)
- [zimoyin/qqbot-sdk](https://github.com/zimoyin/qqbot-sdk)

## 许可证

MIT License
