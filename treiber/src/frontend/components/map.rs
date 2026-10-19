use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;

use crate::GpsPosition;
use crate::frontend::frondend_data::Theme;

static MAP_COUNTER: AtomicUsize = AtomicUsize::new(0);
/// Whether a route map is fullscreen; read by the Android back handler.
pub static MAP_FULLSCREEN: GlobalSignal<bool> = GlobalSignal::new(|| false);

/// Order matches the destructuring in `SHOW_ROUTE_JS`.
const MAP_ASSETS: [Asset; 5] = [
    asset!("/assets/map/map_view.js", AssetOptions::js()),
    asset!("/assets/map/map_style.js", AssetOptions::js()),
    asset!("/assets/map/maplibre-gl.mjs", AssetOptions::js()),
    asset!("/assets/map/pmtiles.js", AssetOptions::js()),
    asset!("/assets/map/maplibre-gl-worker.mjs", AssetOptions::js()),
];

const MAP_CSS: Asset = asset!("/assets/map/maplibre-gl.css");

/// On Android the WebView origin is not the server, so range requests need the absolute URL.
#[cfg(target_os = "android")]
const SERVER_BASE: &str = crate::SERVER_URL;
#[cfg(not(target_os = "android"))]
const SERVER_BASE: &str = "";

/// Interactive map that draws the given GPS positions as a red line.
#[component]
pub fn RouteMap(positions: ReadSignal<Vec<GpsPosition>>) -> Element {
    let id = use_hook(|| format!("route-map-{}", MAP_COUNTER.fetch_add(1, Ordering::Relaxed)));

    let theme = use_context::<Signal<Theme>>();
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
            SERVER_BASE.to_string(),
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
                    onclick: move |_| {
                        *MAP_FULLSCREEN.write() = !is_full;
                    },
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

const SHOW_ROUTE_JS: &str = r#"
const [id, positions, theme, assets, serverUrl] = await dioxus.recv();
const [viewUrl, styleUrl, maplibreUrl, pmtilesUrl, workerUrl] = assets;
window.routeMapServerUrl = serverUrl;
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

/// Installs `window.routeMapRangeRequest` and answers requests coming from Rust.
const RANGE_BRIDGE_JS: &str = r#"
const pendingRanges = new Map();
let nextRangeId = 0;
window.routeMapRangeRequest = (key, offset, length) => new Promise((resolve, reject) => {
    const id = nextRangeId++;
    pendingRanges.set(id, { resolve, reject });
    dioxus.send([id, key, offset, length]);
});
while (true) {
    const [id, bytes] = await dioxus.recv();
    const pending = pendingRanges.get(id);
    pendingRanges.delete(id);
    if (!pending) continue;
    if (bytes === null) pending.reject(new Error("native range request failed"));
    else pending.resolve(new Uint8Array(bytes).buffer);
}
"#;

/// Serves PMTiles range reads over native HTTP, which is not subject to WebView mixed-content rules.
/// Only active on Android; runs for the lifetime of `App`.
pub async fn run_range_bridge() {
    #[cfg(target_os = "android")]
    {
        let client = reqwest::Client::new();
        let mut eval = document::eval(RANGE_BRIDGE_JS);
        while let Ok((id, key, offset, length)) = eval.recv::<(u64, String, u64, u64)>().await {
            let bytes = fetch_range(&client, &key, offset, length).await;
            let _ = eval.send((id, bytes));
        }
    }
}

/// Requests `/maps/{key}` from the server with a byte range and returns the body.
#[cfg(target_os = "android")]
async fn fetch_range(client: &reqwest::Client, key: &str, offset: u64, length: u64) -> Option<Vec<u8>> {
    if length == 0 {
        return Some(Vec::new());
    }
    let response = client
        .get(format!("{}/maps/{key}", crate::SERVER_URL))
        .header(
            reqwest::header::RANGE,
            format!("bytes={}-{}", offset, offset + length - 1),
        )
        .send()
        .await
        .ok()?;
    if response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        return None;
    }
    Some(response.bytes().await.ok()?.to_vec())
}