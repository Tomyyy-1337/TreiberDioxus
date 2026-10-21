use dioxus::prelude::*;

use crate::{
    FahrtInfo, backend::api::toggle_fahrt::{Fahrt, toggle_fahrt}, frontend::{
        app::{CURRENT_STATS, FAHRT, WEBSOCKET_CONNECTED}, frondend_data::Theme, settings_pages::settings_camera::SettingsCameraComponent,
    },
};

#[component]
pub fn Camera() -> Element {
    let mut theme = use_context::<Signal<Theme>>();

    rsx! {
        div { 
            class: "grid gap-3",
            
            CameraSettingsSection {}   
            FahrtSection {}
        }
    }
}

#[component]
fn FahrtSection() -> Element {
    let connected = WEBSOCKET_CONNECTED();
    let running = FAHRT().is_some();
    let distance = FAHRT().map(|fahrt| fahrt.distance_traveled).unwrap_or(0.0);
    let speed = CURRENT_STATS().velocity;

    let (label, color) = if !connected {
        ("Nicht verfügbar", "bg-border text-foreground/60")
    } else if running {
        ("Fahrt beenden", "border-red-500 bg-red-500/10 text-foreground")
    } else {
        ("Fahrt starten", "border-green-500 bg-green-500/10 text-foreground")
    };

    rsx! {
        div { class: "grid gap-3 rounded-2xl border border-border bg-card p-4",
            button {
                class: format!("rounded-lg border-2 px-4 py-3 font-bold disabled:cursor-not-allowed {color}"),
                disabled: !connected,
                onclick: move |_| {
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
                "{label}"
            }
            div { class: "grid grid-cols-2 gap-3",
                StatBox { title: "Distanz", value: format!("{distance:.2} km") }
                StatBox { title: "Geschwindigkeit", value: format!("{speed:.1} km/h") }
            }
        }
    }
}

#[component]
fn CameraSettingsSection() -> Element {
    let mut open = use_signal(|| false);
    let grid_rows = if open() { "grid-rows-[1fr]" } else { "grid-rows-[0fr]" };
    let chevron = if open() { "rotate-180" } else { "" };

    rsx! {
        div { class: "rounded-2xl border border-border bg-card p-4",
            button {
                class: "flex w-full items-center justify-between font-bold",
                onclick: move |_| *open.write() = !open(),
                span { "Camera Settings" }
                span { class: "inline-flex h-4 w-4 items-center justify-center text-muted-foreground transition-transform duration-300 {chevron}",
                    svg {
                        class: "h-4 w-4",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path { d: "m6 9 6 6 6-6" }
                    }
                }
            }
            div { class: "grid transition-[grid-template-rows] duration-300 ease-in-out {grid_rows}",
                div { class: "min-h-0 overflow-hidden",
                    div { class: "pt-3",
                        SettingsCameraComponent {}
                    }
                }
            }
        }
    }
}

#[component]
fn StatBox(title: &'static str, value: String) -> Element {
    rsx! {
        div { class: "grid gap-1 rounded-lg border border-border bg-background p-2.5",
            span { class: "text-xs font-bold uppercase tracking-widest text-foreground/60", "{title}" }
            span { class: "text-xl font-semibold", "{value}" }
        }
    }
}