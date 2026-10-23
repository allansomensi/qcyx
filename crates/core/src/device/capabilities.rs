//! Model-specific restrictions on commands accepted by otherwise compatible devices.

use crate::command::{AncScene, opcode};
use crate::error::CoreError;
use crate::protocol::Command;

/// Supported control surfaces for the connected device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceCapabilities {
    pub transparency: bool,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        // Preserve existing behavior for models without a known restriction.
        Self { transparency: true }
    }
}

impl DeviceCapabilities {
    /// The first two QCY manufacturer-data bytes contain the big-endian vendor ID.
    /// Prefer that identity over the editable Bluetooth name. The public QCY
    /// product catalog identifies AilyBuds Pro as 0x4D48 / HT10ANC APP as 0x4D4C;
    /// Pro+ has separate IDs and is deliberately not included in this restriction.
    pub fn from_advertisement(name: Option<&str>, manufacturer_data: Option<&[u8]>) -> Self {
        let vendor_id = manufacturer_data
            .and_then(|data| data.get(..2))
            .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]));

        let ailybuds_pro = match vendor_id {
            Some(id) => matches!(id, 0x4D48 | 0x4D4C),
            None => name.is_some_and(|name| {
                let name = name.trim().to_ascii_lowercase();
                let name = name.strip_suffix("-app").unwrap_or(&name).trim();
                matches!(name, "qcy ailybuds pro" | "qcy ht10" | "qcy ht10anc app")
            }),
        };

        Self {
            transparency: !ailybuds_pro,
        }
    }

    pub fn supports_anc_scene(self, scene: AncScene) -> bool {
        self.transparency || !matches!(scene, AncScene::Transparency(_))
    }

    /// Validate at the write boundary, including commands issued through profiles
    /// or directly through `send_anc_and_confirm`, rather than only hiding a button.
    pub fn validate_command(self, command: &Command) -> Result<(), CoreError> {
        if !self.transparency
            && command.opcode == opcode::ANC_SETTING
            && command.parameters.first() == Some(&0x03)
        {
            return Err(CoreError::TransparencyUnsupported);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{self, NoiseCancellingMode, TransparencyMode};

    #[test]
    fn identifies_renamed_ailybuds_pro_by_vendor_id() {
        for data in [[0x4D, 0x48], [0x4D, 0x4C]] {
            assert!(
                !DeviceCapabilities::from_advertisement(Some("My earbuds"), Some(&data))
                    .transparency
            );
        }
    }

    #[test]
    fn recognizes_exact_name_aliases_without_a_complete_vendor_id() {
        for name in [
            "QCY AilyBuds Pro",
            " QCY AilyBuds Pro-APP ",
            "qcy ht10-app",
            "QCY HT10ANC APP",
        ] {
            for data in [None, Some(&[][..]), Some(&[0x4D][..])] {
                assert!(!DeviceCapabilities::from_advertisement(Some(name), data).transparency);
            }
        }
    }

    #[test]
    fn does_not_extend_the_restriction_to_pro_plus_or_other_models() {
        for name in [
            None,
            Some("QCY AilyBuds Pro+"),
            Some("QCY AilyBuds Pro+-APP"),
            Some("QCY MeloBuds Pro"),
            Some("QCY AilyBuds Pro Max"),
        ] {
            assert!(DeviceCapabilities::from_advertisement(name, None).transparency);
        }
        // A name changed to that of an HT10 must not override another vendor ID.
        for data in [[0x4D, 0x4B], [0x4D, 0x4D], [0x12, 0x34]] {
            assert!(
                DeviceCapabilities::from_advertisement(Some("QCY AilyBuds Pro"), Some(&data))
                    .transparency
            );
        }
    }

    #[test]
    fn blocks_all_transparency_variants_before_the_write() {
        let capabilities = DeviceCapabilities::from_advertisement(None, Some(&[0x4D, 0x48]));
        for mode in [
            TransparencyMode::VocalEnhancement,
            TransparencyMode::AmbientSound { level: 1 },
            TransparencyMode::AmbientSound { level: 6 },
        ] {
            let scene = AncScene::Transparency(mode);
            assert!(!capabilities.supports_anc_scene(scene));
            assert!(matches!(
                capabilities.validate_command(&command::anc_scene(scene)),
                Err(CoreError::TransparencyUnsupported)
            ));
        }
        // An unrecognized transparency sub-scene is still an unsafe mode-3 write.
        assert!(matches!(
            capabilities.validate_command(&Command::new(opcode::ANC_SETTING, vec![3, 255, 255])),
            Err(CoreError::TransparencyUnsupported)
        ));
    }

    #[test]
    fn keeps_supported_anc_and_queries_available() {
        let capabilities = DeviceCapabilities::from_advertisement(None, Some(&[0x4D, 0x48]));
        for scene in [
            AncScene::Normal,
            AncScene::NoiseCancelling(NoiseCancellingMode::Adaptive),
        ] {
            assert!(capabilities.supports_anc_scene(scene));
            assert!(
                capabilities
                    .validate_command(&command::anc_scene(scene))
                    .is_ok()
            );
        }
        assert!(
            capabilities
                .validate_command(&Command::new(0xFE, vec![3]))
                .is_ok()
        );
        let transparency =
            command::anc_scene(AncScene::Transparency(TransparencyMode::VocalEnhancement));
        assert!(
            DeviceCapabilities::default()
                .validate_command(&transparency)
                .is_ok()
        );
    }
}
