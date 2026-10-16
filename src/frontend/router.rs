use dioxus::prelude::*;

use super::android::back_button::BackButton;
use super::navbar::NavbarLayout;
use super::pages::camera::Camera;
use super::pages::fahrtenbuch::Fahrtenbuch;
use super::pages::einstellungen::Einstellungen;
use super::settings_pages::settings_camera::SettingsCamera;
use super::settings_pages::settings_anzeige::SettingsAnzeige;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {    
    #[layout(BackButton)]
        #[layout(NavbarLayout)]
            #[route("/")]
            Camera {},
            #[route("/fahrtenbuch")]
            Fahrtenbuch {},
            #[route("/einstellungen")]
            Einstellungen {},
        #[end_layout]

        #[route("/settings_camera")]
        SettingsCamera {},
        #[route("/settings_anzeige")]
        SettingsAnzeige {},
    #[end_layout]

    #[route("/:..segments", NotFound)]
    NotFound { segments: Vec<String> },
}

/// Unknown paths are replaced with the camera page.
#[component]
fn NotFound(segments: Vec<String>) -> Element {
    let nav = navigator();
    use_effect(move || {
        nav.replace(Route::Camera {});
    });
    rsx! {}
}
