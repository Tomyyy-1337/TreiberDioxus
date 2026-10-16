// Read battery state from the I2C interface and update the sender periodically.
pub async fn update_battery_state(battery_sender: tokio::sync::watch::Sender<crate::BatteryState>) {
    let mut timer = tokio::time::interval(tokio::time::Duration::from_secs(30));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        let battery_state = crate::backend::server::I2C_INTERFACE.read_battery_status().await;
        let _ = battery_sender.send(battery_state);

        timer.tick().await;
    }
}
