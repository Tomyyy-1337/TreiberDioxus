use std::{str::FromStr, time::Duration};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
lazy_static::lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::builder().build().unwrap();
}

/// Send updates to the camera when the camera interface changes
#[cfg(feature = "server")]
pub async fn update_camera(mut camera_receiver: tokio::sync::watch::Receiver<CameraInterface>) {    
    camera_receiver.mark_changed();

    while let Ok(_) = camera_receiver.changed().await {
        let camera_interface = camera_receiver.borrow().clone();
        if let Err(_e) = camera_interface.send_to_camera().await {
            println!("Camera can not be reached");
        }
    }
}


#[post("/api/set_focal_length", ext: dioxus::server::axum::Extension<tokio::sync::watch::Sender<CameraInterface>>)]
pub async fn set_focal_length(focal_length: u8) -> Result<()> { 
    ext.0.send_modify(|interface| {
        interface.focal_length = focal_length;
    });
    
    Ok(())
}

#[post("/api/set_metering_mode", ext: dioxus::server::axum::Extension<tokio::sync::watch::Sender<CameraInterface>>)]
pub async fn set_metering_mode(metering_mode: Metering) -> Result<()> {
    ext.0.send_modify(|interface| {
        interface.metering_mode = metering_mode;
    });

    Ok(())
}

#[post("/api/set_focus_mode", ext: dioxus::server::axum::Extension<tokio::sync::watch::Sender<CameraInterface>>)]
pub async fn set_focus_mode(focus_mode: FocusMode) -> Result<()> {
    ext.0.send_modify(|interface| {
        interface.focus_mode = focus_mode;
    });

    Ok(())
}

#[post("/api/set_bitrate", ext: dioxus::server::axum::Extension<tokio::sync::watch::Sender<CameraInterface>>)]
pub async fn set_bitrate(bitrate: u32) -> Result<()> {
    ext.0.send_modify(|interface| {
        interface.bitrate = bitrate;
    });

    Ok(())
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CameraInterface {
    pub hdr_enabled: bool,
    pub exposure_compenstion: f32,
    pub metering_mode: Metering,
    pub focus_mode: FocusMode,
    pub focal_length: u8,
    pub bitrate: u32,
}

#[derive(Serialize, Debug, Copy, Clone, PartialEq, Eq, Deserialize)]
pub enum Metering {
    Center,
    Average,
}

#[derive(Serialize, Debug, Copy, Clone, PartialEq, Eq, Deserialize)]
pub enum FocusMode {
    Auto,
    Fixed,
}

#[cfg(feature = "server")]
impl CameraInterface {
    pub fn new() -> Self {
        CameraInterface {
            hdr_enabled: true,
            exposure_compenstion: 0.0,
            metering_mode: Metering::Average,
            focus_mode: FocusMode::Fixed,
            focal_length: 28,
            bitrate: 3_200_000,
        }
    }

    pub async fn send_to_camera(&self) -> Result<(), reqwest::Error> {
        let config = CameraConfig::from(self);

        let response = CLIENT
            .patch("http://127.0.0.1:9997/v3/config/paths/patch/stream")
            .json(&config)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            println!("API error: {status}: {body}");
        }
        Ok(())
    }

    fn get_roi(&self) -> String {
        const BASE_FOCAL_LENGTH: f32 = 28.0;
        
        let crop = BASE_FOCAL_LENGTH / self.focal_length as f32;
        let offset = (1.0 - crop) / 2.0;

        format!("{offset:.6},{offset:.6},{crop:.6},{crop:.6}")
    }
}
    

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
struct CameraConfig {
    pub rpiCameraHDR: bool,
    pub rpiCameraEV: f32,
    pub rpiCameraMetering: String,
    pub rpiCameraAfMode: String,
    pub rpiCameraROI: String,
    pub rpiCameraBitrate: u32,
}

#[cfg(feature = "server")]
impl From<&CameraInterface> for CameraConfig {
    fn from(camera: &CameraInterface) -> Self {
        CameraConfig {
            rpiCameraHDR: camera.hdr_enabled,
            rpiCameraEV: camera.exposure_compenstion,
            rpiCameraMetering: camera.metering_mode.to_string(),
            rpiCameraAfMode: camera.focus_mode.to_string(),
            rpiCameraROI: camera.get_roi(),
            rpiCameraBitrate: camera.bitrate,
        }
    }
}

#[cfg(feature = "server")]
impl ToString for Metering {
    fn to_string(&self) -> String {
        match self {
            Metering::Center => "centre".to_owned(),
            Metering::Average => "matrix".to_owned(),
        }
    }
}

#[cfg(feature = "server")]
impl FromStr for Metering {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "centre" => Ok(Metering::Center),
            "matrix" => Ok(Metering::Average),
            _ => Err(()),
        }
    }
}

#[cfg(feature = "server")]
impl ToString for FocusMode {
    fn to_string(&self) -> String {
        match self {
            FocusMode::Auto => "continuous".to_owned(),
            FocusMode::Fixed => "manual".to_owned(),
        }
    }
}

#[cfg(feature = "server")]
impl FromStr for FocusMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "continuous" => Ok(FocusMode::Auto),
            "manual" => Ok(FocusMode::Fixed),
            _ => Err(()),
        }
    }
}