# QQ Bot 流式消息实现指南

根据 hermes-agent-rs 项目的实现，QQ Bot 支持 C2C（私聊）流式消息，这是一个强大的功能，可以实现类似 ChatGPT 的打字效果。

## 流式消息原理

### API 端点
与普通消息相同，但增加 `stream` 字段：

```json
POST /v2/users/{user_openid}/messages

{
  "msg_type": 0,  // 0=文本, 2=Markdown
  "content": "流式内容...",
  "msg_seq": 12345,
  "stream": {
    "id": "stream_uuid",     // 流 ID（首次可省略）
    "index": 0,              // 块索引（从 0 开始递增）
    "state": 1,              // 1=进行中, 10=完成
    "reset": false           // 是否重置流
  }
}
```

### 流式状态机

```
开始 → 发送第一块(state=1, index=0)
    → 发送第二块(state=1, index=1, id=响应中的id)
    → ...
    → 发送最后块(state=10, index=N)
    → 结束（自动重置状态）
```

## 实现方案

### 1. 添加流式消息数据结构

```rust
// src/types.rs 添加

/// 流式消息配置
#[derive(Debug, Serialize, Clone)]
pub struct StreamPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub index: u64,
    pub state: u8,  // 1=streaming, 10=done
    #[serde(default)]
    pub reset: bool,
}

/// 流式消息状态
#[derive(Debug, Clone, Default)]
pub struct StreamState {
    pub id: Option<String>,
    pub index: u64,
    pub active: bool,
}
```

### 2. 实现流式 API

```rust
// src/api.rs 添加

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct QQBotApi {
    // ... 现有字段
    stream_states: Arc<Mutex<HashMap<String, StreamState>>>,
}

impl QQBotApi {
    /// 发送流式消息块
    pub async fn send_stream_chunk(
        &self,
        user_openid: &str,
        content: &str,
        is_final: bool,
    ) -> Result<MessageResponse> {
        self.apply_rate_limit().await;

        let mut states = self.stream_states.lock().await;
        let state = states.entry(user_openid.to_string()).or_default();

        let stream_payload = StreamPayload {
            id: state.id.clone(),
            index: state.index,
            state: if is_final { 10 } else { 1 },
            reset: false,
        };

        let msg = MessageRequest {
            content: Some(content.to_string()),
            msg_type: Some(0),
            stream: Some(stream_payload),
            msg_seq: Some(Self::next_msg_seq(user_openid)),
            ..Default::default()
        };

        let response: MessageResponse = self.request(
            reqwest::Method::POST,
            &format!("/v2/users/{}/messages", user_openid),
            Some(msg),
        )
        .await?;

        // 更新状态
        if is_final {
            states.remove(user_openid);
        } else {
            state.id = Some(response.id.clone());
            state.index += 1;
            state.active = true;
        }

        Ok(response)
    }

    /// 发送流式 Markdown 消息块
    pub async fn send_stream_markdown_chunk(
        &self,
        user_openid: &str,
        markdown_content: &str,
        is_final: bool,
    ) -> Result<MessageResponse> {
        self.apply_rate_limit().await;

        let mut states = self.stream_states.lock().await;
        let state = states.entry(user_openid.to_string()).or_default();

        let stream_payload = StreamPayload {
            id: state.id.clone(),
            index: state.index,
            state: if is_final { 10 } else { 1 },
            reset: false,
        };

        let markdown = Markdown {
            content: markdown_content.to_string(),
            custom_template_id: None,
            params: None,
        };

        let msg = MessageRequest {
            msg_type: Some(2),
            markdown: Some(markdown),
            stream: Some(stream_payload),
            msg_seq: Some(Self::next_msg_seq(user_openid)),
            ..Default::default()
        };

        let response: MessageResponse = self.request(
            reqwest::Method::POST,
            &format!("/v2/users/{}/messages", user_openid),
            Some(msg),
        )
        .await?;

        // 更新状态
        if is_final {
            states.remove(user_openid);
        } else {
            state.id = Some(response.id.clone());
            state.index += 1;
            state.active = true;
        }

        Ok(response)
    }

    /// 取消流式消息
    pub async fn cancel_stream(&self, user_openid: &str) -> Result<()> {
        let mut states = self.stream_states.lock().await;
        states.remove(user_openid);
        Ok(())
    }
}
```

