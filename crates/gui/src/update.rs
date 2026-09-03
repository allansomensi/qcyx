use crate::{app::App, message::Message, state::AppState};
use iced::Task;
use qcyx_core::client::AncConfirmation;
use qcyx_core::session;
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
    }
}
