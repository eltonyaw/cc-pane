use crate::models::{ChannelConfig, NotifyPayload};

/// Slack Webhook：发送 Block Kit 消息
pub fn send(config: &ChannelConfig, payload: &NotifyPayload) -> Result<(), String> {
    let body = serde_json::json!({
        "blocks": [{
            "type": "header",
            "text": { "type": "plain_text", "text": payload.title }
        }, {
            "type": "section",
            "text": { "type": "mrkdwn", "text": payload.body }
        }]
    });

    ureq::post(&config.url)
        .header("Content-Type", "application/json")
        .send(body.to_string().as_bytes())
        .map_err(|e| format!("Slack 发送失败: {}", e))?;

    Ok(())
}
