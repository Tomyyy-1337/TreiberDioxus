// Lazily loads MapLibre and PMTiles on first use and renders GPS routes.

const PMTILES_FILES = ["germany.pmtiles"];

const routeMaps = new Map();
const interactive = new Set();
let librariesPromise = null;

function loadScript(src) {
    return new Promise((resolve, reject) => {
        const script = document.createElement("script");
        script.src = src;
        script.onload = resolve;
        script.onerror = () => reject(new Error(`failed to load ${src}`));
        document.head.appendChild(script);
    });
}

// Reads PMTiles byte ranges directly when offline, otherwise through the Rust bridge.
class BridgeSource {
    constructor(key) {
        this.key = key;
    }

    getKey() {
        return this.key;
    }

    async getBytes(offset, length) {
        if (window.routeMapDirect) {
            const response = await fetch(`https://maps.passberg.net/${this.key}`, {
                headers: { Range: `bytes=${offset}-${offset + length - 1}` },
            });
            if (response.status !== 206) {
                throw new Error(`range request failed: ${response.status}`);
            }
            return { data: await response.arrayBuffer() };
        }
        while (!window.routeMapBridge) {
            await new Promise((resolve) => setTimeout(resolve, 50));
        }
        const data = await window.routeMapBridge.read(offset, length);
        return { data };
    }
}

// pmtiles.js is an IIFE that exposes a global `pmtiles`; maplibre-gl.mjs is an ES module.
function loadLibraries(urls) {
    if (!librariesPromise) {
        librariesPromise = (async () => {
            const [maplibre, style] = await Promise.all([
                import(urls.maplibreUrl),
                import(urls.styleUrl),
            ]);
            maplibre.setWorkerUrl(urls.workerUrl);
            if (!window.pmtiles) {
                await loadScript(urls.pmtilesUrl);
            }
            const protocol = new window.pmtiles.Protocol();
            for (const file of PMTILES_FILES) {
                protocol.add(new window.pmtiles.PMTiles(new BridgeSource(file)));
            }
            maplibre.addProtocol("pmtiles", protocol.tile);
            return { maplibre, style };
        })();
        // Allow a retry if loading failed.
        librariesPromise.catch(() => (librariesPromise = null));
    }
    return librariesPromise;
}

function buildStyle(theme, style) {
    return {
        version: 8,
        sources: style.buildSources(PMTILES_FILES),
        layers: style.buildLayers(PMTILES_FILES, theme),
    };
}

function routeGeoJson(positions) {
    return {
        type: "Feature",
        properties: {},
        geometry: {
            type: "LineString",
            coordinates: positions.map((p) => [p.longitude, p.latitude]),
        },
    };
}

export async function showRoute(id, positions, theme, urls) {
    const { maplibre, style } = await loadLibraries(urls);
    if (!document.getElementById(id)) {
        return;
    }

    applyInteractive(id);

    let entry = routeMaps.get(id);
    if (!entry) {
        const map = new maplibre.Map({
            container: id,
            style: buildStyle(theme, style),
            center: [10.45, 51.16],
            zoom: 5,
            attributionControl: false,
        });
        map.on("error", (e) => showError(id, e.error ?? e));
        entry = {
            map,
            nav: new maplibre.NavigationControl({ showCompass: false }),
            navShown: false,
            ready: new Promise((resolve) => map.once("style.load", resolve)),
        };
        routeMaps.set(id, entry);
    }
    await entry.ready;

    const { map } = entry;
    style.applyThemeToMap(map, PMTILES_FILES, theme);
    map.getSource("waypoint-path").setData(routeGeoJson(positions));

    entry.positions = positions;
    fitRoute(map, positions);
}

function showError(id, err) {
    const container = document.getElementById(id);
    if (!container) {
        return;
    }
    console.error("maplibre error", err);
    container.style.position = "relative";
    let box = container.querySelector(".route-map-error");
    if (!box) {
        box = document.createElement("div");
        box.className = "route-map-error";
        box.style.cssText = "position:absolute;inset:0;padding:12px;overflow:auto;z-index:10;background:rgba(0,0,0,0.7);color:#ff8080;font:12px monospace;white-space:pre-wrap;";
        container.appendChild(box);
    }
    box.textContent = `Map error: ${err?.message ?? err}`;
}

function fitRoute(map, positions) {
    if (positions.length === 0) {
        return;
    }
    const lngs = positions.map((p) => p.longitude);
    const lats = positions.map((p) => p.latitude);
    map.fitBounds(
        [[Math.min(...lngs), Math.min(...lats)], [Math.max(...lngs), Math.max(...lats)]],
        { padding: 40, maxZoom: 15, duration: 0 },
    );
}

// Pointer events are off outside fullscreen so the map cannot be panned or zoomed there.
function applyInteractive(id) {
    const el = document.getElementById(id);
    if (el) {
        el.style.pointerEvents = interactive.has(id) ? "auto" : "none";
    }
}

export function setInteractive(id, enabled) {
    if (enabled) {
        interactive.add(id);
    } else {
        interactive.delete(id);
    }
    applyInteractive(id);
    const entry = routeMaps.get(id);
    if (entry && entry.navShown !== enabled) {
        entry.navShown = enabled;
        if (enabled) {
            entry.map.addControl(entry.nav, "top-right");
        } else {
            entry.map.removeControl(entry.nav);
        }
    }
    if (entry) {
        entry.map.resize();
        fitRoute(entry.map, entry.positions ?? []);
    }
}

export function disposeRoute(id) {
    const entry = routeMaps.get(id);
    if (entry) {
        entry.map.remove();
        routeMaps.delete(id);
    }
}

// Exposed for the eval scripts, which cannot resolve module URLs.
window.routeMap = { showRoute, disposeRoute, setInteractive };
