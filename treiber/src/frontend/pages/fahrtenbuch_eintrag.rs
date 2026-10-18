use dioxus::prelude::*;

use crate::{frontend::{components::map::RouteMap, frondend_data::{Fahrtenbuch, Indexed}}};

#[component]
pub fn FahrtenbuchEintrag(index: usize) -> Element {
    let fahrtenbuch = use_context::<Signal<Fahrtenbuch>>();

    let eintrag = fahrtenbuch().entries.iter().find(|Indexed{index: i, ..}| *i == index).cloned();

    if let Some(Indexed { index, value }) = eintrag {
        rsx!(
            div {
                class: "grid gap-3",
                h1 { "Fahrtenbuch Eintrag {index}" }

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