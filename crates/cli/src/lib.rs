pub mod args;

use args::{
    AncSceneArg, Cli, Commands, NoiseCancellingArg, PowerOffArg, ToggleArg, TouchActionArg,
    TouchControlArg,
};
use clap::Parser;
use qcyx_core::command::{AncScene, NcLevel, NoiseCancellingMode, TransparencyMode};
use qcyx_core::disconnect_power_off::DisconnectPowerOff;
use qcyx_core::eq::EqPreset;
use qcyx_core::error::CoreError;
use qcyx_core::game_mode::GameMode;
use qcyx_core::ldac::Ldac;
use qcyx_core::multipoint::Multipoint;
use qcyx_core::notification_volume::NotificationVolume;
use qcyx_core::scheduled_power_off::ScheduledPowerOff;
use qcyx_core::sleep_mode::SleepMode;
use qcyx_core::touch_action::{TouchAction, TouchControl};
use qcyx_core::wear_detection::WearDetection;
use qcyx_i18n::fl;
use std::io::Write;

pub async fn run() -> Result<(), CoreError> {
    let cli = Cli::parse();

    println!("{}", fl!("cli-welcome"));

    match cli.command {
        Commands::Anc { scene } => {
            let scene = anc_scene_from_arg(scene);
            qcyx_core::client::set_anc_scene(scene).await
        }
        Commands::Battery => {
            let status = qcyx_core::client::get_battery().await?;
            print_battery(status);
            Ok(())
        }
        Commands::Version => {
            let info = qcyx_core::client::get_version().await?;
            print_version(info);
            Ok(())
        }
        Commands::Balance { value } => {
            qcyx_core::client::set_balance(value).await?;
            println!("{}", fl!("cli-balance-set", value = value.to_string()));
            Ok(())
        }
        Commands::Eq { preset } => {
            let preset = eq_preset_from_arg(preset);
            qcyx_core::client::set_eq_preset(preset).await?;
            println!("{}", fl!("cli-eq-set"));
            Ok(())
        }
        Commands::ResetDefault => {
            qcyx_core::client::reset_default().await?;
            println!("{}", fl!("cli-reset-default-done"));
            Ok(())
        }
        Commands::FactoryReset { yes } => {
            if !yes && !confirm(&fl!("cli-factory-reset-confirm")) {
                println!("{}", fl!("cli-factory-reset-cancelled"));
                return Ok(());
            }
            qcyx_core::client::factory_reset().await?;
            println!("{}", fl!("cli-factory-reset-done"));
            Ok(())
        }
        Commands::Rename { name } => {
            qcyx_core::client::set_name(&name).await?;
            println!("{}", fl!("cli-rename-done", name = name));
            Ok(())
        }
        Commands::NotificationVolume { level } => {
            let level = notification_volume_from_arg(level);
            qcyx_core::client::set_notification_volume(level).await?;
            println!("{}", fl!("cli-notification-volume-set"));
            Ok(())
        }
        Commands::ScheduledPowerOff { value } => {
            let value = match value {
                PowerOffArg::Disabled => ScheduledPowerOff::Disabled,
                PowerOffArg::Minutes(m) => ScheduledPowerOff::Minutes(m),
            };
            qcyx_core::client::set_scheduled_power_off(value).await?;
            println!("{}", fl!("cli-scheduled-power-off-set"));
            Ok(())
        }
        Commands::DisconnectPowerOff { value } => {
            let value = match value {
                PowerOffArg::Disabled => DisconnectPowerOff::Never,
                PowerOffArg::Minutes(m) => DisconnectPowerOff::Minutes(m),
            };
            qcyx_core::client::set_disconnect_power_off(value).await?;
            println!("{}", fl!("cli-disconnect-power-off-set"));
            Ok(())
        }
        Commands::WearDetection {
            enabled,
            anc_on_wear,
        } => {
            qcyx_core::client::set_wear_detection(WearDetection {
                wear_detection: enabled,
                anc_on_wear,
            })
            .await?;
            println!("{}", fl!("cli-wear-detection-set"));
            Ok(())
        }
        Commands::GameMode { state } => {
            let state = match state {
                ToggleArg::On => GameMode::On,
                ToggleArg::Off => GameMode::Off,
            };
            qcyx_core::client::set_game_mode(state).await?;
            println!("{}", fl!("cli-game-mode-set"));
            Ok(())
        }
        Commands::SleepMode { state } => {
            let state = match state {
                ToggleArg::On => SleepMode::On,
                ToggleArg::Off => SleepMode::Off,
            };
            qcyx_core::client::set_sleep_mode(state).await?;
            println!("{}", fl!("cli-sleep-mode-set"));
            Ok(())
        }
        Commands::Ldac { state } => {
            let state = match state {
                ToggleArg::On => Ldac::On,
                ToggleArg::Off => Ldac::Off,
            };
            qcyx_core::client::set_ldac(state).await?;
            println!("{}", fl!("cli-ldac-set"));
            Ok(())
        }
        Commands::Multipoint { state } => {
            let state = match state {
                ToggleArg::On => Multipoint::On,
                ToggleArg::Off => Multipoint::Off,
            };
            qcyx_core::client::set_multipoint(state).await?;
            println!("{}", fl!("cli-multipoint-set"));
            Ok(())
        }
        Commands::TouchAction { control, action } => {
            let control = touch_control_from_arg(control);
            let action = touch_action_from_arg(action);
            qcyx_core::client::set_touch_action(control, action).await?;
            println!("{}", fl!("cli-touch-action-set"));
            Ok(())
        }
    }
}

