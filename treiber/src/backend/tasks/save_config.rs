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
    loop {
        tokio::select! {
            _ = camera_interface.changed() => {}
            _ = wlan_settings.changed() => {}
        }

        let config = Config {
            camera_settings: (*camera_interface.borrow()).clone(),
            wlan_settings: (*wlan_settings.borrow()).clone(),
        };

        let config_bytes = serde_json::to_vec(&config).unwrap();
        let config_len = config_bytes.len();
        let config_checksums: u8 = config_bytes.iter().fold(0, |acc, &b| acc.wrapping_add(b));

        info!("Saving config to EEPROM");
        I2C_INTERFACE.write_eeprom_async(0, vec![(config_len >> 8) as u8, config_len as u8]).await;
        I2C_INTERFACE.write_eeprom_async(3, vec![config_checksums]).await;
        I2C_INTERFACE.write_eeprom_async(4, config_bytes).await;
    }
}

pub async fn read_config() -> Option<(CameraInterface, WlanSettings)> {
    let config_len_bytes = I2C_INTERFACE.read_eeprom_async(0, 2).await;
    let config_len = ((config_len_bytes[0] as usize) << 8) | (config_len_bytes[1] as usize); 
    let checksums = I2C_INTERFACE.read_eeprom_async(3, 1).await[0];

    let config_bytes = I2C_INTERFACE.read_eeprom_async(4, config_len).await;
    let calculated_checksum: u8 = config_bytes.iter().fold(0, |acc, &b| acc.wrapping_add(b));
    if calculated_checksum != checksums {
        eprintln!("Config checksum mismatch: expected {}, got {}", checksums, calculated_checksum);
        return None;
    }
    let config: Config = match serde_json::from_slice(&config_bytes) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("Failed to parse config: {:?}", err);
            return None;
        }
    };

    Some((config.camera_settings, config.wlan_settings))
}