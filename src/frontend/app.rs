use dioxus::fullstack::{WebSocketOptions, use_websocket};
use dioxus::prelude::*;
use dioxus_sdk::storage::{SessionStorage, use_storage};

use crate::backend::battery::BatteryState;
use crate::backend::socket::{ServerMessage, websocket_handler};

use super::theme::*;
use super::router::Route;
use super::battery_indicator::BatteryIndicator;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

pub static BATTERY_STATE: GlobalSignal<Option<BatteryState>> = GlobalSignal::new(|| None);
pub static GPS_COUNT: GlobalSignal<Option<u8>> = GlobalSignal::new(|| None);

#[component]
pub fn App() -> Element {
    let theme = use_storage::<SessionStorage, _>(String::from("theme"), || Theme::Dark);
    let theme = use_context_provider(|| theme);

    let mut socket = use_websocket(|| websocket_handler(WebSocketOptions::new()));
    use_future(move || async move {
        while let Ok(msg) = socket.recv().await {
            match msg {
                ServerMessage::BatteryUpdate(batery) => {
                    *BATTERY_STATE.write() = Some(batery);
                }
                ServerMessage::GpsCountUpdate(gps_count) => {
                    *GPS_COUNT.write() = Some(gps_count.count);
                }
            }
        }
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS },
        main { 
            class: format!("{} min-h-screen bg-background text-foreground antialiased", theme().to_string()),
            div {
                class: "mx-auto w-full max-w-4xl px-2 py-4",
                BatteryIndicator {}
                Router::<Route> {}
            }
        }
    }
}
