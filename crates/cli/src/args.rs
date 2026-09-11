use clap::{ArgAction, Parser, Subcommand};
use qcyx_core::command::{AMBIENT_LEVEL_MAX, AMBIENT_LEVEL_MIN};
use qcyx_core::scheduled_power_off::{MAX_MINUTES, MIN_MINUTES};

#[derive(Parser, Debug)]
#[command(author, version, about = "Control QCY earbuds over BLE")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Sets the active ANC scene
    Anc {
        #[command(subcommand)]
        scene: AncSceneArg,
    },
    /// Reads the current battery level of both earbuds
    Battery,
    /// Reads the device name and firmware version of both earbuds
    Version,
    /// Sets the channel balance (0 = full left, 100 = full right, 50 = centered)
    Balance {
        #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
        value: u8,
    },
    /// Sets an equalizer preset
    Eq {
        #[arg(value_enum)]
        preset: EqPresetArg,
    },
    /// Sets a custom 10-band equalizer curve
    EqCustom {
        /// Gains in dB (-8 to 8), one per band, in order:
        /// 31 62 125 250 500 1k 2k 4k 8k 16k Hz
        #[arg(
            num_args = 10,
            allow_hyphen_values = true,
            value_name = "DB",
            value_parser = clap::value_parser!(i16).range(-8..=8)
        )]
        bands: Vec<i16>,
    },
    /// Resets settings to default
    ResetDefault,
    /// Factory-resets the device
    FactoryReset {
        /// Skips the confirmation prompt
        #[arg(long)]
        yes: bool,
    },
    /// Renames the device's Bluetooth pairing name
    Rename { name: String },
    /// Sets the notification volume
    NotificationVolume {
        #[arg(value_enum)]
        level: NotificationVolumeArg,
    },
    /// Sets the scheduled power-off timer (minutes, or "off" to disable)
    ScheduledPowerOff {
        /// Minutes until power-off (1-65534), or "off" to disable
        value: PowerOffArg,
    },
    /// Sets the power-off-after-disconnect timer (minutes, or "never")
    DisconnectPowerOff {
        /// Minutes after disconnect until power-off (1-65534), or "never"
        value: PowerOffArg,
    },
    /// Sets in-ear wear detection and its ANC-on-wear sub-toggle
    WearDetection {
        #[arg(long)]
        enabled: bool,
        /// Re-applies the last ANC scene when the earbuds are put back in.
        /// Takes an explicit value, e.g. `--anc-on-wear=false`.
        #[arg(
            long,
            action = ArgAction::Set,
            num_args = 0..=1,
            default_value_t = true,
            default_missing_value = "true"
        )]
        anc_on_wear: bool,
    },
    /// Toggles game (low-latency) mode
    GameMode {
        #[arg(value_enum)]
        state: ToggleArg,
    },
    /// Toggles sleep mode
    SleepMode {
        #[arg(value_enum)]
        state: ToggleArg,
    },
    /// Toggles the LDAC codec
    Ldac {
        #[arg(value_enum)]
        state: ToggleArg,
    },
    /// Toggles dual-device (multipoint) connection
    Multipoint {
        #[arg(value_enum)]
        state: ToggleArg,
    },
    /// Assigns a tap action to one earbud/click-count control
    TouchAction {
        #[arg(value_enum)]
        control: TouchControlArg,
        #[arg(value_enum)]
        action: TouchActionArg,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum NotificationVolumeArg {
    Low,
    Medium,
    High,
    Max,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum ToggleArg {
    On,
    Off,
}

/// A power-off timer value: a number of minutes, or a disabling keyword
/// (`off`/`never` — both accepted, since the two timers use different words
/// for it in the official app).
#[derive(Clone, Copy, Debug)]
pub enum PowerOffArg {
    Disabled,
    Minutes(u16),
}

impl std::str::FromStr for PowerOffArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "off" | "never" | "disabled" => Ok(PowerOffArg::Disabled),
            // `0` was never observed on the wire, and `65535` is the sentinel
            // both timers use for "disabled" — it would turn the timer off
            // instead of arming it.
            _ => s
                .parse::<u16>()
                .ok()
                .filter(|minutes| (MIN_MINUTES..=MAX_MINUTES).contains(minutes))
                .map(PowerOffArg::Minutes)
                .ok_or_else(|| {
                    format!(
                        "'{s}' is not 'off' or a number of minutes ({MIN_MINUTES}-{MAX_MINUTES})"
                    )
                }),
        }
    }
}

