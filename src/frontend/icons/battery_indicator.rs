use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::backend::tasks::battery::current_battery;
use crate::{frontend::app::{BATTERY_STATE, WEBSOCKET_CONNECTED}};

#[component]
pub fn BatteryIndicator() -> Element {
    #[cfg(feature = "server")]
    let initial_value = use_server_future(current_battery)?.unwrap()?;
    #[cfg(not(feature = "server"))]
    let initial_value = 0;

    let display_value = use_memo(move || 
        match BATTERY_STATE() {
            Some(value) => value.percentage,
            None => initial_value,
        }
    );

    let color = use_memo(move || match display_value() {
        0..=14 => "text-red-500",
        15..=30 => "text-orange-500",
        31..=50 => "text-yellow-500",
        _ => "text-green-500",
    });
    let width = use_memo(move || u8::min(12, display_value() / 8));

    rsx! {
        if WEBSOCKET_CONNECTED() {
            div {
                class: format!("flex items-center gap-1 font-bold leading-none {}", color()),
                svg {
                    class: "block h-[1.1rem] w-[1.1rem] shrink-0 fill-none stroke-current",
                    stroke_width: "1.8",
                    view_box: "0 0 24 24",
                    rect { x: "3", y: "7", width: "16", height: "10", rx: "2" }
                    rect { x: "19", y: "10", width: "2", height: "4", rx: "1" }
                    rect {
                        x: "5",
                        y: "9",
                        width: width(),
                        height: "6",
                        rx: "1",
                        fill: "currentColor",
                    }
                }
                span {
                    class: "inline-flex min-w-[2ch] items-center justify-center text-center text-[0.85rem]",
                    "{display_value()}%"
                }
            }
        }
    }
}
