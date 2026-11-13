use dioxus::prelude::*;

use crate::{FahrtInfo, backend::api::toggle_fahrt::toggle_fahrt, frontend::app::{FAHRT, WEBSOCKET_CONNECTED}};

#[component]
pub fn FahrtStatusIndicator() -> Element {
    let fahrt = *FAHRT.read();
    let connected = WEBSOCKET_CONNECTED();

    rsx! {
        button {
            class: "pointer-events-auto flex items-center justify-center rounded-lg bg-black/50 p-2 disabled:opacity-40",
            disabled: !connected,
            onclick: move |evt| {
                evt.stop_propagation();
                let current_time = chrono::Local::now();
                spawn(async move {
                    let _ = toggle_fahrt(current_time.into()).await;
                });

                // Update the local state to reflect the new Fahrt status instantly
                if FAHRT().is_some() {
                    FAHRT().take();
                } else {
                    *FAHRT.write() = Some(FahrtInfo::default());
                }
            },
        match fahrt {
            None => rsx! {
                svg {
                    class: "h-8 w-8 shrink-0 text-red-500",
                    view_box: "0 0 24 24",
                    rect { x: "5", y: "5", width: "14", height: "14", rx: "2", fill: "currentColor" }
                }
            },
            Some(info) if info.paused => rsx! {
                svg {
                    class: "h-8 w-8 shrink-0 text-yellow-500",
                    view_box: "0 0 24 24",
                    rect { x: "6", y: "4", width: "4", height: "16", rx: "1", fill: "currentColor" }
                    rect { x: "14", y: "4", width: "4", height: "16", rx: "1", fill: "currentColor" }
                }
            },
            Some(_) => rsx! {
                svg {
                    class: "h-8 w-8 shrink-0 text-green-500",
                    view_box: "0 0 24 24",
                    path { d: "M7 4 L20 12 L7 20 Z", fill: "currentColor" }
                }
            },
        }
        }
    }
}
