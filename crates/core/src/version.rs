//! Firmware version reading and parsing for [`crate::device::profile::VERSION_UUID`].
//!
//! Bytes 0-2 encode the left earbud's `major.minor.patch`.
//! Bytes 3-5 encode the right earbud's `major.minor.patch`.

/// Firmware version for each earbud.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirmwareVersion {
    pub left: Option<String>,
    pub right: Option<String>,
}

impl FirmwareVersion {
    /// Parses the raw payload.
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        let left = (data.len() >= 3).then(|| format!("{}.{}.{}", data[0], data[1], data[2]));
        let right = (data.len() >= 6).then(|| format!("{}.{}.{}", data[3], data[4], data[5]));

        Some(Self { left, right })
    }

    /// Formats both earbuds' versions for display, collapsing them into a
    /// single number when both sides report the same firmware — showing
    /// `"1.2.3 / 1.2.3"` is redundant when there's only one version in play.
    pub fn display(&self) -> Option<String> {
        match (&self.left, &self.right) {
            (Some(left), Some(right)) if left == right => Some(left.clone()),
            (Some(left), Some(right)) => Some(format!("{left} / {right}")),
            (Some(left), None) => Some(left.clone()),
            (None, Some(right)) => Some(right.clone()),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_earbuds() {
        let v = FirmwareVersion::parse(&[1, 2, 3, 4, 5, 6]).unwrap();
        assert_eq!(v.left.as_deref(), Some("1.2.3"));
        assert_eq!(v.right.as_deref(), Some("4.5.6"));
    }

    #[test]
    fn left_only_when_payload_short() {
        let v = FirmwareVersion::parse(&[9, 9, 9]).unwrap();
        assert_eq!(v.left.as_deref(), Some("9.9.9"));
        assert_eq!(v.right, None);
    }

    #[test]
    fn empty_payload_is_none() {
        assert_eq!(FirmwareVersion::parse(&[]), None);
    }

    #[test]
    fn display_collapses_matching_versions() {
        let v = FirmwareVersion::parse(&[1, 2, 3, 1, 2, 3]).unwrap();
        assert_eq!(v.display().as_deref(), Some("1.2.3"));
    }

    #[test]
    fn display_shows_both_when_different() {
        let v = FirmwareVersion::parse(&[1, 2, 3, 4, 5, 6]).unwrap();
        assert_eq!(v.display().as_deref(), Some("1.2.3 / 4.5.6"));
    }

    #[test]
    fn display_none_when_both_missing() {
        let v = FirmwareVersion {
            left: None,
            right: None,
        };
        assert_eq!(v.display(), None);
    }
}
