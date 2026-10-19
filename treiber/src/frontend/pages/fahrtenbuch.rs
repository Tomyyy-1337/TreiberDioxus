use dioxus::prelude::*;

use crate::{FinishedFahrt, frontend::{frondend_data::{Fahrtenbuch, Indexed}, router::Route}};

#[component]
pub fn FahrtenbuchPage() -> Element {
    let fahrtenbuch = use_context::<Signal<Fahrtenbuch>>();
    let entries = fahrtenbuch();

    // Newest first, grouped by consecutive entries sharing a year and month.
    let mut months: Vec<((u16, u8), String, Vec<&Indexed<FinishedFahrt>>)> = Vec::new();
    for entry in entries.entries.iter().rev() {
        let key = entry.value.client_start_time.year_month();
        match months.last_mut() {
            Some((last_key, _, list)) if *last_key == key => list.push(entry),
            _ => months.push((key, entry.value.client_start_time.month_label(), vec![entry])),
        }
    }

    let trip_count = entries.entries.len();
    let total_km = if trip_count == 0 {
        0.0
    } else {
        entries.entries.iter().map(|e| e.value.distance_traveled).sum()
    };

    rsx! {
        div {
            class: "grid gap-6",
            section {
                class: "grid grid-cols-2 gap-3 rounded-2xl border border-border bg-card p-4",
                h1 {
                    class: "col-span-2 text-center text-xl font-semibold tracking-tight",
                    "Fahrtenbuch"
                }
                div {
                    class: "grid gap-1 rounded-xl bg-background p-3 text-center",
                    span { class: "text-xs uppercase tracking-wide text-muted-foreground", "Fahrten" }
                    span { class: "text-xl font-semibold tabular-nums", "{trip_count}" }
                }
                div {
                    class: "grid gap-1 rounded-xl bg-background p-3 text-center",
                    span { class: "text-xs uppercase tracking-wide text-muted-foreground", "Strecke" }
                    span { class: "text-xl font-semibold tabular-nums", "{total_km:.1} km" }
                }
            }
            if months.is_empty() {
                p {
                    class: "text-center text-muted-foreground",
                    "Noch keine Fahrten"
                }
            }
            for (_, label, list) in months.iter() {
                section {
                    class: "grid gap-3",
                    div {
                        class: "flex items-baseline justify-between border-b border-border pb-1",
                        h2 {
                            class: "text-sm font-semibold uppercase tracking-wide text-muted-foreground",
                            "{label}"
                        }
                        span {
                            class: "text-xs tabular-nums text-muted-foreground",
                            "{list.len()} Fahrten"
                        }
                    }
                    for Indexed{index, value} in list.iter() {
                        Link {
                            to: Route::FahrtenbuchEintrag {index: *index},
                            class: "grid gap-3 rounded-2xl border border-border bg-card p-4 transition-opacity hover:opacity-80",
                            div {
                                class: "flex items-baseline justify-between gap-2",
                                h3 {
                                    class: "text-base font-semibold",
                                    "Fahrt am {value.client_start_time.date_string()}"
                                }
                                span {
                                    class: "text-sm tabular-nums text-muted-foreground",
                                    "{value.client_start_time.hour_minute_string()} Uhr"
                                }
                            }
                            div {
                                class: "flex flex-wrap gap-2",
                                span {
                                    class: "inline-flex items-center rounded-full bg-background px-3 py-1 text-sm font-medium tabular-nums",
                                    "{value.active_time_string()}"
                                }
                                span {
                                    class: "inline-flex items-center rounded-full bg-background px-3 py-1 text-sm font-medium tabular-nums",
                                    "{value.distance_traveled:.1} km"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}