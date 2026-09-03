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
