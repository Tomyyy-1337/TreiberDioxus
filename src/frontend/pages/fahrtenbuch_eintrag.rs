use dioxus::prelude::*;

use crate::frontend::{frondend_data::{Fahrtenbuch, Indexed}, router::Route};

#[component]
pub fn FahrtenbuchEintrag(index: usize) -> Element {
    let fahrtenbuch = use_context::<Signal<Fahrtenbuch>>();

    let eintrag = fahrtenbuch().entries.iter().find(|Indexed{index: i, ..}| *i == index).cloned();

    if let Some(eintrag) = eintrag {
        rsx!(
            div {
                h1 { "Fahrtenbuch Eintrag {index}" }

            }
        )
    } else {
        let nav = navigator();
        nav.replace(Route::FahrtenbuchPage {});
        rsx!()
    }
}