use dioxus::prelude::*;

use crate::frontend::frondend_data::{OverlayElementTypes, OverlayPosition, OverlaySettings, Theme};

use super::settings_layout::SettingsWrapper;

#[component]
pub fn SettingsAnzeige() -> Element {
    rsx! {
        SettingsWrapper {
            title: "Anzeige",
            description: "Anzeige Einstellungen",
            children: rsx! {
                ThemeToggle {}
                OverlayVisibilityToggle {}
                OverlayPositionSelect {}
                OverlayOrder {}
            },
        }
    }
}

fn element_label(element_type: OverlayElementTypes) -> &'static str {
    match element_type {
        OverlayElementTypes::CurrentSpeed => "Geschwindigkeit",
        OverlayElementTypes::CurrentBPM => "Schlagzahl",
        OverlayElementTypes::SessionDistance => "Strecke",
        OverlayElementTypes::SessionTime => "Zeit",
        OverlayElementTypes::Split500m => "500m Split",
        OverlayElementTypes::DistancePerStroke => "Distanz pro Schlag",
        OverlayElementTypes::FahrtSchlagCount => "Anzahl Schläge"
    }
}

// Row height (h-12) plus gap-2.
const ROW_STEP: f64 = 56.0;

#[derive(Clone, Copy)]
struct Drag {
    from: usize,
    start_y: f64,
    offset: f64,
}

fn target_index(drag: Drag, len: usize) -> usize {
    let steps = (drag.offset / ROW_STEP).round() as i64;
    (drag.from as i64 + steps).clamp(0, len as i64 - 1) as usize
}

// Dragged row follows the finger vertically; rows in between shift by one step.
fn row_offset(drag: Option<Drag>, index: usize, len: usize) -> f64 {
    let Some(d) = drag else { return 0.0 };
    if index == d.from {
        return d.offset;
    }
    let target = target_index(d, len);
    if index > d.from && index <= target {
        -ROW_STEP
    } else if index < d.from && index >= target {
        ROW_STEP
    } else {
        0.0
    }
}

fn row_transition(drag: Option<Drag>, index: usize) -> &'static str {
    match drag {
        Some(d) if d.from != index => "transform 150ms",
        _ => "none",
    }
}

fn row_z(drag: Option<Drag>, index: usize) -> i32 {
    match drag {
        Some(d) if d.from == index => 10,
        _ => 0,
    }
}

fn segment_class(selected: bool) -> &'static str {
    if selected {
        "flex-1 rounded-md bg-primary/20 px-3 py-2 text-sm font-medium text-foreground"
    } else {
        "flex-1 rounded-md border border-border px-3 py-2 text-sm font-medium text-foreground/60"
    }
}

fn section_class(overlay_enabled: bool) -> &'static str {
    if overlay_enabled {
        "grid gap-2"
    } else {
        "grid gap-2 opacity-40 pointer-events-none"
    }
}

#[component]
fn ThemeToggle() -> Element {
    let mut theme = use_context::<Signal<Theme>>();
    let current = theme();

    rsx! {
        div { class: "grid gap-2",
            span { class: "text-xs uppercase tracking-widest text-foreground/60", "Design" }
            div { class: "flex gap-2",
                button {
                    class: segment_class(current == Theme::Light),
                    onclick: move |_| theme.set(Theme::Light),
                    "Hell"
                }
                button {
                    class: segment_class(current == Theme::Dark),
                    onclick: move |_| theme.set(Theme::Dark),
                    "Dunkel"
                }
            }
        }
    }
}

#[component]
fn OverlayVisibilityToggle() -> Element {
    let mut overlay_settings = use_context::<Signal<OverlaySettings>>();
    let current = overlay_settings.read().get_show_overlay();

    rsx! {
        div { class: "grid gap-2",
            span { class: "text-xs uppercase tracking-widest text-foreground/60", "Overlay Sichtbarkeit" }
            div { class: "flex gap-2",
                button {
                    class: segment_class(current),
                    onclick: move |_| overlay_settings.write().set_show_overlay(true),
                    "An"
                }
                button {
                    class: segment_class(!current),
                    onclick: move |_| overlay_settings.write().set_show_overlay(false),
                    "Aus"
                }
            }
        }
    }
}