### 3. 命令行接口

```rust
// src/main.rs 添加

#[derive(Subcommand)]
enum Commands {
    // ... 现有命令
    
    /// 发送流式消息（模拟打字效果）
    SendStream {
        /// 用户 OpenID
        user_id: String,
        /// 消息内容
        message: String,
        /// 每次发送的字符数
        #[arg(short = 'c', long, default_value = "10")]
        chunk_size: usize,
        /// 每块之间的延迟（毫秒）
        #[arg(short = 'd', long, default_value = "200")]
        delay_ms: u64,
        /// 使用 Markdown 格式
        #[arg(short, long)]
        markdown: bool,
    },
}

// 在 main 函数中处理
Commands::SendStream { user_id, message, chunk_size, delay_ms, markdown } => {
    let bot = qqbot::QQBot::new(config);
    tracing::info!("开始流式发送消息到用户 {}...", user_id);
    
    bot.send_stream_message(
        &user_id,
        &message,
        chunk_size,
        delay_ms,
        markdown
    ).await?;
    
    tracing::info!("✓ 流式消息发送完成");
}
```

### 4. 高级封装

```rust
// src/qqbot.rs 添加

impl QQBot {
    /// 发送流式消息
    pub async fn send_stream_message(
        &self,
        user_openid: &str,
        content: &str,
        chunk_size: usize,
        delay_ms: u64,
        markdown: bool,
    ) -> Result<()> {
        let chars: Vec<char> = content.chars().collect();
        let total_chunks = (chars.len() + chunk_size - 1) / chunk_size;
        
        for (idx, chunk) in chars.chunks(chunk_size).enumerate() {
            let chunk_text: String = chunk.iter().collect();
            let is_final = idx == total_chunks - 1;
            
            if markdown {
                self.api.send_stream_markdown_chunk(
                    user_openid,
                    &chunk_text,
                    is_final
                ).await?;
            } else {
                self.api.send_stream_chunk(
                    user_openid,
                    &chunk_text,
                    is_final
                ).await?;
            }
            
            // 延迟下一块
            if !is_final {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }
        
        Ok(())
    }

    /// 流式发送生成器产出的内容
    pub async fn send_stream_from_generator<S>(
        &self,
        user_openid: &str,
        mut stream: S,
        markdown: bool,
    ) -> Result<()>
    where
        S: Stream<Item = String> + Unpin,
    {
        use futures::StreamExt;
        
        let mut accumulated = String::new();
        let mut is_first = true;
        
        while let Some(chunk) = stream.next().await {
            accumulated.push_str(&chunk);
            
            // 每 50 个字符或首次发送
            if is_first || accumulated.len() >= 50 {
                if markdown {
                    self.api.send_stream_markdown_chunk(
                        user_openid,
                        &accumulated,
                        false
                    ).await?;
                } else {
                    self.api.send_stream_chunk(
                        user_openid,
                        &accumulated,
                        false
                    ).await?;
                }
                is_first = false;
                
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
        }
        
        // 发送最终消息
        if markdown {
            self.api.send_stream_markdown_chunk(
                user_openid,
                &accumulated,
                true
            ).await?;
        } else {
            self.api.send_stream_chunk(
                user_openid,
                &accumulated,
                true
            ).await?;
        }
        
        Ok(())
    }
}
```

## Markdown 渲染支持

### 在 Daemon 中渲染 Markdown

