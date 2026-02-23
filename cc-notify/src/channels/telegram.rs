use crate::models::{ChannelConfig, NotifyPayload};

/// Telegram Bot API：通过 sendMessage 接口发送消息
pub fn send(config: &ChannelConfig, payload: &NotifyPayload) -> Result<(), String> {
    let token = config
        .token
        .as_ref()
        .ok_or_else(|| "Telegram 需要 bot token".to_string())?;
    let chat_id = config
        .chat_id
        .as_ref()
        .ok_or_else(|| "Telegram 需要 chat_id".to_string())?;

    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let text = format!("*{}*\n{}", payload.title, payload.body);

    let body = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "Markdown"
    });

    ureq::post(&url)
        .header("Content-Type", "application/json")
        .send(body.to_string().as_bytes())
        .map_err(|e| format!("Telegram 发送失败: {}", e))?;

    Ok(())
}
