use dioxus::prelude::*;

use crate::{FinishedFahrt, frontend::{frondend_data::{Fahrtenbuch, Indexed}, router::Route}};

const PAGE_SIZE: usize = 25;
const SCROLL_RESTORE_JS: &str = "window.__fahrtActive = true; if (!window.__fahrtScrollBound) { window.__fahrtScrollBound = true; window.addEventListener('scroll', () => { if (window.__fahrtActive) window.__fahrtScrollY = window.scrollY; }, { passive: true }); } requestAnimationFrame(() => window.scrollTo(0, window.__fahrtScrollY ?? 0));";

pub static FAHRTENBUCH_SHOWN: GlobalSignal<usize> = GlobalSignal::new(|| PAGE_SIZE);

/// Resets paging and scroll position, used when the page is opened from the navbar.
pub fn reset_view() {
    *FAHRTENBUCH_SHOWN.write() = PAGE_SIZE;
    document::eval("window.__fahrtScrollY = 0;");
}

#[component]
pub fn FahrtenbuchPage() -> Element {
    let fahrtenbuch = use_context::<Signal<Fahrtenbuch>>();
    let entries = fahrtenbuch.read();
    let shown_count = *FAHRTENBUCH_SHOWN.read();

    use_effect(|| {
        document::eval(SCROLL_RESTORE_JS);
    });

    use_drop(|| {
        document::eval("window.__fahrtActive = false;");
    });

    // Newest first, grouped by consecutive entries sharing a year and month.
    let mut months: Vec<((u16, u8), String, usize, Vec<&Indexed<FinishedFahrt>>)> = Vec::new();
    for (position, entry) in entries.entries.iter().rev().enumerate() {
        let key = entry.value.client_start_time.year_month();
        let visible = position < shown_count;
        match months.last_mut() {
            Some((last_key, _, count, list)) if *last_key == key => {
                *count += 1;
                if visible {
                    list.push(entry);
                }
            }
            _ => months.push((
                key,
                entry.value.client_start_time.month_label(),
                1,
                if visible { vec![entry] } else { Vec::new() },
            )),
        }
    }
    months.retain(|(_, _, _, list)| !list.is_empty());

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
            for (_, label, count, list) in months.iter() {
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
                            "{count} Fahrten"
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
            if shown_count < entries.entries.len() {
                button {
                    class: "mx-auto rounded-xl border border-border bg-card px-4 py-2 text-sm font-medium",
                    onclick: move |_| *FAHRTENBUCH_SHOWN.write() += PAGE_SIZE,
                    "Mehr anzeigen"
                }
            }
        }
    }
}