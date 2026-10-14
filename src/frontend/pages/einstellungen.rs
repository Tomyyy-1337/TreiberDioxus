use dioxus::prelude::*;

use crate::frontend::router::Route;

#[component]
pub fn Einstellungen() -> Element {
    rsx! {
        div {
            class:"grid gap-3",
            SettingsLink {
                title: "Kamera",
                description: "Kamera Einstellungen",
                to: Route::SettingsCamera {},
            }
            SettingsLink {
                title: "Anzeige",
                description: "Anzeige Einstellungen",
                to: Route::SettingsAnzeige {},
            }
        }
    }
}

#[component]
pub fn SettingsLink(title: &'static str, description: &'static str, to: Route) -> Element {
    rsx! {
        Link {
            class: "grid gap-1 rounded-2xl border border-border bg-card px-4 py-3 text-base text-foreground no-underline",
            to: to,
            
            h2 { 
                class:"text-base font-semibold leading-6 text-foreground",
                "{title}" 
            }
            p { 
                class: "text-sm leading-5 text-muted-foreground",
                "{description}" 
            }
            
        }
    }
}