# QQ Bot 命令速查表

## 快速启动

```bash
# 1. 编辑配置文件，填入 AppID 和 ClientSecret
vim config.toml

# 2. 测试连接
qq_warning test

# 3. 发送第一条消息
qq_warning send-user <USER_OPENID> "Hello!"
```

## 常用命令

### 消息发送

```bash
# 私聊文本
qq_warning send-user <USER_ID> "消息内容"

# 私聊 Markdown
qq_warning send-user <USER_ID> "# 标题" --markdown

# 私聊图片
qq_warning send-user <USER_ID> "图片" --image "URL"

# 群消息
qq_warning send-group <GROUP_ID> "消息内容"
```

### 消息管理

```bash
# 撤回私聊消息
qq_warning recall user <USER_ID> <MESSAGE_ID>

# 撤回群消息（隐藏提示）
qq_warning recall group <GROUP_ID> <MESSAGE_ID> --hidetip
```

### 频道管理

```bash
# 禁言用户 60 秒
qq_warning guild mute <GUILD_ID> <USER_ID> -s 60

# 全员禁言 1 小时
qq_warning guild mute-all <GUILD_ID> -s 3600

# 添加精华
qq_warning guild pin-add <CHANNEL_ID> <MESSAGE_ID>

# 查看精华列表
qq_warning guild pin-list <CHANNEL_ID>

# 删除精华
qq_warning guild pin-delete <CHANNEL_ID> <MESSAGE_ID>

# 创建公告
qq_warning guild announce-create <GUILD_ID> <CHANNEL_ID> <MESSAGE_ID>

# 删除公告
qq_warning guild announce-delete <GUILD_ID> <MESSAGE_ID>

# 添加表情反应（1=系统表情 2=emoji）
qq_warning guild reaction-add <CHANNEL_ID> <MESSAGE_ID> 1 "123"
```

### 实时服务

```bash
# 启动 WebSocket 后台服务
qq_warning daemon

# 使用自定义配置
qq_warning -c /path/to/config.toml daemon
```

## 脚本集成

### Bash 一行命令

```bash
# 发送通知
qq_warning send-user "$USER_ID" "任务完成"

# 条件发送
[ $? -eq 0 ] && qq_warning send-user "$USER_ID" "✅ 成功" || qq_warning send-user "$USER_ID" "❌ 失败"

# 管道输入
echo "报告内容" | xargs -I {} qq_warning send-user "$USER_ID" "{}"
```

### Python 调用

```python
import subprocess

# 发送消息
subprocess.run(["qq_warning", "send-user", user_id, message])

# 发送 Markdown
subprocess.run(["qq_warning", "send-user", user_id, markdown_text, "--markdown"])
```

### Cron 定时

```cron
# 每小时执行
0 * * * * qq_warning send-group "$GROUP_ID" "定时报告"

# 每天 8 点
0 8 * * * qq_warning send-user "$USER_ID" "早安！"
```

## 配置文件模板

```toml
[bot]
app_id = "your_app_id"
client_secret = "your_secret"

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
level = "info"              # trace, debug, info, warn, error
file = "qq_warning.log"     # 留空则仅输出到控制台

[features]
auto_download_media = true
media_dir = "./media"
```

## 获取 OpenID

### 方法 1: 启动 daemon 查看

```bash
qq_warning daemon
# 让用户发消息给机器人，控制台会显示 OpenID
```

### 方法 2: 从日志提取

```bash
# 启动 daemon 并将输出保存到文件
qq_warning daemon 2>&1 | tee bot.log

# 从日志中提取 OpenID
grep "发送者" bot.log
```

## 环境变量

```bash
# 使用环境变量覆盖配置文件路径
export QQ_BOT_CONFIG=/path/to/config.toml
qq_warning test
```

## Systemd 服务（Linux）

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

启动：

```bash
sudo systemctl enable qqbot
sudo systemctl start qqbot
sudo systemctl status qqbot
```

## 日志查看

```bash
# 实时查看日志
tail -f qq_warning.log

# 搜索错误
grep ERROR qq_warning.log

# 查看最近 50 条
tail -n 50 qq_warning.log
```

## 常见问题

### 认证失败

```bash
# 检查配置
cat config.toml

# 测试连接
qq_warning test

# 查看详细日志
RUST_LOG=debug qq_warning test
```

### 发送失败

```bash
# 检查速率限制（默认 1 秒/条）
# 可在 config.toml 中调整 min_interval_secs

# 检查 OpenID 是否正确
qq_warning daemon  # 启动后查看收到的消息中的 ID
```

### 连接问题

```bash
# 检查网络
ping api.sgroup.qq.com

# 检查防火墙
sudo iptables -L

# 使用代理（如需要）
export https_proxy=http://proxy:port
qq_warning test
```

## 帮助命令

```bash
# 查看所有命令
qq_warning --help

# 查看子命令帮助
qq_warning send-user --help
qq_warning guild --help
qq_warning guild mute --help
```

## 快速脚本示例

### 监控脚本

```bash
#!/bin/bash
USER_ID="your_user_openid"

# CPU 使用率
CPU=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}')
if (( $(echo "$CPU > 80" | bc -l) )); then
    qq_warning send-user "$USER_ID" "⚠️ CPU 使用率: $CPU%"
fi

# 磁盘空间
DISK=$(df -h / | awk 'NR==2 {print $5}' | sed 's/%//')
if [ $DISK -gt 80 ]; then
    qq_warning send-user "$USER_ID" "⚠️ 磁盘使用率: $DISK%"
fi
```

### 备份通知

```bash
#!/bin/bash
USER_ID="your_user_openid"

qq_warning send-user "$USER_ID" "🔄 开始备份..."

# 执行备份
tar -czf backup.tar.gz /data

if [ $? -eq 0 ]; then
    SIZE=$(du -h backup.tar.gz | cut -f1)
    qq_warning send-user "$USER_ID" "✅ 备份完成 ($SIZE)"
else
    qq_warning send-user "$USER_ID" "❌ 备份失败"
fi
```

## 更多资源

- 📖 完整文档: `README.md`
- 🚀 快速开始: `QUICKSTART.md`
- 💡 使用示例: `EXAMPLES.md`
- 📋 API 清单: `API_COVERAGE.md`
- 🏢 官方文档: https://bot.q.qq.com/wiki/

---

**提示**: 将此速查表保存为书签，或打印出来放在手边！
