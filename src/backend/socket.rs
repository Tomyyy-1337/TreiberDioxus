use dioxus::{fullstack::{CborEncoding, WebSocketOptions, Websocket}, prelude::*};

use crate::backend::{camera_interface::CameraInterface, fahrt::{Fahrt, FahrtInfo, FinishedFahrt}, gps_receiver::SateliteCount, tasks::{battery::BatteryState, current_stats::CurrentStats, read_accelerometer::CurrentBpm}};

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
    mut battery: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<BatteryState>>, 
    mut gps_count: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<SateliteCount>>,
    mut camera_interface: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<CameraInterface>>,
    mut finished_fahrt: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<FinishedFahrt>>,
    mut fahrt_info: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<Option<FahrtInfo>>>,
    mut current_stats: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<CurrentStats>>, 
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