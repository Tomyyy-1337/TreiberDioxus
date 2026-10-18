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

    rsx! {
        document::Link { rel: "stylesheet", href: MAP_CSS }
        div {
            id: "{id}",
            class: "h-80 w-full overflow-hidden rounded-2xl border border-border",
        }
    }
}

static MAP_COUNTER: AtomicUsize = AtomicUsize::new(0);
static MAP_BRIDGE_STARTED: AtomicBool = AtomicBool::new(false);


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