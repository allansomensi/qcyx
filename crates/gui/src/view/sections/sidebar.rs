use crate::app::App;
use crate::message::Message;
use crate::state::{AppState, Tab};
use crate::view::components::{badge, nav, signal};
use iced::widget::{Space, column, container, pick_list, row, scrollable, text};
use iced::{Alignment, Element, Length, Theme};
use qcyx_i18n::fl;

pub fn view(app: &App) -> Element<'_, Message> {
    let status_row: Element<'_, Message> = if matches!(app.state, AppState::Connected) {
        row![
            status_badge(app),
            Space::new().width(Length::Fill),
            signal::indicator(app.rssi),
        ]
        .align_y(Alignment::Center)
        .into()
    } else {
        row![status_badge(app)].into()
    };

    let mut header_items = column![
        text(fl!("app-title"))
            .size(26)
            .font(iced::font::Font::MONOSPACE),
        text(fl!("app-subtitle")).size(11).style(text::secondary),
        Space::new().height(10),
        status_row,
    ]
    .spacing(2);

    if let Some(name) = active_profile_label(app) {
        // Purely informational here — it's not interactive. Deactivating
        // a profile is a Profiles-tab action (see profiles.rs), not
        // something this badge does.
        header_items = header_items
            .push(Space::new().height(6))
            .push(badge::status(
                fl!("sidebar-active-profile", name = name),
                badge::Tone::Accent,
            ));
    }

    let nav_items = column![
        nav::item("🏠", fl!("nav-home"), Tab::Home, app.tab),
        nav::item("🎧", fl!("nav-anc"), Tab::Anc, app.tab),
        nav::item("🎚️", fl!("nav-equalizer"), Tab::Equalizer, app.tab),
        nav::item("🗂️", fl!("nav-profiles"), Tab::Profiles, app.tab),
        nav::item("⚙️", fl!("nav-settings"), Tab::Settings, app.tab),
        nav::item("ℹ️", fl!("nav-about"), Tab::About, app.tab),
    ]
    .spacing(4);

    // The header and footer (theme/language pickers) must always stay
    // fully visible, so only the nav list itself scrolls when the window
    // is too short to fit everything — it used to just get silently
    // clipped by the sidebar's `clip(true)` instead.
    let nav_scroll = scrollable(nav_items)
        .height(Length::Fill)
        .width(Length::Fill);

    let theme_picker = column![
        text(fl!("theme-label")).size(11).style(text::secondary),
        pick_list(Theme::ALL, Some(&app.theme), Message::ThemeSelected)
            .text_size(13)
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(6);

    let language_picker = column![
        text(fl!("language-label")).size(11).style(text::secondary),
        pick_list(
            LanguageOption::ALL,
            LanguageOption::from_id(&app.language),
            |option| Message::LanguageSelected(option.id().to_string()),
        )
        .text_size(13)
        .padding(8)
        .width(Length::Fill),
    ]
    .spacing(6);

    container(column![
        header_items,
        Space::new().height(24),
        nav_scroll,
        Space::new().height(10),
        language_picker,
        Space::new().height(10),
        theme_picker,
    ])
    .width(Length::Fixed(230.0))
    .height(Length::Fill)
    .padding(20)
    .clip(true)
    .style(|theme: &Theme| container::Style {
        background: Some(theme.extended_palette().background.weak.color.into()),
        border: iced::Border {
            color: theme.extended_palette().background.strong.color,
            width: 1.0,
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

fn status_badge(app: &App) -> Element<'_, Message> {
    match app.state {
        AppState::Connecting => badge::status(fl!("badge-connecting"), badge::Tone::Warning),
        AppState::Connected => badge::status(fl!("badge-connected"), badge::Tone::Success),
        AppState::Error => badge::status(fl!("badge-error"), badge::Tone::Danger),
    }
}

/// The label to show in the "active profile" pill.
fn active_profile_label(app: &App) -> Option<String> {
    app.active_profile
        .as_deref()
        .map(crate::localize::profile_label)
}

/// Wraps a Fluent language id with a localized `Display` for [`pick_list`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LanguageOption(&'static str);

impl LanguageOption {
    const ALL: [LanguageOption; 2] = [LanguageOption("en"), LanguageOption("pt-BR")];

    fn from_id(id: &str) -> Option<LanguageOption> {
        Self::ALL.into_iter().find(|option| option.0 == id)
    }

    fn id(self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for LanguageOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self.0 {
            "en" => "English",
            "pt-BR" => "Português (Brasil)",
            other => other,
        };
        write!(f, "{label}")
    }
}
