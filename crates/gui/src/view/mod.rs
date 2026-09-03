pub mod components;
pub mod screens;
pub mod sections;
pub mod tabs;

use crate::{app::App, message::Message, state::AppState};
use iced::Element;

pub fn view(app: &App) -> Element<'_, Message> {
    match app.state {
        AppState::Connecting => screens::connecting::view(app),
        AppState::Connected => screens::shell::view(app),
        AppState::Error => screens::error::view(app),
    }
}
