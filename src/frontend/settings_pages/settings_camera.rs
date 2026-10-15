use dioxus::prelude::*;

use crate::{backend::camera_interface::{FocusMode, Metering, set_bitrate, set_exposure, set_focal_length, set_focus_mode, set_metering_mode, toggle_hdr}, frontend::{app::{SETTINGS_CAMERA, WEBSOCKET_CONNECTED}, frondend_data::AutoRotate, settings_pages::settings_layout::SettingsWrapper}};

#[component]
pub fn SettingsCamera() -> Element {
    rsx! {
        SettingsWrapper {
            title: "Kamera Einstellungen",
            description: "Einstellungen für Belichtung, Fokus, Bitrate und weitere Kameraoptionen.",
            children: rsx! {
                SettingsCameraComponent {}
            },
        }
    }
}

#[component]
pub fn SettingsCameraComponent() -> Element {
    let disconnected = !WEBSOCKET_CONNECTED();

    rsx! {
        fieldset { class: "contents", disabled: disconnected,
            div {
                class: if disconnected { "opacity-50 pointer-events-none" },
                class: "grid gap-3 grid-cols-2",

                FocalLength {}
                MeteringMode {}
                FocusModeSettings {}
                BitrateSettings {}
                HdrSettings {}
                AutoLevelSettings {}
                ExposureSettings {}
            }
        }
    }
}

#[component]
pub fn SelectWidget<T: 'static + Clone + PartialEq>(
    title: &'static str,
    options: &'static [(T, &'static str)],
    selected: T,
    onchange: EventHandler<T>,
) -> Element {
    let name = title
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join("_");

    rsx! {
        label { class: "grid min-w-0 gap-1 rounded-lg border border-border bg-card p-2.5",
            span { class: "block text-xs font-bold uppercase tracking-widest text-foreground/60",
                "{title}"
            }
            select {
                class: "rounded-md border border-border bg-card px-3 py-2 text-base text-foreground outline-none focus:outline-none focus:ring-0",
                name: "{name}_select",
                onchange: move |event: Event<FormData>| {
                    if let Some((value, _)) = event
                        .value()
                        .parse::<usize>()
                        .ok()
                        .and_then(|i| options.get(i))
                    {
                        onchange.call(value.clone());
                    }
                },
                for (index, (value, label)) in options.iter().enumerate() {
                    option {
                        class: "bg-card text-base text-foreground",
                        value: "{index}",
                        selected: selected == *value,
                        "{label}"
                    }
                }
            }
        }
    }
}

#[component]
pub fn CheckboxWidget(
    title: &'static str,
    checked: bool,
    onchange: EventHandler<bool>,
) -> Element {
    let name = title
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join("_");

    let status = if checked { "Aktiv" } else { "Aus" };

    rsx! {
        label { class: "flex min-h-18 cursor-pointer items-center justify-between gap-3 rounded-lg border border-border bg-card px-2.5 py-2.5 transition-colors",
            span { class: "grid min-w-0 gap-1",
                span { class: "text-sm font-medium", "{title}" }
                small { class: "text-[0.68rem] leading-none text-foreground/60", "{status}" }
            }
            input {
                class: "peer sr-only",
                name: "{name}_checkbox",
                r#type: "checkbox",
                checked: checked,
                onchange: move |event: Event<FormData>| onchange.call(event.checked()),
            }
            span {
                class: "relative h-5 w-9 shrink-0 rounded-full bg-border transition-colors after:absolute after:left-1 after:top-1 after:h-3 after:w-3 after:rounded-full after:bg-background after:transition-transform peer-checked:bg-primary peer-checked:after:translate-x-4 peer-focus-visible:outline peer-focus-visible:outline-primary",
                aria_hidden: "true",
            }
        }
    }
}

#[component]
pub fn FocalLength() -> Element {
    static FOCAL_LENGTH_OPTIONS: [(u8, &str); 3] = [
        (28, "28 mm"),
        (35, "35 mm"),
        (42, "42 mm"),
    ];

    let selected = use_memo(move ||
        SETTINGS_CAMERA().focal_length
    );
    
    rsx! {
        SelectWidget::<u8> {
            title: "Brennweite",
            options: &FOCAL_LENGTH_OPTIONS,
            selected: selected(),
            onchange: move |focal_length: u8| {
                spawn(async move {
                    let _ = set_focal_length(focal_length).await;
                });
            },
        }
    }
}

#[component]
pub fn MeteringMode() -> Element {
    static METERING_OPTIONS: [(Metering, &str); 2] = [
        (Metering::Average, "Durchschnitt"),
        (Metering::Center, "Mittenbetont"),
    ];

    let selected = use_memo(move ||
        SETTINGS_CAMERA().metering_mode
    );

    rsx! {
        SelectWidget::<Metering> {
            title: "Messung",
            options: &METERING_OPTIONS,
            selected: selected(),
            onchange: move |metering_mode: Metering| {
                spawn(async move {
                    let _ = set_metering_mode(metering_mode).await;
                });
            },
        }
    }
}

