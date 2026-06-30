# 快速上手指南

## 步骤 1: 获取 QQ 机器人凭证

1. 访问 [QQ 开放平台](https://q.qq.com/)
2. 登录并创建新机器人
3. 记录下 **App ID** 和 **Client Secret**

## 步骤 2: 配置

编辑 `config.toml`，填入你的凭证：

```toml
[bot]
app_id = "102xxxxx"           # 替换为你的 App ID
client_secret = "xxxxxx"      # 替换为你的 Client Secret

[api]
base_url = "https://api.sgroup.qq.com"
auth_url = "https://bots.qq.com/app/getAppAccessToken"

[rate_limit]
min_interval_secs = 1
```

## 步骤 3: 测试连接

```bash
./qq_warning test
```

如果看到 `✓ 连接成功`，说明配置正确。

## 步骤 4: 发送消息

### 发送私聊消息

需要先获取用户的 OpenID（可以通过机器人接收到的消息查看）：

```bash
./qq_warning send-user "用户OpenID" "你好！"
```

### 发送群聊消息

需要先获取群的 OpenID：

```bash
./qq_warning send-group "群OpenID" "大家好！"
```

## 步骤 5: 启动后台服务（可选）

如果需要接收消息，启动 daemon 服务：

```bash
./qq_warning daemon
```

服务会保持运行并显示接收到的所有消息。按 Ctrl+C 停止服务。

## 如何获取 OpenID？

### 方法 1: 通过 daemon 服务

1. 启动 `./qq_warning daemon`
2. 让用户给机器人发消息或在群里 @ 机器人
3. 控制台会显示消息信息，包括发送者的 OpenID

示例输出：
```
收到消息:
  消息 ID: 08c8e5...
  发送者: 张三 (11D0B9E0-0CF2-78DC-...)
  内容: 你好
```

其中 `11D0B9E0-0CF2-78DC-...` 就是用户的 OpenID。

### 方法 2: 通过 QQ 开放平台控制台

在开放平台的机器人管理页面可以查看机器人加入的群和用户信息。

## 常见问题

### Q: 认证失败怎么办？

A: 检查以下几点：
1. App ID 和 Client Secret 是否正确
2. 机器人是否已发布（未发布的机器人只能在沙箱环境使用）
3. 网络连接是否正常

### Q: 发送消息失败？

A: 可能的原因：
1. OpenID 不正确
2. 机器人未加入目标群聊
3. 触发了速率限制

### Q: daemon 无法连接？

A: 检查：
1. 认证是否成功（先运行 `qq_warning test`）
2. 防火墙是否阻止 WebSocket 连接
3. 查看错误日志

## 后台运行 daemon（Linux/macOS）

使用 systemd 或 screen：

### 使用 screen

```bash
# 创建新的 screen 会话
screen -S qqbot

# 在 screen 中运行
./qq_warning daemon

# 按 Ctrl+A, D 分离会话
# 重新连接：screen -r qqbot
```

### 使用 systemd

创建 `/etc/systemd/system/qq_warning.service`：

```ini
[Unit]
Description=QQ Warning Bot
After=network.target

[Service]
Type=simple
User=yourusername
WorkingDirectory=/path/to/qq_warning
ExecStart=/path/to/qq_warning/qq_warning daemon
Restart=always

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable qq_warning
sudo systemctl start qq_warning
sudo systemctl status qq_warning
```

## 下一步

- 查看 [README.md](README.md) 了解完整功能
- 阅读 [PROJECT.md](PROJECT.md) 了解技术细节
- 访问 [QQ 机器人开发文档](https://bot.q.qq.com/wiki/) 学习更多 API
