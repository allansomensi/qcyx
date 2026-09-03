use crate::{app::App, message::Message};
use iced::widget::{Space, column, container, text};
use iced::{Alignment, Element, Length};
use qcyx_i18n::fl;

pub fn view(_app: &App) -> Element<'_, Message> {
    let content = column![
        text("🎧").size(40),
        Space::new().height(10),
        text(fl!("waiting-title")).size(26),
        text(fl!("waiting-subtitle"))
            .size(13)
            .style(text::secondary),
    ]
    .align_x(Alignment::Center)
    .spacing(6);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
