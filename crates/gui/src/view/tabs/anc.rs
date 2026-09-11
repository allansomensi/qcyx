use crate::app::App;
use crate::message::Message;
use crate::view::components::{badge, card};
use iced::widget::{Space, column, container, row, slider, text, toggler};
use iced::{Alignment, Element, Length};
use qcyx_core::command::{
    AMBIENT_LEVEL_MAX, AMBIENT_LEVEL_MIN, AncScene, NcLevel, NoiseCancellingMode, TransparencyMode,
};
use qcyx_i18n::fl;

pub fn view(app: &App) -> Element<'_, Message> {
    let header = column![
        text(fl!("anc-title")).size(20),
        text(fl!("anc-subtitle")).size(13).style(text::secondary),
    ]
    .spacing(4);

    let modes = card::panel(
        row![
            mode_button(app, "🔊", fl!("gui-anc-normal"), AncScene::Normal),
            mode_button(
                app,
                "🌬️",
                fl!("gui-anc-transparency"),
                AncScene::Transparency(TransparencyMode::VocalEnhancement)
            ),
        ]
        .spacing(12),
    );

    let status_text = app
        .status_log
        .clone()
        .unwrap_or_else(|| fl!("gui-anc-hint"));

    let status = container(text(status_text).size(13).style(text::secondary))
        .width(Length::Fill)
        .align_x(Alignment::Center);

    let mut content = column![header, Space::new().height(6), modes].spacing(18);

    let transparency_active = matches!(app.active_scene, Some(AncScene::Transparency(_)))
        || matches!(app.pending_scene, Some(AncScene::Transparency(_)));
    if transparency_active {
        content = content.push(transparency_section(app));
    }

    content
        .push(noise_cancelling_section(app))
        .push(status)
        .width(Length::Fill)
        .into()
}

fn transparency_section(app: &App) -> Element<'_, Message> {
    let vocal_enhancement_on = matches!(
        app.active_scene,
        Some(AncScene::Transparency(TransparencyMode::VocalEnhancement))
    );

    let ambient_level = app.transparency_level;

    let toggle_row = row![
        column![
            text(fl!("anc-vocal-enhancement-label")).size(14),
            text(fl!("anc-vocal-enhancement-desc"))
                .size(11)
                .style(text::secondary),
        ]
        .spacing(2)
        .width(Length::Fill),
        toggler(vocal_enhancement_on).on_toggle(move |on| {
            Message::SetAnc(AncScene::Transparency(if on {
                TransparencyMode::VocalEnhancement
            } else {
                // Back to ambient sound at the level the slider shows.
                TransparencyMode::AmbientSound {
                    level: ambient_level,
                }
            }))
        }),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let mut content = column![
        text(fl!("anc-transparency-detail-title"))
            .size(12)
            .style(text::secondary),
        toggle_row,
    ]
    .spacing(12);

    if !vocal_enhancement_on {
        content = content.push(
            column![
                text(fl!("anc-ambient-level-label"))
                    .size(12)
                    .style(text::secondary),
                slider(
                    AMBIENT_LEVEL_MIN..=AMBIENT_LEVEL_MAX,
                    app.transparency_level,
                    Message::TransparencyLevelChanged
                )
                .on_release(Message::SetAnc(AncScene::Transparency(
                    TransparencyMode::AmbientSound {
                        level: app.transparency_level,
                    }
                )))
                .step(1u8),
            ]
            .spacing(6),
        );
    }

    card::panel(content)
}

fn noise_cancelling_section(app: &App) -> Element<'_, Message> {
    let simple_row = row![
        submode_button(
            app,
            "🔄",
            fl!("anc-nc-adaptive"),
            AncScene::NoiseCancelling(NoiseCancellingMode::Adaptive)
        ),
        submode_button(
            app,
            "🌀",
            fl!("anc-nc-wind"),
            AncScene::NoiseCancelling(NoiseCancellingMode::WindResistance)
        ),
    ]
    .spacing(10);

    card::panel(
        column![
            text(fl!("anc-nc-title")).size(12).style(text::secondary),
            simple_row,
            level_row(app, "🏠", fl!("anc-nc-indoor"), NoiseCancellingMode::Indoor),
            level_row(
                app,
                "🚗",
                fl!("anc-nc-daily-commute"),
                NoiseCancellingMode::DailyCommute
            ),
            level_row(app, "🏙️", fl!("anc-nc-noisy"), NoiseCancellingMode::Noisy),
        ]
        .spacing(12),
    )
}

