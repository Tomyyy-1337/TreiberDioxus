use dioxus::prelude::*;

use crate::frontend::frondend_data::{Fahrtenbuch, Indexed};

#[component]
pub fn FahrtenbuchPage() -> Element {
    let fahrtenbuch = use_context::<Signal<Fahrtenbuch>>();
    
    rsx! {
        div {
            "Fahrtenbuch"

            for Indexed{value, ..} in fahrtenbuch().entries {
                div { 
                    h2 {
                        "Start time {value.client_start_time}"
                    }
                }
            }
        }
    }
}