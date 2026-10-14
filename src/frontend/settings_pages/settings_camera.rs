use dioxus::prelude::*;

use crate::frontend::{router::Route, settings_pages::settings_layout::SettingsWrapper};

#[component]
pub fn SettingsCamera() -> Element {
    rsx! {
        SettingsWrapper {
            title: "Settings Camera",
            description: "Configure your camera settings here.",
            settings_page: Route::Einstellungen {}, 
            children: rsx! {
                SettingsCameraComponent {}
            },
        }
    }
}

#[component]
pub fn SettingsCameraComponent() -> Element {
    rsx! {
        div {
            "Camera Settings"
        }
    }
}