use crate::channels;
use crate::models::{ChannelConfig, NotifyPayload};
use std::sync::mpsc;
use std::thread;

/// 内部消息类型
enum Message {
    Send {
        channels: Vec<ChannelConfig>,
        payload: NotifyPayload,
    },
    Stop,
}

/// 后台通知发送器
///
/// 通过 mpsc channel 发送消息到后台线程，fire-and-forget 模式。
/// 不阻塞调用线程，适合在同步代码中使用。
pub struct NotifySender {
    tx: mpsc::Sender<Message>,
}

impl NotifySender {
    /// 创建新的 NotifySender，启动后台工作线程
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Message>();

        thread::Builder::new()
            .name("cc-notify-worker".to_string())
            .spawn(move || {
                Self::worker(rx);
            })
            .expect("Failed to spawn notify worker thread");

        Self { tx }
    }

    /// 异步发送通知（非阻塞，fire-and-forget）
    pub fn send(&self, channels: Vec<ChannelConfig>, payload: NotifyPayload) {
        let _ = self.tx.send(Message::Send { channels, payload });
    }

    /// 停止后台线程
    pub fn stop(&self) {
        let _ = self.tx.send(Message::Stop);
    }

    /// 后台工作线程：循环接收消息并分发到各渠道
    fn worker(rx: mpsc::Receiver<Message>) {
        while let Ok(msg) = rx.recv() {
            match msg {
                Message::Send { channels, payload } => {
                    for config in &channels {
                        if config.enabled {
                            let result = channels::send_to_channel(config, &payload);
                            if !result.success {
                                eprintln!(
                                    "[cc-notify] 发送失败 channel={} error={:?}",
                                    result.channel_id, result.error
                                );
                            }
                        }
                    }
                }
                Message::Stop => break,
            }
        }
    }
}

impl Default for NotifySender {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for NotifySender {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sender_creation_and_stop() {
        let sender = NotifySender::new();
        sender.stop();
        // 验证 sender 可以正常创建和停止，不 panic
    }

    #[test]
    fn test_sender_send_to_no_channels() {
        let sender = NotifySender::new();
        let payload = NotifyPayload {
            event: crate::models::NotifyEvent::TaskCompleted,
            title: "Test".to_string(),
            body: "Test body".to_string(),
            workspace_name: None,
            project_path: None,
            session_id: None,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };
        // 发送到空渠道列表应该不 panic
        sender.send(vec![], payload);
        sender.stop();
    }
}
