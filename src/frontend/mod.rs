pub mod app;
mod android_back;
#[cfg(target_os = "android")]
mod android_status_bar;
mod icons;
mod navbar;
mod frondend_data;
mod settings_pages;
mod router; 
mod pages;