fn level_row<'a>(
    app: &'a App,
    icon: &'static str,
    label: String,
    constructor: fn(NcLevel) -> NoiseCancellingMode,
) -> Element<'a, Message> {
    row![
        row![text(icon).size(14), text(label).size(13)]
            .spacing(8)
            .align_y(Alignment::Center)
            .width(Length::Fill),
        level_button(app, "1", constructor(NcLevel::One)),
        level_button(app, "2", constructor(NcLevel::Two)),
        level_button(app, "3", constructor(NcLevel::Three)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn level_button<'a>(
    app: &'a App,
    label: &'static str,
    mode: NoiseCancellingMode,
) -> Element<'a, Message> {
    let scene = AncScene::NoiseCancelling(mode);
    let is_active = app.active_scene == Some(scene);

    let mut btn = iced::widget::button(text(label).size(13))
        .padding([6, 14])
        .style(move |theme: &iced::Theme, status| pill_style(theme, status, is_active));

    if app.pending_scene.is_none() {
        btn = btn.on_press(Message::SetAnc(scene));
    }
    btn.into()
}

fn submode_button<'a>(
    app: &'a App,
    icon: &'static str,
    label: String,
    scene: AncScene,
) -> Element<'a, Message> {
    let is_active = app.active_scene == Some(scene);

    let content = row![text(icon).size(14), text(label).size(13)]
        .spacing(8)
        .align_y(Alignment::Center);

    let mut btn = iced::widget::button(content)
        .padding([10, 16])
        .style(move |theme: &iced::Theme, status| pill_style(theme, status, is_active));

    if app.pending_scene.is_none() {
        btn = btn.on_press(Message::SetAnc(scene));
    }
    btn.into()
}

fn pill_style(
    theme: &iced::Theme,
    status: iced::widget::button::Status,
    is_active: bool,
) -> iced::widget::button::Style {
    let ext = theme.extended_palette();
    let hovered = status == iced::widget::button::Status::Hovered;

    let (background, text_color, border_color) = if is_active {
        (
            ext.primary.weak.color.scale_alpha(0.35),
            ext.primary.strong.color,
            ext.primary.strong.color,
        )
    } else {
        (
            if hovered {
                ext.background.strong.color.scale_alpha(0.5)
            } else {
                ext.background.base.color
            },
            theme.palette().text,
            ext.background.strong.color,
        )
    };

    iced::widget::button::Style {
        background: Some(background.into()),
        text_color,
        border: iced::Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn mode_button<'a>(
    app: &'a App,
    icon: &'static str,
    label: String,
    scene: AncScene,
) -> Element<'a, Message> {
    let is_active = match (app.active_scene, scene) {
        (Some(AncScene::Transparency(_)), AncScene::Transparency(_)) => true,
        (active, scene) => active == Some(scene),
    };
    let is_pending = app.pending_scene == Some(scene);

    let label_text = if is_pending {
        format!("{label}…")
    } else {
        label
    };

    let body = column![text(icon).size(22), text(label_text).size(13),]
        .spacing(8)
        .align_x(Alignment::Center);

    let footer: Element<'a, Message> =
        badge::status_slot(fl!("anc-active"), badge::Tone::Success, is_active);

    let mut btn = iced::widget::button(
        column![body, Space::new().height(6), footer]
            .spacing(0)
            .align_x(Alignment::Center)
            .width(Length::Fill),
    )
    .padding([18, 12])
    .width(Length::Fill)
    .style(move |theme: &iced::Theme, status| pill_style(theme, status, is_active));

    if app.pending_scene.is_none() {
        btn = btn.on_press(Message::SetAnc(scene));
    }

    btn.into()
}
