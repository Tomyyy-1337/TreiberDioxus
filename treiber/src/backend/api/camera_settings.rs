use dioxus::prelude::*;

use crate::{CameraInterface, FocusMode, Metering};

#[cfg(feature = "server")]
use dioxus::server::axum::Extension;
#[cfg(feature = "server")]
use tokio::sync::watch;

#[post("/api/set_focal_length", ext: Extension<watch::Sender<CameraInterface>>)]
pub async fn set_focal_length(focal_length: u8) -> Result<()> { 
    ext.0.send_modify(|interface| {
        interface.focal_length = focal_length;
    });
    
    Ok(())
}

#[post("/api/set_metering_mode", ext: Extension<watch::Sender<CameraInterface>>)]
pub async fn set_metering_mode(metering_mode: Metering) -> Result<()> {
    ext.0.send_modify(|interface| {
        interface.metering_mode = metering_mode;
    });

    Ok(())
}

#[post("/api/set_focus_mode", ext: Extension<watch::Sender<CameraInterface>>)]
pub async fn set_focus_mode(focus_mode: FocusMode) -> Result<()> {
    ext.0.send_modify(|interface| {
        interface.focus_mode = focus_mode;
    });

    Ok(())
}

#[post("/api/set_bitrate", ext: Extension<watch::Sender<CameraInterface>>)]
pub async fn set_bitrate(bitrate: u32) -> Result<()> {
    ext.0.send_modify(|interface| {
        interface.bitrate = bitrate;
    });

    Ok(())
}

#[post("/api/toggle_hdr", ext: Extension<watch::Sender<CameraInterface>>)]
pub async fn toggle_hdr() -> Result<bool> {
    let mut hdr_enabled = false;
    ext.0.send_modify(|interface| {
        interface.hdr_enabled = !interface.hdr_enabled;
        hdr_enabled = interface.hdr_enabled;
    });

    Ok(hdr_enabled)
}

#[post("/api/set_exposure", ext: Extension<watch::Sender<CameraInterface>>)]
pub async fn set_exposure(exposure_compenstion: f32) -> Result<()> {
    ext.0.send_modify(|interface| {
        interface.exposure_compenstion = exposure_compenstion.clamp(-3.0, 3.0);
    });

    Ok(())
}
