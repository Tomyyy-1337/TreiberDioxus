use dioxus::prelude::*;

use crate::{CurrentStats, frontend::{app::{CURRENT_STATS, FAHRT, OVERLAY_SETTINGS}, frondend_data::{OverlayElementTypes, OverlayPosition}, icons::{battery_indicator::BatteryIndicator, fahrt_status_indicator::FahrtStatusIndicator, gps_indicator::GpsIndicator}}};

#[component]
pub fn FullscreenOverlay() -> Element {
    let CurrentStats{ bpm, velocity} = *CURRENT_STATS.read();

    let fahrt= *FAHRT.read();
   
    let distance_traveled = fahrt.map_or("-.-".to_string(), |f| format!("{:.1}", f.distance_traveled));
    let active_time_seconds = fahrt.map(|f| f.active_time.as_secs());
    let formated_active_time = if let Some(active_time_seconds) = active_time_seconds {
        format!("{:02}:{:02}", active_time_seconds / 60, active_time_seconds % 60)
    } else {
        "--:--".to_string()
    };

    let position_class = match OVERLAY_SETTINGS.read().get_overlay_position() {
        OverlayPosition::Top => "top-0",
        OverlayPosition::Bottom => "bottom-0",
    };

    let stats: Vec<(&str, String, &str)> = OVERLAY_SETTINGS
        .read()
        .get_visible_elements()
        .map(|element| match element {
            OverlayElementTypes::CurrentSpeed => ("Current Speed", format!("{:.1}", velocity), "km/h"),
            OverlayElementTypes::CurrentBPM => ("Current BPM", format!("{:.1}", bpm), "bpm"),
            OverlayElementTypes::SessionDistance => ("Session Distance", distance_traveled.to_owned(), "km"),
            OverlayElementTypes::SessionTime => ("Session Time", format!("{formated_active_time}"), ""),
        })
        .collect();

    rsx! {
        div { class: "pointer-events-none absolute inset-x-0 z-10 flex bg-black/50 py-2 {position_class}",
            for (label, value, unit) in stats {
                div { class: "flex flex-1 flex-col items-center",
                    span { class: "text-sm font-semibold text-white drop-shadow-lg", "{label}" }
                    span { class: "text-3xl font-bold text-white drop-shadow-lg", "{value}" }
                    if !unit.is_empty() {
                        span { class: "text-sm text-white drop-shadow-lg", "{unit}" }
                    }
                }
            }
        }
        div { class: "pointer-events-none absolute left-2 top-1/2 z-10 flex -translate-y-1/2 flex-col items-start gap-2",
            BatteryIndicator {}
            GpsIndicator {}
        }
        div { class: "pointer-events-none absolute right-2 top-1/2 z-10 -translate-y-1/2",
            FahrtStatusIndicator {}
        }
    }
}
