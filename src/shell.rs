//! Shell detection and mapping helpers for completion generation.
//!
//! This module provides a small abstraction over clap_complete::Shell for
//! selecting one or more target shells from a user-friendly CLI value.

use clap::ValueEnum;
use clap_complete::Shell;
use std::env;
use std::path::Path;

/// Which shell(s) to target when generating/installing completions.
///
/// Values:
/// - Auto: detect the current shell from $SHELL (Unix) and use it; if detection fails, fall back to Bash, Zsh and Fish.
/// - Bash/Zsh/Fish: target only that specific shell.
/// - All: target Bash, Zsh and Fish.
#[derive(Clone, ValueEnum, Debug)]
pub enum ShellTarget {
    Auto,
    Bash,
    Zsh,
    Fish,
    All,
}

impl ShellTarget {
    /// Attempt to detect the current interactive shell from $SHELL.
    ///
    /// Returns Some(Bash|Zsh|Fish) if the basename of $SHELL matches one of the
    /// supported shells, otherwise None.
    pub fn detect() -> Option<Self> {
        env::var("SHELL")
            .ok()
            .and_then(|p| {
                Path::new(&p)
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
            })
            .and_then(|name| match name.as_str() {
                "bash" => Some(ShellTarget::Bash),
                "zsh" => Some(ShellTarget::Zsh),
                "fish" => Some(ShellTarget::Fish),
                _ => None,
            })
    }

    /// Convert the high-level target into one or more clap_complete shells.
    ///
    /// For `Auto`, tries detection first; if it fails, includes all shells so
    /// completions are still generated in a best-effort manner.
    pub fn to_shells(&self) -> Vec<Shell> {
        match self {
            ShellTarget::Bash => vec![Shell::Bash],
            ShellTarget::Zsh => vec![Shell::Zsh],
            ShellTarget::Fish => vec![Shell::Fish],
            ShellTarget::Auto => Self::detect()
                .map(|t| Self::to_shells(&t))
                .unwrap_or_else(|| vec![Shell::Bash, Shell::Zsh, Shell::Fish]),
            ShellTarget::All => vec![Shell::Bash, Shell::Zsh, Shell::Fish],
        }
    }
}
