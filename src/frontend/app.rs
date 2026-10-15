use std::time::Duration;

use dioxus::CapturedError;
use dioxus::fullstack::{CborEncoding, WebSocketOptions, Websocket, use_websocket};
use dioxus::prelude::*;
use dioxus_sdk::storage::{LocalStorage, use_storage};

use crate::backend::battery::BatteryState;
use crate::backend::camera_interface::CameraInterface;
use crate::backend::socket::{ServerMessage, health, websocket_handler};

use super::frondend_data::*;
use super::router::Route;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

pub static BATTERY_STATE: GlobalSignal<Option<BatteryState>> = GlobalSignal::new(|| None);
pub static GPS_COUNT: GlobalSignal<Option<u8>> = GlobalSignal::new(|| None);
pub static WEBSOCKET_DISCONNECTED: GlobalSignal<bool> = GlobalSignal::new(|| false);
pub static SETTINGS_CAMERA: GlobalSignal<Option<CameraInterface>> = GlobalSignal::new(|| None);

fn handle_websocket_message(message: ServerMessage) {
    match message {
        ServerMessage::BatteryUpdate(battery_message) => {
            *BATTERY_STATE.write() = Some(battery_message);
        }
        ServerMessage::GpsCountUpdate(gps_count_message) => {
            *GPS_COUNT.write() = Some(gps_count_message.count);
        }
        ServerMessage::SettingsCameraUpdate(camera_message) => {
            *SETTINGS_CAMERA.write() = Some(camera_message);
        }
    }
}

#[component]
pub fn App() -> Element {
    let theme = use_storage::<LocalStorage, _>("theme".to_string(), || Theme::Dark);
    let theme = use_context_provider(|| theme);

    let auto_rotate = use_storage::<LocalStorage, _>("auto_rotate".to_string(), || AutoRotate { enabled: false });
    let auto_rotate = use_context_provider(|| auto_rotate);

    let mut socket = use_websocket(|| connect_websocket());
    use_future(move || async move {
        loop {
            match socket.recv().await {
                Ok(message) => handle_websocket_message(message),
                Err(_) => {
                    if !socket.is_closed() {
                        continue;
                    }
                    *WEBSOCKET_DISCONNECTED.write() = true;
                    loop {
                        let result = connect_websocket().await;
                        let connected = result.is_ok();
                        socket.set(result);
                        if connected {
                            break;
                        }
                        #[cfg(feature = "web")]
                        wasmtimer::tokio::sleep(WS_RETRY_DELAY).await;
                        #[cfg(not(feature = "web"))]
                        tokio::time::sleep(WS_RETRY_DELAY).await;
                    }
                    *WEBSOCKET_DISCONNECTED.write() = false;
                }
            }
        }
    });

    use_memo(move || {
        if WEBSOCKET_DISCONNECTED() {
            *SETTINGS_CAMERA.write() = None;
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
const WS_RETRY_DELAY: Duration = Duration::from_secs(5);

#[cfg(feature = "web")]
async fn connect_websocket() -> Result<Websocket<(), ServerMessage, CborEncoding>> {
    match wasmtimer::tokio::timeout(WS_CONNECT_TIMEOUT, health()).await {
        Ok(Ok(())) => {}
        _ => return Err(CapturedError::from_display("server unreachable")),
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
        _ => return Err(CapturedError::from_display("server unreachable")),
    }
    match tokio::time::timeout(WS_CONNECT_TIMEOUT, websocket_handler(WebSocketOptions::new())).await {
        Ok(socket) => socket,
        Err(_) => Err(CapturedError::from_display("websocket connect timed out")),
    }
}
