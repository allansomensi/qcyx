use crate::app::App;
use crate::message::Message;
use crate::state::{AppState, Tab};
use crate::view::components::{badge, nav};
use iced::widget::{Space, column, container, pick_list, text};
use iced::{Element, Length, Theme};
use qcyx_i18n::fl;

pub fn view(app: &App) -> Element<'_, Message> {
    let header = column![
        text(fl!("app-title"))
            .size(26)
            .font(iced::font::Font::MONOSPACE),
        text(fl!("app-subtitle")).size(11).style(text::secondary),
        Space::new().height(10),
        status_badge(app),
    ]
    .spacing(2);

    let nav_items = column![
        nav::item("🏠", fl!("nav-home"), Tab::Home, app.tab),
        nav::item("🎧", fl!("nav-anc"), Tab::Anc, app.tab),
        nav::item("🎚️", fl!("nav-equalizer"), Tab::Equalizer, app.tab),
        nav::item("⚙️", fl!("nav-settings"), Tab::Settings, app.tab),
        nav::item("ℹ️", fl!("nav-about"), Tab::About, app.tab),
    ]
    .spacing(4);

    let theme_picker = column![
        text(fl!("theme-label")).size(11).style(text::secondary),
        pick_list(Theme::ALL, Some(&app.theme), Message::ThemeSelected)
            .text_size(13)
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(6);

    container(column![
        header,
        Space::new().height(24),
        nav_items,
        Space::new().height(Length::Fill),
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
