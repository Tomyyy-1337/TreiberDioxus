use dioxus::prelude::*;

use crate::{
    FocusMode, backend::api::livestream::whep_offer, frontend::{app::{SETTINGS_CAMERA, WEBSOCKET_CONNECTED}, components::{double_tap::register_tap, fullscreen_overlay::FullscreenOverlay}, frondend_data::{OverlayPosition, OverlaySettings}},
};

const VIDEO_ID: &str = "livestream-video";
const CONTAINER_ID: &str = "livestream-container";

/// Negotiates WHEP: the browser creates the offer, Rust forwards it, and the answer is applied here.
/// Runs one WHEP session until it ends and reports (reason, uptime in ms).
const WHEP_JS: &str = r#"
const [id, delay] = await dioxus.recv();
await new Promise((resolve) => setTimeout(resolve, delay));
const video = document.getElementById(id);
// Black poster replaces the default placeholder shown until the first frame.
const c = document.createElement("canvas");
c.width = c.height = 2;
const ctx = c.getContext("2d");
ctx.fillStyle = 'black';
ctx.fillRect(0, 0, 2, 2);
video.poster = c.toDataURL();
video.style.visibility = "hidden";
video.srcObject = null;
window.livestreamSeq = (window.livestreamSeq ?? 0) + 1;
const session = window.livestreamSeq;
window.livestreamSession = session;
window.livestreamPc?.close();
const pc = new RTCPeerConnection();
window.livestreamPc = pc;
pc.addTransceiver("video", { direction: "recvonly" });
pc.ontrack = (e) => {
    // Keep the jitter buffer minimal so latency does not build up on a weak link.
    e.receiver.playoutDelayHint = 0;
    if ("jitterBufferTarget" in e.receiver) e.receiver.jitterBufferTarget = 0;
    video.muted = true;
    video.playsInline = true;
    video.controls = false;
    video.removeAttribute("controls");
    video.autoplay = true;
    video.style.background = 'black';
    video.addEventListener("playing", () => { video.style.visibility = "visible"; }, { once: true });
    video.srcObject = e.streams[0];
    video.play().catch(() => {});
};
await pc.setLocalDescription(await pc.createOffer());
await new Promise((resolve) => {
    if (pc.iceGatheringState === "complete") return resolve();
    pc.addEventListener("icegatheringstatechange", () => {
        if (pc.iceGatheringState === "complete") resolve();
    });
});
dioxus.send(pc.localDescription.sdp);
const answer = await dioxus.recv();
await pc.setRemoteDescription({ type: "answer", sdp: answer });

// The publisher can restart after a settings change and leave the peer silently dead.
let connectedAt = null;
let disconnectTimer = null;
let watchdog = null;
const reason = await new Promise((resolve) => {
    pc.onconnectionstatechange = () => {
        clearTimeout(disconnectTimer);
        if (pc.connectionState === "connected") connectedAt ??= Date.now();
        if (pc.connectionState === "failed" || pc.connectionState === "closed") resolve("failed");
        if (pc.connectionState === "disconnected") disconnectTimer = setTimeout(() => resolve("disconnected"), 1000);
    };
    const started = Date.now();
    let lastTime = -1;
    let lastProgress = Date.now();
    watchdog = setInterval(() => {
        if (window.livestreamSession !== session) return resolve("stopped");
        if (pc.connectionState !== "connected") {
            if (Date.now() - started > 5000) resolve("timeout");
            return;
        }
        if (video.currentTime !== lastTime) {
            lastTime = video.currentTime;
            lastProgress = Date.now();
        } else if (Date.now() - lastProgress > 2000) {
            resolve("stalled");
        }
    }, 250);
});
clearInterval(watchdog);
clearTimeout(disconnectTimer);
pc.close();
if (window.livestreamPc === pc) window.livestreamPc = null;
dioxus.send([reason, connectedAt ? Date.now() - connectedAt : 0]);
"#;

const WHEP_STOP_JS: &str = "window.livestreamSession = null; window.livestreamPc?.close(); window.livestreamPc = null; const v = document.getElementById('livestream-video'); if (v) { v.srcObject = null; v.style.visibility = 'hidden'; }";

/// Sessions that stayed up this long reset the reconnect backoff.
const HEALTHY_UPTIME_MS: u64 = 10_000;

/// Whether the livestream container is in fullscreen. Read by the Android back handler.
pub static LIVESTREAM_FULLSCREEN: GlobalSignal<bool> = GlobalSignal::new(|| false);

const ENTER_FULLSCREEN_JS: &str = "document.getElementById('livestream-container')?.requestFullscreen?.().catch(() => {});";
const EXIT_FULLSCREEN_JS: &str = "if (document.fullscreenElement) document.exitFullscreen?.().catch(() => {});";
/// Reports the real fullscreen state back to Rust, so exits by the system are reflected too.
const FULLSCREEN_SYNC_JS: &str = "const sync = () => dioxus.send(!!document.fullscreenElement); if (window.livestreamFsSync) document.removeEventListener('fullscreenchange', window.livestreamFsSync); window.livestreamFsSync = sync; document.addEventListener('fullscreenchange', sync); sync();";

#[cfg(target_os = "android")]
const SCREEN_ORIENTATION_SENSOR_LANDSCAPE: i32 = 6;

#[cfg(target_os = "android")]
fn set_native_fullscreen(on: bool) {
    let orientation = if on {
        SCREEN_ORIENTATION_SENSOR_LANDSCAPE
    } else {
        crate::frontend::android::orientation::SCREEN_ORIENTATION_PORTRAIT
    };
    if let Err(err) = crate::frontend::android::orientation::set_requested_orientation(orientation) {
        eprintln!("failed to set orientation: {err}");
    }
    if let Err(err) = crate::frontend::android::orientation::set_system_bars_hidden(on) {
        eprintln!("failed to set system bars: {err}");
    }
}

