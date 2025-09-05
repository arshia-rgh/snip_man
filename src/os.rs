//! OS utilities for detecting the current platform and simple helpers.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsKind {
    /// Linux or other Unix-like distros using the linux target.
    Linux,
    /// Apple macOS.
    Macos,
    /// Microsoft Windows.
    Windows,
    /// Any other 
    Unknown(&'static str),
}

impl OsKind {
    /// Returns the current OS as detected from `std::env::consts::OS`.
    pub fn current() -> Self {
        match std::env::consts::OS {
            "linux" => OsKind::Linux,
            "macos" => OsKind::Macos,
            "windows" => OsKind::Windows,
            other => OsKind::Unknown(other),
        }
    }

    /// Whether this platform is Unix-like (Linux or macOS).
    pub fn is_unix(&self) -> bool {
        matches!(self, OsKind::Linux | OsKind::Macos)
    }

    /// Returns a short, human-readable name for the OS.
    pub fn as_str(&self) -> &'static str {
        match self {
            OsKind::Linux => "linux",
            OsKind::Macos => "macos",
            OsKind::Windows => "windows",
            OsKind::Unknown(s) => s,
        }
    }
}

impl fmt::Display for OsKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Convenience wrapper returning the current [`OsKind`].
pub fn current_os() -> OsKind {
    OsKind::current()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_current_os() {
        let os = current_os();
        assert_eq!(os.as_str(), std::env::consts::OS);
    }
}
