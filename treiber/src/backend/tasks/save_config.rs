use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::{CameraInterface, WlanSettings, backend::server::I2C_INTERFACE};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    camera_settings: CameraInterface,
    wlan_settings: WlanSettings,
}   

pub async fn save_config_task(
    mut camera_interface: watch::Receiver<CameraInterface>,
    mut wlan_settings: watch::Receiver<WlanSettings>,
) {
    let mut rate_limiter = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        rate_limiter.tick().await;

        tokio::select! {
            _ = camera_interface.changed() => {}
            _ = wlan_settings.changed() => {}
        }

        let config = Config {
            camera_settings: (*camera_interface.borrow()).clone(),
            wlan_settings: (*wlan_settings.borrow()).clone(),
        };

        let config_bytes = serde_json::to_vec(&config).unwrap();

        info!("Saving config to EEPROM");
        I2C_INTERFACE.write_eeprom_async(0, config_bytes).await;
    }
}