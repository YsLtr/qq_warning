mod api;
mod config;
mod qqbot;
mod types;
mod utils;
mod websocket;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser)]
#[command(name = "qq_warning")]
#[command(about = "QQ Bot 命令行工具 - 完整的 QQ 机器人管理工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// 配置文件路径
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[derive(Subcommand)]
enum Commands {
    /// 发送消息
    Send {
        #[command(subcommand)]
        target: SendTarget,
    },
    /// 撤回消息
    Recall {
        /// 目标 ID（用户或群）
        target_id: String,
        /// 消息 ID
        message_id: String,
        /// 是否隐藏撤回提示
        #[arg(short = 't', long)]
        hidetip: bool,
    },
    /// 频道管理
    Guild {
        #[command(subcommand)]
        action: GuildAction,
    },
    /// 测试连接
    Test,
    /// 启动 WebSocket 服务（保持连接，接收消息）
    Daemon,
}

#[derive(Subcommand)]
enum SendTarget {
    /// 发送到指定 ID（自动识别用户/群）
    To {
        /// 目标 ID（用户 OpenID 或群 OpenID）
        target_id: String,
        /// 消息内容
        message: String,
        /// 使用 Markdown 格式
        #[arg(short, long)]
        markdown: bool,
        /// 附加图片 URL
        #[arg(short, long)]
        image: Option<String>,
        /// 流式发送（模拟打字效果）
        #[arg(short, long)]
        stream: bool,
        /// 流式发送：每次发送的字符数
        #[arg(long, default_value = "50")]
        chunk_size: usize,
        /// 流式发送：每块之间的延迟（毫秒）
        #[arg(long, default_value = "300")]
        delay_ms: u64,
    },
    /// 明确发送到用户（C2C）
    User {
        /// 用户 OpenID
        user_id: String,
        /// 消息内容
        message: String,
        /// 使用 Markdown 格式
        #[arg(short, long)]
        markdown: bool,
        /// 附加图片 URL
        #[arg(short, long)]
        image: Option<String>,
        /// 流式发送
        #[arg(short, long)]
        stream: bool,
        #[arg(long, default_value = "50")]
        chunk_size: usize,
        #[arg(long, default_value = "300")]
        delay_ms: u64,
    },
    /// 明确发送到群
    Group {
        /// 群 OpenID
        group_id: String,
        /// 消息内容
        message: String,
        /// 附加图片 URL
        #[arg(short, long)]
        image: Option<String>,
    },
}

#[derive(Subcommand)]
enum GuildAction {
    /// 禁言用户
    Mute {
        /// 频道 ID
        guild_id: String,
        /// 用户 ID
        user_id: String,
        /// 禁言时长（秒）
        #[arg(short, long)]
        seconds: u64,
    },
    /// 禁言全员
    MuteAll {
        /// 频道 ID
        guild_id: String,
        /// 禁言时长（秒）
        #[arg(short, long)]
        seconds: u64,
    },
    /// 添加精华消息
    PinAdd {
        /// 子频道 ID
        channel_id: String,
        /// 消息 ID
        message_id: String,
    },
    /// 删除精华消息
    PinDelete {
        /// 子频道 ID
        channel_id: String,
        /// 消息 ID
        message_id: String,
    },
    /// 查看精华消息列表
    PinList {
        /// 子频道 ID
        channel_id: String,
    },
    /// 创建公告
    AnnounceCreate {
        /// 频道 ID
        guild_id: String,
        /// 子频道 ID
        channel_id: String,
        /// 消息 ID
        message_id: String,
    },
    /// 删除公告
    AnnounceDelete {
        /// 频道 ID
        guild_id: String,
        /// 消息 ID
        message_id: String,
    },
    /// 添加表情反应
    ReactionAdd {
        /// 子频道 ID
        channel_id: String,
        /// 消息 ID
        message_id: String,
        /// 表情类型：1=系统表情 2=emoji
        emoji_type: u8,
        /// 表情 ID
        emoji_id: String,
    },
}

