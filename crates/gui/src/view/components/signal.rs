//! A small Wi-Fi-style signal-strength indicator driven by the connected
//! peripheral's BLE RSSI, with a hover tooltip explaining the reading.

use crate::message::Message;
use iced::widget::{column, container, row, text, tooltip};
use iced::{Alignment, Element, Length, Theme};
use qcyx_i18n::fl;

/// Number of bars drawn.
const BAR_COUNT: usize = 4;

/// How many bars (0..=4) light up for a given RSSI reading, in dBm.
/// Thresholds follow the common rule of thumb for BLE: closer to 0 is
/// stronger, -100 or below is effectively unusable.
fn bars_for_rssi(rssi: i16) -> usize {
    match rssi {
        r if r >= -60 => 4,
        r if r >= -70 => 3,
        r if r >= -80 => 2,
        r if r >= -90 => 1,
        _ => 0,
    }
}

/// The signal icon, wrapped in a tooltip. `rssi` is the last successful
/// reading in dBm (`None` while unknown or unsupported by the platform).
pub fn indicator<'a>(rssi: Option<i16>) -> Element<'a, Message> {
    let lit = rssi.map(bars_for_rssi).unwrap_or(0);
    let known = rssi.is_some();

    let bars = row((0..BAR_COUNT)
        .map(|i| bar(i, lit, known))
        .collect::<Vec<_>>())
    .spacing(2)
    .align_y(Alignment::End);

    let hint = hint_text(rssi);

    tooltip(
        container(bars).padding([2, 2]),
        container(
            column![
                text(fl!("signal-tooltip-title")).size(12),
                text(hint).size(11).style(text::secondary),
            ]
            .spacing(2),
        )
        .padding(10)
        .max_width(220.0)
        .style(|theme: &Theme| {
            let ext = theme.extended_palette();
            container::Style {
                background: Some(ext.background.strong.color.into()),
                border: iced::Border {
                    color: ext.background.base.color,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        }),
        tooltip::Position::Bottom,
    )
    .into()
}

/// One vertical bar. Bars grow taller left-to-right, like a Wi-Fi icon;
/// `index < lit` decides whether it's painted "on".
fn bar<'a>(index: usize, lit: usize, known: bool) -> Element<'a, Message> {
    let height = 5.0 + index as f32 * 3.0;
    let active = known && index < lit;

    container(text(""))
        .width(Length::Fixed(3.0))
        .height(Length::Fixed(height))
        .style(move |theme: &Theme| {
            let ext = theme.extended_palette();
            let color = if active {
                ext.success.strong.color
            } else {
                ext.background.strong.color.scale_alpha(0.5)
            };
            container::Style {
                background: Some(color.into()),
                border: iced::Border {
                    radius: 1.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

/// Explains the current reading in the tooltip body.
fn hint_text(rssi: Option<i16>) -> String {
    match rssi {
        Some(value) => fl!("signal-tooltip-value", rssi = value.to_string()),
        None => fl!("signal-tooltip-unknown"),
    }
}
