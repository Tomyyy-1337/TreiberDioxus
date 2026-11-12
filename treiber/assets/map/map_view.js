// Lazily loads MapLibre and PMTiles on first use and renders GPS routes.

const PMTILES_FILES = ["germany.pmtiles"];

const routeMaps = new Map();
const interactive = new Set();
const lastRequests = new Map();
let librariesPromise = null;
// Replaced after a failed read, so PMTiles does not keep its failed header cache.
let pmtilesProtocol = null;

function loadScript(src) {
    return new Promise((resolve, reject) => {
        const script = document.createElement("script");
        script.src = src;
        script.onload = resolve;
        script.onerror = () => reject(new Error(`failed to load ${src}`));
        document.head.appendChild(script);
    });
}

// Reads PMTiles byte ranges from /maps, or from the public host when offline.
class RangeSource {
    constructor(key) {
        this.key = key;
    }

    getKey() {
        return this.key;
    }

    async getBytes(offset, length) {
        try {
            return await this.fetchBytes(offset, length);
        } catch (error) {
            markSourceUnavailable();
            throw error;
        }
    }

    async fetchBytes(offset, length) {
        if (!window.routeMapDirect && window.routeMapRangeRequest) {
            try {
                return { data: await window.routeMapRangeRequest(this.key, offset, length) };
            } catch (error) {
                throw new Error(`native range request for ${this.key} failed: ${error.message}`);
            }
        }
        const base = window.routeMapDirect ? "https://maps.passberg.net" : `${window.routeMapServerUrl ?? ""}/maps`;
        const url = `${base}/${this.key}`;
        let response;
        try {
            response = await fetch(url, {
                headers: { Range: `bytes=${offset}-${offset + length - 1}` },
            });
        } catch (error) {
            throw new Error(`range fetch to ${url} from ${location.origin} failed: ${error.message}`);
        }
        if (response.status !== 206) {
            throw new Error(`range request failed: ${response.status}`);
        }
        return { data: await response.arrayBuffer() };
    }
}

function createPmtilesProtocol() {
    const protocol = new window.pmtiles.Protocol();
    for (const file of PMTILES_FILES) {
        protocol.add(new window.pmtiles.PMTiles(new RangeSource(file)));
    }
    return protocol;
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
            maplibre.addProtocol("pmtiles", (params, abort) => pmtilesProtocol.tile(params, abort));
            pmtilesProtocol = createPmtilesProtocol();
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
    lastRequests.set(id, { positions, theme, urls });
    const { maplibre, style } = await loadLibraries(urls);
    if (!document.getElementById(id)) {
        return;
    }

    applyInteractive(id);

    let entry = routeMaps.get(id);
    if (!entry) {
        document.getElementById(id)?.querySelector(".route-map-unavailable")?.remove();
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

// A failed range read means the PMTiles source is unreachable; every route map shows the notice.
function markSourceUnavailable() {
    pmtilesProtocol = createPmtilesProtocol();
    for (const id of [...routeMaps.keys()]) {
        showUnavailable(id);
    }
}

function showUnavailable(id) {
    const entry = routeMaps.get(id);
    if (entry) {
        entry.map.remove();
        routeMaps.delete(id);
    }
    const container = document.getElementById(id);
    if (!container || container.querySelector(".route-map-unavailable")) {
        return;
    }
    const box = document.createElement("div");
    box.className = "route-map-unavailable";
    box.style.cssText = "display:flex;align-items:center;justify-content:center;height:100%;width:100%;color:#888;font:14px sans-serif;";
    box.textContent = "Map not available";
    container.replaceChildren(box);
}

function showError(id, err) {
    const container = document.getElementById(id);
    if (!container) {
        return;
    }
    console.error("maplibre error", err);
    if (!routeMaps.has(id)) {
        return;
    }
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

// Retries a map that showed the unavailable notice; called when the user toggles fullscreen.
function retryUnavailable(id) {
    const request = lastRequests.get(id);
    const box = document.getElementById(id)?.querySelector(".route-map-unavailable");
    if (request && box) {
        showRoute(id, request.positions, request.theme, request.urls).catch((err) => console.error("route map retry failed", err));
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
    if (!entry) {
        retryUnavailable(id);
        return;
    }
    if (entry.navShown !== enabled) {
        entry.navShown = enabled;
        if (enabled) {
            entry.map.addControl(entry.nav, "top-right");
        } else {
            entry.map.removeControl(entry.nav);
        }
    }
    entry.map.resize();
    fitRoute(entry.map, entry.positions ?? []);
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
