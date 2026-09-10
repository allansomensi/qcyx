use crate::app::App;
use crate::localize::profile_label;
use crate::message::{Message, ProfileScope};
use crate::view::components::card;
use iced::widget::{Space, button, column, container, row, slider, text, text_input};
use iced::{Alignment, Element, Length, Theme};
use qcyx_core::eq::{CUSTOM_BAND_FREQS_HZ, CUSTOM_GAIN_MAX_DB, CUSTOM_GAIN_MIN_DB, EqPreset};
use qcyx_core::profile::Profile;
use qcyx_i18n::fl;

const PRESETS: [EqPreset; 7] = [
    EqPreset::Spatial,
    EqPreset::Default,
    EqPreset::Popular,
    EqPreset::BassBoost,
    EqPreset::Rock,
    EqPreset::Soft,
    EqPreset::Classic,
];

/// Returns the localized label for a preset.
fn preset_label(preset: EqPreset) -> String {
    match preset {
        EqPreset::Spatial => fl!("eq-preset-spatial"),
        EqPreset::Default => fl!("eq-preset-default"),
        EqPreset::Popular => fl!("eq-preset-popular"),
        EqPreset::BassBoost => fl!("eq-preset-bass"),
        EqPreset::Rock => fl!("eq-preset-rock"),
        EqPreset::Soft => fl!("eq-preset-soft"),
        EqPreset::Classic => fl!("eq-preset-classic"),
    }
}

