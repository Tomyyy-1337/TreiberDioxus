use dioxus::prelude::*;

use crate::frontend::router::Route;

#[component]
pub fn Einstellungen() -> Element {
    rsx! {
        div {
            p { 
                "Kamera Einstellungen"
            }

            Link {
                to: Route::SettingsCamera {},
                "Einstellungen"
            }
        }
    }
}