use dioxus::prelude::*;

use crate::frontend::router::Route;
use crate::frontend::icons::battery_indicator::BatteryIndicator;
use crate::frontend::icons::gps_indicator::GpsIndicator;

#[component]
pub fn NavbarLayout() -> Element {
    rsx! {
        section {
            class:"grid grid-cols-[1fr_auto_1fr] items-center rounded-2xl border border-border bg-card p-4",
            div { 
                class: "justify-self-start",
                BatteryIndicator {}
            }
            h1 {
                id:"main-title", 
                class:"whitespace-nowrap text-center text-xl font-semibold tracking-tight", 
                "Ruder Cam Beta"
            }
            div {
                class:"justify-self-end",
                GpsIndicator {}
            }
        }

        Navbar { }
        
        Outlet::<Route> {}
    }
}

#[component]
pub fn Navbar() -> Element {
    rsx! {
        section { 
            class: "my-4 rounded-2xl border border-border bg-card p-2",
            div {
                class: "relative grid grid-cols-3 gap-0 ",
                Link {
                    active_class: "text-purple-600",
                    class: "z-10 flex h-11 min-w-30 items-center justify-center whitespace-nowrap rounded-xl px-4 text-center text-base font-semibold text-foreground",
                    to: Route::Camera {},
                    "Home"
                }     
                Link {
                    active_class: "text-purple-600",
                    class: "z-10 flex h-11 min-w-30 items-center justify-center whitespace-nowrap rounded-xl px-4 text-center text-base font-semibold text-foreground",
                    to: Route::Fahrtenbuch {},
                    "Fahrtenbuch"
                }
                Link {
                    active_class: "text-purple-600",
                    class: "z-10 flex h-11 min-w-30 items-center justify-center whitespace-nowrap rounded-xl px-4 text-center text-base font-semibold text-foreground",
                    to: Route::Einstellungen {},
                    "Einstellungen"
                }
            }
        }
    }
}