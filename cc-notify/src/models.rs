use serde::{Deserialize, Serialize};

/// 通知事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyEvent {
    TerminalWaiting,
    TerminalExited,
    TaskCompleted,
    Custom(String),
}

/// 通知载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyPayload {
    pub event: NotifyEvent,
    pub title: String,
    pub body: String,
    pub workspace_name: Option<String>,
    pub project_path: Option<String>,
    pub session_id: Option<String>,
    pub timestamp: String, // RFC3339
}

/// 渠道类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    Webhook,
    Telegram,
    Dingtalk,
    Lark,
    Slack,
}

/// 渠道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub id: String,
    pub channel_type: ChannelType,
    pub name: String,
    pub url: String,
    pub token: Option<String>,
    pub chat_id: Option<String>,
    pub enabled: bool,
}

/// 发送结果
#[derive(Debug)]
pub struct SendResult {
    pub channel_id: String,
    pub success: bool,
    pub error: Option<String>,
}
