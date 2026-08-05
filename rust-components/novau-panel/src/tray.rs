//! StatusNotifierItem (tray) D-Bus listener.
//! Minimal placeholder: full SNI implementation is a follow-up.

#[derive(Debug, Clone)]
pub struct TrayItem {
    pub id: String,
    pub icon: Option<String>,
    pub tooltip: Option<String>,
}

pub async fn watch() -> Vec<TrayItem> {
    // Placeholder: returns empty tray. Full SNI implementation comes later.
    Vec::new()
}
