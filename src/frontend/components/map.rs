use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use dioxus::prelude::*;

use crate::GpsPosition;
use crate::backend::api::map_range::map_range;
use crate::frontend::frondend_data::Theme;

/// Interactive map that draws the given GPS positions as a red line.
#[component]
pub fn RouteMap(positions: ReadSignal<Vec<GpsPosition>>) -> Element {
    let id = use_hook(|| format!("route-map-{}", MAP_COUNTER.fetch_add(1, Ordering::Relaxed)));

    let theme = use_context::<Signal<Theme>>();
    use_hook(|| {
        if !MAP_BRIDGE_STARTED.swap(true, Ordering::SeqCst) {
            let _ = dioxus::core::spawn_forever(run_map_bridge());
        }
    });
    let is_full = *MAP_FULLSCREEN.read();
    let interactive_id = id.clone();
    use_effect(move || {
        let on = *MAP_FULLSCREEN.read();
        document::eval(&format!(r#"window.routeMap?.setInteractive("{interactive_id}", {on});"#));
    });
    let effect_id = id.clone();
    use_effect(move || {
        let eval = document::eval(SHOW_ROUTE_JS);
        let _ = eval.send((
            effect_id.clone(),
            positions(),
            theme().to_string(),
            MAP_ASSETS.map(|a| a.to_string()),
        ));
    });

    let drop_id = id.clone();
    use_drop(move || {
        document::eval(&format!(
            r#"window.routeMap?.disposeRoute("{drop_id}");"#
        ));
    });

    let icon_path = if is_full { "M8 3v3a2 2 0 0 1-2 2H3M21 8h-3a2 2 0 0 1-2-2V3M3 16h3a2 2 0 0 1 2 2v3M16 21v-3a2 2 0 0 1 2-2h3" } else { "M8 3H5a2 2 0 0 0-2 2v3M21 8V5a2 2 0 0 0-2-2h-3M3 16v3a2 2 0 0 0 2 2h3M16 21h3a2 2 0 0 0 2-2v-3" };
    let title = if is_full { "Exit fullscreen" } else { "Fullscreen" };
    rsx! {
        document::Link { rel: "stylesheet", href: MAP_CSS }
        div {
            class: if is_full {
                "fixed inset-0 z-50 bg-background"
            } else {
                "relative h-full w-full overflow-hidden rounded-2xl border border-border"
            },
            div {
                id: "{id}",
                class: "h-full w-full",
            }
            div {
                class: "maplibregl-ctrl maplibregl-ctrl-group",
                style: "position: absolute; right: 10px; bottom: 10px; z-index: 20;",
                button {
                    title: "{title}",
                    style: "display: flex; align-items: center; justify-content: center; width: 29px; height: 29px; padding: 0; color: #333;",
                    onclick: move |_| *MAP_FULLSCREEN.write() = !is_full,
                    svg {
                        width: "18",
                        height: "18",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        path { d: "{icon_path}" }
                    }
                }
            }
        }
    }
}

static MAP_COUNTER: AtomicUsize = AtomicUsize::new(0);
static MAP_BRIDGE_STARTED: AtomicBool = AtomicBool::new(false);
/// Whether a route map is fullscreen; read by the Android back handler.
pub static MAP_FULLSCREEN: GlobalSignal<bool> = GlobalSignal::new(|| false);


/// Serves PMTiles byte-range reads from the page: each request is sent to Rust and answered in order.
const MAP_BRIDGE_JS: &str = r#"
if (!window.routeMapBridge) {
    const pending = [];
    window.routeMapBridge = {
        read(offset, length) {
            return new Promise((resolve, reject) => {
                pending.push({ resolve, reject });
                dioxus.send([offset, length]);
            });
        },
    };
    while (true) {
        const response = await dioxus.recv();
        const request = pending.shift();
        if (response.Err) request.reject(new Error(response.Err));
        else request.resolve(new Uint8Array(response.Ok).buffer);
    }
}
"#;

async fn run_map_bridge() {
    let mut eval = document::eval(MAP_BRIDGE_JS);
    while let Ok((offset, length)) = eval.recv::<(u64, u64)>().await {
        let response = map_range(offset, length).await.map_err(|err| err.to_string());
        if eval.send(response).is_err() {
            break;
        }
    }
}

/// Order matches the destructuring in `SHOW_ROUTE_JS`.
const MAP_ASSETS: [Asset; 5] = [
    asset!("/assets/map/map_view.js", AssetOptions::js()),
    asset!("/assets/map/map_style.js", AssetOptions::js()),
    asset!("/assets/map/maplibre-gl.mjs", AssetOptions::js()),
    asset!("/assets/map/pmtiles.js", AssetOptions::js()),
    asset!("/assets/map/maplibre-gl-worker.mjs", AssetOptions::js()),
];

const MAP_CSS: Asset = asset!("/assets/map/maplibre-gl.css");

const SHOW_ROUTE_JS: &str = r#"
const [id, positions, theme, assets] = await dioxus.recv();
const [viewUrl, styleUrl, maplibreUrl, pmtilesUrl, workerUrl] = assets;
try {
    window.routeMapLoading ??= new Promise((resolve, reject) => {
        const script = document.createElement("script");
        script.type = "module";
        script.src = viewUrl;
        script.onload = resolve;
        script.onerror = () => reject(new Error("could not load " + viewUrl));
        document.head.appendChild(script);
    });
    await window.routeMapLoading;
    await window.routeMap.showRoute(id, positions, theme, { styleUrl, maplibreUrl, pmtilesUrl, workerUrl });
} catch (err) {
    console.error("route map failed", err);
    const el = document.getElementById(id);
    if (el) el.textContent = `Map failed to load: ${err}`;
}
"#;