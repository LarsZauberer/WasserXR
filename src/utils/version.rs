//! This is a helper utility struct that help you define versions numbers in a standardized way

use std::fmt::Display;

/// A simple utility struct that represents a version number by defining the major, minor and patch
/// number as usize. It is FFI compatible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}.{}.{}", self.major, self.minor, self.patch))
    }
}

#[cfg(test)]
mod test {
    use crate::utils::version::Version;
    use rstest::rstest;

    #[rstest]
    #[case(1, 2, 3, "1.2.3")]
    #[case(0, 1, 0, "0.1.0")]
    fn test_version_formatting(
        #[case] major: usize,
        #[case] minor: usize,
        #[case] patch: usize,
        #[case] expected: &str,
    ) {
        let version: Version = Version {
            major,
            minor,
            patch,
        };
        assert_eq!(format!("{}", version), expected);
    }
}
