use dioxus::prelude::*;

use crate::frontend::{frondend_data::{Fahrtenbuch, Indexed}, router::Route};

#[component]
pub fn FahrtenbuchPage() -> Element {
    let fahrtenbuch = use_context::<Signal<Fahrtenbuch>>();
    
    rsx! {
        div {
            class: "grid gap-3",
            for Indexed{index, value} in fahrtenbuch().entries.iter().rev() {
                Link {
                    to: Route::FahrtenbuchEintrag {index: *index},
                    class: "grid gap-1 rounded-2xl border border-border bg-card p-3",
                    h2 {
                        class: "text-base font-semibold",
                        "Fahrt am {value.client_start_time}"
                    }
                    div {
                        class: "flex flex-wrap gap-1",
                        span {
                            class: "inline-flex items-center rounded-full border border-border bg-background px-3 py-1 text-sm tabular-nums",
                            "{value.active_time_string()}"
                        }
                        span {
                            class: "inline-flex items-center rounded-full border border-border bg-background px-3 py-1 text-sm tabular-nums",
                            "{value.distance_traveled} km"
                        }
                    }
                }
            }
        }
    }
}