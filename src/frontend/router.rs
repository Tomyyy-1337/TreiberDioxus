use dioxus::prelude::*;

use super::navbar::NavbarLayout;
use super::pages::camera::Camera;
use super::pages::fahrtenbuch::Fahrtenbuch;
use super::pages::einstellungen::Einstellungen;
use super::settings_pages::settings_camera::SettingsCamera;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {    
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
}
