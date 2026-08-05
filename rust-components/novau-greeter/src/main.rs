//! novau-greeter — Wayland-native login screen.
//!
//! Replaces `gdm3` / `sddm` / `lightdm`.
//!
//! Architecture:
//!   - UI built with `iced` (wayland backend, winit)
//!   - Auth via PAM (service: `novau-greeter`)
//!   - Session enumeration from `/usr/share/wayland-sessions` and
//!     `/usr/share/xsessions` (latter for X11 fallback only)
//!   - User list from `/etc/passwd` (UID ≥ 1000) augmented with
//!     face images from `/var/lib/AccountsService/icons/<user>`
//!
//! Lifecycle:
//!   1. systemd starts `novau-greeter.service` at `graphical.target`
//!   2. Greeter connects to the active VT, launches iced UI
//!   3. On successful PAM auth, spawns the selected session via
//!      `pam_open_session` + `systemd-run --scope --uid=<uid>`
//!   4. Greeter exits; the session takes over the VT.

mod auth;
mod session;
mod ui;
mod users;

use anyhow::Result;
use iced::{Application, Settings};

fn main() -> Result<()> {
    novau_common::init_logging("greeter");
    log::info!(
        "starting {} greeter v{}",
        novau_common::DISTRO_NAME,
        novau_common::DISTRO_VERSION
    );

    // Refuse to run as non-root — we need root to call pam_open_session
    // and switch VTs. In production this runs as a dedicated `novau-greeter`
    // user with the right polkit rules; for the dev branch we require root.
    if !nix::unistd::geteuid().is_root() {
        log::error!("novau-greeter must run as root (or via setcap)");
        std::process::exit(1);
    }

    let users = users::enumerate()?;
    let sessions = session::enumerate()?;

    log::info!("found {} users, {} sessions", users.len(), sessions.len());

    let state = ui::GreeterState::new(users, sessions);
    ui::Greeter::run(Settings::with_flags(state))?;
    Ok(())
}
