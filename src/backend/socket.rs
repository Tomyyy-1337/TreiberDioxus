use dioxus::{fullstack::{CborEncoding, WebSocketOptions, Websocket}, prelude::*};

use crate::backend::{battery::BatteryState, camera_interface::{self, CameraInterface}, fahrt::{Fahrt, FahrtInfo}, gps_receiver::SateliteCount, tasks::read_accelerometer::CurrentBpm};

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ServerMessage {
    BatteryUpdate(BatteryState),
    GpsCountUpdate(SateliteCount),
    SettingsCameraUpdate(CameraInterface),
    FahrtUpdate(Option<FahrtInfo>),
    BpmUpdate(CurrentBpm),
}

#[get("/api/health")]
pub async fn health() -> Result<()> {
    Ok(())
}

#[get(
    "/ws", 
    battery: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<BatteryState>>, 
    gps_count: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<SateliteCount>>,
    camera_interface: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<CameraInterface>>,
    fahrt_info: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<Option<FahrtInfo>>>,
    bpm: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<CurrentBpm>>,
)]
pub async fn websocket_handler(options: WebSocketOptions) -> Result<Websocket<(), ServerMessage, CborEncoding>> {
    Ok(
        options.on_upgrade(move |mut socket| async move {
            let mut battery_receiver = battery.0;
            battery_receiver.mark_changed();

            let mut gps_count_receiver = gps_count.0;
            gps_count_receiver.mark_changed();

            let mut camera_interface = camera_interface.0;
            camera_interface.mark_changed();

            let mut fahrt_receiver = fahrt_info.0;
            fahrt_receiver.mark_changed();

            let mut bpm_receiver = bpm.0;
            bpm_receiver.mark_changed();

            loop {
                let message = tokio::select! {
                    _ = battery_receiver.changed() => {
                        ServerMessage::BatteryUpdate(*battery_receiver.borrow())
                    }
                    _ = gps_count_receiver.changed() => {
                        ServerMessage::GpsCountUpdate(*gps_count_receiver.borrow())
                    }
                    _ = camera_interface.changed() => {
                        ServerMessage::SettingsCameraUpdate(*camera_interface.borrow())
                    }
                    _ = fahrt_receiver.changed() => {
                        ServerMessage::FahrtUpdate(*fahrt_receiver.borrow())
                    }
                    _ = bpm_receiver.changed() => {
                        ServerMessage::BpmUpdate(*bpm_receiver.borrow())
                    }
                };
                if socket.send(message).await.is_err() {
                    break;
                }
            }
        })
    )   
}