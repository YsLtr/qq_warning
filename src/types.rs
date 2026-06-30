use serde::{Deserialize, Serialize};

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Text = 0,
    Markdown = 2,
    Ark = 3,
    Embed = 4,
    Media = 7,
}

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

/// 媒体类型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Media {
    pub file_info: String,
}

/// Ark 模板消息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ark {
    pub template_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kv: Option<Vec<ArkKv>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArkKv {
    pub key: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obj: Option<Vec<ArkObj>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArkObj {
    #[serde(rename = "objKv")]
    pub obj_kv: Vec<ArkObjKv>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArkObjKv {
    pub key: String,
    pub value: String,
}

/// Embed 富文本消息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Embed {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<MessageEmbedThumbnail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<MessageEmbedField>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageEmbedThumbnail {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageEmbedField {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Markdown 消息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Markdown {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<MarkdownParam>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarkdownParam {
    pub key: String,
    pub values: Vec<String>,
}

/// 消息引用
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageReference {
    pub message_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_get_message_error: Option<bool>,
}

/// 键盘组件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Keyboard {
    pub id: Option<String>,
    pub content: KeyboardContent,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyboardContent {
    pub rows: Vec<KeyboardRow>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyboardRow {
    pub buttons: Vec<Button>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Button {
    pub id: String,
    pub render_data: RenderData,
    pub action: Action,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenderData {
    pub label: String,
    pub visited_label: String,
    pub style: u8, // 0: 灰色, 1: 蓝色
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: u8, // 0: http/https, 1: 回调, 2: 指令
    pub permission: Permission,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enter: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Permission {
    #[serde(rename = "type")]
    pub permission_type: u8, // 0: 指定用户, 1: 管理员, 2: 所有人, 3: 指定身份组
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specify_role_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specify_user_ids: Option<Vec<String>>,
}

/// 表情回应类型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageReaction {
    pub emoji_id: String,
    pub emoji_type: u8, // 1: 系统表情, 2: emoji表情
}

/// 用户信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub union_openid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub union_user_account: Option<String>,
}

/// 成员信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Member {
    pub user: Option<User>,
    pub nick: Option<String>,
    pub roles: Vec<String>,
    pub joined_at: String,
}

/// 频道信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub id: String,
    pub guild_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: u8,
    pub sub_type: u8,
    pub position: u32,
    pub owner_id: String,
    pub private_type: u8,
}

/// 子频道权限
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelPermissions {
    pub channel_id: String,
    pub user_id: Option<String>,
    pub role_id: Option<String>,
    pub permissions: String,
}

/// 身份组
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub color: u32,
    pub hoist: u8,
    pub number: u32,
    pub member_limit: u32,
}

/// 禁言对象
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MuteOptions {
    pub mute_end_timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute_seconds: Option<String>,
}

/// 公告
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Announce {
    pub guild_id: String,
    pub channel_id: String,
    pub message_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub announces_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommend_channels: Option<Vec<RecommendChannel>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecommendChannel {
    pub channel_id: String,
    pub introduce: String,
}

/// 精华消息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PinsMessage {
    pub guild_id: String,
    pub channel_id: String,
    pub message_ids: Vec<String>,
}

/// 日程
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Schedule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub start_timestamp: String,
    pub end_timestamp: String,
    pub creator: Member,
    pub jump_channel_id: String,
    pub remind_type: String,
}

/// API 响应包装
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    #[serde(flatten)]
    pub data: T,
}

/// 消息撤回原因
#[derive(Debug, Serialize)]
pub struct MessageDelete {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidetip: Option<bool>,
}
