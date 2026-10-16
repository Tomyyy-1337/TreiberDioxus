use dioxus::prelude::*;

#[component]
pub fn FahrtenbuchEintrag(index: usize) -> Element {
    rsx!(
        div {
            "Fahrtenbuch Eintrag {index}"
        }
    )
}