use crate::models::{ChannelConfig, NotifyPayload};

/// 通用 Webhook：POST JSON 到指定 URL
pub fn send(config: &ChannelConfig, payload: &NotifyPayload) -> Result<(), String> {
    let body = serde_json::to_string(payload).map_err(|e| format!("序列化失败: {}", e))?;

    let mut req = ureq::post(&config.url).header("Content-Type", "application/json");

    if let Some(ref token) = config.token {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }

    req.send(body.as_bytes())
        .map_err(|e| format!("Webhook 发送失败: {}", e))?;

    Ok(())
}
