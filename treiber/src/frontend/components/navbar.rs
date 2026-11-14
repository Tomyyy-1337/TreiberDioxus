use dioxus::prelude::*;

use crate::frontend::components::title_bar::TitleBar;
use crate::frontend::pages::fahrtenbuch::reset_view;
use crate::frontend::router::Route;

#[component]
pub fn NavbarLayout() -> Element {
    rsx! {
        TitleBar { title: "Ruder Cam Beta" }
        Navbar { }
        Outlet::<Route> {}
    }
}

fn tab_class(active: bool) -> &'static str {
    if active {
        "z-10 flex h-11 items-center justify-center whitespace-nowrap rounded-xl px-4 text-center text-base font-semibold text-background transition-colors duration-300"
    } else {
        "z-10 flex h-11 items-center justify-center whitespace-nowrap rounded-xl px-4 text-center text-base font-semibold text-foreground transition-colors duration-300"
    }
}

#[component]
pub fn Navbar() -> Element {
    let route = use_route::<Route>();
    let active = match route {
        Route::Camera {} => 0,
        Route::FahrtenbuchPage {} => 1,
        Route::Einstellungen {} => 2,
        _ => usize::MAX,
    };
    let pill_left = if active == usize::MAX { 0.0 } else { active as f64 * 100.0 / 3.0 };
    let pill_opacity = if active == usize::MAX { "0" } else { "1" };

    rsx! {
        section {
            class: "my-4 rounded-2xl border border-border bg-card p-2",
            div {
                class: "relative grid grid-cols-3 gap-0",
                div {
                    class: "absolute top-0 h-11 rounded-xl bg-foreground transition-all duration-300 ease-out",
                    style: "left: {pill_left}%; width: 33.3333%; opacity: {pill_opacity};",
                }
                Link {
                    class: tab_class(active == 0),
                    to: Route::Camera {},
                    "Home"
                }
                Link {
                    class: tab_class(active == 1),
                    to: Route::FahrtenbuchPage {},
                    onclick: move |_| reset_view(),
                    "Fahrtenbuch"
                }
                Link {
                    class: tab_class(active == 2),
                    to: Route::Einstellungen {},
                    "Settings"
                }
            }
        }
    }
}