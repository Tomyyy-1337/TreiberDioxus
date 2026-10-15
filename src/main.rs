mod backend;

mod frontend;

fn main() {
    #[cfg(not(feature = "server"))]
    init_storage();

    #[cfg(all(not(feature = "server"), target_os = "android"))]
    dioxus::fullstack::set_server_url(SERVER_URL);

    #[cfg(not(feature = "server"))]
    dioxus::launch(crate::frontend::app::App);

    #[cfg(feature = "server")]
    backend::server::start_server();
}


/// `10.0.2.2` is the emulator's alias for the host. Override with `ANDROID_SERVER_URL` for a physical device.
#[cfg(target_os = "android")]
const SERVER_URL: &str = match option_env!("ANDROID_SERVER_URL") {
    Some(url) => url,
    None => "http://10.0.2.2:8080",
};

/// App-private internal storage; writable without runtime permissions. Must match the Android `applicationId`.
#[cfg(target_os = "android")]
const ANDROID_STORAGE_DIR: &str = "/data/data/com.example.RuderCamBeta/files";

/// Sets the persistent storage directory before any `LocalStorage` access.
#[cfg(not(feature = "server"))]
fn init_storage() {
    #[cfg(target_os = "android")]
    dioxus_sdk::storage::set_directory(ANDROID_STORAGE_DIR.into());

    // iOS sandbox: HOME points to the app container.
    #[cfg(target_os = "ios")]
    dioxus_sdk::storage::set_directory(
        std::path::PathBuf::from(std::env::var("HOME").expect("HOME is set in the iOS sandbox"))
            .join("Library/Application Support/ruder_cam_beta"),
    );

    #[cfg(not(any(target_os = "android", target_os = "ios", target_family = "wasm")))]
    dioxus_sdk::storage::set_dir!();
}