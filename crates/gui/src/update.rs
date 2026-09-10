use crate::message::ProfileScope;
use crate::{app::App, message::Message, state::AppState};
use iced::Task;
use qcyx_core::client::AncConfirmation;
use qcyx_core::command::{AncScene, TransparencyMode};
use qcyx_core::profile::Profile;
use qcyx_core::scheduled_power_off::ScheduledPowerOff;
use qcyx_core::session;
use qcyx_core::touch_action::TouchControl;
use qcyx_core::wear_detection::WearDetection;
use qcyx_i18n::fl;
use tracing::{debug, error, info};

/// Max digit count accepted by the custom scheduled-power-off input.
const SCHEDULED_POWER_OFF_CUSTOM_MAX_DIGITS: usize = 3;

pub fn handle_message(app: &mut App, message: Message) -> Task<Message> {
    debug!("Received message: {message:?}");

    match message {
        Message::Connected(info) => {
            info!(
                "Device connected (name: {:?}, initial ANC scene: {:?})",
                info.device_name, info.initial_anc_scene
            );
            app.state = AppState::Connected;
            app.error_log = None;
            app.active_scene = info.initial_anc_scene;
            if let Some(AncScene::Transparency(TransparencyMode::AmbientSound { level })) =
                info.initial_anc_scene
            {
                app.transparency_level = level;
            }
            app.balance = info.initial_balance.unwrap_or(50);
            app.rename_input = info.device_name.clone().unwrap_or_default();
            app.rename_editing = false;
            app.device_name = info.device_name;
            app.firmware_version = info.firmware_version;
            app.wear_detection = info.initial_wear_detection;
            app.notification_volume = info.initial_notification_volume;
            app.scheduled_power_off = info.initial_scheduled_power_off;
            app.disconnect_power_off = info.initial_disconnect_power_off;
            app.game_mode = info.initial_game_mode;
            app.sleep_mode = info.initial_sleep_mode;
            app.ldac = info.initial_ldac;
            app.multipoint = info.initial_multipoint;
            app.touch_actions = info.initial_touch_actions;
            Task::none()
        }
        Message::BatteryResult(result) => {
            match result {
                Ok(status) => {
                    debug!("Battery read: {status:?}");
                    app.battery = Some(status);
                    app.battery_error = None;
                }
                Err(e) => {
                    error!("Failed to read battery: {e}");
                    app.battery_error = Some(e);
                }
            }
            Task::none()
        }
        Message::RssiResult(result) => {
            match result {
                Ok(value) => {
                    debug!("RSSI read: {value:?}");
                    app.rssi = value;
                }
                Err(e) => {
                    debug!("Failed to read RSSI: {e}");
                    app.rssi = None;
                }
            }
            Task::none()
        }
        Message::Disconnected => {
            info!("Device disconnected");
            app.state = AppState::Connecting;
            app.battery = None;
            app.battery_error = None;
            app.rssi = None;
            app.device_name = None;
            app.firmware_version = None;
            app.active_scene = None;
            app.pending_scene = None;
            app.transparency_level = 1;
            app.eq_preset = None;
            app.eq_status = None;
            app.balance = 50;
            app.rename_input = String::new();
            app.rename_status = None;
            app.rename_editing = false;
            app.reset_default_armed = false;
            app.reset_default_status = None;
            app.factory_reset_armed = false;
            app.factory_reset_status = None;
            app.wear_detection = None;
            app.wear_detection_status = None;
            app.notification_volume = None;
            app.notification_volume_status = None;
            app.scheduled_power_off = None;
            app.scheduled_power_off_status = None;
            app.scheduled_power_off_custom_input = String::new();
            app.disconnect_power_off = None;
            app.disconnect_power_off_status = None;
            app.game_mode = None;
            app.game_mode_status = None;
            app.sleep_mode = None;
            app.sleep_mode_status = None;
            app.ldac = None;
            app.ldac_status = None;
            app.multipoint = None;
            app.multipoint_status = None;
            app.touch_actions = None;
            app.touch_actions_status = None;
            app.status_log = None;
            app.active_profile = None;
            app.profiles_status = None;
            app.eq_profiles_status = None;
            Task::none()
        }
        Message::ConnectionError(err) => {
            error!("Failed to connect to device: {err}");
            app.state = AppState::Error;
            app.error_log = Some(err);
            Task::none()
        }
        Message::Retry => {
            info!("Retrying connection");
            app.state = AppState::Connecting;
            app.error_log = None;
            Task::none()
        }
        Message::SetAnc(scene) => {
            if app.pending_scene.is_some() {
                return Task::none();
            }

            info!("Requesting ANC scene: {scene:?}");
            app.pending_scene = Some(scene);
            app.status_log = None;

            Task::perform(
                async move {
                    session::set_anc_scene(scene)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |result| Message::AncResult(scene, result),
            )
        }
        Message::AncResult(scene, result) => {
            app.pending_scene = None;

            match result {
                Ok(AncConfirmation::Applied) => {
                    info!("ANC scene applied: {scene:?}");
                    app.active_scene = Some(scene);
                    app.status_log = Some(fl!("gui-anc-applied"));
                }
                Ok(AncConfirmation::Rejected) => {
                    app.status_log = Some(fl!("gui-anc-unconfirmed"));
                }
                Ok(AncConfirmation::EchoedOnly) => {
                    app.active_scene = Some(scene);
                    app.status_log = Some(fl!("gui-anc-echoed"));
                }
                Ok(AncConfirmation::NoResponse) => {
                    app.status_log = Some(fl!("gui-anc-timeout"));
                }
                Err(e) => {
                    error!("Failed to set ANC scene: {e}");
                    app.status_log = Some(fl!("gui-anc-error", error = e));
                }
            }

            Task::none()
        }
        Message::TabSelected(tab) => {
            app.tab = tab;
            app.reset_default_armed = false;
            app.factory_reset_armed = false;
            Task::none()
        }
        Message::ThemeSelected(theme) => {
            info!("Theme changed to: {theme}");
            app.theme = theme.clone();
            let mut settings = crate::store::load_settings();
            settings.theme = Some(theme.to_string());
            crate::store::save_settings(&settings);
            Task::none()
        }
        Message::BalanceChanged(value) => {
            app.balance = value;
            Task::none()
        }
        Message::BalanceCommit(value) => {
            app.balance = value;
            Task::perform(
                async move { session::set_balance(value).await.map_err(|e| e.to_string()) },
                Message::BalanceResult,
            )
        }
        Message::BalanceResult(Err(e)) => {
            error!("Failed to set channel balance: {e}");
            app.status_log = Some(fl!("gui-balance-error", error = e));
            Task::none()
        }
        Message::BalanceResult(Ok(())) => Task::none(),
        Message::RenameInputChanged(value) => {
            app.rename_input = value;
            Task::none()
        }
        Message::RenameEditToggled => {
            if app.rename_editing {
                // "Salvar" was pressed: commit the typed name. The field
                // stays enabled until the write actually succeeds
                // (RenameResult flips rename_editing back off), so a failed
                // write leaves the user able to correct and retry.
                let name = app.rename_input.trim().to_string();
                if name.is_empty() {
                    return Task::none();
                }
                app.rename_status = None;
                Task::perform(
                    async move { session::set_name(&name).await.map_err(|e| e.to_string()) },
                    Message::RenameResult,
                )
            } else {
                app.rename_editing = true;
                app.rename_status = None;
                Task::none()
            }
        }
        Message::RenameEditCancelled => {
            app.rename_input = app.device_name.clone().unwrap_or_default();
            app.rename_editing = false;
            app.rename_status = None;
            Task::none()
        }
        Message::RenameResult(result) => {
            match result {
                Ok(()) => {
                    info!("Device renamed");
                    app.rename_status = Some(fl!("settings-rename-done"));
                    app.rename_editing = false;
                }
                Err(e) => {
                    error!("Failed to rename device: {e}");
                    app.rename_status = Some(fl!("settings-rename-error", error = e));
                }
            }
            Task::none()
        }
        Message::ResetDefaultPressed => {
            if app.reset_default_armed {
                app.reset_default_armed = false;
                app.reset_default_status = None;
                Task::perform(
                    async { session::reset_default().await.map_err(|e| e.to_string()) },
                    Message::ResetDefaultResult,
                )
            } else {
                app.reset_default_armed = true;
                Task::none()
            }
        }
        Message::ResetDefaultResult(result) => {
            match result {
                Ok(()) => {
                    info!("Settings reset to default");
                    app.reset_default_status = Some(fl!("settings-reset-default-done"));
                }
                Err(e) => {
                    error!("Failed to reset settings: {e}");
                    app.reset_default_status = Some(fl!("settings-reset-default-error", error = e));
                }
            }
            Task::none()
        }
        Message::FactoryResetPressed => {
            if app.factory_reset_armed {
                app.factory_reset_armed = false;
                app.factory_reset_status = None;
                Task::perform(
                    async { session::factory_reset().await.map_err(|e| e.to_string()) },
                    Message::FactoryResetResult,
                )
            } else {
                app.factory_reset_armed = true;
                Task::none()
            }
        }
        Message::FactoryResetResult(result) => {
            match result {
                Ok(()) => {
                    info!("Factory reset sent");
                    app.factory_reset_status = Some(fl!("settings-factory-reset-done"));
                }
                Err(e) => {
                    error!("Failed to factory reset: {e}");
                    app.factory_reset_status = Some(fl!("settings-factory-reset-error", error = e));
                }
            }
            Task::none()
        }
        Message::TransparencyLevelChanged(value) => {
            app.transparency_level = value;
            Task::none()
        }
        Message::SetEqPreset(preset) => {
            info!("Requesting EQ preset: {preset:?}");
            app.eq_preset = Some(preset);
            app.eq_status = None;
            Task::perform(
                async move {
                    session::set_eq_preset(preset)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |result| Message::EqPresetResult(preset, result),
            )
        }
        Message::EqPresetResult(preset, result) => {
            match result {
                Ok(()) => {
                    info!("EQ preset applied: {preset:?}");
                    app.eq_status = Some(fl!("eq-preset-applied"));
                }
                Err(e) => {
                    error!("Failed to set EQ preset: {e}");
                    app.eq_status = Some(fl!("eq-preset-error", error = e));
                }
            }
            Task::none()
        }
        Message::EqCustomBandChanged(band, value) => {
            if let Some(slot) = app.eq_custom_bands.get_mut(band) {
                *slot = value;
            }
            Task::none()
        }
        Message::EqCustomBandCommit(band, value) => {
            if let Some(slot) = app.eq_custom_bands.get_mut(band) {
                *slot = value;
            }
            // A custom edit deselects any built-in preset chip — the device
            // is no longer on one of the 7 named curves.
            app.eq_preset = None;
            app.eq_status = None;
            let bands = app.eq_custom_bands;
            Task::perform(
                async move {
                    session::set_eq_custom(bands)
                        .await
                        .map_err(|e| e.to_string())
                },
                Message::EqCustomResult,
            )
        }
        Message::EqCustomReset => {
            app.eq_custom_bands = [0; qcyx_core::eq::CUSTOM_BAND_COUNT];
            app.eq_preset = None;
            app.eq_status = None;
            Task::perform(
                async move {
                    session::set_eq_custom([0; qcyx_core::eq::CUSTOM_BAND_COUNT])
                        .await
                        .map_err(|e| e.to_string())
                },
                Message::EqCustomResult,
            )
        }
        Message::EqCustomResult(result) => {
            match result {
                Ok(()) => {
                    info!("Custom EQ applied");
                    app.eq_status = Some(fl!("eq-custom-applied"));
                }
                Err(e) => {
                    error!("Failed to set custom EQ: {e}");
                    app.eq_status = Some(fl!("eq-custom-error", error = e));
                }
            }
            Task::none()
        }
        Message::WearDetectionToggled(enabled) => {
            // Preserves the ANC-on-wear sub-flag, which this toggle doesn't
            // expose — defaults to on, matching the device's own
            // connect-time default when no read has happened yet.
            let anc_on_wear = app.wear_detection.map(|s| s.anc_on_wear).unwrap_or(true);
            let new_state = WearDetection {
                wear_detection: enabled,
                anc_on_wear,
            };
            app.wear_detection = Some(new_state);
            app.wear_detection_status = None;
            Task::perform(
                async move {
                    session::set_wear_detection(new_state)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |result| Message::WearDetectionResult(new_state, result),
            )
        }
        Message::WearDetectionResult(state, result) => {
            match result {
                Ok(()) => {
                    info!("Wear detection set: {state:?}");
                }
                Err(e) => {
                    error!("Failed to set wear detection: {e}");
                    app.wear_detection_status = Some(fl!("settings-inear-toggle-error", error = e));
                }
            }
            Task::none()
        }
        Message::SetNotificationVolume(level) => {
            info!("Requesting notification volume: {level:?}");
            app.notification_volume = Some(level);
            app.notification_volume_status = None;
            Task::perform(
                async move {
                    session::set_notification_volume(level)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |result| Message::NotificationVolumeResult(level, result),
            )
        }
        Message::NotificationVolumeResult(level, result) => {
            match result {
                Ok(()) => info!("Notification volume set: {level:?}"),
                Err(e) => {
                    error!("Failed to set notification volume: {e}");
                    app.notification_volume_status =
                        Some(fl!("settings-notification-volume-error", error = e));
                }
            }
            Task::none()
        }
        Message::SetScheduledPowerOff(value) => {
            info!("Requesting scheduled power-off: {value:?}");
            app.scheduled_power_off = Some(value);
            app.scheduled_power_off_status = None;
            Task::perform(
                async move {
                    session::set_scheduled_power_off(value)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |result| Message::ScheduledPowerOffResult(value, result),
            )
        }
        Message::ScheduledPowerOffResult(value, result) => {
            match result {
                Ok(()) => info!("Scheduled power-off set: {value:?}"),
                Err(e) => {
                    error!("Failed to set scheduled power-off: {e}");
                    app.scheduled_power_off_status =
                        Some(fl!("settings-scheduled-poweroff-error", error = e));
                }
            }
            Task::none()
        }
        Message::ScheduledPowerOffCustomInputChanged(value) => {
            // Digits only, capped at 3 characters (max "999" minutes,
            // ~16.5h) — long enough for any sensible timer, short enough
            // that a mistyped value can't turn into a bogus multi-day one.
            let filtered: String = value
                .chars()
                .filter(char::is_ascii_digit)
                .take(SCHEDULED_POWER_OFF_CUSTOM_MAX_DIGITS)
                .collect();
            app.scheduled_power_off_custom_input = filtered;
            Task::none()
        }
        Message::ScheduledPowerOffCustomSubmit => {
            let Ok(minutes) = app.scheduled_power_off_custom_input.trim().parse::<u16>() else {
                return Task::none();
            };
            let value = ScheduledPowerOff::Minutes(minutes);
            info!("Requesting scheduled power-off: {value:?}");
            app.scheduled_power_off = Some(value);
            app.scheduled_power_off_status = None;
            Task::perform(
                async move {
                    session::set_scheduled_power_off(value)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |result| Message::ScheduledPowerOffResult(value, result),
            )
        }
        Message::SetDisconnectPowerOff(value) => {
            info!("Requesting disconnect power-off: {value:?}");
            app.disconnect_power_off = Some(value);
            app.disconnect_power_off_status = None;
            Task::perform(
                async move {
                    session::set_disconnect_power_off(value)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |result| Message::DisconnectPowerOffResult(value, result),
            )
        }
        Message::DisconnectPowerOffResult(value, result) => {
            match result {
                Ok(()) => info!("Disconnect power-off set: {value:?}"),
                Err(e) => {
                    error!("Failed to set disconnect power-off: {e}");
                    app.disconnect_power_off_status =
                        Some(fl!("settings-disconnect-poweroff-error", error = e));
                }
            }
            Task::none()
        }
        Message::SetGameMode(state) => {
            info!("Requesting game mode: {state:?}");
            app.game_mode = Some(state);
            app.game_mode_status = None;
            Task::perform(
                async move {
                    session::set_game_mode(state)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |result| Message::GameModeResult(state, result),
            )
        }
        Message::GameModeResult(state, result) => {
            match result {
                Ok(()) => info!("Game mode set: {state:?}"),
                Err(e) => {
                    error!("Failed to set game mode: {e}");
                    app.game_mode_status = Some(fl!("settings-game-mode-error", error = e));
                }
            }
            Task::none()
        }
        Message::SetSleepMode(state) => {
            info!("Requesting sleep mode: {state:?}");
            app.sleep_mode = Some(state);
            app.sleep_mode_status = None;
            Task::perform(
                async move {
                    session::set_sleep_mode(state)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |result| Message::SleepModeResult(state, result),
            )
        }
        Message::SleepModeResult(state, result) => {
            match result {
                Ok(()) => info!("Sleep mode set: {state:?}"),
                Err(e) => {
                    error!("Failed to set sleep mode: {e}");
                    app.sleep_mode_status = Some(fl!("settings-sleep-mode-error", error = e));
                }
            }
            Task::none()
        }
        Message::SetLdac(state) => {
            info!("Requesting LDAC: {state:?}");
            app.ldac = Some(state);
            app.ldac_status = None;
            Task::perform(
                async move { session::set_ldac(state).await.map_err(|e| e.to_string()) },
                move |result| Message::LdacResult(state, result),
            )
        }
        Message::LdacResult(state, result) => {
            match result {
                Ok(()) => info!("LDAC set: {state:?}"),
                Err(e) => {
                    error!("Failed to set LDAC: {e}");
                    app.ldac_status = Some(fl!("settings-ldac-error", error = e));
                }
            }
            Task::none()
        }
        Message::SetMultipoint(state) => {
            info!("Requesting multipoint: {state:?}");
            app.multipoint = Some(state);
            app.multipoint_status = None;
            Task::perform(
                async move {
                    session::set_multipoint(state)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |result| Message::MultipointResult(state, result),
            )
        }
        Message::MultipointResult(state, result) => {
            match result {
                Ok(()) => info!("Multipoint set: {state:?}"),
                Err(e) => {
                    error!("Failed to set multipoint: {e}");
                    app.multipoint_status = Some(fl!("settings-multipoint-error", error = e));
                }
            }
            Task::none()
        }
        Message::SetTouchAction(control, action) => {
            info!("Requesting touch action: {control:?} -> {action:?}");
            if let Some(map) = app.touch_actions.as_mut() {
                match control {
                    TouchControl::LeftSingle => map.left_single = Some(action),
                    TouchControl::RightSingle => map.right_single = Some(action),
                    TouchControl::LeftDouble => map.left_double = Some(action),
                    TouchControl::RightDouble => map.right_double = Some(action),
                    TouchControl::LeftTriple => map.left_triple = Some(action),
                    TouchControl::RightTriple => map.right_triple = Some(action),
                }
            }
            app.touch_actions_status = None;
            Task::perform(
                async move {
                    session::set_touch_action(control, action)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |result| Message::TouchActionResult(control, action, result),
            )
        }
        Message::TouchActionResult(control, action, result) => {
            match result {
                Ok(()) => info!("Touch action set: {control:?} -> {action:?}"),
                Err(e) => {
                    error!("Failed to set touch action: {e}");
                    app.touch_actions_status = Some(fl!("settings-touch-action-error", error = e));
                }
            }
            Task::none()
        }
        Message::LanguageSelected(language) => {
            match qcyx_i18n::set_language(&language) {
                Ok(()) => {
                    info!("Language changed to: {language}");
                    app.language = language.clone();
                    let mut settings = crate::store::load_settings();
                    settings.language = Some(language);
                    crate::store::save_settings(&settings);
                }
                Err(e) => {
                    error!("Failed to set language '{language}': {e}");
                }
            }
            Task::none()
        }
        Message::ApplyProfile(scope, profile) => {
            info!("Applying profile ({scope:?}): {}", profile.name);

            if let Some(scene) = profile.anc_scene {
                app.active_scene = Some(scene);
                app.pending_scene = None;
                if let AncScene::Transparency(TransparencyMode::AmbientSound { level }) = scene {
                    app.transparency_level = level;
                }
            }
            if let Some(bands) = profile.eq_custom {
                app.eq_custom_bands = bands;
                app.eq_preset = None;
            } else if let Some(preset) = profile.eq_preset {
                app.eq_preset = Some(preset);
            }
            if let Some(balance) = profile.balance {
                app.balance = balance;
            }
            if let Some(level) = profile.notification_volume {
                app.notification_volume = Some(level);
            }
            if let Some(state) = profile.game_mode {
                app.game_mode = Some(state);
            }

            match scope {
                ProfileScope::Full => app.profiles_status = None,
                ProfileScope::Eq => app.eq_profiles_status = None,
            }

            let name = profile.name.clone();
            Task::perform(apply_profile_to_device(profile), move |result| {
                Message::ApplyProfileResult(scope, name, result)
            })
        }
        Message::ApplyProfileResult(scope, name, result) => {
            match (scope, result) {
                (ProfileScope::Full, Ok(())) => {
                    info!("Profile applied: {name}");
                    app.active_profile = Some(name);
                    app.profiles_status = Some(fl!("profiles-applied"));
                }
                (ProfileScope::Full, Err(e)) => {
                    error!("Failed to apply profile '{name}': {e}");
                    app.profiles_status = Some(fl!("profiles-apply-error", error = e));
                }
                (ProfileScope::Eq, Ok(())) => {
                    info!("EQ profile applied: {name}");
                    app.eq_profiles_status = Some(fl!("eq-profiles-applied"));
                }
                (ProfileScope::Eq, Err(e)) => {
                    error!("Failed to apply EQ profile '{name}': {e}");
                    app.eq_profiles_status = Some(fl!("eq-profiles-apply-error", error = e));
                }
            }
            Task::none()
        }
        Message::NewProfileNameChanged(scope, value) => {
            match scope {
                ProfileScope::Full => app.new_profile_name = value,
                ProfileScope::Eq => app.new_eq_profile_name = value,
            }
            Task::none()
        }
        Message::SaveCurrentAsProfile(scope) => {
            match scope {
                ProfileScope::Full => {
                    let name = app.new_profile_name.trim().to_string();
                    if name.is_empty() {
                        return Task::none();
                    }
                    let custom_eq = app.eq_preset.is_none().then_some(app.eq_custom_bands);
                    let profile = Profile::from_current(
                        name.clone(),
                        app.active_scene,
                        app.eq_preset,
                        custom_eq,
                        Some(app.balance),
                        app.notification_volume,
                        app.game_mode,
                    );
                    app.profiles.retain(|p| p.name != name);
                    app.profiles.push(profile);
                    crate::store::save_profiles(&app.profiles);
                    app.new_profile_name = String::new();
                    app.profiles_status = Some(fl!("profiles-saved", name = name));
                }
                ProfileScope::Eq => {
                    let name = app.new_eq_profile_name.trim().to_string();
                    if name.is_empty() {
                        return Task::none();
                    }
                    let custom_eq = app.eq_preset.is_none().then_some(app.eq_custom_bands);
                    let profile = Profile::from_current_eq(name.clone(), app.eq_preset, custom_eq);
                    app.eq_profiles.retain(|p| p.name != name);
                    app.eq_profiles.push(profile);
                    crate::store::save_eq_profiles(&app.eq_profiles);
                    app.new_eq_profile_name = String::new();
                    app.eq_profiles_status = Some(fl!("eq-profiles-saved", name = name));
                }
            }
            Task::none()
        }
        Message::DeleteProfile(scope, name) => {
            match scope {
                ProfileScope::Full => {
                    app.profiles.retain(|p| p.name != name);
                    crate::store::save_profiles(&app.profiles);
                    if app.active_profile.as_deref() == Some(name.as_str()) {
                        app.active_profile = None;
                    }
                }
                ProfileScope::Eq => {
                    app.eq_profiles.retain(|p| p.name != name);
                    crate::store::save_eq_profiles(&app.eq_profiles);
                }
            }
            Task::none()
        }
        Message::ExportProfile(scope, profile) => Task::perform(
            async move {
                tokio::task::spawn_blocking(move || crate::store::export_profile_blocking(&profile))
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()))
            },
            move |result| Message::ExportProfileResult(scope, result),
        ),
        Message::ExportProfileResult(scope, result) => {
            let status = Some(match result {
                Ok(()) => fl!("profiles-export-done"),
                Err(e) => fl!("profiles-export-error", error = e),
            });
            match scope {
                ProfileScope::Full => app.profiles_status = status,
                ProfileScope::Eq => app.eq_profiles_status = status,
            }
            Task::none()
        }
        Message::ImportProfile(scope) => Task::perform(
            async move {
                tokio::task::spawn_blocking(crate::store::import_profile_blocking)
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()))
            },
            move |result| Message::ImportProfileResult(scope, result),
        ),
        Message::ImportProfileResult(scope, result) => {
            match result {
                Ok(profile) => {
                    let name = profile.name.clone();
                    match scope {
                        ProfileScope::Full => {
                            app.profiles.retain(|p| p.name != name);
                            app.profiles.push(profile);
                            crate::store::save_profiles(&app.profiles);
                            app.profiles_status = Some(fl!("profiles-import-done", name = name));
                        }
                        ProfileScope::Eq => {
                            app.eq_profiles.retain(|p| p.name != name);
                            app.eq_profiles.push(profile);
                            crate::store::save_eq_profiles(&app.eq_profiles);
                            app.eq_profiles_status =
                                Some(fl!("eq-profiles-import-done", name = name));
                        }
                    }
                }
                Err(e) => {
                    let status = Some(fl!("profiles-import-error", error = e));
                    match scope {
                        ProfileScope::Full => app.profiles_status = status,
                        ProfileScope::Eq => app.eq_profiles_status = status,
                    }
                }
            }
            Task::none()
        }
        Message::ClearActiveProfile => {
            info!("Clearing active profile indicator");
            app.active_profile = None;
            app.profiles_status = None;
            Task::none()
        }
    }
}

/// Sequentially applies every field a profile sets, over the shared BLE
/// session. Fields left `None` in the profile are left untouched on the
/// device — this is what lets an EQ-only profile safely share the same
/// apply path as a full-device one.
async fn apply_profile_to_device(profile: Profile) -> Result<(), String> {
    if let Some(scene) = profile.anc_scene {
        session::set_anc_scene(scene)
            .await
            .map(|_: AncConfirmation| ())
            .map_err(|e| e.to_string())?;
    }
    if let Some(bands) = profile.eq_custom {
        session::set_eq_custom(bands)
            .await
            .map_err(|e| e.to_string())?;
    } else if let Some(preset) = profile.eq_preset {
        session::set_eq_preset(preset)
            .await
            .map_err(|e| e.to_string())?;
    }
    if let Some(balance) = profile.balance {
        session::set_balance(balance)
            .await
            .map_err(|e| e.to_string())?;
    }
    if let Some(level) = profile.notification_volume {
        session::set_notification_volume(level)
            .await
            .map_err(|e| e.to_string())?;
    }
    if let Some(state) = profile.game_mode {
        session::set_game_mode(state)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
