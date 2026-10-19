use dioxus::prelude::*;

use crate::{frontend::{components::map::RouteMap, frondend_data::{Fahrtenbuch, Indexed}, router::Route}};

#[component]
pub fn FahrtenbuchEintrag(index: usize) -> Element {
    let fahrtenbuch = use_context::<Signal<Fahrtenbuch>>();

    let eintrag = fahrtenbuch().entries.iter().find(|Indexed{index: i, ..}| *i == index).cloned();

    if let Some(Indexed { index, value }) = eintrag {
        let minutes = value.active_time.as_secs_f32() / 60.0;
        let hours = minutes / 60.0;
        let schlagzahl = if minutes > 0.0 { value.schlag_count as f32 / minutes } else { 0.0 };
        let speed = if hours > 0.0 { value.distance_traveled / hours } else { 0.0 };

        rsx!(
            div {
                class: "grid w-full gap-6 py-4",
                div {
                    class: "flex items-start justify-between gap-4",
                    header {
                        class: "grid gap-1",
                        h1 {
                            class: "text-2xl font-semibold tracking-tight text-foreground",
                            "Fahrt am {value.client_start_time.date_string()}"
                        }
                        p {
                            class: "text-sm text-muted-foreground",
                            "{value.client_start_time.hour_minute_string()} Uhr"
                        }
                    }
                    Link {
                        to: Route::FahrtenbuchPage {},
                        class: "inline-flex h-12 w-12 shrink-0 items-center justify-center rounded-full border border-border bg-card text-3xl font-medium leading-none text-muted-foreground",
                        "×"
                    }
                }
                div { class: "border-t border-border", aria_hidden: "true" }

                div {
                    class: "grid gap-4",
                    div {
                        class: "h-80 w-full",
                        if value.gps_history.len() >= 2 {
                            RouteMap { positions: value.gps_history.clone() }
                        } else {
                            div {
                                class: "flex h-full w-full items-center justify-center rounded-2xl border border-border bg-card text-sm",
                                "Kein GPS-Verlauf vorhanden"
                            }
                        }
                    }
                    div {
                        class: "flex items-baseline justify-between rounded-2xl border border-border bg-card px-4 py-3",
                        span { class: "text-xs uppercase tracking-wide text-muted-foreground", "Gesamtstrecke" }
                        span { class: "text-xl font-semibold tabular-nums", "{value.distance_traveled:.1} km" }
                    }

                    div {
                        class: "grid grid-cols-2 gap-4",
                        StatCard { label: "Zeit", value: value.active_time_string() }
                        StatCard { label: "Schläge", value: value.schlag_count.to_string() }
                        StatCard { label: "Ø Schlagzahl", value: format!("{schlagzahl:.1} /min") }
                        StatCard { label: "Ø Geschwindigkeit", value: format!("{speed:.1} km/h") }
                    }
                }
            }
        )
    } else {
        rsx!(
            div {
                class: "grid gap-3",
                h1 { "Fahrtenbuch Eintrag nicht gefunden" }
            }
        )
    }
}

#[component]
fn StatCard(label: &'static str, value: String) -> Element {
    rsx!(
        div {
            class: "grid gap-1 rounded-2xl border border-border bg-card px-4 py-3",
            span { class: "text-xs uppercase tracking-wide text-muted-foreground", "{label}" }
            span { class: "text-xl font-semibold tabular-nums", "{value}" }
        }
    )
}

