use crate::view::{sections::sidebar, tabs};
use crate::{app::App, message::Message, state::Tab};
use iced::widget::{row, scrollable};
use iced::{Element, Length, Theme};

pub fn view(app: &App) -> Element<'_, Message> {
    let content = match app.tab {
        Tab::Home => tabs::home::view(app),
        Tab::Anc => tabs::anc::view(app),
        Tab::Equalizer => tabs::equalizer::view(app),
        Tab::Settings => tabs::settings::view(app),
        Tab::About => tabs::about::view(app),
    };

    let scrollable_content = scrollable(
        iced::widget::container(content)
            .width(Length::Fill)
            .padding([28, 36]),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    row![
        sidebar::view(app),
        iced::widget::container(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|theme: &Theme| iced::widget::container::Style {
                background: Some(theme.extended_palette().background.base.color.into()),
                ..Default::default()
            }),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
