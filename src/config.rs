//! Multi-path `.env` loader for Nisaba binaries.
//!
//! All three binaries (dashboard, scraper, sidecar) previously called
//! `dotenvy::dotenv().ok()` which only searches the current working
//! directory. That works for `cargo run` from the project root but
//! breaks on installed binaries whose CWD is `C:\Program Files\...`
//! (no `.env` there).
//!
//! v12.3 — call `load_env_multi_path()` instead. Searches in priority order:
//!
//! 1. **Per-user config**:
//!    - Windows: `%APPDATA%\Nisaba Terminal\.env`
//!    - Linux: `$XDG_CONFIG_HOME/nisaba-terminal/.env` or `$HOME/.config/nisaba-terminal/.env`
//!    - macOS: `$HOME/Library/Application Support/Nisaba Terminal/.env`
//! 2. **All-users config**:
//!    - Windows: `%PROGRAMDATA%\Nisaba Capital Charting\Nisaba Terminal\.env`
//!    - Linux: `/etc/nisaba-terminal/.env`
//!    - macOS: `/Library/Application Support/Nisaba Terminal/.env`
//! 3. **Project-root `.env`** (via `dotenvy::dotenv()`) — for `cargo run` dev mode
//!
//! Returns the loaded path so the caller can log which file was used.
//! Multiple files can be loaded; later loads do NOT overwrite earlier
//! values (dotenvy default behavior).

use std::path::PathBuf;

/// Per-user config directory for Nisaba Terminal.
pub fn user_config_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("Nisaba Terminal"))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|p| {
            PathBuf::from(p)
                .join("Library")
                .join("Application Support")
                .join("Nisaba Terminal")
        })
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg).join("nisaba-terminal"));
        }
        std::env::var_os("HOME")
            .map(|p| PathBuf::from(p).join(".config").join("nisaba-terminal"))
    }
}

/// All-users (system-wide) config directory for Nisaba Terminal.
pub fn system_config_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("PROGRAMDATA").map(|p| {
            PathBuf::from(p)
                .join("Nisaba Capital Charting")
                .join("Nisaba Terminal")
        })
    }
    #[cfg(target_os = "macos")]
    {
        Some(
            PathBuf::from("/Library/Application Support").join("Nisaba Terminal"),
        )
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        Some(PathBuf::from("/etc/nisaba-terminal"))
    }
}

/// Source from which `.env` values were loaded.
#[derive(Debug, Clone)]
pub enum EnvSource {
    /// Per-user config: `%APPDATA%\Nisaba Terminal\.env` (or XDG/macOS equivalent)
    User(PathBuf),
    /// All-users config: `%PROGRAMDATA%\Nisaba Capital Charting\...\.env`
    System(PathBuf),
    /// Project-root `.env` via `dotenvy::dotenv()` — dev mode
    Cwd,
    /// No `.env` file found at any path
    None,
}

/// Load `.env` from the first path that exists, searching in priority:
/// per-user → all-users → CWD. Returns the source for logging.
///
/// Multiple files can be loaded in sequence; existing env vars are NOT
/// overwritten (dotenvy default). Caller decides which to log.
pub fn load_env_multi_path() -> EnvSource {
    if let Some(dir) = user_config_dir() {
        let path = dir.join(".env");
        if path.exists() {
            if dotenvy::from_path(&path).is_ok() {
                return EnvSource::User(path);
            }
        }
    }

    if let Some(dir) = system_config_dir() {
        let path = dir.join(".env");
        if path.exists() {
            if dotenvy::from_path(&path).is_ok() {
                return EnvSource::System(path);
            }
        }
    }

    // Fall back to CWD `.env` (dev mode via cargo run)
    if dotenvy::dotenv().is_ok() {
        return EnvSource::Cwd;
    }

    EnvSource::None
}

/// Wrapper that loads `.env` and prints a one-line diagnostic to stderr.
/// Use this from binary `main()` for visibility during install debugging.
pub fn load_env_and_log(binary_name: &str) {
    match load_env_multi_path() {
        EnvSource::User(p) => {
            eprintln!("[{binary_name}] env loaded from user config: {}", p.display())
        }
        EnvSource::System(p) => {
            eprintln!("[{binary_name}] env loaded from system config: {}", p.display())
        }
        EnvSource::Cwd => eprintln!("[{binary_name}] env loaded from CWD .env (dev mode)"),
        EnvSource::None => {
            eprintln!("[{binary_name}] WARNING: no .env file found at any path");
            eprintln!("[{binary_name}]   tried: user config, system config, CWD");
            eprintln!("[{binary_name}]   create one of these to provide API keys + DATABASE_URL:");
            if let Some(d) = user_config_dir() {
                eprintln!("[{binary_name}]   - {}\\.env (per-user, no admin needed)", d.display());
            }
            if let Some(d) = system_config_dir() {
                eprintln!("[{binary_name}]   - {}\\.env (all-users, needs admin)", d.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_config_dir_resolves() {
        // Should resolve to some path on every platform with HOME or APPDATA set
        let dir = user_config_dir();
        assert!(dir.is_some(), "user_config_dir should resolve on this platform");
    }

    #[test]
    fn system_config_dir_resolves() {
        let dir = system_config_dir();
        assert!(dir.is_some(), "system_config_dir should resolve on this platform");
    }
}
