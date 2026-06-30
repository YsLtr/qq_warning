# QQ Bot API 功能清单

本文档列出了工具已实现的所有 QQ Bot API 功能。

## ✅ 已实现功能

### 消息 API

#### 发送消息
- ✅ `POST /v2/users/{user_openid}/messages` - 发送私聊消息
  - 支持文本消息 (msg_type: 0)
  - 支持 Markdown 消息 (msg_type: 2)
  - 支持 Ark 模板消息 (msg_type: 3)
  - 支持图片消息 (msg_type: 7)
  - 支持键盘组件 (Keyboard)
  - 支持"正在输入"提示 (msg_type: 6)
  
- ✅ `POST /v2/groups/{group_openid}/messages` - 发送群消息
  - 支持文本消息
  - 支持"正在输入"提示

#### 消息管理
- ✅ `DELETE /v2/users/{user_openid}/messages/{message_id}` - 撤回私聊消息
  - 支持隐藏撤回提示 (hidetip 参数)
  
- ✅ `DELETE /v2/groups/{group_openid}/messages/{message_id}` - 撤回群消息
  - 支持隐藏撤回提示

### 频道管理 API

#### 成员管理
- ✅ `GET /guilds/{guild_id}/members/{user_id}` - 获取频道成员信息
- ✅ `PATCH /guilds/{guild_id}/members/{user_id}/mute` - 禁言指定成员
  - 支持指定禁言时长（秒）
  
- ✅ `PATCH /guilds/{guild_id}/mute` - 全员禁言
  - 支持指定禁言时长

#### 身份组管理
- ✅ `GET /guilds/{guild_id}/roles` - 获取频道身份组列表
- ✅ `PUT /guilds/{guild_id}/members/{user_id}/roles/{role_id}` - 添加身份组成员
  - 支持指定子频道范围

### 消息管理 API

#### 精华消息
- ✅ `PUT /channels/{channel_id}/pins/{message_id}` - 添加精华消息
- ✅ `DELETE /channels/{channel_id}/pins/{message_id}` - 删除精华消息
- ✅ `GET /channels/{channel_id}/pins` - 获取精华消息列表

#### 公告
- ✅ `POST /guilds/{guild_id}/announces` - 创建公告
  - 指定消息 ID 和子频道
  
- ✅ `DELETE /guilds/{guild_id}/announces/{message_id}` - 删除公告

### 表情反应 API
- ✅ `PUT /channels/{channel_id}/messages/{message_id}/reactions/{type}/{id}` - 添加表情反应
  - 支持系统表情 (type: 1)
  - 支持 emoji 表情 (type: 2)
  
- ✅ `DELETE /channels/{channel_id}/messages/{message_id}/reactions/{type}/{id}` - 删除表情反应

### 日程 API
- ✅ `GET /channels/{channel_id}/schedules` - 获取日程列表
  - 支持 since 参数过滤
  
- ✅ `POST /channels/{channel_id}/schedules` - 创建日程
- ✅ `DELETE /channels/{channel_id}/schedules/{schedule_id}` - 删除日程

### WebSocket API
- ✅ `GET /gateway` - 获取 WebSocket Gateway URL
- ✅ WebSocket 连接和事件订阅
  - Opcode 10: Hello - 接收心跳间隔
  - Opcode 2: Identify - 鉴权认证
  - Opcode 1: Heartbeat - 心跳保活
  - Opcode 11: Heartbeat ACK - 心跳确认
  - Opcode 0: Dispatch - 事件分发
  - Opcode 7: Reconnect - 重连请求
  - Opcode 9: Invalid Session - 会话无效

#### 支持的事件类型
- ✅ `READY` - 机器人就绪
- ✅ `MESSAGE_CREATE` - 频道消息
- ✅ `C2C_MESSAGE_CREATE` - 私聊消息
- ✅ `GROUP_AT_MESSAGE_CREATE` - 群聊 @ 消息

## 命令行映射

### 消息相关

| 命令 | API 端点 | 说明 |
|------|---------|------|
| `send-user <id> <msg>` | POST /v2/users/{id}/messages | 发送私聊文本消息 |
| `send-user <id> <msg> --markdown` | POST /v2/users/{id}/messages | 发送 Markdown 消息 |
| `send-user <id> <msg> --image <url>` | POST /v2/users/{id}/messages | 发送图片消息 |
| `send-group <id> <msg>` | POST /v2/groups/{id}/messages | 发送群消息 |
| `recall user <id> <msg_id>` | DELETE /v2/users/{id}/messages/{msg_id} | 撤回私聊消息 |
| `recall group <id> <msg_id>` | DELETE /v2/groups/{id}/messages/{msg_id} | 撤回群消息 |

### 频道管理

