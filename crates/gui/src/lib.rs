use crate::app::App;

pub mod app;
mod localize;
mod message;
pub mod state;
mod store;
mod update;
mod view;
mod worker;

pub fn run() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    tracing::info!("Starting QCYx...");

    let icon = iced::window::icon::from_file_data(
        include_bytes!("../../../pkg/windows/Product.ico"),
        None,
    )
    .inspect_err(|err| tracing::warn!(%err, "failed to load window icon"))
    .ok();

    iced::application(App::new, App::update, App::view)
        .title(|_app: &App| "QCYx".to_string())
        .theme(App::theme)
        .subscription(|app: &App| app.subscription())
        .window(iced::window::Settings {
            min_size: Some(iced::Size::new(900.0, 680.0)),
            icon,
            ..Default::default()
        })
        .centered()
        .window_size((1000.0, 700.0))
        .resizable(true)
        .antialiasing(true)
        .run()
}
