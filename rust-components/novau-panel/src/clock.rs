//! Clock task: emit `Local::now()` every second.

use chrono::{DateTime, Local};
use std::time::Duration;

pub async fn tick_loop() -> DateTime<Local> {
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        return Local::now();
    }
}