| 命令 | API 端点 | 说明 |
|------|---------|------|
| `guild mute <gid> <uid> --seconds <s>` | PATCH /guilds/{gid}/members/{uid}/mute | 禁言用户 |
| `guild mute-all <gid> --seconds <s>` | PATCH /guilds/{gid}/mute | 全员禁言 |
| `guild pin-add <cid> <mid>` | PUT /channels/{cid}/pins/{mid} | 添加精华 |
| `guild pin-delete <cid> <mid>` | DELETE /channels/{cid}/pins/{mid} | 删除精华 |
| `guild pin-list <cid>` | GET /channels/{cid}/pins | 查看精华列表 |
| `guild announce-create <gid> <cid> <mid>` | POST /guilds/{gid}/announces | 创建公告 |
| `guild announce-delete <gid> <mid>` | DELETE /guilds/{gid}/announces/{mid} | 删除公告 |
| `guild reaction-add <cid> <mid> <type> <id>` | PUT /channels/{cid}/messages/{mid}/reactions/{type}/{id} | 添加表情 |

### 其他

| 命令 | 功能 | 说明 |
|------|------|------|
| `test` | 认证测试 | 测试 API 连接和 Token 获取 |
| `daemon` | WebSocket 服务 | 启动长连接，接收实时消息 |

## 高级功能

### Token 管理
- ✅ 自动获取 Access Token
- ✅ Token 缓存（提前 60 秒刷新）
- ✅ 过期自动刷新

### 速率限制
- ✅ 消息发送速率限制（可配置）
- ✅ 自动排队等待

### WebSocket 功能
- ✅ 自动心跳（间隔的 80%）
- ✅ 断线自动重连（5 秒延迟）
- ✅ Session 状态管理
- ✅ Sequence 序列号追踪

### 消息处理
- ✅ 自动下载媒体文件（可配置）
- ✅ 桌面通知（Linux/macOS，可配置）
- ✅ 消息类型识别（文本/Markdown/富媒体）
- ✅ 附件信息显示（文件名、大小、类型）

### 配置管理
- ✅ TOML 配置文件
- ✅ 日志级别配置
- ✅ 日志文件输出
- ✅ 通知开关配置
- ✅ 媒体自动下载配置

## 类型支持

### 消息类型
- ✅ MessageType::Text (0) - 文本消息
- ✅ MessageType::Markdown (2) - Markdown 消息
- ✅ MessageType::Ark (3) - Ark 模板消息
- ✅ MessageType::Embed (4) - Embed 富文本
- ✅ MessageType::Media (7) - 媒体消息

### 复杂类型
- ✅ Markdown - Markdown 内容和自定义模板
- ✅ Keyboard - 按钮键盘组件
- ✅ Button - 按钮定义（链接/回调/命令）
- ✅ Ark - Ark 模板消息
- ✅ MessageReference - 消息引用
- ✅ MessageReaction - 表情反应
- ✅ User - 用户信息
- ✅ Member - 成员信息
- ✅ Role - 身份组
- ✅ Channel - 频道信息
- ✅ Schedule - 日程
- ✅ Announce - 公告
- ✅ PinsMessage - 精华消息

## 待实现功能

以下功能类型定义已完成，命令行接口待实现：

### 频道相关
- ⏳ 创建频道
- ⏳ 修改频道信息
- ⏳ 删除频道
- ⏳ 获取子频道列表
- ⏳ 创建子频道
- ⏳ 修改子频道权限

### 身份组相关
- ⏳ 创建身份组
- ⏳ 修改身份组
- ⏳ 删除身份组
- ⏳ 移除身份组成员

### 消息相关
- ⏳ 获取消息详情
- ⏳ 发送 Embed 富文本消息
- ⏳ 群消息图片发送支持

### 日程相关
- ⏳ 修改日程

### 音频相关
- ⏳ 音频控制 API（需要音频机器人权限）

### 事件订阅
- ⏳ 更多事件类型支持
- ⏳ 成员变更事件
- ⏳ 频道变更事件
- ⏳ 消息审核事件

## 参考文档

- [QQ Bot API 官方文档](https://bot.q.qq.com/wiki/)
- [WebSocket Gateway](https://bot.q.qq.com/wiki/develop/api/gateway/reference.html)
- [消息 API](https://bot.q.qq.com/wiki/develop/api/openapi/message/model.html)
- [频道管理 API](https://bot.q.qq.com/wiki/develop/api/openapi/guild/model.html)

## 版本历史

### v0.1.0 (当前版本)
- ✅ 基础消息发送（私聊/群聊）
- ✅ Markdown 消息支持
- ✅ 图片消息支持
- ✅ 消息撤回
- ✅ 频道管理（禁言、精华、公告、表情）
- ✅ WebSocket 实时消息接收
- ✅ 自动下载媒体文件
- ✅ 桌面通知
- ✅ 完整的类型定义
- ✅ 速率限制和重连机制

## 贡献指南

如需添加新功能：

1. 在 `src/types.rs` 中定义相关类型
2. 在 `src/api.rs` 中实现 API 调用
3. 在 `src/qqbot.rs` 中封装高级接口
4. 在 `src/main.rs` 中添加命令行选项
5. 更新文档和示例

## 技术债务

- [ ] 添加单元测试
- [ ] 添加集成测试
- [ ] 完善错误处理和用户提示
- [ ] 支持配置文件热重载
- [ ] 添加消息队列支持
- [ ] 支持多机器人实例
- [ ] Web 管理界面
- [ ] 插件系统
