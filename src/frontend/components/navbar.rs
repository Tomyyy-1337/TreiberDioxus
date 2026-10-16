use dioxus::prelude::*;

use crate::frontend::components::title_bar::TitleBar;
use crate::frontend::router::Route;

#[component]
pub fn NavbarLayout() -> Element {
    rsx! {
        TitleBar { title: "Ruder Cam Beta" }
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