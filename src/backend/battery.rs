use dioxus::prelude::*;
use serde::{Deserialize, Serialize};


#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct BatteryState {
    pub percentage: u8,
}

#[cfg(feature = "server")]
pub async fn update_battery_state(battery_sender: tokio::sync::watch::Sender<BatteryState>) {
    loop {
        let new_percentage = (battery_sender.borrow().percentage + 10) % 101;
        let _ = battery_sender.send(BatteryState { percentage: new_percentage });
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await; 
    }
}

#[server(ext: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<BatteryState>>)]
pub async fn current_battery() -> Result<u8> {
    Ok(ext.0.borrow().percentage)
}
