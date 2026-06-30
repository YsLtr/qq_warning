use crate::config::Config;
use crate::qqbot::QQBot;
use crate::utils;
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::tungstenite::Message;

type WebSocketWriter = Arc<Mutex<futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    tokio_tungstenite::tungstenite::Message,
>>>;

#[derive(Debug, Deserialize)]
struct GatewayResponse {
    url: String,
}

#[derive(Debug, Serialize)]
struct IdentifyPayload {
    op: u8,
    d: IdentifyData,
}

#[derive(Debug, Serialize)]
struct IdentifyData {
    token: String,
    intents: u32,
    shard: [u8; 2],
}

#[derive(Debug, Serialize)]
struct HeartbeatPayload {
    op: u8,
    d: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct WebSocketMessage {
    op: u8,
    d: Option<serde_json::Value>,
    s: Option<u64>,
    t: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HelloData {
    heartbeat_interval: u64,
}

#[derive(Debug, Deserialize)]
struct MessageData {
    id: String,
    #[serde(default)]
    content: String,
    author: Author,
    #[serde(default)]
    attachments: Vec<Attachment>,
    #[serde(default)]
    msg_type: u8,
}

#[derive(Debug, Deserialize, Clone)]
struct Attachment {
    url: String,
    #[serde(default)]
    filename: Option<String>,
    #[serde(default)]
    content_type: Option<String>,
    #[serde(default)]
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct Author {
    id: String,
    username: String,
}

pub struct WebSocketService {
    bot: Arc<QQBot>,
    config: Config,
    session_id: Arc<Mutex<Option<String>>>,
    sequence: Arc<Mutex<Option<u64>>>,
}

impl WebSocketService {
    pub fn new(config: Config) -> Self {
        let bot = Arc::new(QQBot::new(config.clone()));
        Self {
            bot,
            config,
            session_id: Arc::new(Mutex::new(None)),
            sequence: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取 WebSocket Gateway URL
    async fn get_gateway_url(&self) -> Result<String> {
        let token = self.bot.get_access_token().await?;
        let url = format!("{}/gateway", self.config.api.base_url);

        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .header("Authorization", format!("QQBot {}", token))
            .send()
            .await
            .context("获取 Gateway URL 失败")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("获取 Gateway URL 失败 ({}): {}", status, body);
        }

        let gateway_resp: GatewayResponse = resp.json().await?;
        Ok(gateway_resp.url)
    }

    /// 启动 WebSocket 服务
    pub async fn start(&self) -> Result<()> {
        loop {
            tracing::info!("正在连接 WebSocket Gateway...");

            match self.connect_and_run().await {
                Ok(_) => {
                    tracing::info!("WebSocket 连接正常关闭");
                }
                Err(e) => {
                    tracing::error!("WebSocket 连接错误: {}", e);
                }
            }

            // 重连策略：等待后重试
            tracing::info!("5 秒后重新连接...");
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn connect_and_run(&self) -> Result<()> {
        use tokio_tungstenite::connect_async;

        // 获取 Gateway URL
        let gateway_url = self.get_gateway_url().await?;
        tracing::info!("连接到: {}", gateway_url);

        // 连接 WebSocket
        let (ws_stream, _) = connect_async(&gateway_url)
            .await
            .context("WebSocket 连接失败")?;

        tracing::info!("✓ WebSocket 连接成功");

        let (write, mut read) = ws_stream.split();
        let write = Arc::new(Mutex::new(write));

        // 处理消息
        while let Some(msg) = read.next().await {
            let msg = msg.context("接收消息失败")?;

            if let Message::Text(text) = msg {
                if let Err(e) = self.handle_message(&text, &write).await {
                    tracing::error!("处理消息错误: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn handle_message(&self, text: &str, write: &WebSocketWriter) -> Result<()> {
        let ws_msg: WebSocketMessage = serde_json::from_str(text)
            .context("解析 WebSocket 消息失败")?;

        // 更新 sequence
        if let Some(s) = ws_msg.s {
            let mut seq = self.sequence.lock().await;
            *seq = Some(s);
        }

        match ws_msg.op {
            10 => {
                // Hello - 开始心跳并认证
                tracing::info!("收到 Hello 消息");

                let hello_data: HelloData = serde_json::from_value(
                    ws_msg.d.context("Hello 消息缺少 data")?,
                )?;

                tracing::info!("心跳间隔: {} ms", hello_data.heartbeat_interval);

                // 发送 Identify
                self.send_identify(write).await?;

                // 启动心跳
                self.start_heartbeat(write.clone(), hello_data.heartbeat_interval)
                    .await;
            }
            11 => {
                // Heartbeat ACK
                tracing::debug!("收到心跳 ACK");
            }
            0 => {
                // Dispatch - 事件
                if let Some(event_type) = &ws_msg.t {
                    self.handle_event(event_type, ws_msg.d).await?;
                }
            }
            7 => {
                // Reconnect
                tracing::warn!("收到重连请求");
                return Err(anyhow::anyhow!("服务器要求重连"));
            }
            9 => {
                // Invalid Session
                tracing::warn!("会话无效，清除状态");
                let mut session = self.session_id.lock().await;
                *session = None;
                return Err(anyhow::anyhow!("会话无效"));
            }
            _ => {
                tracing::debug!("未知 opcode: {}", ws_msg.op);
            }
        }

        Ok(())
    }

    async fn send_identify(&self, write: &WebSocketWriter) -> Result<()> {
        let token = self.bot.get_access_token().await?;

        let identify = IdentifyPayload {
            op: 2,
            d: IdentifyData {
                token: format!("QQBot {}", token),
                intents: (1 << 25) | (1 << 30) | (1 << 12) | (1 << 26), // C2C + Group + Guild messages
                shard: [0, 1],
            },
        };

        let payload = serde_json::to_string(&identify)?;
        let mut writer = write.lock().await;
        writer
            .send(Message::Text(payload))
            .await
            .context("发送 Identify 失败")?;

        tracing::info!("✓ 已发送 Identify");
        Ok(())
    }

    async fn start_heartbeat(&self, write: WebSocketWriter, interval_ms: u64) {
        let sequence = self.sequence.clone();

        tokio::spawn(async move {
            // 使用 80% 的间隔发送心跳
            let interval = Duration::from_millis(interval_ms * 8 / 10);

            loop {
                sleep(interval).await;

                let seq = sequence.lock().await.clone();
                let heartbeat = HeartbeatPayload { op: 1, d: seq };

                match serde_json::to_string(&heartbeat) {
                    Ok(payload) => {
                        let mut writer = write.lock().await;
                        if let Err(e) = writer.send(Message::Text(payload)).await {
                            tracing::error!("发送心跳失败: {}", e);
                            break;
                        }
                        tracing::debug!("发送心跳 (seq: {:?})", seq);
                    }
                    Err(e) => {
                        tracing::error!("序列化心跳失败: {}", e);
                        break;
                    }
                }
            }
        });
    }

    async fn handle_event(
        &self,
        event_type: &str,
        data: Option<serde_json::Value>,
    ) -> Result<()> {
        match event_type {
            "READY" => {
                tracing::info!("✓ 机器人已准备就绪");
                if let Some(d) = data {
                    if let Some(session_id) = d.get("session_id").and_then(|v| v.as_str()) {
                        let mut session = self.session_id.lock().await;
                        *session = Some(session_id.to_string());
                        tracing::info!("Session ID: {}", session_id);
                    }
                }
            }
            "MESSAGE_CREATE" | "C2C_MESSAGE_CREATE" | "GROUP_AT_MESSAGE_CREATE" => {
                if let Some(d) = data {
                    if let Ok(msg_data) = serde_json::from_value::<MessageData>(d.clone()) {
                        self.handle_message_event(&msg_data).await?;
                    }
                }
            }
            _ => {
                tracing::debug!("收到事件: {}", event_type);
            }
        }

        Ok(())
    }

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
            tracing::info!("  内容: {}", msg.content);
        }

        // 处理附件
        if !msg.attachments.is_empty() {
            tracing::info!("  附件数量: {}", msg.attachments.len());
            for (idx, att) in msg.attachments.iter().enumerate() {
                let size_str = att.size
                    .map(|s| utils::format_size(s))
                    .unwrap_or_else(|| "未知".to_string());

                let filename = att.filename.as_deref().unwrap_or("未命名");
                let content_type = att.content_type.as_deref().unwrap_or("未知");

                tracing::info!(
                    "    [{}] {} ({}, {})",
                    idx + 1,
                    filename,
                    content_type,
                    size_str
                );
                tracing::info!("        URL: {}", att.url);

                // 自动下载
                if self.config.features.auto_download_media {
                    if let Err(e) = self.download_attachment(att, &msg.id).await {
                        tracing::error!("下载附件失败: {}", e);
                    }
                }
            }
        }

        // 发送桌面通知
        let notification_body = if !msg.content.is_empty() {
            msg.content.clone()
        } else if !msg.attachments.is_empty() {
            format!("[{}个附件]", msg.attachments.len())
        } else {
            "[空消息]".to_string()
        };

        let _ = utils::send_notification(
            &format!("来自 {}", msg.author.username),
            &notification_body,
            self.config.notifications.enabled,
            self.config.notifications.sound,
        );

        Ok(())
    }

    async fn download_attachment(&self, att: &Attachment, msg_id: &str) -> Result<()> {
        let media_dir = PathBuf::from(&self.config.features.media_dir);

        let filename = att.filename.as_deref().unwrap_or_else(|| {
            // 从 URL 提取文件名
            att.url
                .split('/')
                .last()
                .and_then(|s| s.split('?').next())
                .unwrap_or("unknown")
        });

        let save_path = media_dir.join(msg_id).join(filename);

        utils::download_file(&att.url, &save_path).await?;

        Ok(())
    }
}
