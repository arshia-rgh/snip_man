use crate::os::OsKind;
use crate::shell::ShellTarget;
use crate::Cli;
use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::{env, fs, io};

pub mod state {
    use crate::init::user_dirs;
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use std::{fs, io};

    /// Persisted metadata written by `snipman install` to indicate that the
    /// one-time initialization has completed.
    #[derive(Serialize, Deserialize)]
    struct InstallState {
        /// Package version at install time (from CARGO_PKG_VERSION)
        version: String,
        /// Unix epoch seconds when installation finished
        installed_at_unix: u64,
    }

    /// Location of the JSON install-stamp file. Ensures the parent directory exists.
    pub fn install_stamp_path() -> io::Result<PathBuf> {
        let dirs = user_dirs()?;
        fs::create_dir_all(&dirs.data_root)?;
        Ok(dirs.data_root.join("install_state.json"))
    }

    /// Write the install-stamp with version and timestamp for gating.
    pub fn write_install_stamp() -> io::Result<()> {
        let stamp_path = install_stamp_path()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let state = InstallState {
            version: env!("CARGO_PKG_VERSION").to_string(),
            installed_at_unix: now,
        };
        let json = serde_json::to_string_pretty(&state)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(stamp_path, json)
    }

    /// Return true if the one-time installation has completed.
    ///
    /// Used by `main.rs` to gate functional commands until `snipman install` runs.
    pub fn is_installed() -> bool {
        install_stamp_path().map(|p| p.exists()).unwrap_or(false)
    }
}

/// Convenience holder for user-specific directories used during installation.
///
/// Notes (platform-specific):
/// - data_root:
///   - Linux: $XDG_DATA_HOME or ~/.local/share, joined with ".snipman"
///   - macOS: ~/Library/Application Support/.snipman
///   - Windows: %APPDATA% or %USERPROFILE%/AppData/Roaming, joined with ".snipman"
/// - config_root:
///   - Linux: $XDG_CONFIG_HOME or ~/.config/snipman
///   - macOS: ~/Library/Preferences/snipman
///   - Windows: %APPDATA%/snipman
struct UserDirs {
    home: PathBuf,
    man1: PathBuf,
    bash: PathBuf,
    zsh: PathBuf,
    fish: PathBuf,
    data_root: PathBuf,
    config_root: PathBuf,
}

/// Derive the per-user directories used for installing assets and state.
fn user_dirs() -> io::Result<UserDirs> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))?;

    // Data root (mirrors snippets storage root on Linux/macOS/Windows)
    let data_root = match OsKind::current() {
        OsKind::Windows => env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| {
                env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join("AppData").join("Roaming"))
            })
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".snipman"),
        OsKind::Macos => home
            .join("Library")
            .join("Application Support")
            .join(".snipman"),
        OsKind::Linux | OsKind::Unknown(_) => env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| Some(home.join(".local").join("share")))
            .unwrap()
            .join(".snipman"),
    };

    let config_root = match OsKind::current() {
        OsKind::Windows => env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("snipman"),
        OsKind::Macos => home.join("Library").join("Preferences").join("snipman"),
        OsKind::Linux | OsKind::Unknown(_) => env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| Some(home.join(".config")))
            .unwrap()
            .join("snipman"),
    };

    Ok(UserDirs {
        home: home.clone(),
        man1: home.join(".local/share/man/man1"),
        bash: home.join(".local/share/bash-completion/completions"),
        zsh: home.join(".local/share/zsh/site-functions"),
        fish: home.join(".config/fish/completions"),
        data_root,
        config_root,
    })
}

/// Ensure a unique, idempotent block is present in a text file.
///
/// If a block delimited by markers `# BEGIN {marker} (snipman)` and
/// `# END {marker} (snipman)` is not present, append one with the given body.
fn ensure_block_in_file(file: &Path, marker: &str, body: &str) -> io::Result<()> {
    let start = format!("# BEGIN {marker} (snipman)");
    let end = format!("# END {marker} (snipman)");
    let mut contents = fs::read_to_string(file).unwrap_or_default();
    if contents.contains(&start) {
        return Ok(());
    }
    if !contents.ends_with('\n') {
        contents.push('\n');
    }
    contents.push_str(&format!("\n{start}\n{body}\n{end}\n"));
    fs::write(file, contents)
}

