use crate::message::Message;
use iced::widget::container;
use iced::{Element, Length, Theme};

/// The standard rounded, faintly-outlined panel used to group content
/// throughout the app (settings groups, battery tiles, the ANC panel...).
pub fn panel<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(18)
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let ext = theme.extended_palette();
            container::Style {
                background: Some(ext.background.weak.color.into()),
                border: iced::Border {
                    color: ext.background.strong.color,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// A slightly more subdued panel, used for nested/secondary groupings
/// inside a [`panel`] (e.g. a single battery tile inside the battery row).
pub fn tile<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(14)
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let ext = theme.extended_palette();
            container::Style {
                background: Some(ext.background.base.color.into()),
                border: iced::Border {
                    color: ext.background.strong.color.scale_alpha(0.6),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}
