use dioxus::prelude::*;

use crate::frontend::router::Route;

#[component]
pub fn SettingsWrapper(
    title: String,
    description: String,
    settings_page: Route,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "grid w-full gap-6 py-4",
            div {
                class: "flex items-start justify-between gap-4",
                header {
                    class: "grid gap-1",
                    h1 {
                        class: "text-2xl font-semibold tracking-tight text-foreground",
                        "{title}"
                    }
                    if !description.is_empty() {
                        p {
                            class: "text-sm text-muted-foreground",
                            "{description}"
                        }
                    }
                }
                Link {
                    to: settings_page,
                    class: "inline-flex h-12 w-12 shrink-0 items-center justify-center rounded-full border border-border bg-card text-3xl font-medium leading-none text-muted-foreground",
                    "×"
                }
            }
            div { class: "border-t border-border", aria_hidden: "true" }
            div {
                class: "grid gap-4",
                {children}
            }
        }
    }
}