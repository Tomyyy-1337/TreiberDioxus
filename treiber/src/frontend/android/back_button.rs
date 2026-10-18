use dioxus::prelude::*;

use crate::frontend::router::Route;

/// Mirrors router navigation into WebView history so the stock Android back key pops it via `popstate`.
#[cfg(target_os = "android")]
#[component]
pub fn BackButton() -> Element {
    let nav = navigator();
    let route = use_route::<Route>();
    let mut skip_push = use_signal(|| false);
    let mut mounted = use_signal(|| false);
    let mut at_root = use_signal(|| true);

    use_effect(use_reactive!(|route| {
        let _ = &route;
        at_root.set(matches!(&route, Route::Camera {}));
        if !*mounted.peek() {
            mounted.set(true);
            document::eval("history.pushState(null, '', location.href);");
            return;
        }
        if *skip_push.peek() {
            skip_push.set(false);
            return;
        }
        document::eval("history.pushState(null, '', location.href);");
    }));

    use_future(move || async move {
        let mut popstate = document::eval("window.addEventListener('popstate', () => dioxus.send('back'));");
        while popstate.recv::<String>().await.is_ok() {
            if *crate::frontend::components::map::MAP_FULLSCREEN.peek() {
                *crate::frontend::components::map::MAP_FULLSCREEN.write() = false;
                document::eval("history.pushState(null, '', location.href);");
            } else if nav.can_go_back() {
                skip_push.set(true);
                nav.go_back();
            } else if !*at_root.peek() {
                nav.replace(Route::Camera {});
            } else {
                document::eval("history.pushState(null, '', location.href);");
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
