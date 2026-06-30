use anyhow::Result;
use std::path::Path;

/// 发送桌面通知
pub fn send_notification(title: &str, body: &str, enabled: bool, _sound: bool) -> Result<()> {
    if !enabled {
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        use notify_rust::Notification;
        Notification::new()
            .summary(title)
            .body(body)
            .show()?;
    }

    #[cfg(target_os = "windows")]
    {
        // Windows 通知暂时使用日志替代
        tracing::info!("通知: {} - {}", title, body);
    }

    Ok(())
}

/// 下载文件
pub async fn download_file(url: &str, save_path: &Path) -> Result<()> {
    tracing::info!("开始下载文件: {} -> {:?}", url, save_path);

    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        anyhow::bail!("下载失败: HTTP {}", response.status());
    }

    let bytes = response.bytes().await?;

    // 确保目录存在
    if let Some(parent) = save_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::write(save_path, bytes).await?;

    tracing::info!("文件下载完成: {:?}", save_path);

    Ok(())
}

/// 格式化文件大小
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}
