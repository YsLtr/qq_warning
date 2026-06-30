# QQ Bot 使用示例

本文档包含各种实际使用场景的示例代码。

## 1. 基础消息发送

### 发送简单文本消息

```bash
# 发送私聊消息
qq_warning send-user "user_openid_here" "Hello, World!"

# 发送群消息
qq_warning send-group "group_openid_here" "大家好！"
```

### 发送 Markdown 格式消息

```bash
qq_warning send-user "user_openid" "# 系统报告
## 状态
- ✅ CPU: 正常
- ✅ 内存: 正常
- ⚠️ 磁盘: 85% 使用率

**建议**: 清理日志文件" --markdown
```

### 发送图片

```bash
qq_warning send-user "user_openid" "截图如下" --image "https://example.com/screenshot.png"
```

## 2. 系统监控脚本

### 磁盘空间监控

```bash
#!/bin/bash
# disk_monitor.sh - 监控磁盘使用率

USER_ID="your_user_openid"
THRESHOLD=80

# 获取磁盘使用率
USAGE=$(df -h / | awk 'NR==2 {print $5}' | sed 's/%//')

if [ "$USAGE" -gt "$THRESHOLD" ]; then
    MESSAGE="⚠️ 磁盘空间警告\n当前使用率: ${USAGE}%\n建议及时清理"
    qq_warning send-user "$USER_ID" "$MESSAGE"
fi
```

### CPU 温度监控

```bash
#!/bin/bash
# cpu_temp_monitor.sh

USER_ID="your_user_openid"
TEMP=$(sensors | awk '/Core 0/ {print $3}' | sed 's/+//;s/°C//')

if (( $(echo "$TEMP > 80" | bc -l) )); then
    qq_warning send-user "$USER_ID" "🔥 CPU 温度过高: ${TEMP}°C"
fi
```

### 服务状态检查

```bash
#!/bin/bash
# service_check.sh

USER_ID="your_user_openid"
SERVICES=("nginx" "mysql" "redis")

for service in "${SERVICES[@]}"; do
    if ! systemctl is-active --quiet "$service"; then
        qq_warning send-user "$USER_ID" "⚠️ 服务 $service 未运行！"
        
        # 尝试重启服务
        sudo systemctl restart "$service"
        sleep 3
        
        if systemctl is-active --quiet "$service"; then
            qq_warning send-user "$USER_ID" "✅ 服务 $service 已自动重启"
        else
            qq_warning send-user "$USER_ID" "❌ 服务 $service 重启失败，需要人工介入"
        fi
    fi
done
```

## 3. Python 集成

### 基础封装

```python
import subprocess
import json
from typing import Optional

class QQBot:
    def __init__(self, binary_path: str = "qq_warning"):
        self.binary = binary_path
    
    def send_user_message(self, user_id: str, message: str, 
                         markdown: bool = False, 
                         image: Optional[str] = None) -> bool:
        """发送私聊消息"""
        cmd = [self.binary, "send-user", user_id, message]
        
        if markdown:
            cmd.append("--markdown")
        if image:
            cmd.extend(["--image", image])
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        return result.returncode == 0
    
    def send_group_message(self, group_id: str, message: str) -> bool:
        """发送群消息"""
        cmd = [self.binary, "send-group", group_id, message]
        result = subprocess.run(cmd, capture_output=True, text=True)
        return result.returncode == 0
    
    def recall_message(self, target_type: str, target_id: str, 
                      message_id: str, hidetip: bool = False) -> bool:
        """撤回消息"""
        cmd = [self.binary, "recall", target_type, target_id, message_id]
        if hidetip:
            cmd.append("--hidetip")
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        return result.returncode == 0

# 使用示例
bot = QQBot()

# 发送普通消息
bot.send_user_message("user_id", "Hello from Python!")

# 发送 Markdown 消息
bot.send_user_message("user_id", "# 标题\n**粗体**", markdown=True)

# 发送图片
bot.send_user_message("user_id", "图片", image="https://example.com/img.jpg")
```

### Web 服务器集成

