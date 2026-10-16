use dioxus::prelude::*;

use super::android::back_button::BackButton;
use super::components::navbar::NavbarLayout;
use super::pages::camera::Camera;
use super::pages::fahrtenbuch::FahrtenbuchPage;
use super::pages::einstellungen::Einstellungen;
use super::settings_pages::settings_camera::SettingsCamera;
use super::settings_pages::settings_anzeige::SettingsAnzeige;
use super::pages::fahrtenbuch_eintrag::FahrtenbuchEintrag;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {    
    #[layout(BackButton)]
        #[layout(NavbarLayout)]
            #[route("/")]
            Camera {},
            #[route("/fahrtenbuch")]
            FahrtenbuchPage {},
            #[route("/einstellungen")]
            Einstellungen {},
        #[end_layout]

        #[route("/settings_camera")]
        SettingsCamera {},
        #[route("/settings_anzeige")]
        SettingsAnzeige {},
        #[route("/fahrtenbuch_eintrag/:index")]
        FahrtenbuchEintrag { index: usize },

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
