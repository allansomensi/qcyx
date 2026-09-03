use crate::app::App;
use crate::message::Message;
use crate::view::components::{badge, card};
use iced::widget::{Space, button, column, row, text, text_input, toggler};
use iced::{Alignment, Element, Length, Theme};
use qcyx_i18n::fl;

pub fn view(app: &App) -> Element<'_, Message> {
    let header = column![
        text(fl!("settings-title")).size(20),
        text(fl!("settings-subtitle"))
            .size(13)
            .style(text::secondary),
    ]
    .spacing(4);

    let rename_section = card::panel(
        column![
            text(fl!("settings-device-name-label"))
                .size(13)
                .style(text::secondary),
            row![
                text_input(&fl!("settings-device-name-placeholder"), &app.rename_input)
                    .on_input(Message::RenameInputChanged)
                    .on_submit(Message::RenameSubmit)
                    .padding(10)
                    .width(Length::Fill),
                button(text(fl!("settings-save-button")).size(13))
                    .padding([10, 18])
                    .style(button::secondary)
                    .on_press(Message::RenameSubmit),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            status_line(&app.rename_status),
        ]
        .spacing(10),
    );

    let toggles_section = card::panel(
        column![
            toggle_row(
                fl!("settings-inear-toggle-label"),
                fl!("settings-inear-toggle-desc")
            ),
            toggle_row(
                fl!("settings-doubletap-toggle-label"),
                fl!("settings-doubletap-toggle-desc")
            ),
            toggle_row(
                fl!("settings-voice-prompts-toggle-label"),
                fl!("settings-voice-prompts-toggle-desc")
            ),
        ]
        .spacing(16),
    );

    let firmware_section = card::panel(
        column![
            section_label(fl!("settings-firmware-section-title")),
            row![
                column![
                    text(fl!("settings-firmware-version-label"))
                        .size(12)
                        .style(text::secondary),
                    text(firmware_line(app)).size(14),
                ]
                .spacing(2)
                .width(Length::Fill),
                button(text(fl!("settings-firmware-check-button")).size(13))
                    .padding([10, 18])
                    .style(button::secondary),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(10),
    );

    let reset_section = card::panel(
        column![
            row![
                text(fl!("settings-reset-default-desc"))
                    .size(12)
                    .style(text::secondary)
                    .width(Length::Fill),
                button(text(confirmable_label(
                    fl!("settings-reset-default-button"),
                    app.reset_default_armed
                )))
                .padding([10, 18])
                .style(button::secondary)
                .on_press(Message::ResetDefaultPressed),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            status_line(&app.reset_default_status),
        ]
        .spacing(10),
    );

    let danger_section = card::panel(
        column![
            text(fl!("settings-danger-title"))
                .size(13)
                .style(|theme: &Theme| {
                    text::Style {
                        color: Some(theme.extended_palette().danger.strong.color),
                    }
                }),
            row![
                text(fl!("settings-factory-reset-desc"))
                    .size(12)
                    .style(text::secondary)
                    .width(Length::Fill),
                button(text(confirmable_label(
                    fl!("settings-factory-reset-button"),
                    app.factory_reset_armed
                )))
                .padding([10, 18])
                .style(button::danger)
                .on_press(Message::FactoryResetPressed),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            status_line(&app.factory_reset_status),
        ]
        .spacing(10),
    );

    column![
        header,
        rename_section,
        toggles_section,
        firmware_section,
        reset_section,
        danger_section
    ]
    .spacing(18)
    .width(Length::Fill)
    .into()
}

/// `"Reset to default"` normally, `"Tap again to confirm"` once armed —
/// same pattern for both reset actions.
fn confirmable_label(label: String, armed: bool) -> String {
    if armed {
        fl!("settings-confirm-again")
    } else {
        label
    }
}

fn firmware_line(app: &App) -> String {
    match &app.firmware_version {
        Some(v) => match (&v.left, &v.right) {
            (Some(left), Some(right)) => format!("{left} / {right}"),
            (Some(left), None) => left.clone(),
            (None, Some(right)) => right.clone(),
            (None, None) => fl!("home-firmware-unknown"),
        },
        None => fl!("home-firmware-unknown"),
    }
}

fn status_line(status: &Option<String>) -> Element<'_, Message> {
    match status {
        Some(s) => text(s.clone()).size(12).style(text::secondary).into(),
        None => Space::new().height(0).into(),
    }
}

fn section_label(label: String) -> Element<'static, Message> {
    row![
        text(label).size(13).style(text::secondary),
        badge::coming_soon(fl!("badge-coming-soon")),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn toggle_row(label: String, description: String) -> Element<'static, Message> {
    row![
        column![
            text(label).size(14),
            text(description).size(11).style(text::secondary),
        ]
        .spacing(2)
        .width(Length::Fill),
        badge::coming_soon(fl!("badge-coming-soon")),
        Space::new().width(6),
        toggler(false),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}
