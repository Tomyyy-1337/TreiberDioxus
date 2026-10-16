use futures::StreamExt;
use tokio::{sync::watch, pin};

use crate::{SateliteCount, backend::{api::toggle_fahrt::Fahrt, hardware_interface::gps_interface::{GPSMessage, GPSModule}}};

pub async fn read_gps_task(
    gps_sender: watch::Sender<SateliteCount>,
    velocity_sender: watch::Sender<Velocity>,
    fahrt_sender: watch::Sender<Option<Fahrt>>,
) {
    let mut gps_module = GPSModule::new();
    let gps_stream = gps_module.messages_stream().await;
    pin!(gps_stream);

    let mut last_satelite_count = 0;

    while let Some(message) = gps_stream.next().await {
        match message {
            GPSMessage::GGA { satellites } => {
                let _ = gps_sender.send(SateliteCount { count: satellites as u8 });
                last_satelite_count = satellites as u8;
                if satellites as u8 <= 4 {
                    let _ = velocity_sender.send(Velocity(0.0));
                    let _ = fahrt_sender.send_modify(|fahrt| {
                        if let Some(fahrt) = fahrt {
                            fahrt.no_satellites();
                        }
                    });
                }
            }
            GPSMessage::RMC { latitude, longitude, speed_kmh } => {
                let velocity = if speed_kmh > 2.0 && last_satelite_count > 4 {speed_kmh} else {0.0};
                let _ = velocity_sender.send(Velocity(velocity));

                fahrt_sender.send_modify(|fahrt| {
                    if let Some(fahrt) = fahrt {
                        fahrt.add_gps_data(latitude, longitude, velocity);
                    }
                });
            }
        }
    }
}

pub struct Velocity(pub f32);