#[cfg(not(target_os = "android"))]
fn set_native_fullscreen(_on: bool) {}

async fn run_whep_session(mut status: Signal<&'static str>, delay_ms: u64) -> Option<(String, u64)> {
    status.set("Verbindung...");
    let mut eval = document::eval(WHEP_JS);
    eval.send((VIDEO_ID, delay_ms)).ok()?;
    let offer = eval.recv::<String>().await.ok()?;
    let answer = whep_offer(offer).await.ok()?;
    eval.send(answer).ok()?;
    status.set("");
    eval.recv::<(String, u64)>().await.ok()
}

fn start_livestream(mut status: Signal<&'static str>, generation: Signal<u32>, my_generation: u32) {
    spawn(async move {
        let mut failures: u32 = 0;
        while generation() == my_generation {
            let delay_ms = if failures == 0 { 0 } else { (100u64 << (failures - 1).min(4)).min(1_000) };
            let ended = run_whep_session(status, delay_ms).await;
            if generation() != my_generation {
                break;
            }
            let healthy = matches!(ended, Some((_, uptime)) if uptime >= HEALTHY_UPTIME_MS);
            failures = if healthy { 0 } else { failures + 1 };
            status.set("Verbindung wird erneut aufgebaut...");
        }
    });
}

#[component]
pub fn LivestreamSection() -> Element {
    let overlay_settings = use_context::<Signal<OverlaySettings>>();

    let connected = WEBSOCKET_CONNECTED();
    let mut status = use_signal(|| "");
    // Each start or stop bumps this so older reconnect loops exit.
    let mut generation = use_signal(|| 0u32);
    // Last (connected, focal_length, hdr_enabled, focus_mode) the stream was started with.
    let mut applied = use_signal(|| None::<(bool, u8, bool, FocusMode)>);
    let last_tap = use_signal(|| None::<i64>);
    let tap_hint = use_signal(|| false);
    let fullscreen_button_class = {
        let overlay_at_bottom = overlay_settings.read().get_show_overlay()
            && overlay_settings.read().get_overlay_position() == OverlayPosition::Bottom;
        if overlay_at_bottom { "absolute top-2 right-2 z-10 grid h-9 w-9 place-items-center rounded-lg bg-black/60 text-white" } else { "absolute bottom-2 right-2 z-10 grid h-9 w-9 place-items-center rounded-lg bg-black/60 text-white" }
    };

    use_effect(move || {
        let settings = SETTINGS_CAMERA();
        let connected = WEBSOCKET_CONNECTED();
        let key = (connected, settings.focal_length, settings.hdr_enabled, settings.focus_mode);
        // Only focal length, HDR and focus mode require the MediaMTX path to restart.
        if applied.peek().as_ref() == Some(&key) {
            return;
        }
        applied.set(Some(key));
        *generation.write() += 1;
        let my_generation = *generation.peek();
        if connected {
            start_livestream(status, generation, my_generation);
        } else {
            document::eval(WHEP_STOP_JS);
            status.set("");
        }
    });

    use_effect(move || {
        let fullscreen = LIVESTREAM_FULLSCREEN();
        set_native_fullscreen(fullscreen);
        document::eval(if fullscreen { ENTER_FULLSCREEN_JS } else { EXIT_FULLSCREEN_JS });
    });

    use_future(move || async move {
        let mut changes = document::eval(FULLSCREEN_SYNC_JS);
        while let Ok(on) = changes.recv::<bool>().await {
            *LIVESTREAM_FULLSCREEN.write() = on;
        }
    });

    use_drop(|| {
        *LIVESTREAM_FULLSCREEN.write() = false;
        document::eval(EXIT_FULLSCREEN_JS);
        set_native_fullscreen(false);
        document::eval(WHEP_STOP_JS);
    });

    rsx! {
        div { class: "grid gap-3 rounded-2xl border border-border bg-card p-4",
            div {
                id: CONTAINER_ID,
                class: if LIVESTREAM_FULLSCREEN() {
                    "relative h-full w-full overflow-hidden bg-black"
                } else {
                    "relative aspect-video w-full overflow-hidden rounded-lg border border-border bg-black"
                },
                onclick: move |_| {
                    if !*LIVESTREAM_FULLSCREEN.peek() && register_tap(last_tap, tap_hint) {
                        *LIVESTREAM_FULLSCREEN.write() = true;
                    }
                },
                video {
                    id: VIDEO_ID,
                    class: "h-full w-full object-contain bg-black",
                    controls: false,
                }
                if LIVESTREAM_FULLSCREEN() && overlay_settings.read().get_show_overlay() {
                    FullscreenOverlay {}
                }
                if !connected || !status().is_empty() {
                    div { class: "absolute inset-0 grid place-items-center bg-black text-sm font-bold text-foreground/60",
                        if connected {
                            "{status}"
                        } else {
                            "Stream nicht verfügbar"
                        }
                    }
                }
                if tap_hint() {
                    div { class: "pointer-events-none absolute bottom-12 left-1/2 z-20 -translate-x-1/2 rounded-lg bg-black/70 px-3 py-1 text-sm text-white",
                        "Tap again for fullscreen"
                    }
                }
                // Rendered last so it stays above the status overlay.
                button {
                    class: fullscreen_button_class,
                    aria_label: "Vollbild",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        let fullscreen = *LIVESTREAM_FULLSCREEN.peek();
                        *LIVESTREAM_FULLSCREEN.write() = !fullscreen;
                    },
                    svg {
                        class: "h-5 w-5",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path { d: "M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3" }
                    }
                }
            }
        }
    }
}
