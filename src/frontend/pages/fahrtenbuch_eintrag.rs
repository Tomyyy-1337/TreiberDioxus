use dioxus::prelude::*;

use crate::{GpsPosition, frontend::{components::map::RouteMap, frondend_data::{Fahrtenbuch, Indexed}}};

#[component]
pub fn FahrtenbuchEintrag(index: usize) -> Element {
    let fahrtenbuch = use_context::<Signal<Fahrtenbuch>>();

    let eintrag = fahrtenbuch().entries.iter().find(|Indexed{index: i, ..}| *i == index).cloned();

    if let Some(Indexed { index, value }) = eintrag {
        let mut test_eintrag = value.clone();
        test_eintrag.gps_history.push(GpsPosition { latitude: 49.0, longitude: 8.0 });
        test_eintrag.gps_history.push(GpsPosition { latitude: 50.0, longitude: 9.0 });


        rsx!(
            div {
                class: "grid gap-3",
                h1 { "Fahrtenbuch Eintrag {index}" }
                RouteMap { positions: test_eintrag.gps_history.clone() }
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