/// Returns a representative icon for each preset.
fn preset_icon(preset: EqPreset) -> &'static str {
    match preset {
        EqPreset::Spatial => "🌐",
        EqPreset::Default => "🎚️",
        EqPreset::Popular => "🔥",
        EqPreset::BassBoost => "🥁",
        EqPreset::Rock => "🎸",
        EqPreset::Soft => "🍃",
        EqPreset::Classic => "🎻",
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    let header = column![
        text(fl!("eq-title")).size(20),
        text(fl!("eq-subtitle")).size(13).style(text::secondary),
    ]
    .spacing(4);

    let presets = card::panel(
        column![
            text(fl!("eq-preset-label")).size(12).style(text::secondary),
            row(PRESETS
                .iter()
                .map(|p| preset_chip(app, *p))
                .collect::<Vec<_>>())
            .spacing(10)
            .wrap(),
        ]
        .spacing(10),
    );

    let custom_note = card::panel(
        column![
            row![
                column![
                    text(fl!("eq-custom-title")).size(14),
                    text(fl!("eq-custom-desc")).size(11).style(text::secondary),
                ]
                .spacing(2)
                .width(Length::Fill),
                button(text(fl!("eq-custom-reset")).size(12))
                    .padding([6, 12])
                    .on_press(Message::EqCustomReset),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            column(
                CUSTOM_BAND_FREQS_HZ
                    .iter()
                    .enumerate()
                    .map(|(i, &freq)| band_row(app, i, freq))
                    .collect::<Vec<_>>()
            )
            .spacing(10),
        ]
        .spacing(14),
    );

    let status_text = app.eq_status.clone().unwrap_or_else(|| fl!("eq-hint"));

    let status = container(text(status_text).size(13).style(text::secondary))
        .width(Length::Fill)
        .align_x(Alignment::Center);

    let eq_profiles_section = eq_profiles_section(app);

    column![header, presets, status, custom_note, eq_profiles_section]
        .spacing(18)
        .width(Length::Fill)
        .into()
}

/// EQ-only profiles: save the currently-applied curve under a name, apply
/// a saved one, and import/export to JSON — a lighter-weight sibling of
/// the full-device profiles on the "Profiles" tab, scoped to just the EQ.
fn eq_profiles_section(app: &App) -> Element<'_, Message> {
    let rows: Vec<Element<'_, Message>> = app
        .eq_profiles
        .iter()
        .cloned()
        .map(eq_profile_row)
        .collect();

    let list = if rows.is_empty() {
        column![
            text(fl!("eq-profiles-empty"))
                .size(12)
                .style(text::secondary)
        ]
    } else {
        column(rows).spacing(10)
    };

    let status = app
        .eq_profiles_status
        .clone()
        .map(|s| -> Element<'_, Message> { text(s).size(12).style(text::secondary).into() })
        .unwrap_or_else(|| Space::new().height(0).into());

    card::panel(
        column![
            row![
                text(fl!("eq-profiles-title"))
                    .size(13)
                    .style(text::secondary),
                Space::new().width(Length::Fill),
                button(text(fl!("profiles-import-button")).size(12))
                    .padding([6, 12])
                    .style(button::secondary)
                    .on_press(Message::ImportProfile(ProfileScope::Eq)),
            ]
            .align_y(Alignment::Center),
            list,
            row![
                text_input(&fl!("profiles-name-placeholder"), &app.new_eq_profile_name)
                    .on_input(|value| Message::NewProfileNameChanged(ProfileScope::Eq, value))
                    .on_submit(Message::SaveCurrentAsProfile(ProfileScope::Eq))
                    .padding(10)
                    .width(Length::Fill),
                button(text(fl!("eq-profiles-save-button")).size(13))
                    .padding([10, 18])
                    .style(button::secondary)
                    .on_press(Message::SaveCurrentAsProfile(ProfileScope::Eq)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            status,
        ]
        .spacing(10),
    )
}

fn eq_profile_row(profile: Profile) -> Element<'static, Message> {
    let name = profile.name.clone();
    let label = profile_label(&profile.name);

    card::tile(
        row![
            text(label).size(13).width(Length::Fill),
            button(text(fl!("profiles-apply-button")).size(12))
                .padding([6, 10])
                .style(button::secondary)
                .on_press(Message::ApplyProfile(ProfileScope::Eq, profile.clone())),
            button(text(fl!("profiles-export-button")).size(12))
                .padding([6, 10])
                .style(button::text)
                .on_press(Message::ExportProfile(ProfileScope::Eq, profile)),
            button(text(fl!("profiles-delete-button")).size(12))
                .padding([6, 10])
                .style(button::text)
                .on_press(Message::DeleteProfile(ProfileScope::Eq, name)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
}

fn preset_chip(app: &App, preset: EqPreset) -> Element<'_, Message> {
    let selected = app.eq_preset == Some(preset);

    let content = row![
        text(preset_icon(preset)).size(14),
        text(preset_label(preset)).size(13),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    button(content)
        .padding([8, 16])
        .style(move |theme: &Theme, _status| {
            let ext = theme.extended_palette();
            if selected {
                iced::widget::button::Style {
                    background: Some(ext.primary.weak.color.scale_alpha(0.35).into()),
                    text_color: ext.primary.strong.color,
                    border: iced::Border {
                        color: ext.primary.strong.color,
                        width: 1.0,
                        radius: 16.0.into(),
                    },
                    ..Default::default()
                }
            } else {
                iced::widget::button::Style {
                    background: Some(ext.background.base.color.into()),
                    text_color: theme.palette().text.scale_alpha(0.8),
                    border: iced::Border {
                        color: ext.background.strong.color,
                        width: 1.0,
                        radius: 16.0.into(),
                    },
                    ..Default::default()
                }
            }
        })
        .on_press(Message::SetEqPreset(preset))
        .into()
}

/// One custom-EQ band: frequency label, gain slider, and the current
/// gain readout. `band` is the index into [`CUSTOM_BAND_FREQS_HZ`] /
/// `app.eq_custom_bands` (0 = 31 Hz .. 9 = 16 kHz).
fn band_row(app: &App, band: usize, freq_hz: u16) -> Element<'_, Message> {
    let value = app.eq_custom_bands[band];

    row![
        container(text(freq_label(freq_hz)).size(12).style(text::secondary)).width(46),
        slider(CUSTOM_GAIN_MIN_DB..=CUSTOM_GAIN_MAX_DB, value, move |v| {
            Message::EqCustomBandChanged(band, v)
        })
        .on_release(Message::EqCustomBandCommit(band, value))
        .step(1i16)
        .width(Length::Fill),
        container(text(gain_label(value)).size(12).style(text::secondary)).width(40),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

/// Formats a band's center frequency the way the device's own app does:
/// whole kHz above 1000 Hz ("2k", "16k"), plain Hz below it ("31", "500").
fn freq_label(freq_hz: u16) -> String {
    if freq_hz >= 1000 {
        format!("{}k", freq_hz / 1000)
    } else {
        freq_hz.to_string()
    }
}

/// Formats a gain in dB with an explicit sign, e.g. "+3", "-8", "0".
fn gain_label(gain_db: i16) -> String {
    match gain_db.cmp(&0) {
        std::cmp::Ordering::Greater => format!("+{gain_db}"),
        _ => gain_db.to_string(),
    }
}
