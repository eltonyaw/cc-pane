use crate::models::{ChannelConfig, NotifyPayload};

/// 钉钉 Webhook：发送 Markdown 消息
pub fn send(config: &ChannelConfig, payload: &NotifyPayload) -> Result<(), String> {
    let body = serde_json::json!({
        "msgtype": "markdown",
        "markdown": {
            "title": payload.title,
            "text": format!("### {}\n{}", payload.title, payload.body)
        }
    });

    ureq::post(&config.url)
        .header("Content-Type", "application/json")
        .send(body.to_string().as_bytes())
        .map_err(|e| format!("钉钉发送失败: {}", e))?;

    Ok(())
}
