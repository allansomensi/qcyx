use crate::message::Message;
use crate::state::Tab;
use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length, Theme};

/// One entry in the sidebar navigation list. Highlighted with an accent
/// background + left bar when it's the active [`Tab`].
pub fn item<'a>(icon: &'static str, label: String, tab: Tab, active: Tab) -> Element<'a, Message> {
    let is_active = tab == active;

    let content = row![text(icon).size(16), text(label).size(14)]
        .spacing(12)
        .align_y(Alignment::Center);

    let bar_width: f32 = if is_active { 3.0 } else { 0.0 };

    let indicator = container(text(""))
        .width(bar_width)
        .height(Length::Fill)
        .style(move |theme: &Theme| container::Style {
            background: Some(theme.extended_palette().primary.strong.color.into()),
            border: iced::Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    let inner = row![indicator, container(content).padding([10, 12]).clip(true)]
        .spacing(0)
        .width(Length::Fill)
        .align_y(Alignment::Center);

    button(inner)
        .on_press(Message::TabSelected(tab))
        .width(Length::Fill)
        .padding(0)
        .clip(true)
        .style(move |theme: &Theme, status| {
            let ext = theme.extended_palette();

            let background = if is_active {
                Some(ext.primary.weak.color.scale_alpha(0.35).into())
            } else {
                match status {
                    button::Status::Hovered => {
                        Some(ext.background.strong.color.scale_alpha(0.5).into())
                    }
                    _ => None,
                }
            };

            let text_color = if is_active {
                ext.primary.strong.color
            } else {
                theme.palette().text
            };

            button::Style {
                background,
                text_color,
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}
