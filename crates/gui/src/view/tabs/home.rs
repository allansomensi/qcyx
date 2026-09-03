use crate::app::App;
use crate::message::Message;
use crate::view::components::{badge, battery, card};
use iced::widget::{Space, button, column, container, row, slider, text};
use iced::{Alignment, Element, Length, Theme};
use qcyx_core::battery::{BatteryComponent, BatteryStatus};
use qcyx_i18n::fl;

pub fn view(app: &App) -> Element<'_, Message> {
    let device_name = app
        .device_name
        .clone()
        .unwrap_or_else(|| fl!("home-device-fallback-name"));

    let device_header = card::panel(
        row![
            text("🎧").size(34),
            column![
                text(device_name).size(18),
                row![
                    text(fl!("home-firmware-label"))
                        .size(12)
                        .style(text::secondary),
                    text(firmware_display(app)).size(12).style(text::secondary),
                ]
                .spacing(4),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            status_pill(app),
        ]
        .spacing(16)
        .align_y(Alignment::Center),
    );

    let battery_row = card::panel(
        column![
            row![
                text(fl!("home-battery-title"))
                    .size(12)
                    .style(text::secondary),
                Space::new().width(Length::Fill),
                battery_status_hint(app),
            ]
            .align_y(Alignment::Center),
            row![
                card::tile(battery::gauge(
                    battery::Side::Left,
                    fl!("home-battery-left"),
                    battery_level(app, |b| b.left)
                )),
                card::tile(battery::gauge(
                    battery::Side::Right,
                    fl!("home-battery-right"),
                    battery_level(app, |b| b.right)
                )),
            ]
            .spacing(12),
        ]
        .spacing(12),
    );

    let balance_section = card::panel(
        column![
            row![
                text(fl!("anc-balance-title"))
                    .size(12)
                    .style(text::secondary),
                Space::new().width(Length::Fill),
                balance_reset_button(app),
            ]
            .align_y(Alignment::Center),
            row![
                text("L").size(12).style(text::secondary),
                slider(0..=100u8, app.balance, Message::BalanceChanged)
                    .on_release(Message::BalanceCommit(app.balance))
                    .step(1u8),
                text("R").size(12).style(text::secondary),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            container(
                text(balance_label(app.balance))
                    .size(12)
                    .style(text::secondary)
            )
            .width(Length::Fill)
            .align_x(Alignment::Center),
        ]
        .spacing(10),
    );

    let quick_actions = card::panel(
        column![
            text(fl!("home-quick-actions-title"))
                .size(12)
                .style(text::secondary),
            row![
                action_button("🔎", fl!("home-action-find-device")),
                action_button("⬆️", fl!("home-action-check-update")),
            ]
            .spacing(12),
        ]
        .spacing(12),
    );

    column![device_header, battery_row, balance_section, quick_actions]
        .spacing(18)
        .width(Length::Fill)
        .into()
}

/// Formats the firmware line.
fn firmware_display(app: &App) -> String {
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

/// Extracts one component's charge level as `0.0..=1.0`.
fn battery_level(app: &App, pick: impl Fn(&BatteryStatus) -> BatteryComponent) -> Option<f32> {
    app.battery
        .as_ref()
        .map(|status| pick(status).level as f32 / 100.0)
}

/// A small "stale" hint shown when the last battery read failed.
fn battery_status_hint(app: &App) -> Element<'_, Message> {
    if app.battery_error.is_some() && app.battery.is_some() {
        badge::status(fl!("home-battery-stale"), badge::Tone::Warning)
    } else {
        Space::new().height(0).into()
    }
}

fn status_pill(app: &App) -> Element<'_, Message> {
    match app.active_scene {
        Some(_) => badge::status(fl!("home-status-ready"), badge::Tone::Success),
        None => badge::status(fl!("home-status-idle"), badge::Tone::Neutral),
    }
}

/// "Reset to center" button for the balance slider.
fn balance_reset_button(app: &App) -> Element<'_, Message> {
    let is_centered = app.balance == 50;

    let content = row![
        text("↺").size(13),
        text(fl!("balance-reset-button")).size(12),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let mut btn =
        iced::widget::button(content)
            .padding([4, 10])
            .style(move |theme: &Theme, status| {
                let ext = theme.extended_palette();
                let hovered = matches!(status, iced::widget::button::Status::Hovered);

                iced::widget::button::Style {
                    background: if hovered {
                        Some(ext.background.strong.color.scale_alpha(0.4).into())
                    } else {
                        None
                    },
                    text_color: theme.palette().text.scale_alpha(if is_centered {
                        0.25
                    } else {
                        0.7
                    }),
                    border: iced::Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });

    if !is_centered {
        btn = btn.on_press(Message::BalanceCommit(50));
    }

    btn.into()
}

fn balance_label(value: u8) -> String {
    match value.cmp(&50) {
        std::cmp::Ordering::Equal => fl!("anc-balance-centered"),
        std::cmp::Ordering::Greater => {
            fl!(
                "anc-balance-right",
                percent = ((value - 50) * 2).to_string()
            )
        }
        std::cmp::Ordering::Less => {
            fl!("anc-balance-left", percent = ((50 - value) * 2).to_string())
        }
    }
}

fn action_button(icon: &'static str, label: String) -> Element<'static, Message> {
    column![
        button(
            column![text(icon).size(20), text(label).size(12)]
                .spacing(6)
                .align_x(Alignment::Center)
                .width(Length::Fill)
        )
        .padding([14, 18])
        .width(Length::Fill)
        .style(|theme: &Theme, status| {
            let ext = theme.extended_palette();
            let is_disabled = matches!(status, iced::widget::button::Status::Disabled);
            iced::widget::button::Style {
                background: Some(ext.background.base.color.into()),
                text_color: ext.background.base.text.scale_alpha(if is_disabled {
                    0.35
                } else {
                    0.5
                }),
                border: iced::Border {
                    color: ext.background.strong.color.scale_alpha(if is_disabled {
                        0.6
                    } else {
                        1.0
                    }),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        }),
        container(badge::coming_soon(fl!("badge-coming-soon")))
            .width(Length::Fill)
            .align_x(Alignment::Center),
    ]
    .spacing(6)
    .width(Length::Fill)
    .into()
}
