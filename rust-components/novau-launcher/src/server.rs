//! IPC server: listens on a Unix socket so the panel can wake the launcher.

use crate::desktop::App;
use std::sync::Arc;

pub async fn listen(_apps: Arc<Vec<App>>) {
    let path = novau_common::paths::runtime().join("launcher.sock");
    let _ = std::fs::remove_file(&path);
    let listener = match tokio::net::UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            log::warn!("failed to bind launcher socket at {path:?}: {e}");
            return;
        }
    };
    log::info!("launcher IPC listening at {path:?}");

    loop {
        match listener.accept().await {
            Ok((_s, _)) => {
                // Wake the launcher window — in production we'd raise via
                // the layer-shell set_keyboard_interactivity. Here we just
                // log.
                log::debug!("launcher wake event received");
            }
            Err(e) => {
                log::warn!("accept: {e}");
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        }
    }
}
