use std::{str::FromStr, time::Duration};
use dioxus::prelude::*;
use serde::Serialize;

use crate::CameraInterface;
use crate::{FocusMode, Metering};

lazy_static::lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::builder().build().unwrap();
}

/// Send updates to the camera when the camera interface changes
pub async fn update_camera_task(mut camera_receiver: tokio::sync::watch::Receiver<CameraInterface>) {    
    camera_receiver.mark_changed();

    while let Ok(_) = camera_receiver.changed().await {
        let camera_interface = camera_receiver.borrow().clone();
        if let Err(_e) = camera_interface.send_to_camera().await {
            println!("Camera can not be reached");
        }
    }
}

impl CameraInterface {
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

impl ToString for Metering {
    fn to_string(&self) -> String {
        match self {
            Metering::Center => "centre".to_owned(),
            Metering::Average => "matrix".to_owned(),
        }
    }
}

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

impl ToString for FocusMode {
    fn to_string(&self) -> String {
        match self {
            FocusMode::Auto => "continuous".to_owned(),
            FocusMode::Fixed => "manual".to_owned(),
        }
    }
}

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