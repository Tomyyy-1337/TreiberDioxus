use dioxus::prelude::*;

use crate::frontend::theme::Theme;

#[component]
pub fn Camera() -> Element {
    let mut theme = use_context::<Signal<Theme>>();

    rsx! {
        p {
            class: "bg-background p-4 text-foreground", 
            "Welcome to Ruder Cam Beta v0.3!" 
        }
        p { 
            class: "bg-background p-4 text-foreground", 
        }
        button {
            onclick: move |_| { 
                let t = theme();
                *theme.write() = match t {
                    Theme::Light => Theme::Dark,
                    Theme::Dark => Theme::Light,
                };
            },
            "Toggle theme!"
        }
    }
}