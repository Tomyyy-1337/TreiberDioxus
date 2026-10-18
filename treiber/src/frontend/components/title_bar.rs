use dioxus::prelude::*;

use crate::frontend::icons::{battery_indicator::BatteryIndicator, gps_indicator::GpsIndicator};

#[component]
pub fn TitleBar(
    title: &'static str,
) -> Element {
    rsx! {
        section { class:"grid grid-cols-[1fr_auto_1fr] items-center rounded-2xl border border-border bg-card p-4",
            div { class: "justify-self-start",
                BatteryIndicator {}
            }
            h1 { class:"whitespace-nowrap text-center text-xl font-semibold tracking-tight", 
                id:"main-title", 
                "{title}"
            }
            div { class:"justify-self-end",
                GpsIndicator {}
            }
        }
    }
}