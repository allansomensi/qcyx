use crate::app::App;
use crate::localize::profile_label;
use crate::message::{Message, ProfileScope};
use crate::view::components::card;
use iced::widget::{Space, button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length, Theme};
use qcyx_core::profile::Profile;
use qcyx_i18n::fl;

pub fn view(app: &App) -> Element<'_, Message> {
    let header = column![
        text(fl!("profiles-title")).size(20),
        text(fl!("profiles-subtitle"))
            .size(13)
            .style(text::secondary),
    ]
    .spacing(4);

    let built_in_section = card::panel(
        column![
            text(fl!("profiles-built-in-title"))
                .size(13)
                .style(text::secondary),
            column(
                Profile::built_in()
                    .into_iter()
                    .map(|profile| {
                        let is_active =
                            app.active_profile.as_deref() == Some(profile.name.as_str());
                        profile_row(profile, true, is_active)
                    })
                    .collect::<Vec<_>>()
            )
            .spacing(10),
        ]
        .spacing(10),
    );

    let custom_section = card::panel(
        column![
            row![
                text(fl!("profiles-custom-title"))
                    .size(13)
                    .style(text::secondary),
                Space::new().width(Length::Fill),
                button(text(fl!("profiles-import-button")).size(12))
                    .padding([6, 12])
                    .style(button::secondary)
                    .on_press(Message::ImportProfile(ProfileScope::Full)),
            ]
            .align_y(Alignment::Center),
            if app.profiles.is_empty() {
                column![text(fl!("profiles-empty")).size(12).style(text::secondary)]
            } else {
                column(
                    app.profiles
                        .iter()
                        .cloned()
                        .map(|profile| {
                            let is_active =
                                app.active_profile.as_deref() == Some(profile.name.as_str());
                            profile_row(profile, false, is_active)
                        })
                        .collect::<Vec<_>>(),
                )
                .spacing(10)
            },
        ]
        .spacing(10),
    );

    let save_section = card::panel(
        column![
            text(fl!("profiles-save-current-title"))
                .size(13)
                .style(text::secondary),
            row![
                text_input(&fl!("profiles-name-placeholder"), &app.new_profile_name)
                    .on_input(|value| Message::NewProfileNameChanged(ProfileScope::Full, value))
                    .on_submit(Message::SaveCurrentAsProfile(ProfileScope::Full))
                    .padding(10)
                    .width(Length::Fill),
                button(text(fl!("profiles-save-button")).size(13))
                    .padding([10, 18])
                    .style(button::secondary)
                    .on_press(Message::SaveCurrentAsProfile(ProfileScope::Full)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(10),
    );

    let status = app
        .profiles_status
        .clone()
        .map(|s| -> Element<'_, Message> { text(s).size(12).style(text::secondary).into() })
        .unwrap_or_else(|| Space::new().height(0).into());

    column![
        header,
        built_in_section,
        custom_section,
        save_section,
        status
    ]
    .spacing(18)
    .width(Length::Fill)
    .into()
}

/// One profile row: icon + name, plus Apply/Export/Delete actions.
/// Built-in profiles have no Delete action and use a representative icon.
fn profile_row(profile: Profile, built_in: bool, is_active: bool) -> Element<'static, Message> {
    let icon = if built_in {
        built_in_icon(&profile.name)
    } else {
        "🗂️"
    };
    let label = profile_label(&profile.name);

    let mut actions = row![apply_indicator(is_active)].spacing(8);

    if is_active {
        actions = actions.push(
            button(text(fl!("profiles-deactivate-button")).size(12))
                .padding([6, 10])
                .style(button::text)
                .on_press(Message::ClearActiveProfile),
        );
    } else {
        actions = actions.push(
            button(text(fl!("profiles-apply-button")).size(12))
                .padding([6, 12])
                .style(move |theme: &Theme, _status| {
                    let ext = theme.extended_palette();
                    iced::widget::button::Style {
                        background: Some(ext.primary.weak.color.scale_alpha(0.25).into()),
                        text_color: ext.primary.strong.color,
                        border: iced::Border {
                            color: ext.primary.strong.color,
                            width: 1.0,
                            radius: 14.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .on_press(Message::ApplyProfile(ProfileScope::Full, profile.clone())),
        );
    }

    actions = actions.push(
        button(text(fl!("profiles-export-button")).size(12))
            .padding([6, 10])
            .style(button::text)
            .on_press(Message::ExportProfile(ProfileScope::Full, profile.clone())),
    );

    if !built_in {
        let name = profile.name.clone();
        actions = actions.push(
            button(text(fl!("profiles-delete-button")).size(12))
                .padding([6, 10])
                .style(button::text)
                .on_press(Message::DeleteProfile(ProfileScope::Full, name)),
        );
    }

    card::tile(
        row![
            row![text(icon).size(16), text(label).size(14)]
                .spacing(10)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            actions,
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
}

/// The non-interactive "Ativo" pill — a plain status indicator, not a
/// button. Deactivating lives in its own separate button (see
/// [`profile_row`]) so it's an unambiguous, deliberate action rather
/// than an easy-to-misclick side effect of the indicator itself.
fn apply_indicator(is_active: bool) -> Element<'static, Message> {
    if !is_active {
        return Space::new().width(0).into();
    }

    container(text(fl!("profiles-active-label")).size(12))
        .padding([6, 12])
        .style(|theme: &Theme| {
            let ext = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(ext.success.weak.color.scale_alpha(0.35).into()),
                text_color: Some(ext.success.strong.color),
                border: iced::Border {
                    color: ext.success.strong.color,
                    width: 1.0,
                    radius: 14.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Maps a built-in profile's key to a representative icon.
fn built_in_icon(name: &str) -> &'static str {
    match name {
        "built-in-focus" => "🎯",
        "built-in-calls" => "🗣️",
        "built-in-workout" => "🏃",
        "built-in-gaming" => "🎮",
        _ => "🗂️",
    }
}