/// The three top-level ANC modes. `Transparency` and `NoiseCancelling` each
/// open further choices — see [`NoiseCancellingArg`] and `Transparency`'s
/// own `--level` flag.
#[derive(Subcommand, Debug)]
pub enum AncSceneArg {
    /// Plain listening, no ANC/transparency processing.
    Normal,
    /// Transparency / ambient sound passthrough. With no flags, this is
    /// vocal-enhancement mode. Pass `--level` to switch to ambient-sound
    /// mode at that intensity instead (valid range: 1 to 6).
    Transparency {
        /// Ambient-sound intensity, 1 to 6 (omit this flag for
        /// vocal-enhancement mode instead)
        #[arg(
            long,
            value_parser = clap::value_parser!(u8)
                .range(i64::from(AMBIENT_LEVEL_MIN)..=i64::from(AMBIENT_LEVEL_MAX))
        )]
        level: Option<u8>,
    },
    /// The Noise Cancelling submenu.
    NoiseCancelling {
        #[command(subcommand)]
        mode: NoiseCancellingArg,
    },
}

/// Noise Cancelling submenu — five sub-modes, three of which
/// take a 1-3 intensity level.
#[derive(Subcommand, Debug)]
pub enum NoiseCancellingArg {
    /// AI-driven, no manual level
    Adaptive,
    /// No manual level
    Wind,
    Indoor {
        #[arg(value_parser = clap::value_parser!(u8).range(1..=3))]
        level: u8,
    },
    DailyCommute {
        #[arg(value_parser = clap::value_parser!(u8).range(1..=3))]
        level: u8,
    },
    Noisy {
        #[arg(value_parser = clap::value_parser!(u8).range(1..=3))]
        level: u8,
    },
}

/// Which earbud and click count a touch control addresses.
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum TouchControlArg {
    LeftSingle,
    RightSingle,
    LeftDouble,
    RightDouble,
    LeftTriple,
    RightTriple,
}

/// The action a tap gesture triggers.
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum TouchActionArg {
    None,
    PlayPause,
    Previous,
    Next,
    VoiceAssistant,
    VolumeUp,
    VolumeDown,
    GameMode,
    Anc,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum EqPresetArg {
    Spatial,
    Default,
    Popular,
    BassBoost,
    Rock,
    Soft,
    Classic,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("qcyx-cli").chain(args.iter().copied()))
    }

    #[test]
    fn command_definition_is_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn transparency_level_is_range_checked() {
        assert!(parse(&["anc", "transparency", "--level", "1"]).is_ok());
        assert!(parse(&["anc", "transparency", "--level", "6"]).is_ok());
        assert!(parse(&["anc", "transparency", "--level", "0"]).is_err());
        assert!(parse(&["anc", "transparency", "--level", "7"]).is_err());
    }

    #[test]
    fn power_off_minutes_exclude_zero_and_the_disabled_sentinel() {
        assert!(matches!(
            "1".parse::<PowerOffArg>(),
            Ok(PowerOffArg::Minutes(1))
        ));
        assert!(matches!(
            "65534".parse::<PowerOffArg>(),
            Ok(PowerOffArg::Minutes(65534))
        ));
        assert!(matches!(
            "Never".parse::<PowerOffArg>(),
            Ok(PowerOffArg::Disabled)
        ));
        assert!("0".parse::<PowerOffArg>().is_err());
        assert!("65535".parse::<PowerOffArg>().is_err());
        assert!("soon".parse::<PowerOffArg>().is_err());
        assert_eq!(qcyx_core::disconnect_power_off::MAX_MINUTES, MAX_MINUTES);
    }

    #[test]
    fn anc_on_wear_takes_an_explicit_value() {
        let anc_on_wear = |args: &[&str]| match parse(args).map(|cli| cli.command) {
            Ok(Commands::WearDetection { anc_on_wear, .. }) => Some(anc_on_wear),
            _ => None,
        };

        assert_eq!(anc_on_wear(&["wear-detection", "--enabled"]), Some(true));
        assert_eq!(
            anc_on_wear(&["wear-detection", "--enabled", "--anc-on-wear=false"]),
            Some(false)
        );
        assert_eq!(
            anc_on_wear(&["wear-detection", "--anc-on-wear"]),
            Some(true)
        );
    }
}
