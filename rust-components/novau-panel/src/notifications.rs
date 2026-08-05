//! Notification receiver. Listens on the Novau IPC Unix socket.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub app: String,
    pub summary: String,
    pub body: String,
}

pub async fn listen() -> Notification {
    // Block forever waiting for a notification. If the socket doesn't
    // exist, we just sleep — the panel still renders.
    loop {
        let path = novau_common::paths::runtime().join("notify.sock");
        if let Ok(mut s) = tokio::net::UnixStream::connect(&path).await {
            use tokio::io::AsyncBufReadExt;
            let mut reader = tokio::io::BufReader::new(&mut s);
            let mut line = String::new();
            if reader.read_line(&mut line).await.is_ok() {
                if let Ok(n) = serde_json::from_str::<Notification>(line.trim()) {
                    return n;
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
