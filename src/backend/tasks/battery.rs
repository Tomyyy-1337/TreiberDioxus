use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct BatteryState {
    pub percentage: u8,
}

#[cfg(feature = "server")]
pub async fn update_battery_state(battery_sender: tokio::sync::watch::Sender<BatteryState>) {
    let mut timer = tokio::time::interval(tokio::time::Duration::from_secs(30));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        let battery_state = crate::backend::server::I2C_INTERFACE.read_battery_status().await;
        println!("Battery state: {:?}", battery_state);
        let _ = battery_sender.send(battery_state);

        timer.tick().await;
    }
}

#[server(ext: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<BatteryState>>)]
pub async fn current_battery() -> Result<u8> {
    Ok(ext.0.borrow().percentage)
}
