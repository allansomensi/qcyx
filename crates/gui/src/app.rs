use crate::{
    message::Message,
    state::{AppState, Tab},
    update, view, worker,
};
use iced::{Subscription, Task};
use qcyx_core::battery::BatteryStatus;
use qcyx_core::command::AncScene;
use qcyx_core::eq::EqPreset;
use qcyx_core::version::FirmwareVersion;

pub struct App {
    pub state: AppState,
    pub tab: Tab,
    pub theme: iced::Theme,
    pub error_log: Option<String>,
    /// Last successfully read battery status.
    pub battery: Option<BatteryStatus>,
    /// Error from the most recent battery read attempt.
    pub battery_error: Option<String>,
    /// The device's advertised BLE name.
    pub device_name: Option<String>,
    /// Firmware version read at connect time.
    pub firmware_version: Option<FirmwareVersion>,
    /// The active ANC scene confirmed by the device.
    pub active_scene: Option<AncScene>,
    /// The ANC scene currently being requested.
    pub pending_scene: Option<AncScene>,
    /// Local value for the Transparency ambient-sound slider.
    pub transparency_level: u8,
    /// Optimistically set equalizer preset.
    pub eq_preset: Option<EqPreset>,
    pub eq_status: Option<String>,
    /// Channel balance (`0..=100`, `50` = centered).
    pub balance: u8,
    /// Text input for device renaming.
    pub rename_input: String,
    pub rename_status: Option<String>,
    /// Tracks confirmation state for reset to default.
    pub reset_default_armed: bool,
    pub reset_default_status: Option<String>,
    /// Tracks confirmation state for factory reset.
    pub factory_reset_armed: bool,
    pub factory_reset_status: Option<String>,
    pub status_log: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::Connecting,
            tab: Tab::Home,
            theme: iced::Theme::Nord,
            error_log: None,
            battery: None,
            battery_error: None,
            device_name: None,
            firmware_version: None,
            active_scene: None,
            pending_scene: None,
            transparency_level: 1,
            eq_preset: None,
            eq_status: None,
            balance: 50,
            rename_input: String::new(),
            rename_status: None,
            reset_default_armed: false,
            reset_default_status: None,
            factory_reset_armed: false,
            factory_reset_status: None,
            status_log: None,
        }
    }
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        update::handle_message(self, message)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            AppState::Connecting => Subscription::run(worker::connect_poll),
            AppState::Connected => Subscription::batch([
                Subscription::run(worker::watch_disconnect),
                Subscription::run(worker::battery_poll),
            ]),
            AppState::Error => Subscription::none(),
        }
    }

    pub fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        view::view(self)
    }
}
