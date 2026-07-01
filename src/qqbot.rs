use crate::api::QQBotApi;
use crate::config::Config;
use crate::types::*;
use anyhow::Result;
use std::time::Duration;

/// QQBot 高级封装
pub struct QQBot {
    api: QQBotApi,
}

impl QQBot {
    pub fn new(config: Config) -> Self {
        Self {
            api: QQBotApi::new(config),
        }
    }

    /// 获取 access token
    pub async fn get_access_token(&self) -> Result<String> {
        self.api.get_access_token().await
    }

    /// 发送文本消息到用户
    pub async fn send_user_message(&self, user_openid: &str, content: &str) -> Result<String> {
        let resp = self.api.send_user_message(user_openid, content).await?;
        Ok(resp.id)
    }

    /// 发送消息到群聊
    pub async fn send_group_message(&self, group_openid: &str, content: &str) -> Result<String> {
        let resp = self.api.send_group_message(group_openid, content).await?;
        Ok(resp.id)
    }

    /// 被动回复群消息（携带 msg_id，5 分钟内有效，无需主动消息权限）
    pub async fn reply_group_message(
        &self,
        group_openid: &str,
        content: &str,
        msg_id: &str,
    ) -> Result<String> {
        let resp = self
            .api
            .reply_group_message(group_openid, content, msg_id)
            .await?;
        Ok(resp.id)
    }

    /// 发送 Markdown 消息到用户（简化版）
    pub async fn send_user_markdown(&self, user_openid: &str, content: &str) -> Result<String> {
        let markdown = Markdown {
            content: content.to_string(),
            custom_template_id: None,
            params: None,
        };
        let resp = self.api.send_user_markdown(user_openid, &markdown).await?;
        Ok(resp.id)
    }

    /// 发送图片到用户
    pub async fn send_user_image(&self, user_openid: &str, image_url: &str) -> Result<String> {
        let resp = self.api.send_user_image(user_openid, image_url).await?;
        Ok(resp.id)
    }

    /// 发送带按钮的消息
    pub async fn send_user_message_with_buttons(
        &self,
        user_openid: &str,
        content: &str,
        buttons: Vec<(String, String, ButtonAction)>,
    ) -> Result<String> {
        let keyboard = self.build_keyboard(buttons);
        let resp = self.api.send_user_message_with_keyboard(user_openid, content, &keyboard).await?;
        Ok(resp.id)
    }

    /// 构建键盘
    fn build_keyboard(&self, buttons: Vec<(String, String, ButtonAction)>) -> Keyboard {
        let buttons: Vec<Button> = buttons
            .into_iter()
            .enumerate()
            .map(|(idx, (label, data, action))| {
                let (action_type, action_data) = match action {
                    ButtonAction::Link(url) => (0, Some(url)),
                    ButtonAction::Callback => (1, Some(data.clone())),
                    ButtonAction::Command => (2, Some(data.clone())),
                };

                Button {
                    id: format!("btn_{}", idx),
                    render_data: RenderData {
                        label: label.clone(),
                        visited_label: label,
                        style: 1, // 蓝色
                    },
                    action: Action {
                        action_type,
                        permission: Permission {
                            permission_type: 2, // 所有人
                            specify_role_ids: None,
                            specify_user_ids: None,
                        },
                        data: action_data,
                        reply: Some(false),
                        enter: Some(true),
                    },
                }
            })
            .collect();

        let rows: Vec<KeyboardRow> = buttons
            .chunks(2)
            .map(|chunk| KeyboardRow {
                buttons: chunk.to_vec(),
            })
            .collect();

        Keyboard {
            id: None,
            content: KeyboardContent { rows },
        }
    }

    /// 撤回用户消息
    pub async fn recall_user_message(&self, user_openid: &str, message_id: &str, hidetip: bool) -> Result<()> {
        self.api.recall_user_message(user_openid, message_id, hidetip).await
    }

    /// 撤回群消息
    pub async fn recall_group_message(&self, group_openid: &str, message_id: &str, hidetip: bool) -> Result<()> {
        self.api.recall_group_message(group_openid, message_id, hidetip).await
    }

    /// 禁言用户
    pub async fn mute_member(&self, guild_id: &str, user_id: &str, seconds: u64) -> Result<()> {
        self.api.mute_member(guild_id, user_id, seconds).await
    }

    /// 禁言全员
    pub async fn mute_all(&self, guild_id: &str, seconds: u64) -> Result<()> {
        self.api.mute_all(guild_id, seconds).await
    }

    /// 添加表情反应
    pub async fn put_message_reaction(&self, channel_id: &str, message_id: &str, emoji_type: u8, emoji_id: &str) -> Result<()> {
        self.api.put_message_reaction(channel_id, message_id, emoji_type, emoji_id).await
    }

