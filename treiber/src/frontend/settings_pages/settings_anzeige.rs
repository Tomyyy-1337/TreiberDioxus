use dioxus::prelude::*;

use crate::frontend::frondend_data::Theme;

use super::settings_layout::SettingsWrapper;

#[component]
pub fn SettingsAnzeige() -> Element {
    let mut theme = use_context::<Signal<Theme>>();

    rsx! {
        SettingsWrapper {
            title: "Anzeige",
            description: "Anzeige Einstellungen",
            children: rsx! {
                button {
                    class: "flex w-full items-center justify-between rounded-lg border border-border bg-card px-4 py-3 text-left",
                    onclick: move |_| {
                        let t = theme();
                        *theme.write() = match t {
                            Theme::Light => Theme::Dark,
                            Theme::Dark => Theme::Light,
                        };
                    },
                    span { class: "text-sm font-medium", "Design" }
                    span { class: "text-xs uppercase tracking-widest text-foreground/60",
                        match theme() {
                            Theme::Light => "Hell",
                            Theme::Dark => "Dunkel",
                        }
                    }
                }
            },
        }
    }
}