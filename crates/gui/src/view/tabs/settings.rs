use crate::app::App;
use crate::message::Message;
use crate::view::components::{badge, card};
use iced::widget::{Space, button, column, row, text, text_input, toggler};
use iced::{Alignment, Border, Element, Length, Theme};
use qcyx_core::disconnect_power_off::DisconnectPowerOff;
use qcyx_core::game_mode::GameMode;
use qcyx_core::notification_volume::NotificationVolume;
use qcyx_core::scheduled_power_off::ScheduledPowerOff;
use qcyx_core::sleep_mode::SleepMode;
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

    // Every toggle here maps to a wire-confirmed opcode — no placeholders.
    let toggles_section = card::panel(
        column![
            wired_toggle_row(
                fl!("settings-inear-toggle-label"),
                fl!("settings-inear-toggle-desc"),
                app.wear_detection
                    .map(|state| state.wear_detection)
                    .unwrap_or(false),
                &app.wear_detection_status,
                Message::WearDetectionToggled,
            ),
            wired_toggle_row(
                fl!("settings-game-mode-label"),
                fl!("settings-game-mode-desc"),
                matches!(app.game_mode, Some(GameMode::On)),
                &app.game_mode_status,
                |on| Message::SetGameMode(if on { GameMode::On } else { GameMode::Off }),
            ),
            wired_toggle_row(
                fl!("settings-sleep-mode-label"),
                fl!("settings-sleep-mode-desc"),
                matches!(app.sleep_mode, Some(SleepMode::On)),
                &app.sleep_mode_status,
                |on| Message::SetSleepMode(if on { SleepMode::On } else { SleepMode::Off }),
            ),
        ]
        .spacing(16),
    );

    let notification_volume_section = card::panel(
        column![
            text(fl!("settings-notification-volume-title"))
                .size(13)
                .style(text::secondary),
            row![
                volume_chip(
                    app,
                    NotificationVolume::Low,
                    fl!("settings-notification-volume-low")
                ),
                volume_chip(
                    app,
                    NotificationVolume::Medium,
                    fl!("settings-notification-volume-medium")
                ),
                volume_chip(
                    app,
                    NotificationVolume::High,
                    fl!("settings-notification-volume-high")
                ),
                volume_chip(
                    app,
                    NotificationVolume::Max,
                    fl!("settings-notification-volume-max")
                ),
            ]
            .spacing(10)
            .wrap(),
            status_line(&app.notification_volume_status),
        ]
        .spacing(10),
    );

    let scheduled_power_off_section = card::panel(
        column![
            column![
                text(fl!("settings-scheduled-poweroff-title")).size(14),
                text(fl!("settings-scheduled-poweroff-desc"))
                    .size(11)
                    .style(text::secondary),
            ]
            .spacing(2),
            row![
                chip(
                    fl!("settings-scheduled-poweroff-off"),
                    app.scheduled_power_off == Some(ScheduledPowerOff::Disabled),
                    Message::SetScheduledPowerOff(ScheduledPowerOff::Disabled),
                ),
                scheduled_minutes_chip(app, 15),
                scheduled_minutes_chip(app, 30),
                scheduled_minutes_chip(app, 60),
                scheduled_minutes_chip(app, 90),
            ]
            .spacing(10)
            .wrap(),
            row![
                text_input(
                    &fl!("settings-scheduled-poweroff-custom-placeholder"),
                    &app.scheduled_power_off_custom_input
                )
                .on_input(Message::ScheduledPowerOffCustomInputChanged)
                .on_submit(Message::ScheduledPowerOffCustomSubmit)
                .padding(10)
                .width(Length::Fixed(160.0)),
                button(text(fl!("settings-scheduled-poweroff-custom-button")).size(13))
                    .padding([10, 18])
                    .style(button::secondary)
                    .on_press(Message::ScheduledPowerOffCustomSubmit),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            status_line(&app.scheduled_power_off_status),
        ]
        .spacing(10),
    );

    let disconnect_power_off_section = card::panel(
        column![
            column![
                text(fl!("settings-disconnect-poweroff-title")).size(14),
                text(fl!("settings-disconnect-poweroff-desc"))
                    .size(11)
                    .style(text::secondary),
            ]
            .spacing(2),
            row![
                chip(
                    fl!("settings-disconnect-poweroff-never"),
                    app.disconnect_power_off == Some(DisconnectPowerOff::Never),
                    Message::SetDisconnectPowerOff(DisconnectPowerOff::Never),
                ),
                disconnect_minutes_chip(app, 5),
                disconnect_minutes_chip(app, 10),
                disconnect_minutes_chip(app, 30),
                disconnect_minutes_chip(app, 60),
            ]
            .spacing(10)
            .wrap(),
            status_line(&app.disconnect_power_off_status),
        ]
        .spacing(10),
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
        notification_volume_section,
        scheduled_power_off_section,
        disconnect_power_off_section,
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

/// A toggle row wired to a real setting — takes the current state and the
/// message to fire on flip. Every toggle in [`view`] uses this; there are no
/// stub/"coming soon" toggles left, since the official app has no more of
/// them beyond what's mapped here.
fn wired_toggle_row<'a>(
    label: String,
    description: String,
    enabled: bool,
    status: &'a Option<String>,
    on_toggle: impl Fn(bool) -> Message + 'static,
) -> Element<'a, Message> {
    column![
        row![
            column![
                text(label).size(14),
                text(description).size(11).style(text::secondary),
            ]
            .spacing(2)
            .width(Length::Fill),
            toggler(enabled).on_toggle(on_toggle),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        status_line(status),
    ]
    .spacing(6)
    .into()
}

fn volume_chip(app: &App, level: NotificationVolume, label: String) -> Element<'static, Message> {
    let selected = app.notification_volume == Some(level);
    chip(label, selected, Message::SetNotificationVolume(level))
}

