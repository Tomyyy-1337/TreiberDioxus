use dioxus::prelude::*;

use crate::{backend::api::toggle_fahrt::toggle_fahrt, frontend::frondend_data::Theme};

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

        button {
            onclick: move |_| {                
                let current_time = chrono::Local::now();
                
                spawn(async move { 
                    let  _ = toggle_fahrt(current_time.into()).await;
                });
            },
            "Toggle Fahrt"
        }
    }
}