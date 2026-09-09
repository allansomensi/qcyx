use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about = "ANC control for QCY earbuds")]
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
        /// Minutes until power-off, or "off" to disable
        value: PowerOffArg,
    },
    /// Sets the power-off-after-disconnect timer (minutes, or "never")
    DisconnectPowerOff {
        /// Minutes after disconnect until power-off, or "never"
        value: PowerOffArg,
    },
    /// Sets in-ear wear detection and its ANC-on-wear sub-toggle
    WearDetection {
        #[arg(long)]
        enabled: bool,
        /// Re-applies the last ANC scene when the earbuds are put back in.
        /// Takes an explicit value, e.g. `--anc-on-wear=false`.
        #[arg(long, default_value_t = true)]
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
            _ => s
                .parse::<u16>()
                .map(PowerOffArg::Minutes)
                .map_err(|_| format!("'{s}' is not a number of minutes or 'off'")),
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
        /// Ambient-sound intensity (omit this flag for vocal-enhancement
        /// mode instead)
        #[arg(long)]
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
