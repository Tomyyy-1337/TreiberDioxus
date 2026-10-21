use dioxus::{fullstack::{CborEncoding, WebSocketOptions, Websocket}, prelude::*};
#[cfg(feature = "server")]
use futures::StreamExt;
#[cfg(feature = "server")]
use tokio_stream::{wrappers::WatchStream};

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
    Ok(options.on_upgrade(move |mut socket| async move {
        battery.0.mark_changed();
        gps_count.0.mark_changed();
        camera_interface.0.mark_changed();
        fahrt_info.0.mark_changed();
        current_stats.0.mark_changed();
        finished_fahrt.0.mark_unchanged();

        let battery_stream = WatchStream::from_changes(battery.0).map(ServerMessage::BatteryUpdate);
        let gps_count_stream = WatchStream::from_changes(gps_count.0).map(ServerMessage::GpsCountUpdate);
        let camera_interface_stream = WatchStream::from_changes(camera_interface.0).map(ServerMessage::SettingsCameraUpdate);
        let fahrt_info_stream = WatchStream::from_changes(fahrt_info.0).map(ServerMessage::FahrtUpdate);
        let current_stats_stream = WatchStream::from_changes(current_stats.0).map(ServerMessage::CurrentStatsUpdate);
        let finished_fahrt_stream = WatchStream::from_changes(finished_fahrt.0).map(ServerMessage::FinishedFahrtUpdate);

        let mut merged_stream = futures::stream::select_all([
            battery_stream.boxed(),
            gps_count_stream.boxed(),
            camera_interface_stream.boxed(),
            fahrt_info_stream.boxed(),
            current_stats_stream.boxed(),
            finished_fahrt_stream.boxed()
        ]);

        while let Some(message) = merged_stream.next().await {
            if socket.send(message).await.is_err() {
                break;
            }
        }
    }))   
}