use crate::message::Message;
use iced::widget::{container, row, text};
use iced::{Alignment, Element, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Success,
    Warning,
    Danger,
    Neutral,
}

impl Tone {
    fn color(self, theme: &Theme) -> iced::Color {
        let ext = theme.extended_palette();
        match self {
            Tone::Success => ext.success.strong.color,
            Tone::Warning => ext.primary.strong.color,
            Tone::Danger => ext.danger.strong.color,
            Tone::Neutral => ext.background.strong.color,
        }
    }
}

/// A small colored pill with a dot and label.
pub fn status<'a>(label: String, tone: Tone) -> Element<'a, Message> {
    status_slot(label, tone, true)
}

/// Same visual footprint as [`status`], but can be rendered fully
/// transparent while `visible` is `false`.
pub fn status_slot<'a>(label: String, tone: Tone, visible: bool) -> Element<'a, Message> {
    let alpha = if visible { 1.0 } else { 0.0 };

    container(
        row![
            container(text(""))
                .width(8)
                .height(8)
                .style(move |theme: &Theme| container::Style {
                    background: Some(tone.color(theme).scale_alpha(alpha).into()),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            text(label)
                .size(12)
                .style(move |theme: &Theme| text::Style {
                    color: Some(theme.palette().text.scale_alpha(alpha)),
                })
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([5, 10])
    .style(move |theme: &Theme| {
        let color = tone.color(theme);
        container::Style {
            background: Some(color.scale_alpha(0.12 * alpha).into()),
            border: iced::Border {
                color: color.scale_alpha(0.35 * alpha),
                width: 1.0,
                radius: 20.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

/// A tiny muted tag marking a control as scaffolding.
pub fn coming_soon<'a>(label: String) -> Element<'a, Message> {
    container(text(label).size(10))
        .padding([2, 8])
        .style(|theme: &Theme| {
            let ext = theme.extended_palette();
            container::Style {
                background: Some(ext.background.strong.color.scale_alpha(0.6).into()),
                text_color: Some(ext.background.base.text.scale_alpha(0.7)),
                border: iced::Border {
                    radius: 10.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}
