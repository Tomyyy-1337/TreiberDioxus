use futures::StreamExt;
use tokio::{sync::watch, pin};

use crate::{SateliteCount, backend::hardware_interface::gps_interface::{GPSMessage, GPSModule}};

pub async fn read_gps_task(gps_sender: watch::Sender<SateliteCount>) {
    let mut gps_module = GPSModule::new();
    let gps_stream = gps_module.messages_stream().await;
    pin!(gps_stream);

    while let Some(message) = gps_stream.next().await {
        match message {
            GPSMessage::GGA { satellites } => {
                let _ = gps_sender.send(SateliteCount { count: satellites as u8 });
            }
            GPSMessage::RMC { latitude, longitude, speed_kmh } => {

            }
        }
    }
}
