mod config;
mod qqbot;
mod websocket;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser)]
#[command(name = "qq_warning")]
#[command(about = "QQ Bot 命令行消息发送工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// 配置文件路径
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[derive(Subcommand)]
enum Commands {
    /// 发送消息到私聊
    SendUser {
        /// 用户 OpenID
        user_id: String,
        /// 消息内容
        message: String,
        /// 使用 Markdown 格式
        #[arg(short, long)]
        markdown: bool,
    },
    /// 发送消息到群聊
    SendGroup {
        /// 群 OpenID
        group_id: String,
        /// 消息内容
        message: String,
    },
    /// 测试连接
    Test,
    /// 启动 WebSocket 服务（保持连接，接收消息）
    Daemon,
}

fn init_logging(config: &config::Config) {
    let log_level = config.logging.level.as_str();

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    let subscriber = fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false);

    if let Some(log_file) = &config.logging.file {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .expect("无法打开日志文件");

        subscriber
            .with_writer(std::sync::Mutex::new(file))
            .with_ansi(false)
            .init();
    } else {
        subscriber.init();
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
        Commands::SendUser { user_id, message, markdown } => {
            let bot = qqbot::QQBot::new(config);
            if markdown {
                tracing::info!("正在发送 Markdown 消息到用户 {}...", user_id);
                bot.send_user_markdown(&user_id, &message).await?;
            } else {
                tracing::info!("正在发送消息到用户 {}...", user_id);
                bot.send_user_message(&user_id, &message).await?;
            }
            tracing::info!("✓ 消息发送成功");
        }
        Commands::SendGroup { group_id, message } => {
            let bot = qqbot::QQBot::new(config);
            tracing::info!("正在发送消息到群 {}...", group_id);
            bot.send_group_message(&group_id, &message).await?;
            tracing::info!("✓ 消息发送成功");
        }
        Commands::Test => {
            let bot = qqbot::QQBot::new(config);
            tracing::info!("正在测试连接...");
            let token = bot.get_access_token().await?;
            tracing::info!("✓ 连接成功");
            tracing::debug!("Access Token: {}...", &token[..20]);
        }
        Commands::Daemon => {
            tracing::info!("启动 WebSocket 后台服务...");
            let service = websocket::WebSocketService::new(config);
            service.start().await?;
        }
    }

    Ok(())
}
