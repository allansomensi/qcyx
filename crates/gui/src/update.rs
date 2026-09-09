use crate::{app::App, message::Message, state::AppState};
use iced::Task;
use qcyx_core::client::AncConfirmation;
use qcyx_core::scheduled_power_off::ScheduledPowerOff;
use qcyx_core::session;
use qcyx_core::wear_detection::WearDetection;
use qcyx_i18n::fl;
use tracing::{debug, error, info};

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
            app.balance = info.initial_balance.unwrap_or(50);
            app.rename_input = info.device_name.clone().unwrap_or_default();
            app.device_name = info.device_name;
            app.firmware_version = info.firmware_version;
            app.wear_detection = info.initial_wear_detection;
            app.notification_volume = info.initial_notification_volume;
            app.scheduled_power_off = info.initial_scheduled_power_off;
            app.disconnect_power_off = info.initial_disconnect_power_off;
            app.game_mode = info.initial_game_mode;
            app.sleep_mode = info.initial_sleep_mode;
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
        Message::Disconnected => {
            info!("Device disconnected");
            app.state = AppState::Connecting;
            app.battery = None;
            app.battery_error = None;
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
            app.status_log = None;
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
            app.theme = theme;
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
        Message::RenameSubmit => {
            let name = app.rename_input.trim().to_string();
            if name.is_empty() {
                return Task::none();
            }
            app.rename_status = None;
            Task::perform(
                async move { session::set_name(&name).await.map_err(|e| e.to_string()) },
                Message::RenameResult,
            )
        }
        Message::RenameResult(result) => {
            match result {
                Ok(()) => {
                    info!("Device renamed");
                    app.rename_status = Some(fl!("settings-rename-done"));
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
            app.scheduled_power_off_custom_input = value;
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
    }
}
