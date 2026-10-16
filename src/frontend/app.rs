use std::time::Duration;

use dioxus::CapturedError;
use dioxus::fullstack::{CborEncoding, WebSocketOptions, Websocket};
use dioxus::prelude::*;
use dioxus_sdk::storage::{LocalStorage, use_storage};

use crate::{BatteryState, CameraInterface, CurrentStats};
use crate::backend::fahrt::FahrtInfo;
use crate::backend::socket::{ServerMessage, health, websocket_handler};

use super::frondend_data::*;
use super::router::Route;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

pub static BATTERY_STATE: GlobalSignal<BatteryState> = GlobalSignal::new(|| BatteryState { percentage: 0});
pub static GPS_COUNT: GlobalSignal<u8> = GlobalSignal::new(|| 0);
pub static WEBSOCKET_CONNECTED: GlobalSignal<bool> = GlobalSignal::new(|| false);
pub static SETTINGS_CAMERA: GlobalSignal<CameraInterface> = GlobalSignal::new(|| CameraInterface::new());
pub static FAHRT: GlobalSignal<Option<FahrtInfo>> = GlobalSignal::new(|| None);
pub static CURRENT_STATS: GlobalSignal<CurrentStats> = GlobalSignal::new(|| CurrentStats { bpm: 0.0, velocity: 0.0 });


fn handle_websocket_message(message: ServerMessage) {
    match message {
        ServerMessage::BatteryUpdate(battery_message) => {
            *BATTERY_STATE.write() = battery_message;
        }
        ServerMessage::GpsCountUpdate(gps_count_message) => {
            *GPS_COUNT.write() = gps_count_message.count;
        }
        ServerMessage::SettingsCameraUpdate(camera_message) => {
            *SETTINGS_CAMERA.write() = camera_message;
        }
        ServerMessage::FahrtUpdate(fahrt_message) => {
            *FAHRT.write() = fahrt_message;
        }
        ServerMessage::CurrentStatsUpdate(current_stats_message) => {
            *CURRENT_STATS.write() = current_stats_message;
        }
        ServerMessage::FinishedFahrtUpdate(finished_fahrt) => {
            // Add to Fahrtenbuch
        }
    }
}

#[component]
pub fn App() -> Element {
    #[cfg(target_os = "android")]
    use_effect(|| {
        if let Err(err) = super::android_status_bar::set_status_bar_color(0xFF000000) {
            eprintln!("failed to set status bar colour: {err}");
        }
    });

    let theme = use_storage::<LocalStorage, _>("theme".to_string(), || Theme::Dark);
    let theme = use_context_provider(|| theme);

    let auto_rotate = use_storage::<LocalStorage, _>("auto_rotate".to_string(), || AutoRotate { enabled: false });
    use_context_provider(|| auto_rotate);

    use_future(move || async move {
        loop {
            let Ok(socket) = connect_websocket().await else {
                continue;
            };
            *WEBSOCKET_CONNECTED.write() = true;
            while let Ok(message) = socket.recv().await {
                handle_websocket_message(message);
            }
            *WEBSOCKET_CONNECTED.write() = false;
        }
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS },
        main { 
            class: format!("{} min-h-screen bg-background text-foreground antialiased", theme().to_string()),
            div {
                class: "mx-auto w-full max-w-4xl px-2 py-4",
                Router::<Route> {}
            }
        }
    }
}

const WS_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const WS_RETRY_DELAY: Duration = Duration::from_secs(3);

#[cfg(feature = "web")]
async fn connect_websocket() -> Result<Websocket<(), ServerMessage, CborEncoding>> {
    match wasmtimer::tokio::timeout(WS_CONNECT_TIMEOUT, health()).await {
        Ok(Ok(())) => {}
        _ => {
            wasmtimer::tokio::sleep(WS_RETRY_DELAY).await;
            return Err(CapturedError::from_display("server unreachable"))
        }
    }
    match wasmtimer::tokio::timeout(WS_CONNECT_TIMEOUT, websocket_handler(WebSocketOptions::new())).await {
        Ok(socket) => socket,
        Err(_) => Err(CapturedError::from_display("websocket connect timed out")),
    }
}

#[cfg(not(feature = "web"))]
async fn connect_websocket() -> Result<Websocket<(), ServerMessage, CborEncoding>> {
    match tokio::time::timeout(WS_CONNECT_TIMEOUT, health()).await {
        Ok(Ok(())) => {}
        _ => {
            tokio::time::sleep(WS_RETRY_DELAY).await;
            return Err(CapturedError::from_display("server unreachable"))
        }
    }   
    match tokio::time::timeout(WS_CONNECT_TIMEOUT, websocket_handler(WebSocketOptions::new())).await {
        Ok(socket) => socket,
        Err(_) => Err(CapturedError::from_display("websocket connect timed out")),
    }
}
