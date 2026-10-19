use dioxus::prelude::*;

use crate::frontend::router::Route;

/// Mirrors router navigation into WebView history so the stock Android back key pops it via `popstate`.
#[cfg(target_os = "android")]
#[component]
pub fn BackButton() -> Element {
    let nav = navigator();
    let route = use_route::<Route>();
    let mut at_root = use_signal(|| true);

    use_effect(use_reactive!(|route| {
        at_root.set(matches!(&route, Route::Camera {}));
    }));

    use_future(move || async move {
        // Chromium skips entries pushed without user activation, so a buffer of entries is pushed from touches; back presses only consume it.
        let mut popstate = document::eval(
            "const DEPTH = 15;
             const fill = () => { for (let i = (history.state?.s ?? 0) + 1; i <= DEPTH; i++) history.pushState({ s: i }, '', location.href); };
             fill();
             window.addEventListener('pointerdown', fill, true);
             window.addEventListener('popstate', () => { if (history.state === null) fill(); dioxus.send('back'); });",
        );
        while popstate.recv::<String>().await.is_ok() {
            if *crate::frontend::components::map::MAP_FULLSCREEN.peek() {
                *crate::frontend::components::map::MAP_FULLSCREEN.write() = false;
            } else if nav.can_go_back() {
                nav.go_back();
            } else if !*at_root.peek() {
                nav.replace(Route::Camera {});
            }
        }
    });

    rsx! {
        Outlet::<Route> {}
    }
}

#[cfg(not(target_os = "android"))]
#[component]
pub fn BackButton() -> Element {
    rsx! {
        Outlet::<Route> {}
    }
}
