use crate::config::Config;
use crate::types::*;
use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(deserialize_with = "deserialize_string_or_number")]
    expires_in: u64,
}

fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Deserialize};

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(u64),
    }

    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => s.parse::<u64>().map_err(de::Error::custom),
        StringOrNumber::Number(n) => Ok(n),
    }
}

#[derive(Debug, Clone)]
struct TokenCache {
    token: String,
    expires_at: Instant,
}

pub struct QQBotApi {
    config: Config,
    client: Client,
    token_cache: Arc<Mutex<Option<TokenCache>>>,
    last_send_time: Arc<Mutex<Option<Instant>>>,
    stream_states: Arc<Mutex<HashMap<String, StreamState>>>,
}

#[derive(Debug, Serialize)]
struct AuthRequest {
    #[serde(rename = "appId")]
    app_id: String,
    #[serde(rename = "clientSecret")]
    client_secret: String,
}

#[derive(Debug, Serialize)]
struct MessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    markdown: Option<Markdown>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keyboard: Option<Keyboard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ark: Option<Ark>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_reference: Option<MessageReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_seq: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media: Option<Media>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<StreamPayload>,
}

#[derive(Debug, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    #[serde(default)]
    pub timestamp: Option<String>,
}

impl QQBotApi {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            token_cache: Arc::new(Mutex::new(None)),
            last_send_time: Arc::new(Mutex::new(None)),
            stream_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取 access token，自动处理缓存和刷新
    pub async fn get_access_token(&self) -> Result<String> {
        let mut cache = self.token_cache.lock().await;

        // 检查缓存是否有效（提前 60 秒刷新）
        if let Some(cached) = &*cache {
            if cached.expires_at > Instant::now() + Duration::from_secs(60) {
                return Ok(cached.token.clone());
            }
        }

        // 获取新 token
        let auth_req = AuthRequest {
            app_id: self.config.bot.app_id.clone(),
            client_secret: self.config.bot.client_secret.clone(),
        };

        let resp = self
            .client
            .post(&self.config.api.auth_url)
            .json(&auth_req)
            .send()
            .await
            .context("认证请求失败")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("认证失败 ({}): {}", status, body);
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .context("解析认证响应失败")?;

        // 更新缓存
        let new_cache = TokenCache {
            token: token_resp.access_token.clone(),
            expires_at: Instant::now() + Duration::from_secs(token_resp.expires_in),
        };

        *cache = Some(new_cache);

        Ok(token_resp.access_token)
    }

