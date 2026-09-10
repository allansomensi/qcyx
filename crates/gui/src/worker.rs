use crate::message::Message;
use iced::futures::{self, SinkExt, channel::mpsc::Sender};
use qcyx_core::error::CoreError;
use qcyx_core::session;
use std::time::Duration;
use tokio::time;

/// First delay after a connection attempt finds nothing.
const RECONNECT_BACKOFF_MIN: Duration = Duration::from_secs(1);

/// Ceiling on the retry delay.
const RECONNECT_BACKOFF_MAX: Duration = Duration::from_secs(30);

/// Actively attempts to establish the shared, persistent BLE session.
///
/// Retries with exponential backoff. The scan now resolves as soon as an
/// advertisement arrives instead of always burning five seconds, so an
/// unthrottled retry loop would hammer the adapter continuously — degrading BLE
/// for the whole system — while the earbuds sit in their case.
pub fn connect_poll() -> impl futures::Stream<Item = Message> {
    iced::stream::channel(1, |mut output: Sender<Message>| async move {
        let mut backoff = RECONNECT_BACKOFF_MIN;

        loop {
            match session::ensure_connected().await {
                Ok(info) => {
                    let _ = output.send(Message::Connected(info)).await;
                    break;
                }
                // Retryable: the device simply isn't advertising yet, or the
                // stack was briefly unresponsive.
                Err(CoreError::DeviceNotFound | CoreError::OperationTimeout(_)) => {
                    time::sleep(backoff).await;
                    backoff = (backoff * 2).min(RECONNECT_BACKOFF_MAX);
                }
                // Terminal: no adapter at all, or something the user has to fix
                // (BLE pairing on Windows). Retrying only hides the message.
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

/// Periodically re-reads the BLE signal strength (RSSI).
pub fn rssi_poll() -> impl futures::Stream<Item = Message> {
    iced::stream::channel(1, |mut output: Sender<Message>| async move {
        loop {
            let result = session::read_rssi().await.map_err(|e| e.to_string());
            if output.send(Message::RssiResult(result)).await.is_err() {
                break;
            }
            time::sleep(Duration::from_secs(10)).await;
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
