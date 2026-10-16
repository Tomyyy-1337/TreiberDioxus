use dioxus::{fullstack::{CborEncoding, WebSocketOptions, Websocket}, prelude::*};

use crate::{BatteryState, CameraInterface, CurrentStats, FahrtInfo, FinishedFahrt, SateliteCount};

#[cfg(feature = "server")]
use dioxus::server::axum::Extension;
#[cfg(feature = "server")]
use tokio::sync::watch;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ServerMessage {
    BatteryUpdate(BatteryState),
    GpsCountUpdate(SateliteCount),
    SettingsCameraUpdate(CameraInterface),
    FahrtUpdate(Option<FahrtInfo>),
    CurrentStatsUpdate(CurrentStats),
    FinishedFahrtUpdate(FinishedFahrt),
}

#[get("/api/health")]
pub async fn health() -> Result<()> {
    Ok(())
}

#[get(
    "/ws", 
    mut battery: Extension<watch::Receiver<BatteryState>>, 
    mut gps_count: Extension<watch::Receiver<SateliteCount>>,
    mut camera_interface: Extension<watch::Receiver<CameraInterface>>,
    mut finished_fahrt: Extension<watch::Receiver<FinishedFahrt>>,
    mut fahrt_info: Extension<watch::Receiver<Option<FahrtInfo>>>,
    mut current_stats: Extension<watch::Receiver<CurrentStats>>, 
)]
pub async fn websocket_handler(options: WebSocketOptions) -> Result<Websocket<(), ServerMessage, CborEncoding>> {
    Ok(
        options.on_upgrade(move |mut socket| async move {
            battery.0.mark_changed();
            gps_count.0.mark_changed();
            camera_interface.0.mark_changed();
            fahrt_info.0.mark_changed();
            current_stats.0.mark_changed();

            loop {
                let message = tokio::select! {
                    _ = battery.0.changed() => {
                        ServerMessage::BatteryUpdate(*battery.0.borrow())
                    }
                    _ = gps_count.0.changed() => {
                        ServerMessage::GpsCountUpdate(*gps_count.0.borrow())
                    }
                    _ = camera_interface.0.changed() => {
                        ServerMessage::SettingsCameraUpdate(*camera_interface.0.borrow())
                    }
                    _ = fahrt_info.0.changed() => {
                        ServerMessage::FahrtUpdate(*fahrt_info.0.borrow())
                    }
                    _ = current_stats.0.changed() => {
                        ServerMessage::CurrentStatsUpdate(*current_stats.0.borrow())
                    }
                    _ = finished_fahrt.0.changed() => {
                        ServerMessage::FinishedFahrtUpdate(*finished_fahrt.0.borrow())
                    }
                };
                if socket.send(message).await.is_err() {
                    break;
                }
            }
        })
    )   
}