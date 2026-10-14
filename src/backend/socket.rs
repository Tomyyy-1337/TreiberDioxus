use dioxus::{fullstack::{CborEncoding, WebSocketOptions, Websocket}, prelude::*};

use crate::backend::{battery::BatteryState, gps_receiver::SateliteCount};

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ServerMessage {
    BatteryUpdate(BatteryState),
    GpsCountUpdate(SateliteCount),
}

#[get(
    "/ws", 
    battery: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<BatteryState>>, 
    gps_count: dioxus::server::axum::Extension<tokio::sync::watch::Receiver<SateliteCount>>
)]
pub async fn websocket_handler(options: WebSocketOptions) -> Result<Websocket<(), ServerMessage, CborEncoding>> {
    Ok(
        options.on_upgrade(move |mut socket| async move {
            let mut battery_receiver = battery.0;
            battery_receiver.mark_changed();

            let mut gps_count_receiver = gps_count.0;
            gps_count_receiver.mark_changed();

            loop {
                let message = tokio::select! {
                    _ = battery_receiver.changed() => {
                        ServerMessage::BatteryUpdate(*battery_receiver.borrow())
                    }
                    _ = gps_count_receiver.changed() => {
                        ServerMessage::GpsCountUpdate(*gps_count_receiver.borrow())
                    }
                };
                if socket.send(message).await.is_err() {
                    break;
                }
            }
        })
    )   
}