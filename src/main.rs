mod config;
mod qqbot;
mod websocket;

use anyhow::Result;
use clap::{Parser, Subcommand};

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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 加载配置
    let config = config::Config::load(&cli.config)?;

    match cli.command {
        Commands::SendUser { user_id, message } => {
            let bot = qqbot::QQBot::new(config);
            println!("正在发送消息到用户 {}...", user_id);
            bot.send_user_message(&user_id, &message).await?;
            println!("✓ 消息发送成功");
        }
        Commands::SendGroup { group_id, message } => {
            let bot = qqbot::QQBot::new(config);
            println!("正在发送消息到群 {}...", group_id);
            bot.send_group_message(&group_id, &message).await?;
            println!("✓ 消息发送成功");
        }
        Commands::Test => {
            let bot = qqbot::QQBot::new(config);
            println!("正在测试连接...");
            let token = bot.get_access_token().await?;
            println!("✓ 连接成功");
            println!("Access Token: {}...", &token[..20]);
        }
        Commands::Daemon => {
            println!("启动 WebSocket 后台服务...");
            let service = websocket::WebSocketService::new(config);
            service.start().await?;
        }
    }

    Ok(())
}
