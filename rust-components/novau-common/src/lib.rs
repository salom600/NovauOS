//! novau-common — shared types, constants, and helpers for NovauOS.
//!
//! Every `novau-*` crate depends on this. Keep it small, dependency-light,
//! and stable across versions. Breaking changes here trigger a workspace
//! version bump.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Distribution name. Hard-coded everywhere; if you rename the distro,
/// change it here and rebuild.
pub const DISTRO_NAME: &str = "NovauOS";
pub const DISTRO_ID: &str = "novauos";
pub const DISTRO_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DISTRO_CODENAME: &str = "aurora";

/// Paths the rest of the system agrees on.
pub mod paths {
    use super::PathBuf;
    use std::sync::OnceLock;

    static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
    static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
    static CACHE_DIR: OnceLock<PathBuf> = OnceLock::new();
    static RUN_DIR: OnceLock<PathBuf> = OnceLock::new();

    /// Per-user config dir: `$XDG_CONFIG_HOME/novau` or `~/.config/novau`.
    pub fn config() -> &'static PathBuf {
        CONFIG_DIR.get_or_init(|| {
            if let Ok(x) = std::env::var("XDG_CONFIG_HOME") {
                PathBuf::from(x).join("novau")
            } else {
                dirs_home().join(".config").join("novau")
            }
        })
    }

    /// Per-user data dir: `$XDG_DATA_HOME/novau` or `~/.local/share/novau`.
    pub fn data() -> &'static PathBuf {
        DATA_DIR.get_or_init(|| {
            if let Ok(x) = std::env::var("XDG_DATA_HOME") {
                PathBuf::from(x).join("novau")
            } else {
                dirs_home().join(".local").join("share").join("novau")
            }
        })
    }

    /// Per-user cache dir: `$XDG_CACHE_HOME/novau` or `~/.cache/novau`.
    pub fn cache() -> &'static PathBuf {
        CACHE_DIR.get_or_init(|| {
            if let Ok(x) = std::env::var("XDG_CACHE_HOME") {
                PathBuf::from(x).join("novau")
            } else {
                dirs_home().join(".cache").join("novau")
            }
        })
    }

    /// Runtime dir: `/run/user/$UID/novau`.
    pub fn runtime() -> &'static PathBuf {
        RUN_DIR.get_or_init(|| {
            if let Ok(x) = std::env::var("XDG_RUNTIME_DIR") {
                PathBuf::from(x).join("novau")
            } else {
                PathBuf::from("/run/user").join(format!("{}", unsafe { libc::getuid() })).join("novau")
            }
        })
    }

    fn dirs_home() -> PathBuf {
        std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"))
    }
}

/// Ensure a directory exists, creating it (and parents) if missing.
pub fn ensure_dir(p: &Path) -> std::io::Result<()> {
    if !p.exists() {
        std::fs::create_dir_all(p)?;
    }
    Ok(())
}

/// Build semver-style version string for the welcome screen & neofetch.
pub fn long_version() -> String {
    format!("{} {} ({})", DISTRO_NAME, DISTRO_VERSION, DISTRO_CODENAME)
}

/// Identify the running session's compositor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Compositor {
    Sway,
    CosmicComp,
    Wayfire,
    Unknown,
}

impl Compositor {
    pub fn detect() -> Self {
        match std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().as_str() {
            "sway" => Self::Sway,
            "cosmic" => Self::CosmicComp,
            "wayfire" => Self::Wayfire,
            _ => Self::Unknown,
        }
    }
}

/// A user visible in the greeter. Mirrors /etc/passwd + accountsservice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemUser {
    pub name: String,
    pub uid: u32,
    pub real_name: Option<String>,
    pub home: PathBuf,
    pub shell: PathBuf,
    pub avatar_path: Option<PathBuf>,
}

/// Errors we surface to the UI.
#[derive(Debug, thiserror::Error)]
pub enum NovauError {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serde JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Serde RON: {0}")]
    Ron(#[from] ron::Error),
    #[error("Other: {0}")]
    Other(String),
}

pub type Result<T, E = NovauError> = std::result::Result<T, E>;

/// Initialize logging for any novau binary.
pub fn init_logging(component: &str) {
    let component = component.to_string();
    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    )
    .format_timestamp_secs()
    .format_target(false)
    .format(move |buf, record| {
        use std::io::Write;
        writeln!(
            buf,
            "[{} {}] {}",
            component,
            record.level(),
            record.args()
        )
    })
    .try_init();
}