#[component]
fn OverlayPositionSelect() -> Element {
    let mut overlay_settings = use_context::<Signal<OverlaySettings>>();
    let current = overlay_settings.read().get_overlay_position();
    let overlay_enabled = overlay_settings.read().get_show_overlay();

    rsx! {
        div { class: section_class(overlay_enabled),
            span { class: "text-xs uppercase tracking-widest text-foreground/60", "Overlay Position" }
            div { class: "flex gap-2",
                button {
                    class: segment_class(current == OverlayPosition::Top),
                    onclick: move |_| overlay_settings.write().set_overlay_position(OverlayPosition::Top),
                    "Oben"
                }
                button {
                    class: segment_class(current == OverlayPosition::Bottom),
                    onclick: move |_| overlay_settings.write().set_overlay_position(OverlayPosition::Bottom),
                    "Unten"
                }
            }
        }
    }
}

#[component]
fn OverlayOrder() -> Element {
    let mut overlay_settings = use_context::<Signal<OverlaySettings>>();
    let mut drag = use_signal(|| None::<Drag>);
    let elements: Vec<_> = overlay_settings.read().get_elements().copied().collect();
    let len = elements.len();
    let overlay_enabled = overlay_settings.read().get_show_overlay();

    rsx! {
        div { class: section_class(overlay_enabled),
            span { class: "text-xs uppercase tracking-widest text-foreground/60", "Overlay Reihenfolge" }
            div { class: "grid gap-2 rounded-xl border border-border p-2",
            for (index, element) in elements.into_iter().enumerate() {
                div {
                    key: "{element_label(element.element_type)}",
                    class: "relative flex h-12 w-full items-center justify-between rounded-lg border border-border bg-card px-3 py-2",
                    style: "touch-action: none; transition: {row_transition(drag(), index)}; z-index: {row_z(drag(), index)}; transform: translateY({row_offset(drag(), index, len)}px);",
                    onpointerdown: move |evt| {
                        drag.set(Some(Drag { from: index, start_y: evt.client_coordinates().y, offset: 0.0 }));
                    },
                    onpointermove: move |evt| {
                        if let Some(mut d) = drag() {
                            let min = -(d.from as f64) * ROW_STEP;
                            let max = (len.saturating_sub(1 + d.from) as f64) * ROW_STEP;
                            d.offset = (evt.client_coordinates().y - d.start_y).clamp(min, max);
                            drag.set(Some(d));
                        }
                    },
                    onpointerup: move |_| {
                        if let Some(d) = drag() {
                            overlay_settings.write().move_element(d.from, target_index(d, len));
                        }
                        drag.set(None);
                    },
                    onpointercancel: move |_| drag.set(None),
                    span { class: "w-28 text-sm font-medium", "{element_label(element.element_type)}" }
                    button {
                        class: if element.visible { "flex h-8 w-8 items-center justify-center rounded-md bg-primary/20 text-foreground" } else { "flex h-8 w-8 items-center justify-center rounded-md border border-border text-foreground/60" },
                        title: if element.visible { "Ausblenden" } else { "Einblenden" },
                        onpointerdown: move |evt| evt.stop_propagation(),
                        onclick: move |_| overlay_settings.write().set_element_visibility(index, !element.visible),
                        svg {
                            class: "h-4 w-4",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" }
                            circle { cx: "12", cy: "12", r: "3" }
                            if !element.visible {
                                line { x1: "2", y1: "2", x2: "22", y2: "22" }
                            }
                        }
                    }
                }
            }
            }
        }
    }
}