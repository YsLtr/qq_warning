# QQ Bot 使用示例

## 功能说明

### 1. 发送消息并获取消息 ID

所有发送消息的命令都会返回消息 ID，你可以用这个 ID 来撤回消息或获取消息详情。

#### 示例：发送各种类型的消息

```bash
# 发送用户消息
./qq_warning send to <user_openid> "你好！" 

# 发送群消息
./qq_warning send group <group_openid> "大家好！"

# 发送 Markdown 消息
./qq_warning send to <user_openid> "**加粗文本**" --markdown

# 发送带图片的消息
./qq_warning send to <user_openid> "看这张图" --image "https://example.com/image.jpg"

# 发送流式消息（模拟打字效果）
./qq_warning send to <user_openid> "这是一条很长的消息..." --stream

# 自定义流式消息的参数
./qq_warning send to <user_openid> "消息内容" --stream --chunk-size 30 --delay-ms 500
```

**消息 ID 的获取：**
- 消息发送成功后，程序内部会返回消息 ID
- 如需在命令行中使用，可以通过查看日志或修改代码来显式输出

### 2. 获取指定消息的详情

⚠️ **重要限制：** QQ Bot API **不支持**批量获取历史消息列表。你只能：
- 获取**已知 message_id** 的单条频道消息
- 通过 **WebSocket 事件**被动接收新消息

```bash
# 获取指定的频道消息
./qq_warning get-message <channel_id> <message_id>
```

#### 输出示例

```
2024-01-01T12:00:00Z INFO 正在获取频道消息 1234567890abcdef...
2024-01-01T12:00:01Z INFO ✓ 消息详情:
2024-01-01T12:00:01Z INFO   ID: 1234567890abcdef
2024-01-01T12:00:01Z INFO   作者: 张三 (user_openid_123)
2024-01-01T12:00:01Z INFO   内容: 你好！
2024-01-01T12:00:01Z INFO   时间: 2024-01-01T11:59:50Z
2024-01-01T12:00:01Z INFO   附件: 1 个
2024-01-01T12:00:01Z INFO     [1] https://example.com/image.jpg
```

### 3. 撤回消息

使用发送消息时获得的消息 ID 来撤回消息。

```bash
# 撤回用户消息
./qq_warning recall <user_openid> <message_id>

# 撤回群消息
./qq_warning recall <group_openid> <message_id>

# 撤回消息并隐藏撤回提示
./qq_warning recall <user_openid> <message_id> --hidetip
```

### 4. 如何获取历史消息？

由于 QQ Bot API 的限制，**无法通过 API 主动拉取历史消息列表**。要获取消息，你需要：

#### 方案 A：使用 WebSocket Daemon 模式实时接收

```bash
# 启动 WebSocket 服务，实时接收并记录消息
./qq_warning daemon
```

在 Daemon 模式下，机器人会：
- 保持与 QQ 服务器的 WebSocket 连接
- 实时接收所有新消息事件
- 你可以在 WebSocket 处理代码中保存消息到数据库

#### 方案 B：在代码中实现消息缓存

修改 `src/websocket.rs` 来缓存接收到的消息：

```rust
use std::collections::VecDeque;
use tokio::sync::RwLock;

// 添加消息缓存结构
pub struct MessageCache {
    messages: RwLock<VecDeque<Message>>,
    max_size: usize,
}

impl MessageCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: RwLock::new(VecDeque::with_capacity(max_size)),
            max_size,
        }
    }

    pub async fn add(&self, msg: Message) {
        let mut messages = self.messages.write().await;
        if messages.len() >= self.max_size {
            messages.pop_front();
        }
        messages.push_back(msg);
    }

    pub async fn get_recent(&self, count: usize) -> Vec<Message> {
        let messages = self.messages.read().await;
        messages.iter().rev().take(count).cloned().collect()
    }
}
```

## 完整工作流示例

### 示例 1：发送消息并撤回