```python
from flask import Flask, request, jsonify
import subprocess

app = Flask(__name__)
BOT_USER_ID = "your_user_openid"

@app.route('/webhook', methods=['POST'])
def webhook():
    """接收 webhook 并发送 QQ 通知"""
    data = request.json
    
    # 构建消息
    message = f"""# {data.get('title', '通知')}
    
**状态**: {data.get('status', 'unknown')}
**时间**: {data.get('timestamp', 'N/A')}
**详情**: {data.get('message', '')}
    """
    
    # 发送通知
    subprocess.run([
        "qq_warning", "send-user", BOT_USER_ID, message, "--markdown"
    ])
    
    return jsonify({"status": "ok"})

@app.errorhandler(500)
def handle_error(error):
    """错误处理，发送通知"""
    subprocess.run([
        "qq_warning", "send-user", BOT_USER_ID,
        f"❌ 服务器错误: {str(error)}"
    ])
    return jsonify({"error": str(error)}), 500

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

### 异步任务通知

```python
import asyncio
import subprocess
from datetime import datetime

async def long_running_task():
    """耗时任务"""
    user_id = "user_openid"
    
    # 任务开始通知
    subprocess.run([
        "qq_warning", "send-user", user_id,
        f"🚀 任务开始: {datetime.now().strftime('%H:%M:%S')}"
    ])
    
    try:
        # 模拟耗时操作
        await asyncio.sleep(10)
        result = "处理完成"
        
        # 成功通知
        subprocess.run([
            "qq_warning", "send-user", user_id,
            f"✅ 任务完成: {result}", "--markdown"
        ])
    except Exception as e:
        # 失败通知
        subprocess.run([
            "qq_warning", "send-user", user_id,
            f"❌ 任务失败: {str(e)}"
        ])

# 运行
asyncio.run(long_running_task())
```

## 4. 日志监控

### 实时日志监控

```bash
#!/bin/bash
# log_monitor.sh - 监控日志文件，发现错误立即通知

USER_ID="your_user_openid"
LOG_FILE="/var/log/myapp.log"

tail -F "$LOG_FILE" | while read line; do
    if echo "$line" | grep -i "error\|exception\|fatal"; then
        qq_warning send-user "$USER_ID" "🔴 日志错误:\n$line"
    fi
done
```

### 定期日志摘要

```bash
#!/bin/bash
# daily_log_summary.sh - 每日日志摘要

USER_ID="your_user_openid"
LOG_FILE="/var/log/nginx/access.log"

# 统计今日访问量
REQUESTS=$(grep "$(date +%d/%b/%Y)" "$LOG_FILE" | wc -l)

# 统计错误数
ERRORS=$(grep "$(date +%d/%b/%Y)" "$LOG_FILE" | grep " 5[0-9][0-9] " | wc -l)

# 发送摘要
MESSAGE="# 每日报告 $(date +%Y-%m-%d)

## 访问统计
- 总请求数: $REQUESTS
- 错误数: $ERRORS
- 错误率: $(echo "scale=2; $ERRORS * 100 / $REQUESTS" | bc)%

---
报告时间: $(date +%H:%M:%S)"

qq_warning send-user "$USER_ID" "$MESSAGE" --markdown
```

## 5. 备份通知

### 数据库备份脚本

```bash
#!/bin/bash
# mysql_backup.sh

USER_ID="your_user_openid"
BACKUP_DIR="/backups/mysql"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="mydb"

# 开始备份通知
qq_warning send-user "$USER_ID" "🔄 开始备份数据库 $DB_NAME"

# 执行备份
mysqldump -u root -p"$DB_PASSWORD" "$DB_NAME" > "$BACKUP_DIR/${DB_NAME}_${DATE}.sql"

if [ $? -eq 0 ]; then
    SIZE=$(du -h "$BACKUP_DIR/${DB_NAME}_${DATE}.sql" | cut -f1)
    qq_warning send-user "$USER_ID" "✅ 数据库备份成功\n文件大小: $SIZE"
else
    qq_warning send-user "$USER_ID" "❌ 数据库备份失败"
fi
```

## 6. Git 仓库监控

### Git Push 通知

```bash
#!/bin/bash
# git_hook.sh - 放在 .git/hooks/post-receive

USER_ID="your_user_openid"
REPO_NAME=$(basename $(pwd) .git)

while read oldrev newrev refname; do
    branch=$(echo $refname | sed 's|refs/heads/||')
    
    # 获取提交信息
    commits=$(git log $oldrev..$newrev --pretty=format:"- %s (%an)" | head -5)
    
    MESSAGE="# 🔔 代码推送通知

