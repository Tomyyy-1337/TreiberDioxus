use std::time::Duration;
use tokio::{sync::watch, time::MissedTickBehavior};

use crate::{BatteryState, backend::server::I2C_INTERFACE};

/// Read battery state from the I2C interface and update the sender periodically.
pub async fn read_battery_task(battery_sender: watch::Sender<BatteryState>) {
    let mut timer = tokio::time::interval(Duration::from_secs(15));
    timer.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        let battery_percentage = I2C_INTERFACE.read_battery_percentage().await;
        let _ = battery_sender.send(BatteryState { percentage: battery_percentage });

        timer.tick().await;
    }
}