fn init_logging(config: &config::Config) {
    let log_level = config.logging.level.as_str();

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    let has_log_file = config.logging.file.as_ref()
        .map(|f| !f.is_empty())
        .unwrap_or(false);

    if has_log_file {
        let log_file = config.logging.file.as_ref().unwrap();
        // 同时输出到控制台和文件
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .expect("无法打开日志文件");

        let file_layer = fmt::layer()
            .with_writer(std::sync::Mutex::new(file))
            .with_ansi(false);

        let stdout_layer = fmt::layer()
            .with_writer(std::io::stdout);

        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        tracing_subscriber::registry()
            .with(filter)
            .with(file_layer)
            .with(stdout_layer)
            .init();
    } else {
        // 仅输出到控制台
        fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_thread_ids(false)
            .init();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 加载配置
    let config = config::Config::load(&cli.config)?;

    // 初始化日志
    init_logging(&config);

    match cli.command {
        Commands::Send { target } => {
            let bot = qqbot::QQBot::new(config);

            match target {
                SendTarget::To { target_id, message, markdown, image, stream, chunk_size, delay_ms } => {
                    // 自动检测目标类型
                    let is_group = looks_like_group(&target_id);

                    if stream {
                        if is_group {
                            tracing::warn!("群聊不支持流式消息，使用普通发送");
                            bot.send_group_message(&target_id, &message).await?;
                        } else {
                            tracing::info!("正在流式发送消息到 {}...", target_id);
                            bot.send_stream_message(&target_id, &message, chunk_size, delay_ms, markdown).await?;
                        }
                    } else if let Some(img_url) = image {
                        if is_group {
                            tracing::warn!("群消息暂不支持直接发送图片");
                            bot.send_group_message(&target_id, &message).await?;
                        } else {
                            tracing::info!("正在发送图片到 {}...", target_id);
                            bot.send_user_image(&target_id, &img_url).await?;
                        }
                    } else if markdown {
                        if is_group {
                            tracing::warn!("群消息可能不支持 Markdown，尝试发送");
                        }
                        tracing::info!("正在发送 Markdown 消息到 {}...", target_id);
                        bot.send_user_markdown(&target_id, &message).await?;
                    } else {
                        if is_group {
                            tracing::info!("正在发送消息到群 {}...", target_id);
                            bot.send_group_message(&target_id, &message).await?;
                        } else {
                            tracing::info!("正在发送消息到用户 {}...", target_id);
                            bot.send_user_message(&target_id, &message).await?;
                        }
                    }
                    tracing::info!("✓ 消息发送成功");
                }

                SendTarget::User { user_id, message, markdown, image, stream, chunk_size, delay_ms } => {
                    if stream {
                        tracing::info!("正在流式发送消息到用户 {}...", user_id);
                        bot.send_stream_message(&user_id, &message, chunk_size, delay_ms, markdown).await?;
                    } else if let Some(img_url) = image {
                        tracing::info!("正在发送图片到用户 {}...", user_id);
                        bot.send_user_image(&user_id, &img_url).await?;
                    } else if markdown {
                        tracing::info!("正在发送 Markdown 消息到用户 {}...", user_id);
                        bot.send_user_markdown(&user_id, &message).await?;
                    } else {
                        tracing::info!("正在发送消息到用户 {}...", user_id);
                        bot.send_user_message(&user_id, &message).await?;
                    }
                    tracing::info!("✓ 消息发送成功");
                }

                SendTarget::Group { group_id, message, image } => {
                    if image.is_some() {
                        tracing::warn!("群消息暂不支持直接发送图片");
                    }
                    tracing::info!("正在发送消息到群 {}...", group_id);
                    bot.send_group_message(&group_id, &message).await?;
                    tracing::info!("✓ 消息发送成功");
                }
            }
        }

        Commands::Recall { target_id, message_id, hidetip } => {
            let bot = qqbot::QQBot::new(config);
            let is_group = looks_like_group(&target_id);

            if is_group {
                tracing::info!("正在撤回群消息 {}...", message_id);
                bot.recall_group_message(&target_id, &message_id, hidetip).await?;
            } else {
                tracing::info!("正在撤回用户消息 {}...", message_id);
                bot.recall_user_message(&target_id, &message_id, hidetip).await?;
            }
            tracing::info!("✓ 消息撤回成功");
        }

        Commands::Guild { action } => {
            let bot = qqbot::QQBot::new(config);
            match action {
                GuildAction::Mute { guild_id, user_id, seconds } => {
                    tracing::info!("正在禁言用户 {} ({} 秒)...", user_id, seconds);
                    bot.mute_member(&guild_id, &user_id, seconds).await?;
                    tracing::info!("✓ 禁言成功");
                }
                GuildAction::MuteAll { guild_id, seconds } => {
                    tracing::info!("正在禁言全员 ({} 秒)...", seconds);
                    bot.mute_all(&guild_id, seconds).await?;
                    tracing::info!("✓ 全员禁言成功");
                }
                GuildAction::PinAdd { channel_id, message_id } => {
                    tracing::info!("正在添加精华消息 {}...", message_id);
                    let pin = bot.add_pin(&channel_id, &message_id).await?;
                    tracing::info!("✓ 精华消息添加成功");
                    tracing::debug!("精华消息列表: {:?}", pin.message_ids);
                }
                GuildAction::PinDelete { channel_id, message_id } => {
                    tracing::info!("正在删除精华消息 {}...", message_id);
                    bot.delete_pin(&channel_id, &message_id).await?;
                    tracing::info!("✓ 精华消息删除成功");
                }
                GuildAction::PinList { channel_id } => {
                    tracing::info!("正在获取精华消息列表...");
                    let pins = bot.get_pins(&channel_id).await?;
                    tracing::info!("✓ 精华消息列表 ({} 条):", pins.message_ids.len());
                    for (idx, msg_id) in pins.message_ids.iter().enumerate() {
                        tracing::info!("  [{}] {}", idx + 1, msg_id);
                    }
                }
                GuildAction::AnnounceCreate { guild_id, channel_id, message_id } => {
                    tracing::info!("正在创建公告...");
                    let announce = bot.create_announce(&guild_id, &channel_id, &message_id).await?;
                    tracing::info!("✓ 公告创建成功");
                    tracing::debug!("公告 ID: {}", announce.message_id);
                }
                GuildAction::AnnounceDelete { guild_id, message_id } => {
                    tracing::info!("正在删除公告 {}...", message_id);
                    bot.delete_announce(&guild_id, &message_id).await?;
                    tracing::info!("✓ 公告删除成功");
                }
                GuildAction::ReactionAdd { channel_id, message_id, emoji_type, emoji_id } => {
                    tracing::info!("正在添加表情反应...");
                    bot.put_message_reaction(&channel_id, &message_id, emoji_type, &emoji_id).await?;
                    tracing::info!("✓ 表情反应添加成功");
                }
            }
        }

        Commands::Test => {
            let bot = qqbot::QQBot::new(config);
            tracing::info!("正在测试连接...");
            let token = bot.get_access_token().await?;
            tracing::info!("✓ 连接成功");
            tracing::debug!("Access Token: {}...", &token[..20.min(token.len())]);
        }

        Commands::Daemon => {
            tracing::info!("启动 WebSocket 后台服务...");
            let service = websocket::WebSocketService::new(config);
            service.start().await?;
        }
    }

    Ok(())
}

/// 根据 ID 判断是否为群聊
fn looks_like_group(id: &str) -> bool {
    let id_lower = id.trim().to_ascii_lowercase();
    id_lower.starts_with("group_")
        || id_lower.starts_with("grp_")
        || id_lower.starts_with("qqgroup_")
        || id_lower.contains("group")
}