    /// 删除表情反应
    pub async fn delete_message_reaction(&self, channel_id: &str, message_id: &str, emoji_type: u8, emoji_id: &str) -> Result<()> {
        self.api.delete_message_reaction(channel_id, message_id, emoji_type, emoji_id).await
    }

    /// 创建公告
    pub async fn create_announce(&self, guild_id: &str, channel_id: &str, message_id: &str) -> Result<Announce> {
        self.api.create_announce(guild_id, channel_id, message_id).await
    }

    /// 删除公告
    pub async fn delete_announce(&self, guild_id: &str, message_id: &str) -> Result<()> {
        self.api.delete_announce(guild_id, message_id).await
    }

    /// 添加精华消息
    pub async fn add_pin(&self, channel_id: &str, message_id: &str) -> Result<PinsMessage> {
        self.api.add_pin(channel_id, message_id).await
    }

    /// 删除精华消息
    pub async fn delete_pin(&self, channel_id: &str, message_id: &str) -> Result<()> {
        self.api.delete_pin(channel_id, message_id).await
    }

    /// 获取精华消息列表
    pub async fn get_pins(&self, channel_id: &str) -> Result<PinsMessage> {
        self.api.get_pins(channel_id).await
    }

    /// 获取日程列表
    pub async fn get_schedules(&self, channel_id: &str, since: Option<u64>) -> Result<Vec<Schedule>> {
        self.api.get_schedules(channel_id, since).await
    }

    /// 创建日程
    pub async fn create_schedule(&self, channel_id: &str, schedule: &Schedule) -> Result<Schedule> {
        self.api.create_schedule(channel_id, schedule).await
    }

    /// 删除日程
    pub async fn delete_schedule(&self, channel_id: &str, schedule_id: &str) -> Result<()> {
        self.api.delete_schedule(channel_id, schedule_id).await
    }

    /// 获取频道成员信息
    pub async fn get_guild_member(&self, guild_id: &str, user_id: &str) -> Result<Member> {
        self.api.get_guild_member(guild_id, user_id).await
    }

    /// 获取频道身份组列表
    pub async fn get_guild_roles(&self, guild_id: &str) -> Result<Vec<Role>> {
        self.api.get_guild_roles(guild_id).await
    }

    /// 添加频道身份组成员
    pub async fn add_guild_role_member(
        &self,
        guild_id: &str,
        role_id: &str,
        user_id: &str,
        channel_id: Option<&str>,
    ) -> Result<()> {
        self.api.add_guild_role_member(guild_id, role_id, user_id, channel_id).await
    }

    /// 发送流式消息（模拟打字效果）
    pub async fn send_stream_message(
        &self,
        user_openid: &str,
        content: &str,
        chunk_size: usize,
        delay_ms: u64,
        markdown: bool,
    ) -> Result<()> {
        let chars: Vec<char> = content.chars().collect();
        let total_len = chars.len();

        if total_len == 0 {
            return Ok(());
        }

        let mut accumulated = String::new();
        let mut last_send_pos = 0;

        for (idx, chunk) in chars.chunks(chunk_size).enumerate() {
            let chunk_text: String = chunk.iter().collect();
            accumulated.push_str(&chunk_text);
            last_send_pos += chunk.len();

            let is_final = last_send_pos >= total_len;

            // 准备发送的内容（限制长度）
            let mut send_content = accumulated.chars().take(4096).collect::<String>();

            // 最终块添加换行，确保渲染完整
            if is_final && !send_content.ends_with('\n') {
                send_content.push('\n');
            }

            if markdown {
                self.api.send_stream_markdown_chunk(
                    user_openid,
                    &send_content,
                    is_final
                ).await?;
            } else {
                self.api.send_stream_chunk(
                    user_openid,
                    &send_content,
                    is_final
                ).await?;
            }

            tracing::debug!("发送流式块 {} ({}/{}) chars={}", idx, last_send_pos, total_len, send_content.len());

            // 延迟下一块，给服务器和客户端足够的处理时间
            if !is_final {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }

        Ok(())
    }

    /// 取消流式消息
    pub async fn cancel_stream(&self, user_openid: &str) -> Result<()> {
        self.api.cancel_stream(user_openid).await
    }

    /// 获取指定的频道消息（单条）
    /// 注意：QQ Bot API 不支持批量获取历史消息，只能获取已知 message_id 的单条消息
    pub async fn get_channel_message(&self, channel_id: &str, message_id: &str) -> Result<Message> {
        self.api.get_channel_message(channel_id, message_id).await
    }

    /// 获取指定的频道私信消息（单条）
    pub async fn get_dms_message(&self, guild_id: &str, message_id: &str) -> Result<Message> {
        self.api.get_dms_message(guild_id, message_id).await
    }
}

/// 按钮动作类型
pub enum ButtonAction {
    Link(String),
    Callback,
    Command,
}
