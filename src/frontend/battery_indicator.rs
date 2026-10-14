use dioxus::prelude::*;

use crate::{backend::battery::{current_battery}, frontend::app::BATTERY_STATE, frontend::app::GPS_COUNT};

#[component]
pub fn BatteryIndicator() -> Element {
    let initial_value = use_server_future(current_battery)?.unwrap()?;

    let display_value = use_memo(move || 
        match BATTERY_STATE() {
            Some(value) => value.percentage,
            None => initial_value,
        }
    );

    
    rsx! {
        p { "{display_value()}%" }

        p { "GPS Count: {GPS_COUNT().unwrap_or(0)}" }
    }
}

