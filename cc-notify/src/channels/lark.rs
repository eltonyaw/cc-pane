use crate::models::{ChannelConfig, NotifyPayload};

/// 飞书 Webhook：发送 Interactive Card 消息
pub fn send(config: &ChannelConfig, payload: &NotifyPayload) -> Result<(), String> {
    let body = serde_json::json!({
        "msg_type": "interactive",
        "card": {
            "header": {
                "title": { "tag": "plain_text", "content": payload.title },
                "template": "blue"
            },
            "elements": [{
                "tag": "div",
                "text": { "tag": "lark_md", "content": payload.body }
            }]
        }
    });

    ureq::post(&config.url)
        .header("Content-Type", "application/json")
        .send(body.to_string().as_bytes())
        .map_err(|e| format!("飞书发送失败: {}", e))?;

    Ok(())
}