```bash
# 1. 发送消息（需要保存返回的 message_id）
./qq_warning send to user_openid_123 "这是一条测试消息"

# 2. 如果需要撤回，使用消息 ID
# （在实际使用中，你需要从发送结果中获取 message_id）
./qq_warning recall user_openid_123 <message_id>
```

### 示例 2：获取频道消息详情

```bash
# 如果你已经知道消息 ID（例如从 WebSocket 事件中获得）
./qq_warning get-message <channel_id> <message_id>
```

### 示例 3：在 Rust 代码中使用

```rust
use qq_warning::{QQBot, config::Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load("config.toml")?;
    let bot = QQBot::new(config);
    
    // 发送消息并获取 ID
    let message_id = bot.send_user_message("user_openid_123", "你好！").await?;
    println!("消息 ID: {}", message_id);
    
    // 获取指定的频道消息
    let message = bot.get_channel_message("channel_id", &message_id).await?;
    println!("消息内容: {:?}", message.content);
    
    // 撤回消息
    bot.recall_user_message("user_openid_123", &message_id, false).await?;
    
    Ok(())
}
```

## API 限制说明

### QQ Bot API 的限制

1. **不支持批量获取历史消息**
   - ❌ 无法获取用户/群的消息列表
   - ❌ 无法分页浏览历史消息
   - ✅ 只能获取已知 message_id 的单条频道消息
   - ✅ 可以通过 WebSocket 实时接收新消息

2. **消息获取范围**
   - 频道消息：可以通过 `get_channel_message` 获取单条
   - 频道私信：可以通过 `get_dms_message` 获取单条
   - 用户私聊：**不支持获取**（只能实时接收）
   - 群消息：**不支持获取**（只能实时接收）

3. **速率限制**
   - 发送消息受配置文件中的速率限制约束
   - 建议在 `config.toml` 中设置 `min_interval_secs` 来避免触发限流

### 解决方案

如果你需要历史消息功能，建议：

1. **使用 Daemon 模式**：保持 WebSocket 连接，实时接收并缓存消息
2. **添加数据库**：使用 SQLite/PostgreSQL 等数据库存储接收到的消息
3. **实现本地缓存**：在内存中保留最近的 N 条消息供查询

## 配置文件示例

```toml
[bot]
app_id = "你的APP_ID"
client_secret = "你的CLIENT_SECRET"

[api]
base_url = "https://api.sgroup.qq.com"
auth_url = "https://bots.qq.com/app/getAppAccessToken"

[rate_limit]
min_interval_secs = 1

[logging]
level = "info"
file = "qq_warning.log"

[notification]
enabled = true
show_preview = true
```

## 故障排查

### 1. 无法获取消息历史

**问题**：尝试获取消息列表时返回 405 错误

**原因**：QQ Bot API 不提供批量获取历史消息的接口

**解决**：
- 使用 `daemon` 模式实时接收消息
- 只能获取已知 message_id 的单条频道消息

### 2. 消息 ID 格式错误

**问题**：使用错误的消息 ID 格式

**解决**：
- 消息 ID 是由 QQ API 返回的字符串
- 不要手动构造或修改消息 ID
- 从发送消息的返回值或 WebSocket 事件中获取

### 3. 撤回消息失败

**可能原因**：
- 只能撤回自己发送的消息
- 消息可能已超过撤回时限
- 没有撤回权限
- 消息 ID 不正确

### 4. 流式消息不显示

**问题**：群聊不支持流式消息

**解决**：
- 流式消息仅支持用户私聊 (C2C)
- 群消息会自动降级为普通发送

## 参考资源

- [QQ Bot 官方文档](https://bot.q.qq.com/wiki/)
- [QQ Official Bot SDK (Node.js)](https://github.com/zhinjs/qq-official-bot)
- [QQBot SDK (Java/Kotlin)](https://github.com/zimoyin/qqbot-sdk)