**仓库**: $REPO_NAME
**分支**: $branch
**提交数**: $(git log $oldrev..$newrev --oneline | wc -l)

## 最近提交
$commits"
    
    qq_warning send-user "$USER_ID" "$MESSAGE" --markdown
done
```

## 7. 定时任务

### Crontab 配置示例

```cron
# 每小时检查系统状态
0 * * * * /home/user/scripts/system_check.sh

# 每天早上 8 点发送每日报告
0 8 * * * /home/user/scripts/daily_report.sh

# 每 5 分钟检查服务状态
*/5 * * * * /home/user/scripts/service_monitor.sh

# 每天凌晨 2 点备份数据库
0 2 * * * /home/user/scripts/backup_database.sh

# 每周一早上 9 点发送周报
0 9 * * 1 /home/user/scripts/weekly_report.sh
```

## 8. 频道管理示例

### 批量禁言

```bash
#!/bin/bash
# batch_mute.sh - 批量禁言违规用户

GUILD_ID="your_guild_id"
MUTE_SECONDS=3600  # 1小时

# 从文件读取用户 ID
while IFS= read -r user_id; do
    echo "禁言用户: $user_id"
    qq_warning guild mute "$GUILD_ID" "$user_id" --seconds "$MUTE_SECONDS"
    sleep 1  # 避免速率限制
done < users_to_mute.txt
```

### 自动精华消息管理

```bash
#!/bin/bash
# auto_pin.sh - 自动将点赞超过阈值的消息设为精华

CHANNEL_ID="your_channel_id"
THRESHOLD=10

# 假设有 API 获取消息点赞数
# 这里简化为从文件读取
while IFS=',' read -r message_id likes; do
    if [ "$likes" -gt "$THRESHOLD" ]; then
        echo "添加精华: $message_id (点赞: $likes)"
        qq_warning guild pin-add "$CHANNEL_ID" "$message_id"
    fi
done < message_stats.csv
```

## 9. 性能监控仪表板

### 生成性能报告

```bash
#!/bin/bash
# performance_report.sh

USER_ID="your_user_openid"

# 收集系统信息
CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
MEM_USAGE=$(free | grep Mem | awk '{printf("%.1f", $3/$2 * 100)}')
DISK_USAGE=$(df -h / | awk 'NR==2 {print $5}' | sed 's/%//')
UPTIME=$(uptime -p)

# 构建 Markdown 报告
REPORT="# 📊 系统性能报告

**生成时间**: $(date '+%Y-%m-%d %H:%M:%S')

## 资源使用情况

| 项目 | 使用率 | 状态 |
|------|--------|------|
| CPU | ${CPU_USAGE}% | $([ ${CPU_USAGE%.*} -lt 80 ] && echo '✅' || echo '⚠️') |
| 内存 | ${MEM_USAGE}% | $([ ${MEM_USAGE%.*} -lt 80 ] && echo '✅' || echo '⚠️') |
| 磁盘 | ${DISK_USAGE}% | $([ $DISK_USAGE -lt 80 ] && echo '✅' || echo '⚠️') |

**运行时间**: $UPTIME

---
*自动生成*"

qq_warning send-user "$USER_ID" "$REPORT" --markdown
```

## 10. 错误处理和重试

### 带重试的消息发送

```bash
#!/bin/bash
# send_with_retry.sh

send_message_with_retry() {
    local user_id=$1
    local message=$2
    local max_retries=3
    local retry_count=0
    
    while [ $retry_count -lt $max_retries ]; do
        if qq_warning send-user "$user_id" "$message"; then
            echo "消息发送成功"
            return 0
        else
            retry_count=$((retry_count + 1))
            echo "发送失败，重试 $retry_count/$max_retries"
            sleep 2
        fi
    done
    
    echo "消息发送失败，已达到最大重试次数"
    return 1
}

# 使用
send_message_with_retry "user_openid" "重要通知"
```

## 总结

这些示例展示了如何在各种场景中使用 QQ Bot 工具：

1. **系统监控** - 实时监控服务器状态
2. **日志分析** - 自动分析日志并报警
3. **备份通知** - 备份任务完成通知
4. **开发协作** - Git 提交通知
5. **定时任务** - 定期报告和检查
6. **频道管理** - 自动化管理频道
7. **性能监控** - 生成性能报告
8. **错误处理** - 健壮的错误处理机制

你可以根据实际需求修改和组合这些示例。
