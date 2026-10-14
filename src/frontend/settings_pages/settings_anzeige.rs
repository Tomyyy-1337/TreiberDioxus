use dioxus::prelude::*;

use crate::frontend::{router::Route, settings_pages::settings_layout::SettingsWrapper};

#[component]
pub fn SettingsAnzeige() -> Element {
    rsx! {
        SettingsWrapper {
            title: "Anzeige",
            description: "Anzeige Einstellungen",
            children: rsx! {
                div {
                    "Anzeige Einstellungen"
                }
            },
        }
    }
}