```rust
// src/websocket.rs 修改

// 添加依赖到 Cargo.toml
// termimad = "0.29"  # Markdown 终端渲染

async fn handle_message_event(&self, msg: &MessageData) -> Result<()> {
    let msg_type_str = match msg.msg_type {
        0 => "文本",
        2 => "Markdown",
        7 => "富媒体",
        _ => "未知",
    };

    tracing::info!("\n收到消息 [{}]:", msg_type_str);
    tracing::info!("  消息 ID: {}", msg.id);
    tracing::info!("  发送者: {} ({})", msg.author.username, msg.author.id);

    if !msg.content.is_empty() {
        if msg.msg_type == 2 {
            // Markdown 渲染
            self.render_markdown(&msg.content);
        } else {
            // 普通文本
            tracing::info!("  内容: {}", msg.content);
        }
    }

    // ... 处理附件等
}

fn render_markdown(&self, content: &str) {
    use termimad::*;
    
    // 创建 Markdown 渲染器
    let skin = MadSkin::default();
    
    println!("  内容:");
    println!("{}", "─".repeat(60));
    
    // 渲染到终端
    skin.print_text(content);
    
    println!("{}", "─".repeat(60));
}
```

## 使用示例

### 1. 基础流式消息

```bash
# 模拟打字效果
qq_warning send-stream <USER_ID> "这是一条流式消息，会逐字显示" \
    --chunk-size 5 --delay-ms 150
```

### 2. 流式 Markdown

```bash
# Markdown 流式消息
qq_warning send-stream <USER_ID> "# 标题\n逐步显示的内容..." \
    --markdown --chunk-size 10 --delay-ms 200
```

### 3. 集成到脚本

```python
import subprocess
import time

def send_progress_stream(user_id, steps):
    """发送进度流式消息"""
    message = "# 任务进度\n\n"
    
    for i, step in enumerate(steps, 1):
        message += f"{i}. {step}\n"
        
        # 发送中间状态（非最终）
        subprocess.run([
            "qq_warning", "api", "stream-chunk",
            user_id, message, "0"  # 0 = 非最终
        ])
        time.sleep(0.3)
    
    # 发送最终状态
    message += "\n✅ 任务完成"
    subprocess.run([
        "qq_warning", "api", "stream-chunk",
        user_id, message, "1"  # 1 = 最终
    ])

# 使用
send_progress_stream("user_id", [
    "初始化环境",
    "下载依赖",
    "编译代码",
    "运行测试"
])
```

## 注意事项

1. **仅支持 C2C（私聊）**
   - 群聊目前不支持流式消息
   - 发送到群会自动降级为普通消息

2. **速率限制**
   - 建议每块间隔 200-500ms
   - 避免发送过快触发限流

3. **消息长度**
   - 单块最大 4096 字符
   - 超长内容需要分块

4. **状态管理**
   - 必须按顺序递增 index
   - 最后一块必须设置 state=10
   - 中断后会自动清理状态

5. **错误处理**
   - Markdown 被拒绝时自动降级为纯文本
   - 网络错误会中断流式发送
   - 建议实现重试机制

## 性能优化

### 智能缓冲

```rust
pub struct StreamBuffer {
    content: String,
    last_flush: Instant,
    flush_threshold: usize,  // 字符数阈值
    time_threshold: Duration, // 时间阈值
}

impl StreamBuffer {
    pub fn should_flush(&self) -> bool {
        self.content.len() >= self.flush_threshold
            || self.last_flush.elapsed() >= self.time_threshold
    }
    
    pub async fn add_and_maybe_flush(
        &mut self,
        chunk: &str,
        sender: impl Fn(&str) -> Result<()>
    ) -> Result<()> {
        self.content.push_str(chunk);
        
        if self.should_flush() {
            sender(&self.content).await?;
            self.last_flush = Instant::now();
        }
        
        Ok(())
    }
}
```

## 总结

流式消息实现后，你的工具将支持：

- ✅ 类似 ChatGPT 的打字效果
- ✅ 实时进度显示
- ✅ 更好的用户体验
- ✅ Markdown 内容逐步渲染
- ✅ 与 AI 流式输出完美配合

这使得工具不仅是简单的消息发送器，还能提供交互式的实时反馈体验！
