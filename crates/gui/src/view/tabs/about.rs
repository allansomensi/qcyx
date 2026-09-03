use crate::app::App;
use crate::message::Message;
use crate::view::components::card;
use iced::widget::{Space, column, row, text};
use iced::{Alignment, Element, Length};
use qcyx_i18n::fl;

pub fn view(_app: &App) -> Element<'_, Message> {
    let header = row![
        text("🎧").size(36),
        column![
            text("QCYx").size(22).font(iced::font::Font::MONOSPACE),
            text(format!("v{}", env!("CARGO_PKG_VERSION")))
                .size(12)
                .style(text::secondary),
        ]
        .spacing(2),
    ]
    .spacing(14)
    .align_y(Alignment::Center);

    let description = card::panel(
        column![
            text(fl!("about-description")).size(13),
            Space::new().height(6),
            info_row(fl!("about-repo-label"), "github.com/allansomensi/qcyx"),
            info_row(fl!("about-license-label"), "MIT"),
        ]
        .spacing(6),
    );

    let protocol_note = card::panel(
        column![
            text(fl!("about-protocol-title")).size(13),
            text(fl!("about-protocol-note"))
                .size(12)
                .style(text::secondary),
        ]
        .spacing(8),
    );

    column![header, description, protocol_note]
        .spacing(18)
        .width(Length::Fill)
        .into()
}

fn info_row(label: String, value: &'static str) -> Element<'static, Message> {
    row![
        text(label).size(12).style(text::secondary).width(120),
        text(value).size(12),
    ]
    .into()
}
