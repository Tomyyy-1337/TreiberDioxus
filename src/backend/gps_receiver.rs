use dioxus::prelude::*;
use serde::{Deserialize, Serialize};


#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct SateliteCount {
    pub count: u8,
}

#[cfg(feature = "server")]
pub async fn update_gps(gps_sender: tokio::sync::watch::Sender<SateliteCount>) {
    loop {
        let new_count = (gps_sender.borrow().count + 1) % 16;
        let _ = gps_sender.send(SateliteCount { count: new_count });
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