/// Install user-scoped assets (man page, shell completions) and write the install stamp.
///
/// What it does:
/// - Creates the necessary directories under the user's home (man1, shell completion dirs, data/config roots).
/// - Generates `man` page from clap (written to ~/.local/share/man/man1/snipman.1 on Unix).
/// - Best-effort refresh of the man database via `mandb -q` (ignored on failure).
/// - Generates shell completions for the requested target(s) and writes them to conventional paths:
///   - Bash: ~/.local/share/bash-completion/completions/snipman
///   - Zsh:  ~/.local/share/zsh/site-functions/_snipman (name determined by clap_complete)
///   - Fish: ~/.config/fish/completions/snipman.fish
///   For Bash, a generated `*.bash` file is renamed to `snipman` for better autoloading.
/// - If `no_modify_rc` is false and the detected shell is Zsh, appends a small block to $ZDOTDIR/.zshrc (or ~/.zshrc)
///   to ensure the zsh completion fpath is set and compinit is invoked. The block is idempotent.
/// - Finally, writes a JSON stamp file under the data root to indicate initialization completed.
///
/// Returns an error only for unrecoverable filesystem operations or generation failures.
pub fn install_user_assets(target: ShellTarget, no_modify_rc: bool) -> io::Result<()> {
    let dirs = user_dirs()?;
    // Ensure dirs
    fs::create_dir_all(&dirs.man1)?;
    fs::create_dir_all(&dirs.bash)?;
    fs::create_dir_all(&dirs.zsh)?;
    fs::create_dir_all(&dirs.fish)?;
    fs::create_dir_all(&dirs.data_root)?;
    fs::create_dir_all(&dirs.config_root)?;

    // Man page
    let man_path = dirs.man1.join("snipman.1");
    {
        let cmd = Cli::command();
        let man = clap_mangen::Man::new(cmd);
        let mut file = fs::File::create(&man_path)?;
        man.render(&mut file)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    }
    // Refresh man DB quietly (best-effort)
    let _ = StdCommand::new("mandb")
        .args([
            "-q",
            dirs.home
                .join(".local/share/man")
                .to_string_lossy()
                .as_ref(),
        ])
        .status();

    // Completions
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    for sh in target.to_shells() {
        let out_dir = match sh {
            Shell::Bash => &dirs.bash,
            Shell::Zsh => &dirs.zsh,
            Shell::Fish => &dirs.fish,
            _ => continue,
        };
        match generate_to(sh, &mut cmd, &bin_name, out_dir) {
            Ok(path) => {
                if sh == Shell::Bash && path.extension().map(|e| e == "bash").unwrap_or(false) {
                    let target = out_dir.join("snipman");
                    let _ = fs::rename(&path, &target);
                    println!("Installed bash completion: {}", target.display());
                } else {
                    println!("Installed {:?} completion: {}", sh, path.display());
                }
            }
            Err(e) => eprintln!("Failed to generate {:?} completion: {}", sh, e),
        }
    }

    if !no_modify_rc {
        if let Some(ShellTarget::Zsh) = ShellTarget::detect() {
            let zshrc = env::var_os("ZDOTDIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| dirs.home.clone())
                .join(".zshrc");
            let block = format!(
                "fpath+=({})\nautoload -Uz compinit\ncompinit -u",
                dirs.zsh.to_string_lossy()
            );
            let _ = ensure_block_in_file(&zshrc, "SNIPMAN_ZSH_FPATH", &block);
        }
    }

    state::write_install_stamp()?;
    println!("Installed man page: {}", man_path.display());
    println!(
        "Install stamp written to {}",
        state::install_stamp_path()?.display()
    );
    Ok(())
}
