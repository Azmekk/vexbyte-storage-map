/// A Linux device number split into its major and minor parts, as `/proc/self/mountinfo` and
/// `/sys/dev/block` write it (`259:3`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceNumber {
    pub major: u32,
    pub minor: u32,
}

impl DeviceNumber {
    /// Splits a raw `dev_t` (`st_dev` or `st_rdev`) the way glibc's `major()` and `minor()` do.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_raw(raw_device_number: u64) -> Self {
        let major = ((raw_device_number >> 8) & 0xfff) | ((raw_device_number >> 32) & !0xfff);
        let minor = (raw_device_number & 0xff) | ((raw_device_number >> 12) & !0xff);
        Self {
            major: major as u32,
            minor: minor as u32,
        }
    }

    /// Parses the `major:minor` form.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let (major, minor) = text.split_once(':')?;
        Some(Self {
            major: major.parse().ok()?,
            minor: minor.parse().ok()?,
        })
    }
}

impl std::fmt::Display for DeviceNumber {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}:{}", self.major, self.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_raw_device_numbers_like_glibc() {
        assert_eq!(
            DeviceNumber::from_raw(0x0801),
            DeviceNumber { major: 8, minor: 1 }
        );
        assert_eq!(
            DeviceNumber::from_raw(0x1_0303),
            DeviceNumber {
                major: 259,
                minor: 3
            }
        );
        assert_eq!(
            DeviceNumber::from_raw(0x10_082c),
            DeviceNumber {
                major: 8,
                minor: 300
            }
        );
    }

    #[test]
    fn parses_and_prints_the_colon_form() {
        let device = DeviceNumber::parse("259:3");
        assert_eq!(
            device,
            Some(DeviceNumber {
                major: 259,
                minor: 3
            })
        );
        assert_eq!(DeviceNumber::parse("nonsense"), None);
        assert_eq!(
            DeviceNumber {
                major: 8,
                minor: 16
            }
            .to_string(),
            "8:16"
        );
    }
}