fn touch_control_from_arg(arg: TouchControlArg) -> TouchControl {
    match arg {
        TouchControlArg::LeftSingle => TouchControl::LeftSingle,
        TouchControlArg::RightSingle => TouchControl::RightSingle,
        TouchControlArg::LeftDouble => TouchControl::LeftDouble,
        TouchControlArg::RightDouble => TouchControl::RightDouble,
        TouchControlArg::LeftTriple => TouchControl::LeftTriple,
        TouchControlArg::RightTriple => TouchControl::RightTriple,
    }
}

fn touch_action_from_arg(arg: TouchActionArg) -> TouchAction {
    match arg {
        TouchActionArg::None => TouchAction::None,
        TouchActionArg::PlayPause => TouchAction::PlayPause,
        TouchActionArg::Previous => TouchAction::Previous,
        TouchActionArg::Next => TouchAction::Next,
        TouchActionArg::VoiceAssistant => TouchAction::VoiceAssistant,
        TouchActionArg::VolumeUp => TouchAction::VolumeUp,
        TouchActionArg::VolumeDown => TouchAction::VolumeDown,
        TouchActionArg::GameMode => TouchAction::GameMode,
        TouchActionArg::Anc => TouchAction::Anc,
    }
}

fn notification_volume_from_arg(arg: args::NotificationVolumeArg) -> NotificationVolume {
    match arg {
        args::NotificationVolumeArg::Low => NotificationVolume::Low,
        args::NotificationVolumeArg::Medium => NotificationVolume::Medium,
        args::NotificationVolumeArg::High => NotificationVolume::High,
        args::NotificationVolumeArg::Max => NotificationVolume::Max,
    }
}

fn anc_scene_from_arg(arg: AncSceneArg) -> AncScene {
    match arg {
        AncSceneArg::Normal => AncScene::Normal,
        AncSceneArg::Transparency { level: None } => {
            AncScene::Transparency(TransparencyMode::VocalEnhancement)
        }
        AncSceneArg::Transparency { level: Some(level) } => {
            AncScene::Transparency(TransparencyMode::AmbientSound { level })
        }
        AncSceneArg::NoiseCancelling { mode } => {
            AncScene::NoiseCancelling(noise_cancelling_mode_from_arg(mode))
        }
    }
}

fn noise_cancelling_mode_from_arg(arg: NoiseCancellingArg) -> NoiseCancellingMode {
    let level = |raw: u8| match raw {
        1 => NcLevel::One,
        2 => NcLevel::Two,
        _ => NcLevel::Three,
    };

    match arg {
        NoiseCancellingArg::Adaptive => NoiseCancellingMode::Adaptive,
        NoiseCancellingArg::Wind => NoiseCancellingMode::WindResistance,
        NoiseCancellingArg::Indoor { level: l } => NoiseCancellingMode::Indoor(level(l)),
        NoiseCancellingArg::DailyCommute { level: l } => {
            NoiseCancellingMode::DailyCommute(level(l))
        }
        NoiseCancellingArg::Noisy { level: l } => NoiseCancellingMode::Noisy(level(l)),
    }
}

fn eq_preset_from_arg(arg: args::EqPresetArg) -> EqPreset {
    match arg {
        args::EqPresetArg::Spatial => EqPreset::Spatial,
        args::EqPresetArg::Default => EqPreset::Default,
        args::EqPresetArg::Popular => EqPreset::Popular,
        args::EqPresetArg::BassBoost => EqPreset::BassBoost,
        args::EqPresetArg::Rock => EqPreset::Rock,
        args::EqPresetArg::Soft => EqPreset::Soft,
        args::EqPresetArg::Classic => EqPreset::Classic,
    }
}

/// Prints `prompt` and reads a `y`/`n` answer from stdin. Anything other
/// than a leading `y`/`Y` (including empty input, EOF, or a read error) is
/// treated as "no" — a destructive confirmation should fail closed.
fn confirm(prompt: &str) -> bool {
    print!("{prompt} [y/N] ");
    if std::io::stdout().flush().is_err() {
        return false;
    }

    let mut answer = String::new();
    if std::io::stdin().read_line(&mut answer).is_err() {
        return false;
    }

    matches!(answer.trim().chars().next(), Some('y' | 'Y'))
}

fn print_battery(status: qcyx_core::battery::BatteryStatus) {
    println!("{}", fl!("cli-battery-title"));
    print_battery_component(fl!("battery-left"), status.left);
    print_battery_component(fl!("battery-right"), status.right);
    print_battery_component(fl!("battery-case"), status.case);
}

fn print_battery_component(label: String, component: qcyx_core::battery::BatteryComponent) {
    let charging = if component.charging {
        format!(" ({})", fl!("battery-charging"))
    } else {
        String::new()
    };
    println!("  {label}: {}%{charging}", component.level);
}

fn print_version(info: qcyx_core::client::DeviceInfo) {
    println!("{}", fl!("cli-version-title"));

    let name = info.name.unwrap_or_else(|| fl!("cli-version-name-unknown"));
    println!("  {}: {name}", fl!("cli-version-name-label"));

    print_version_component(fl!("battery-left"), info.firmware.left);
    print_version_component(fl!("battery-right"), info.firmware.right);
}

fn print_version_component(label: String, version: Option<String>) {
    let version = version.unwrap_or_else(|| fl!("cli-version-unknown"));
    println!("  {label}: {version}");
}
