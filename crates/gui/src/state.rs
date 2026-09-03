/// High-level state machine driving [`crate::view`] and [`crate::app::App::subscription`].
///
/// Reflects the persistent connection's lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// Actively scanning/connecting in the background.
    Connecting,
    /// The persistent session is open and ready to accept commands.
    Connected,
    /// Connection failed.
    Error,
}

/// Main shell sections shown when [`AppState::Connected`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Home,
    Anc,
    Equalizer,
    Settings,
    About,
}

impl Tab {
    pub const ALL: [Tab; 5] = [
        Tab::Home,
        Tab::Anc,
        Tab::Equalizer,
        Tab::Settings,
        Tab::About,
    ];
}
