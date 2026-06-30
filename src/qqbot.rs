use crate::config::Config;
use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
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

pub struct QQBot {
    config: Config,
    client: Client,
    token_cache: Arc<Mutex<Option<TokenCache>>>,
    last_send_time: Arc<Mutex<Option<Instant>>>,
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
    msg_type: u8,
    content: String,
    msg_seq: i64,
}

impl QQBot {
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

    /// 发送消息到用户 (C2C)
    pub async fn send_user_message(&self, user_openid: &str, content: &str) -> Result<()> {
        self.apply_rate_limit().await;

        let token = self.get_access_token().await?;
        let url = format!(
            "{}/v2/users/{}/messages",
            self.config.api.base_url, user_openid
        );

        let msg = MessageRequest {
            msg_type: 0,
            content: content.to_string(),
            msg_seq: Self::next_msg_seq(user_openid),
        };

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("QQBot {}", token))
            .header("Content-Type", "application/json")
            .json(&msg)
            .send()
            .await
            .context("发送用户消息失败")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("消息发送失败 ({}): {}", status, body);
        }

        Ok(())
    }

    /// 发送消息到群聊
    pub async fn send_group_message(&self, group_openid: &str, content: &str) -> Result<()> {
        self.apply_rate_limit().await;

        let token = self.get_access_token().await?;
        let url = format!(
            "{}/v2/groups/{}/messages",
            self.config.api.base_url, group_openid
        );

        let msg = MessageRequest {
            msg_type: 0,
            content: content.to_string(),
            msg_seq: Self::next_msg_seq(group_openid),
        };

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("QQBot {}", token))
            .header("Content-Type", "application/json")
            .json(&msg)
            .send()
            .await
            .context("发送群消息失败")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("消息发送失败 ({}): {}", status, body);
        }

        Ok(())
    }
}
