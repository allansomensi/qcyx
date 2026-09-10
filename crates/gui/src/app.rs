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
use qcyx_core::profile::Profile;
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
    /// Last successfully read BLE signal strength, in dBm. `None` until the
    /// first read completes, or if the platform never reports one.
    pub rssi: Option<i16>,
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
    /// Optimistically set equalizer preset. `None` while a custom curve is
    /// active (no built-in preset chip should be highlighted).
    pub eq_preset: Option<EqPreset>,
    pub eq_status: Option<String>,
    /// Custom (per-band) equalizer gains in dB, [`qcyx_core::eq::CUSTOM_BAND_FREQS_HZ`] order.
    pub eq_custom_bands: [i16; qcyx_core::eq::CUSTOM_BAND_COUNT],
    /// Channel balance (`0..=100`, `50` = centered).
    pub balance: u8,
    /// Text input for device renaming.
    pub rename_input: String,
    pub rename_status: Option<String>,
    /// Whether the device-name field is unlocked for editing. Starts
    /// (and returns to) `false`/disabled so the field reads as a display
    /// value until the user explicitly asks to change it.
    pub rename_editing: bool,
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
    /// Current UI language (Fluent id, e.g. `"en"`, `"pt-BR"`).
    pub language: String,
    /// Name of the profile last successfully applied, shown as a small
    /// indicator in the sidebar. Cleared on disconnect; not automatically
    /// cleared by later manual tweaks (it marks "last applied", not "still
    /// matches exactly").
    pub active_profile: Option<String>,
    /// User-saved full-device profiles, loaded from disk at startup.
    pub profiles: Vec<Profile>,
    pub profiles_status: Option<String>,
    /// Text input for naming a new full-device profile.
    pub new_profile_name: String,
    /// User-saved EQ-only profiles, loaded from disk at startup.
    pub eq_profiles: Vec<Profile>,
    pub eq_profiles_status: Option<String>,
    /// Text input for naming a new EQ-only profile.
    pub new_eq_profile_name: String,
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
            rssi: None,
            device_name: None,
            firmware_version: None,
            active_scene: None,
            pending_scene: None,
            transparency_level: 1,
            eq_preset: None,
            eq_status: None,
            eq_custom_bands: [0; qcyx_core::eq::CUSTOM_BAND_COUNT],
            balance: 50,
            rename_input: String::new(),
            rename_status: None,
            rename_editing: false,
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
            language: String::new(),
            active_profile: None,
            profiles: Vec::new(),
            profiles_status: None,
            new_profile_name: String::new(),
            eq_profiles: Vec::new(),
            eq_profiles_status: None,
            new_eq_profile_name: String::new(),
        }
    }
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();

        let settings = crate::store::load_settings();

        if let Some(saved_theme) = settings.theme.as_deref()
            && let Some(theme) = iced::Theme::ALL
                .iter()
                .find(|t| t.to_string() == saved_theme)
        {
            app.theme = theme.clone();
        }

        if let Some(language) = settings.language.as_deref() {
            match qcyx_i18n::set_language(language) {
                Ok(()) => app.language = language.to_string(),
                Err(e) => {
                    tracing::warn!(error = %e, language, "failed to apply saved language");
                    app.language = qcyx_i18n::current_language();
                }
            }
        } else {
            // No saved preference yet: keep whatever `qcyx_i18n::localize()`
            // picked from the system locale at startup.
            app.language = qcyx_i18n::current_language();
        }

        app.profiles = crate::store::load_profiles();
        app.eq_profiles = crate::store::load_eq_profiles();

        (app, Task::none())
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
                Subscription::run(worker::rssi_poll),
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
