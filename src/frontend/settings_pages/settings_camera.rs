use dioxus::prelude::*;

use crate::{backend::camera_interface::{FocusMode, Metering, set_bitrate, set_focal_length, set_focus_mode, set_metering_mode, toggle_hdr}, frontend::{app::{SETTINGS_CAMERA}, frondend_data::AutoRotate, settings_pages::settings_layout::SettingsWrapper}};

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
    rsx! {
        div { class :"grid gap-3 grid-cols-2",

            FocalLength {}
            MeteringMode {}
            FocusModeSettings {}
            BitrateSettings {}
            HdrSettings {}
            AutoLevelSettings {}

            div { class:"col-span-2",
            
            }
        }
    }
}

#[component]
pub fn SelectWidget<T: 'static + Clone + PartialEq>(
    title: &'static str,
    options: &'static [(T, &'static str)],
    selected: Option<T>,
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
                option {
                    value: "",
                    class: "bg-card text-base text-foreground",
                    disabled: true,
                    hidden: true,
                    selected: selected.is_none(),
                    "-"
                }
                for (index, (value, label)) in options.iter().enumerate() {
                    option {
                        class: "bg-card text-base text-foreground",
                        value: "{index}",
                        selected: selected.as_ref() == Some(value),
                        "{label}"
                    }
                }
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
        SETTINGS_CAMERA().map(|camera| camera.focal_length)
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
        SETTINGS_CAMERA().map(|camera| camera.metering_mode)
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
        SETTINGS_CAMERA().map(|camera| camera.focus_mode)
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
        SETTINGS_CAMERA().map(|camera| camera.bitrate)
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
        SETTINGS_CAMERA().map(|camera| camera.hdr_enabled).unwrap_or(false)
    );

    rsx! {
        label { class: "flex min-h-18 cursor-pointer items-center justify-between gap-3 rounded-lg border border-border bg-card px-2.5 py-2.5 transition-colors",
            span { class: "grid min-w-0 gap-1",
                span { class: "text-sm font-medium", "HDR" }
                small { class: "text-[0.68rem] leading-none text-foreground/60",
                    if hdr_enabled() { "Aktiv" } else { "Aus" }
                }
            }
            input {
                class: "peer sr-only",
                name: "camera-hdr",
                r#type: "checkbox",
                checked: hdr_enabled(),
                onclick: move |_| {
                    spawn(async move {
                        if let Ok(new_hdr_enabled) = toggle_hdr().await {
                            if let Some(camera) = &mut *SETTINGS_CAMERA.write() {
                                camera.hdr_enabled = new_hdr_enabled;
                            }
                        }
                    });
                },
            }
            span {
                class: "relative h-5 w-9 shrink-0 rounded-full bg-border transition-colors after:absolute after:left-1 after:top-1 after:h-3 after:w-3 after:rounded-full after:bg-background after:transition-transform peer-checked:bg-primary peer-checked:after:translate-x-4 peer-focus-visible:outline peer-focus-visible:outline-primary",
                aria_hidden: "true",
            }
        }
    }
}

#[component]
pub fn AutoLevelSettings() -> Element {
    let mut auto_rotate = use_context::<Signal<AutoRotate>>();

    rsx! {
        label { class: "flex min-h-18 cursor-pointer items-center justify-between gap-3 rounded-lg border border-border bg-card px-2.5 py-2.5 transition-colors",
            span { class: "grid min-w-0 gap-1",
                span { class: "text-sm font-medium", "Auto Level" }
                small { class: "text-[0.68rem] leading-none text-foreground/60",
                    if auto_rotate().enabled { "Aktiv" } else { "Aus" }
                }
            }
            input {
                class: "peer sr-only",
                name: "camera-auto-level",
                r#type: "checkbox",
                checked: auto_rotate().enabled,
                onclick: move |_| auto_rotate.with_mut(|auto_rotate| auto_rotate.enabled = !auto_rotate.enabled),
            }
            span {
                class: "relative h-5 w-9 shrink-0 rounded-full bg-border transition-colors after:absolute after:left-1 after:top-1 after:h-3 after:w-3 after:rounded-full after:bg-background after:transition-transform peer-checked:bg-primary peer-checked:after:translate-x-4 peer-focus-visible:outline peer-focus-visible:outline-primary",
                aria_hidden: "true",
            }
        }
    }
}