    /// 生成消息序列号
    fn next_msg_seq(seed: &str) -> i64 {
        let base = Utc::now().timestamp_millis();
        let salt = seed
            .bytes()
            .fold(0_i64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as i64));
        (base.wrapping_add(salt).rem_euclid(65535)).max(1)
    }

    /// 应用速率限制
    async fn apply_rate_limit(&self) {
        let mut last_send = self.last_send_time.lock().await;

        if let Some(last) = *last_send {
            let elapsed = last.elapsed();
            let min_interval = Duration::from_secs(self.config.rate_limit.min_interval_secs);

            if elapsed < min_interval {
                let sleep_duration = min_interval - elapsed;
                tokio::time::sleep(sleep_duration).await;
            }
        }

        *last_send = Some(Instant::now());
    }

    /// 通用请求方法
    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<impl Serialize>,
    ) -> Result<T> {
        let token = self.get_access_token().await?;
        let url = format!("{}{}", self.config.api.base_url, path);

        let mut req = self
            .client
            .request(method, &url)
            .header("Authorization", format!("QQBot {}", token))
            .header("Content-Type", "application/json");

        if let Some(b) = body {
            req = req.json(&b);
        }

        let resp = req.send().await.context("请求失败")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("API 请求失败 ({}): {}", status, body);
        }

        resp.json().await.context("解析响应失败")
    }

    /// 发送文本消息到用户 (C2C)
    pub async fn send_user_message(&self, user_openid: &str, content: &str) -> Result<MessageResponse> {
        self.apply_rate_limit().await;
        let _ = self.send_typing_indicator(user_openid, true).await;

        let msg = MessageRequest {
            content: Some(content.to_string()),
            msg_type: Some(0),
            msg_seq: Some(Self::next_msg_seq(user_openid)),
            ..Default::default()
        };

        self.request(
            reqwest::Method::POST,
            &format!("/v2/users/{}/messages", user_openid),
            Some(msg),
        )
        .await
    }

    /// 发送消息到群聊
    pub async fn send_group_message(&self, group_openid: &str, content: &str) -> Result<MessageResponse> {
        self.apply_rate_limit().await;
        let _ = self.send_typing_indicator(group_openid, false).await;

        let msg = MessageRequest {
            content: Some(content.to_string()),
            msg_type: Some(0),
            msg_seq: Some(Self::next_msg_seq(group_openid)),
            ..Default::default()
        };

        self.request(
            reqwest::Method::POST,
            &format!("/v2/groups/{}/messages", group_openid),
            Some(msg),
        )
        .await
    }

    /// 发送 Markdown 消息到用户
    pub async fn send_user_markdown(&self, user_openid: &str, markdown: &Markdown) -> Result<MessageResponse> {
        self.apply_rate_limit().await;

        let msg = MessageRequest {
            msg_type: Some(2),
            markdown: Some(markdown.clone()),
            msg_seq: Some(Self::next_msg_seq(user_openid)),
            ..Default::default()
        };

        self.request(
            reqwest::Method::POST,
            &format!("/v2/users/{}/messages", user_openid),
            Some(msg),
        )
        .await
    }

    /// 发送带键盘的消息
    pub async fn send_user_message_with_keyboard(
        &self,
        user_openid: &str,
        content: &str,
        keyboard: &Keyboard,
    ) -> Result<MessageResponse> {
        self.apply_rate_limit().await;

        let msg = MessageRequest {
            content: Some(content.to_string()),
            msg_type: Some(0),
            keyboard: Some(keyboard.clone()),
            msg_seq: Some(Self::next_msg_seq(user_openid)),
            ..Default::default()
        };

        self.request(
            reqwest::Method::POST,
            &format!("/v2/users/{}/messages", user_openid),
            Some(msg),
        )
        .await
    }

    /// 发送 Ark 消息
    pub async fn send_user_ark(&self, user_openid: &str, ark: &Ark) -> Result<MessageResponse> {
        self.apply_rate_limit().await;

        let msg = MessageRequest {
            msg_type: Some(3),
            ark: Some(ark.clone()),
            msg_seq: Some(Self::next_msg_seq(user_openid)),
            ..Default::default()
        };

        self.request(
            reqwest::Method::POST,
            &format!("/v2/users/{}/messages", user_openid),
            Some(msg),
        )
        .await
    }

    /// 发送图片
    pub async fn send_user_image(&self, user_openid: &str, image_url: &str) -> Result<MessageResponse> {
        self.apply_rate_limit().await;

        let msg = MessageRequest {
            msg_type: Some(7),
            image: Some(image_url.to_string()),
            msg_seq: Some(Self::next_msg_seq(user_openid)),
            ..Default::default()
        };

        self.request(
            reqwest::Method::POST,
            &format!("/v2/users/{}/messages", user_openid),
            Some(msg),
        )
        .await
    }

    /// 撤回消息 (C2C)
    pub async fn recall_user_message(&self, user_openid: &str, message_id: &str, hidetip: bool) -> Result<()> {
        let delete_req = MessageDelete {
            hidetip: Some(hidetip),
        };

        self.request::<serde_json::Value>(
            reqwest::Method::DELETE,
            &format!("/v2/users/{}/messages/{}", user_openid, message_id),
            Some(delete_req),
        )
        .await?;

        Ok(())
    }

    /// 撤回群消息
    pub async fn recall_group_message(&self, group_openid: &str, message_id: &str, hidetip: bool) -> Result<()> {
        let delete_req = MessageDelete {
            hidetip: Some(hidetip),
        };

        self.request::<serde_json::Value>(
            reqwest::Method::DELETE,
            &format!("/v2/groups/{}/messages/{}", group_openid, message_id),
            Some(delete_req),
        )
        .await?;

        Ok(())
    }

    /// 发送"正在输入"提示
    async fn send_typing_indicator(&self, openid: &str, is_user: bool) -> Result<()> {
        let token = self.get_access_token().await?;
        let url = if is_user {
            format!("{}/v2/users/{}/messages", self.config.api.base_url, openid)
        } else {
            format!("{}/v2/groups/{}/messages", self.config.api.base_url, openid)
        };

        let typing_msg = serde_json::json!({
            "msg_type": 6,
            "msg_seq": Self::next_msg_seq(openid),
        });

        let _ = self
            .client
            .post(&url)
            .header("Authorization", format!("QQBot {}", token))
            .header("Content-Type", "application/json")
            .json(&typing_msg)
            .send()
            .await;

        Ok(())
    }

    /// 设置消息表情回应
    pub async fn put_message_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji_type: u8,
        emoji_id: &str,
    ) -> Result<()> {
        let reaction = MessageReaction {
            emoji_id: emoji_id.to_string(),
            emoji_type,
        };

        self.request::<serde_json::Value>(
            reqwest::Method::PUT,
            &format!("/channels/{}/messages/{}/reactions/{}/{}", channel_id, message_id, emoji_type, emoji_id),
            Some(reaction),
        )
        .await?;

        Ok(())
    }

    /// 删除消息表情回应
    pub async fn delete_message_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji_type: u8,
        emoji_id: &str,
    ) -> Result<()> {
        self.request::<serde_json::Value>(
            reqwest::Method::DELETE,
            &format!("/channels/{}/messages/{}/reactions/{}/{}", channel_id, message_id, emoji_type, emoji_id),
            None::<()>,
        )
        .await?;

        Ok(())
    }

    /// 获取频道成员信息
    pub async fn get_guild_member(&self, guild_id: &str, user_id: &str) -> Result<Member> {
        self.request(
            reqwest::Method::GET,
            &format!("/guilds/{}/members/{}", guild_id, user_id),
            None::<()>,
        )
        .await
    }

    /// 获取频道身份组列表
    pub async fn get_guild_roles(&self, guild_id: &str) -> Result<Vec<Role>> {
        #[derive(Deserialize)]
        struct RolesResponse {
            roles: Vec<Role>,
        }

        let resp: RolesResponse = self.request(
            reqwest::Method::GET,
            &format!("/guilds/{}/roles", guild_id),
            None::<()>,
        )
        .await?;

        Ok(resp.roles)
    }

    /// 增加频道身份组成员
    pub async fn add_guild_role_member(
        &self,
        guild_id: &str,
        role_id: &str,
        user_id: &str,
        channel_id: Option<&str>,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct AddRoleMember {
            #[serde(skip_serializing_if = "Option::is_none")]
            channel: Option<ChannelInfo>,
        }

        #[derive(Serialize)]
        struct ChannelInfo {
            id: String,
        }

        let body = AddRoleMember {
            channel: channel_id.map(|id| ChannelInfo {
                id: id.to_string(),
            }),
        };

        self.request::<serde_json::Value>(
            reqwest::Method::PUT,
            &format!("/guilds/{}/members/{}/roles/{}", guild_id, user_id, role_id),
            Some(body),
        )
        .await?;

        Ok(())
    }

    /// 禁言用户
    pub async fn mute_member(&self, guild_id: &str, user_id: &str, mute_seconds: u64) -> Result<()> {
        let mute_opts = MuteOptions {
            mute_end_timestamp: "".to_string(),
            mute_seconds: Some(mute_seconds.to_string()),
        };

        self.request::<serde_json::Value>(
            reqwest::Method::PATCH,
            &format!("/guilds/{}/members/{}/mute", guild_id, user_id),
            Some(mute_opts),
        )
        .await?;

        Ok(())
    }

    /// 禁言全员
    pub async fn mute_all(&self, guild_id: &str, mute_seconds: u64) -> Result<()> {
        let mute_opts = MuteOptions {
            mute_end_timestamp: "".to_string(),
            mute_seconds: Some(mute_seconds.to_string()),
        };

        self.request::<serde_json::Value>(
            reqwest::Method::PATCH,
            &format!("/guilds/{}/mute", guild_id),
            Some(mute_opts),
        )
        .await?;

        Ok(())
    }

    /// 创建公告
    pub async fn create_announce(&self, guild_id: &str, channel_id: &str, message_id: &str) -> Result<Announce> {
        #[derive(Serialize)]
        struct CreateAnnounce {
            message_id: String,
            channel_id: String,
        }

        let body = CreateAnnounce {
            message_id: message_id.to_string(),
            channel_id: channel_id.to_string(),
        };

        self.request(
            reqwest::Method::POST,
            &format!("/guilds/{}/announces", guild_id),
            Some(body),
        )
        .await
    }

    /// 删除公告
    pub async fn delete_announce(&self, guild_id: &str, message_id: &str) -> Result<()> {
        self.request::<serde_json::Value>(
            reqwest::Method::DELETE,
            &format!("/guilds/{}/announces/{}", guild_id, message_id),
            None::<()>,
        )
        .await?;

        Ok(())
    }

    /// 添加精华消息
    pub async fn add_pin(&self, channel_id: &str, message_id: &str) -> Result<PinsMessage> {
        self.request(
            reqwest::Method::PUT,
            &format!("/channels/{}/pins/{}", channel_id, message_id),
            None::<()>,
        )
        .await
    }

    /// 删除精华消息
    pub async fn delete_pin(&self, channel_id: &str, message_id: &str) -> Result<()> {
        self.request::<serde_json::Value>(
            reqwest::Method::DELETE,
            &format!("/channels/{}/pins/{}", channel_id, message_id),
            None::<()>,
        )
        .await?;

        Ok(())
    }

    /// 获取精华消息
    pub async fn get_pins(&self, channel_id: &str) -> Result<PinsMessage> {
        self.request(
            reqwest::Method::GET,
            &format!("/channels/{}/pins", channel_id),
            None::<()>,
        )
        .await
    }

    /// 获取日程列表
    pub async fn get_schedules(&self, channel_id: &str, since: Option<u64>) -> Result<Vec<Schedule>> {
        let path = if let Some(since_val) = since {
            format!("/channels/{}/schedules?since={}", channel_id, since_val)
        } else {
            format!("/channels/{}/schedules", channel_id)
        };

        self.request(reqwest::Method::GET, &path, None::<()>).await
    }

    /// 创建日程
    pub async fn create_schedule(&self, channel_id: &str, schedule: &Schedule) -> Result<Schedule> {
        self.request(
            reqwest::Method::POST,
            &format!("/channels/{}/schedules", channel_id),
            Some(schedule),
        )
        .await
    }

    /// 删除日程
    pub async fn delete_schedule(&self, channel_id: &str, schedule_id: &str) -> Result<()> {
        self.request::<serde_json::Value>(
            reqwest::Method::DELETE,
            &format!("/channels/{}/schedules/{}", channel_id, schedule_id),
            None::<()>,
        )
        .await?;

        Ok(())
    }

    /// 发送流式消息块（C2C 私聊）
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

    /// 发送流式 Markdown 消息块（C2C 私聊）
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

impl Default for MessageRequest {
    fn default() -> Self {
        Self {
            content: None,
            msg_type: None,
            markdown: None,
            keyboard: None,
            ark: None,
            image: None,
            message_reference: None,
            event_id: None,
            msg_id: None,
            msg_seq: None,
            media: None,
            stream: None,
        }
    }
}
