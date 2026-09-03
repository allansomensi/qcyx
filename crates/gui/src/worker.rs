use crate::message::Message;
use iced::futures::{self, SinkExt, channel::mpsc::Sender};
use qcyx_core::error::CoreError;
use qcyx_core::session;
use std::time::Duration;
use tokio::time;

/// Actively attempts to establish the shared, persistent BLE session.
pub fn connect_poll() -> impl futures::Stream<Item = Message> {
    iced::stream::channel(1, |mut output: Sender<Message>| async move {
        loop {
            match session::ensure_connected().await {
                Ok(info) => {
                    let _ = output.send(Message::Connected(info)).await;
                    break;
                }
                Err(CoreError::DeviceNotFound) => continue,
                Err(e) => {
                    let _ = output.send(Message::ConnectionError(e.to_string())).await;
                    break;
                }
            }
        }
    })
}

/// Periodically re-reads the battery status.
pub fn battery_poll() -> impl futures::Stream<Item = Message> {
    iced::stream::channel(1, |mut output: Sender<Message>| async move {
        loop {
            let result = session::read_battery().await.map_err(|e| e.to_string());
            if output.send(Message::BatteryResult(result)).await.is_err() {
                break;
            }
            time::sleep(Duration::from_secs(30)).await;
        }
    })
}

/// Periodically checks that the persistent connection is still alive.
pub fn watch_disconnect() -> impl futures::Stream<Item = Message> {
    iced::stream::channel(1, |mut output: Sender<Message>| async move {
        let mut interval = time::interval(Duration::from_secs(2));

        loop {
            interval.tick().await;

            if !session::is_connected().await {
                let _ = output.send(Message::Disconnected).await;
                break;
            }
        }
    })
}
