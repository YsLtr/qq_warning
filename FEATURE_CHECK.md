# 功能实现检查报告

检查时间: 2026/06/30

## 检查结果总览

| 功能 | 实现状态 | 说明 |
|-----|---------|------|
| 发送消息提醒 | ❌ 未实现 | 仅有控制台输出，无桌面通知 |
| C2C Stream | ❌ 未实现 | 仅使用标准 HTTP POST |
| 日志记录 | ❌ 未实现 | 仅使用 println/eprintln |
| 接收图片/文件 | ❌ 未实现 | 仅解析文本消息 |

---

## 详细分析

### 1. ❌ 发送消息提醒（桌面通知）

**当前实现**:
```rust
// websocket.rs:291-295
println!("\n收到消息:");
println!("  消息 ID: {}", msg_data.id);
println!("  发送者: {} ({})", msg_data.author.username, msg_data.author.id);
println!("  内容: {}", msg_data.content);
```

**缺少的功能**:
- 桌面通知（libnotify / Windows Toast）
- 系统托盘图标
- 声音提醒
- 未读消息计数

**建议实现**:
- 添加 `notify-rust` 依赖用于桌面通知
- 添加配置选项控制通知行为

---

### 2. ❌ C2C Stream (流式发送)

**当前实现**:
```rust
// qqbot.rs:153-185
pub async fn send_user_message(&self, user_openid: &str, content: &str) -> Result<()> {
    // 使用标准 HTTP POST
    let resp = self.client
        .post(&url)
        .json(&msg)
        .send()
        .await?;
}
```

**缺少的功能**:
- HTTP/2 Server-Sent Events (SSE)
- 流式传输长文本
- 实时传输进度
- 分块发送大消息

**API 限制**:
根据 QQ Bot API 文档，C2C 消息 API 可能不支持流式传输，需要确认官方 API 是否提供 SSE 端点。

---

### 3. ❌ 日志记录

**当前实现**:
- 全部使用 `println!` 和 `eprintln!`
- 无日志级别控制
- 无日志文件输出
- 无日志轮转

**示例**:
```rust
println!("正在连接 WebSocket Gateway...");  // 信息日志
eprintln!("WebSocket 连接错误: {}", e);      // 错误日志
```

**建议改进**:
使用专业日志库：
- `tracing` - 结构化日志和追踪
- `env_logger` - 简单的日志库
- `log` - 标准日志门面

**推荐实现**:
```rust
// 添加依赖
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

// 使用
info!("正在连接 WebSocket Gateway...");
error!("WebSocket 连接错误: {}", e);
debug!("收到消息: {:?}", msg);
```

---

### 4. ❌ 接收图片/文件

**当前实现**:
```rust
// websocket.rs:56-60
#[derive(Debug, Deserialize)]
struct MessageData {
    id: String,
    content: String,  // 仅处理文本
    author: Author,
}
```

**缺少的功能**:
- 图片消息解析
- 文件消息解析
- 富媒体消息（语音、视频）
- 表情消息
- Markdown 消息
- 附件下载

**QQ Bot API 消息结构**:
根据官方文档，消息可能包含：
```json
{
  "id": "...",
  "content": "...",
  "attachments": [
    {
      "url": "https://...",
      "filename": "image.jpg",
      "content_type": "image/jpeg",
      "size": 102400
    }
  ],
  "embeds": [...],
  "msg_type": 0  // 0=文本, 2=Markdown, 7=富媒体
}
```

**建议实现**:
```rust
#[derive(Debug, Deserialize)]
struct MessageData {
    id: String,
    content: String,
    author: Author,
    #[serde(default)]
    attachments: Vec<Attachment>,
    #[serde(default)]
    msg_type: u8,
}

#[derive(Debug, Deserialize)]
struct Attachment {
    url: String,
    filename: Option<String>,
    content_type: Option<String>,
    size: Option<u64>,
}
```

---

## 推荐改进优先级

### 🔴 高优先级

1. **日志系统** (必需)
   - 理由: 生产环境调试和监控必需
   - 工作量: 中等
   - 依赖: `tracing` + `tracing-subscriber`

2. **接收图片/文件** (重要)
   - 理由: 用户常发送图片和文件
   - 工作量: 中等
   - 需要: 扩展 MessageData 结构

### 🟡 中优先级

3. **桌面通知** (有用)
   - 理由: 提升用户体验
   - 工作量: 小
   - 依赖: `notify-rust`
   - 注意: 需要处理跨平台差异

### 🟢 低优先级

4. **C2C Stream** (可选)
   - 理由: QQ Bot API 可能不支持
   - 工作量: 大
   - 需要: 确认官方 API 支持

---

## 建议的下一步实现

### Phase 1: 日志系统 (1-2 小时)

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

```rust
// main.rs
fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
```

### Phase 2: 富媒体消息支持 (2-3 小时)

扩展 MessageData 结构，支持：
- attachments (图片、文件)
- embeds (嵌入内容)
- msg_type (消息类型区分)

### Phase 3: 桌面通知 (1 小时)

```toml
[dependencies]
notify-rust = "4"
```

```rust
use notify_rust::Notification;

Notification::new()
    .summary("新消息")
    .body(&format!("{}: {}", sender, content))
    .show()?;
```

---

## 配置文件扩展建议

```toml
[features]
# 功能开关
enable_notifications = true
enable_file_download = true
enable_logging = true

[logging]
level = "info"  # trace, debug, info, warn, error
file = "qq_warning.log"
max_size = "10MB"
max_backups = 5

[notifications]
enabled = true
sound = true
show_preview = true
```

---

## 总结

当前实现是一个**最小可用原型**，核心消息收发功能完整，但缺少生产环境所需的：

1. **日志系统** - 最迫切需要
2. **富媒体支持** - 用户体验重要
3. **桌面通知** - 锦上添花
4. **流式传输** - 取决于 API 支持

建议按优先级逐步实现，先完成日志系统和富媒体消息支持。