#[component]
pub fn FocusModeSettings() -> Element {
    static FOCUS_MODE_OPTIONS: [(FocusMode, &str); 2] = [
        (FocusMode::Auto, "Auto"),
        (FocusMode::Fixed, "Manual"),
    ];

    let selected = use_memo(move ||
        SETTINGS_CAMERA().focus_mode
    );

    rsx! {
        SelectWidget::<FocusMode> {
            title: "Fokusmodus",
            options: &FOCUS_MODE_OPTIONS,
            selected: selected(),
            onchange: move |focus_mode: FocusMode| {
                spawn(async move {
                    let _ = set_focus_mode(focus_mode).await;
                });
            },
        }
    }
}

#[component]
pub fn BitrateSettings() -> Element {
    static BITRATE_OPTIONS: [(u32, &str); 5] = [
        (1_600_000, "1.6 MB/s"),
        (2_400_000, "2.4 MB/s"),
        (3_200_000, "3.2 MB/s"),
        (4_000_000, "4.0 MB/s"),
        (8_000_000, "8.0 MB/s"),
    ];

    let selected = use_memo(move ||
        SETTINGS_CAMERA().bitrate
    );

    rsx! {
        SelectWidget::<u32> {
            title: "Bitrate",
            options: &BITRATE_OPTIONS,
            selected: selected(),
            onchange: move |bitrate: u32| {
                spawn(async move {
                    let _ = set_bitrate(bitrate).await;
                });
            },
        }
    }
}

#[component]
pub fn HdrSettings() -> Element {
    let hdr_enabled = use_memo(move ||
        SETTINGS_CAMERA().hdr_enabled
    );

    rsx! {
        CheckboxWidget {
            title: "HDR",
            checked: hdr_enabled(),
            onchange: move |_| {
                spawn(async move {
                    if let Ok(new_hdr_enabled) = toggle_hdr().await {
                        SETTINGS_CAMERA.write().hdr_enabled = new_hdr_enabled;
                    }
                });
            },
        }
    }
}

#[component]
pub fn AutoLevelSettings() -> Element {
    let mut auto_rotate = use_context::<Signal<AutoRotate>>();

    rsx! {
        CheckboxWidget {
            title: "Auto Level",
            checked: auto_rotate().enabled,
            onchange: move |_| auto_rotate.with_mut(|auto_rotate| auto_rotate.enabled = !auto_rotate.enabled),
        }
    }
}

#[component]
pub fn ExposureSettings() -> Element {
    const EXPOSURE_LABELS: [&str; 13] = ["-3", "", "-2", "", "-1", "", "0", "", "+1", "", "+2", "", "+3"];

    let hdr_enabled = use_memo(move ||
        SETTINGS_CAMERA().hdr_enabled
    );
    let server_exposure = use_memo(move ||
        SETTINGS_CAMERA().exposure_compenstion
    );
    let mut exposure = use_signal(|| 0.0_f32);

    // Thumb centers travel from 0.5rem to (100% - 0.5rem), so ticks are placed on that span.
    let ticks: Vec<(String, &str)> = EXPOSURE_LABELS
        .iter()
        .enumerate()
        .map(|(index, label)| (format!("calc(0.5rem + (100% - 1rem) * {})", index as f32 / 12.0), *label))
        .collect();

    // Follow server updates; the local signal leads while the slider is dragged.
    use_effect(move || exposure.set(server_exposure()));

    rsx! {
        div { class: "col-span-2",
            if hdr_enabled() {
                p { class: "my-0 flex h-24 items-center justify-center rounded-lg border border-dashed border-border bg-card px-3 py-2 text-center text-xs text-muted-foreground",
                    "Belichtung ist bei HDR nicht verfügbar."
                }
            } else {
                section { class: "my-0 grid h-24 content-start gap-2 rounded-lg border border-border bg-card px-3 py-2",
                    div { class: "flex items-center justify-between",
                        span { class: "text-[0.66rem] font-bold uppercase tracking-widest text-muted-foreground", "Belichtung" }
                        output {
                            class: "text-sm font-semibold tabular-nums text-card-foreground",
                            "{exposure():+.1} EV"
                        }
                    }
                    div { class: "grid gap-0.5",
                        input {
                            id: "camera-exposure-compensation",
                            name: "camera-exposure-compensation",
                            class: "mx-2 h-6 w-[calc(100%-1rem)] cursor-pointer accent-primary",
                            r#type: "range",
                            min: "-3",
                            max: "3",
                            step: "0.5",
                            value: "{exposure()}",
                            oninput: move |event: Event<FormData>| {
                                if let Ok(value) = event.value().parse::<f32>() {
                                    exposure.set(value);
                                }
                            },
                            onchange: move |event: Event<FormData>| {
                                if let Ok(value) = event.value().parse::<f32>() {
                                    spawn(async move {
                                        let _ = set_exposure(value).await;
                                    });
                                }
                            },
                        }
                        div { class: "relative mx-2 h-7 text-[0.7rem] tabular-nums text-muted-foreground", aria_hidden: "true",
                            for (index, (left, label)) in ticks.iter().enumerate() {
                                if index % 2 == 0 {
                                    span { class: "absolute top-0 flex -translate-x-1/2 flex-col items-center", style: "left: {left}",
                                        i { class: "h-2.5 w-px bg-muted-foreground" }
                                        b { class: "font-normal", "{label}" }
                                    }
                                } else {
                                    span { class: "absolute top-0 flex -translate-x-1/2 justify-center", style: "left: {left}",
                                        i { class: "h-2 w-px bg-border" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


