use dioxus::prelude::*;

use crate::frontend::{settings_pages::settings_layout::SettingsWrapper};

#[component]
pub fn SettingsCamera() -> Element {
    rsx! {
        SettingsWrapper {
            title: "Settings Camera",
            description: "Configure your camera settings here.",
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