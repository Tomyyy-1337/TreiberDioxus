mod backend;

mod frontend;

/// `10.0.2.2` is the emulator's alias for the host. Override with `ANDROID_SERVER_URL` for a physical device.
#[cfg(target_os = "android")]
const SERVER_URL: &str = match option_env!("ANDROID_SERVER_URL") {
    Some(url) => url,
    None => "http://10.0.2.2:8080",
};

fn main() {
    // #[cfg(all(not(feature = "server"), not(target_family = "wasm")))]
    // dioxus_sdk::storage::set_dir!("data");

    #[cfg(all(not(feature = "server"), target_os = "android"))]
    dioxus::fullstack::set_server_url(SERVER_URL);

    #[cfg(not(feature = "server"))] 
    dioxus::launch(crate::frontend::app::App);
    
    #[cfg(feature = "server")]
    backend::server::start_server();
}
