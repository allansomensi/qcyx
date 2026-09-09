use crate::{
    message::Message,
    state::{AppState, Tab},
    update, view, worker,
};
use iced::{Subscription, Task};
use qcyx_core::battery::BatteryStatus;
use qcyx_core::command::AncScene;
use qcyx_core::disconnect_power_off::DisconnectPowerOff;
use qcyx_core::eq::EqPreset;
use qcyx_core::game_mode::GameMode;
use qcyx_core::ldac::Ldac;
use qcyx_core::multipoint::Multipoint;
use qcyx_core::notification_volume::NotificationVolume;
use qcyx_core::scheduled_power_off::ScheduledPowerOff;
use qcyx_core::sleep_mode::SleepMode;
use qcyx_core::touch_action::TouchActionMap;
use qcyx_core::version::FirmwareVersion;
use qcyx_core::wear_detection::WearDetection;

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
    /// In-ear wear detection and its ANC-on-wear sub-toggle. `None` until
    /// read at connect time or unless the read failed — the toggle stays
    /// disabled in that case rather than assuming a state.
    pub wear_detection: Option<WearDetection>,
    pub wear_detection_status: Option<String>,
    /// Notification volume, read at connect time.
    pub notification_volume: Option<NotificationVolume>,
    pub notification_volume_status: Option<String>,
    /// Scheduled (idle-independent) power-off timer, read at connect time.
    pub scheduled_power_off: Option<ScheduledPowerOff>,
    pub scheduled_power_off_status: Option<String>,
    /// Text input for a custom scheduled-power-off minutes value.
    pub scheduled_power_off_custom_input: String,
    /// Power-off-after-disconnect timer, read at connect time.
    pub disconnect_power_off: Option<DisconnectPowerOff>,
    pub disconnect_power_off_status: Option<String>,
    /// Game mode, read at connect time.
    pub game_mode: Option<GameMode>,
    pub game_mode_status: Option<String>,
    /// Sleep mode, read at connect time.
    pub sleep_mode: Option<SleepMode>,
    pub sleep_mode_status: Option<String>,
    /// LDAC codec toggle, read at connect time.
    pub ldac: Option<Ldac>,
    pub ldac_status: Option<String>,
    /// Dual-device (multipoint) connection toggle, read at connect time.
    pub multipoint: Option<Multipoint>,
    pub multipoint_status: Option<String>,
    /// Touch-action map, read at connect time.
    pub touch_actions: Option<TouchActionMap>,
    pub touch_actions_status: Option<String>,
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
            wear_detection: None,
            wear_detection_status: None,
            notification_volume: None,
            notification_volume_status: None,
            scheduled_power_off: None,
            scheduled_power_off_status: None,
            scheduled_power_off_custom_input: String::new(),
            disconnect_power_off: None,
            disconnect_power_off_status: None,
            game_mode: None,
            game_mode_status: None,
            sleep_mode: None,
            sleep_mode_status: None,
            ldac: None,
            ldac_status: None,
            multipoint: None,
            multipoint_status: None,
            touch_actions: None,
            touch_actions_status: None,
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
