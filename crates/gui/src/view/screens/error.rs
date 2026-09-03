use crate::{app::App, message::Message};
use iced::widget::{Space, button, column, container, text};
use iced::{Alignment, Element, Length, Theme};
use qcyx_i18n::fl;

pub fn view(app: &App) -> Element<'_, Message> {
    let error_msg = app
        .error_log
        .clone()
        .unwrap_or_else(|| fl!("error-unknown"));

    let content = column![
        text("⚠️").size(40),
        Space::new().height(10),
        text(fl!("error-title")).size(24).style(|theme: &Theme| {
            text::Style {
                color: Some(theme.extended_palette().danger.strong.color),
            }
        }),
        text(error_msg).size(13).style(text::secondary),
        Space::new().height(10),
        button(text(fl!("gui-retry")).size(14))
            .padding([10, 24])
            .style(button::primary)
            .on_press(Message::Retry),
    ]
    .align_x(Alignment::Center)
    .spacing(8);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(30)
        .into()
}
