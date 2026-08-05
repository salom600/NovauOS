//! Window list via `wlr-foreign-toplevel-management-unstable-v1`.
//!
//! Real implementation will use `smithay-client-toolkit` to subscribe to
//! the protocol. For the initial scaffold we read `/proc/*/comm` and
//! return a placeholder list.

#[derive(Debug, Clone)]
pub struct WinInfo {
    pub app_id: String,
    pub title: String,
    pub focused: bool,
}

pub async fn watch() -> Vec<(String, String, bool)> {
    // Placeholder: in production this connects to wlroots foreign toplevel.
    Vec::new()
}