fn scheduled_minutes_chip(app: &App, minutes: u16) -> Element<'static, Message> {
    let selected = app.scheduled_power_off == Some(ScheduledPowerOff::Minutes(minutes));
    chip(
        minutes_label(minutes),
        selected,
        Message::SetScheduledPowerOff(ScheduledPowerOff::Minutes(minutes)),
    )
}

fn disconnect_minutes_chip(app: &App, minutes: u16) -> Element<'static, Message> {
    let selected = app.disconnect_power_off == Some(DisconnectPowerOff::Minutes(minutes));
    chip(
        minutes_label(minutes),
        selected,
        Message::SetDisconnectPowerOff(DisconnectPowerOff::Minutes(minutes)),
    )
}

fn minutes_label(minutes: u16) -> String {
    fl!("settings-minutes-format", minutes = minutes.to_string())
}

/// Shared pill-shaped selectable button — same visual language as the
/// equalizer preset picker.
fn chip(label: String, selected: bool, message: Message) -> Element<'static, Message> {
    button(text(label).size(13))
        .padding([8, 16])
        .style(move |theme: &Theme, _status| {
            let ext = theme.extended_palette();
            if selected {
                iced::widget::button::Style {
                    background: Some(ext.primary.weak.color.scale_alpha(0.35).into()),
                    text_color: ext.primary.strong.color,
                    border: Border {
                        color: ext.primary.strong.color,
                        width: 1.0,
                        radius: 16.0.into(),
                    },
                    ..Default::default()
                }
            } else {
                iced::widget::button::Style {
                    background: Some(ext.background.base.color.into()),
                    text_color: theme.palette().text.scale_alpha(0.8),
                    border: Border {
                        color: ext.background.strong.color,
                        width: 1.0,
                        radius: 16.0.into(),
                    },
                    ..Default::default()
                }
            }
        })
        .on_press(message)
        .into()
}
