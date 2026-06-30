# QQ Bot 消息功能实现总结

## 问题背景

用户希望实现两个功能：
1. 读取最新的几条消息（获取消息历史）
2. 获取自己发送的消息 ID

## 调查结果

### QQ Bot API 限制

通过调研 QQ 官方 Bot API 和参考项目（[zhinjs/qq-official-bot](https://github.com/zhinjs/qq-official-bot) 和 [zimoyin/qqbot-sdk](https://github.com/zimoyin/qqbot-sdk)），发现：

**QQ Bot API 不提供批量获取历史消息的接口。**

#### 官方支持的消息获取方式：

✅ **支持的功能：**
- `GET /channels/{channel_id}/messages/{message_id}` - 获取**单条**频道消息
- `GET /dms/{guild_id}/messages/{message_id}` - 获取**单条**频道私信
- `POST` 发送消息时返回 `message_id`
- WebSocket 实时接收消息事件

❌ **不支持的功能：**
- 获取用户私聊消息历史列表
- 获取群消息历史列表
- 获取频道消息历史列表
- 分页浏览历史消息

## 实现方案

### 1. 获取消息 ID ✅

**状态：已完全实现**

所有发送消息的 API 都会返回 `MessageResponse` 结构体，包含：
- `id: String` - 消息 ID
- `timestamp: Option<String>` - 时间戳

#### 实现位置

- `src/api.rs`: `MessageResponse` 结构体
- `src/qqbot.rs`: 所有 `send_*` 方法返回 `String` (message_id)

#### 使用方式

```rust
let message_id = bot.send_user_message("user_openid", "Hello").await?;
println!("消息 ID: {}", message_id);
```

### 2. 获取消息详情 ✅

**状态：部分实现（受 API 限制）**

实现了获取已知消息 ID 的单条消息：

#### 已实现的功能

```rust
// 获取频道消息
pub async fn get_channel_message(&self, channel_id: &str, message_id: &str) -> Result<Message>

// 获取频道私信
pub async fn get_dms_message(&self, guild_id: &str, message_id: &str) -> Result<Message>
```

#### 命令行使用

```bash
./qq_warning get-message <channel_id> <message_id>
```

### 3. 消息历史功能的替代方案

由于 API 限制，我们提供了以下解决方案：

#### 方案 A：WebSocket Daemon 模式（推荐）

```bash
# 启动 WebSocket 服务，实时接收所有消息
./qq_warning daemon
```

在 `src/websocket.rs` 中，可以添加消息缓存逻辑：

```rust
// 建议添加的功能
pub struct MessageCache {
    messages: Arc<RwLock<VecDeque<Message>>>,
    max_size: usize,
}

// 在 WebSocket 事件处理中保存消息
async fn handle_message_event(&self, event: MessageEvent) {
    self.cache.add(event.into()).await;
    // ... 其他处理逻辑
}
```

#### 方案 B：数据库持久化

将接收到的消息存储到数据库（SQLite/PostgreSQL）供后续查询。

## 代码变更

### 新增文件

1. `USAGE_EXAMPLES.md` - 使用示例文档
2. 本文档 - 实现总结

### 修改的文件

#### 1. `src/types.rs`

新增消息相关类型：

```rust
pub struct Message {
    pub id: String,
    pub content: Option<String>,
    pub timestamp: Option<String>,
    pub author: Option<MessageAuthor>,
    pub attachments: Option<Vec<MessageAttachment>>,
    pub msg_seq: Option<i64>,
}

pub struct MessageAuthor { ... }
pub struct MessageAttachment { ... }
```

#### 2. `src/api.rs`

新增方法：

```rust
pub async fn get_channel_message(&self, channel_id: &str, message_id: &str) -> Result<Message>
pub async fn get_dms_message(&self, guild_id: &str, message_id: &str) -> Result<Message>
```

#### 3. `src/qqbot.rs`

新增方法：

```rust
pub async fn get_channel_message(&self, channel_id: &str, message_id: &str) -> Result<Message>
pub async fn get_dms_message(&self, guild_id: &str, message_id: &str) -> Result<Message>
```

#### 4. `src/main.rs`

新增命令：

```rust
Commands::GetMessage {
    channel_id: String,
    message_id: String,
}
```

## 使用示例

### 获取消息 ID

```bash
# 发送消息
./qq_warning send to <user_openid> "Hello"
# 输出会包含消息 ID（可在日志中查看或修改代码显式输出）
```

### 获取指定消息

```bash
./qq_warning get-message <channel_id> <message_id>
```

### 实时接收消息

```bash
./qq_warning daemon
```

## 限制与注意事项

1. **API 限制**
   - 无法批量获取历史消息列表
   - 只能获取已知 ID 的单条频道消息
   - 用户私聊和群消息无法主动获取

2. **建议的使用方式**
   - 使用 WebSocket Daemon 模式实时接收消息
   - 在应用层实现消息缓存或数据库存储
   - 保存发送消息时返回的 message_id 供后续使用

3. **性能考虑**
   - WebSocket 连接需要保持活跃
   - 大量消息需要合理的存储策略
   - 注意速率限制避免被封禁

## 参考资源

- [QQ Bot 官方文档](https://bot.q.qq.com/wiki/)
- [zhinjs/qq-official-bot](https://github.com/zhinjs/qq-official-bot) - Node.js SDK 参考
- [zimoyin/qqbot-sdk](https://github.com/zimoyin/qqbot-sdk) - Java/Kotlin SDK 参考

## 总结

✅ **已实现：**
- 发送消息时获取消息 ID
- 获取已知 ID 的单条频道消息
- 完整的命令行接口
- 详细的使用文档

❌ **无法实现（API 限制）：**
- 批量获取历史消息列表
- 分页浏览消息
- 获取私聊/群消息详情

💡 **推荐方案：**
- 使用 WebSocket Daemon 模式 + 本地缓存/数据库
- 实时接收并存储所有需要的消息
- 在应用层实现消息查询功能
