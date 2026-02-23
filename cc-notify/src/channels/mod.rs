pub mod dingtalk;
pub mod lark;
pub mod slack;
pub mod telegram;
pub mod webhook;

use crate::models::{ChannelConfig, ChannelType, NotifyPayload, SendResult};

/// 发送通知到指定渠道
pub fn send_to_channel(config: &ChannelConfig, payload: &NotifyPayload) -> SendResult {
    let result = match config.channel_type {
        ChannelType::Webhook => webhook::send(config, payload),
        ChannelType::Telegram => telegram::send(config, payload),
        ChannelType::Dingtalk => dingtalk::send(config, payload),
        ChannelType::Lark => lark::send(config, payload),
        ChannelType::Slack => slack::send(config, payload),
    };

    match result {
        Ok(()) => SendResult {
            channel_id: config.id.clone(),
            success: true,
            error: None,
        },
        Err(e) => SendResult {
            channel_id: config.id.clone(),
            success: false,
            error: Some(e),
        },
    }
}
