use crate::app::App;
use crate::message::Message;
use crate::view::components::{badge, card};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length, Theme};
use qcyx_core::eq::EqPreset;
use qcyx_i18n::fl;

const PRESETS: [EqPreset; 7] = [
    EqPreset::Spatial,
    EqPreset::Default,
    EqPreset::Popular,
    EqPreset::BassBoost,
    EqPreset::Rock,
    EqPreset::Soft,
    EqPreset::Classic,
];

/// Returns the localized label for a preset.
fn preset_label(preset: EqPreset) -> String {
    match preset {
        EqPreset::Spatial => fl!("eq-preset-spatial"),
        EqPreset::Default => fl!("eq-preset-default"),
        EqPreset::Popular => fl!("eq-preset-popular"),
        EqPreset::BassBoost => fl!("eq-preset-bass"),
        EqPreset::Rock => fl!("eq-preset-rock"),
        EqPreset::Soft => fl!("eq-preset-soft"),
        EqPreset::Classic => fl!("eq-preset-classic"),
    }
}

/// Returns a representative icon for each preset.
fn preset_icon(preset: EqPreset) -> &'static str {
    match preset {
        EqPreset::Spatial => "🌐",
        EqPreset::Default => "🎚️",
        EqPreset::Popular => "🔥",
        EqPreset::BassBoost => "🥁",
        EqPreset::Rock => "🎸",
        EqPreset::Soft => "🍃",
        EqPreset::Classic => "🎻",
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    let header = column![
        text(fl!("eq-title")).size(20),
        text(fl!("eq-subtitle")).size(13).style(text::secondary),
    ]
    .spacing(4);

    let presets = card::panel(
        column![
            text(fl!("eq-preset-label")).size(12).style(text::secondary),
            row(PRESETS
                .iter()
                .map(|p| preset_chip(app, *p))
                .collect::<Vec<_>>())
            .spacing(10)
            .wrap(),
        ]
        .spacing(10),
    );

    let custom_note = card::panel(
        row![
            column![
                text(fl!("eq-custom-title")).size(14),
                text(fl!("eq-custom-desc")).size(11).style(text::secondary),
            ]
            .spacing(2)
            .width(Length::Fill),
            badge::coming_soon(fl!("badge-coming-soon")),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    );

    let status_text = app.eq_status.clone().unwrap_or_else(|| fl!("eq-hint"));

    let status = container(text(status_text).size(13).style(text::secondary))
        .width(Length::Fill)
        .align_x(Alignment::Center);

    column![header, presets, status, custom_note]
        .spacing(18)
        .width(Length::Fill)
        .into()
}

fn preset_chip(app: &App, preset: EqPreset) -> Element<'_, Message> {
    let selected = app.eq_preset == Some(preset);

    let content = row![
        text(preset_icon(preset)).size(14),
        text(preset_label(preset)).size(13),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    button(content)
        .padding([8, 16])
        .style(move |theme: &Theme, _status| {
            let ext = theme.extended_palette();
            if selected {
                iced::widget::button::Style {
                    background: Some(ext.primary.weak.color.scale_alpha(0.35).into()),
                    text_color: ext.primary.strong.color,
                    border: iced::Border {
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
                    border: iced::Border {
                        color: ext.background.strong.color,
                        width: 1.0,
                        radius: 16.0.into(),
                    },
                    ..Default::default()
                }
            }
        })
        .on_press(Message::SetEqPreset(preset))
        .into()
}
