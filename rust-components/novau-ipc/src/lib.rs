//! novau-ipc — typed D-Bus and Unix-socket contracts for NovauOS.
//!
//! Components talk to each other via:
//!   1. D-Bus (system or session bus)
//!   2. Wayland protocols (handled in UI crates, not here)
//!   3. Local Unix sockets under `$XDG_RUNTIME_DIR/novau/`
//!
//! This crate only contains trait/struct definitions and the `zbus`
//! `proxy` macros (for client-side typed access). Server-side
//! `#[interface]` impls live in the component crates themselves.

use serde::{Deserialize, Serialize};

/// Bus name we own on the session bus.
pub const BUS_NAME: &str = "org.novau.Session";
pub const BUS_PATH: &str = "/org/novau/Session";

/// ── Panel ──────────────────────────────────────────────────────────────
///
/// The panel publishes the list of open toplevel windows (via
/// `wlr-foreign-toplevel`) and exposes a few control methods.

#[zbus::proxy(
    interface = "org.novau.Panel",
    default_service = "org.novau.Panel",
    default_path = "/org/novau/Panel"
)]
pub trait Panel {
    /// Returns `(app_id, title, focused)` tuples for each toplevel window.
    fn list_windows(&self) -> zbus::Result<Vec<(String, String, bool)>>;

    fn focus_window(&self, app_id: &str) -> zbus::Result<()>;

    fn close_window(&self, app_id: &str) -> zbus::Result<()>;

    fn set_brightness(&self, pct: u32) -> zbus::Result<()>;

    fn toggle_notifications(&self) -> zbus::Result<bool>;
}

/// ── Greeter bridge types ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, zbus::zvariant::Type)]
pub struct GreeterReply {
    pub success: bool,
    /// Empty string means "no error".
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, zbus::zvariant::Type)]
pub struct SessionDescriptor {
    pub id: String,
    pub name: String,
    pub icon: String,
}

/// ── Store ──────────────────────────────────────────────────────────────

#[zbus::proxy(
    interface = "org.novau.Store",
    default_service = "org.novau.Store",
    default_path = "/org/novau/Store"
)]
pub trait Store {
    fn install_package(&self, kind: &str, name: &str) -> zbus::Result<u32>;
    fn cancel_install(&self, job_id: u32) -> zbus::Result<()>;
    fn search(&self, query: &str) -> zbus::Result<Vec<StoreEntry>>;
}

#[derive(Debug, Clone, Serialize, Deserialize, zbus::zvariant::Type)]
pub struct StoreEntry {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub kind: String, // "apt" | "flatpak" | "wine" | "appimage"
    pub icon: String,
    pub rating: f32,
    pub installed: bool,
}

/// ── Unix-socket IPC (greeter ↔ panel ↔ launcher) ──────────────────────
///
/// For low-latency, non-D-Bus chatter (e.g. launcher hot-key events),
/// we use a simple newline-delimited JSON protocol over a Unix socket
/// at `$XDG_RUNTIME_DIR/novau/ipc.sock`.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcMessage {
    LauncherToggle,
    LauncherQuery {
        query: String,
    },
    PanelNotification {
        app: String,
        summary: String,
        body: String,
    },
    SessionLocked,
    SessionUnlocked,
    UserSwitch {
        user: String,
    },
    Quit,
}

impl IpcMessage {
    pub fn to_jsonl(&self) -> String {
        let mut s = serde_json::to_string(self).unwrap_or_default();
        s.push('\n');
        s
    }
}

/// Convenience: open a connection to the local IPC socket.
pub async fn connect_local() -> std::io::Result<tokio::net::UnixStream> {
    let path = novau_common::paths::runtime().join("ipc.sock");
    tokio::net::UnixStream::connect(&path).await
}
