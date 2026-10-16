use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct FinishedFahrt {
    pub client_start_time: String,
    pub schlag_count: u32,
    pub distance_traveled: f32,
    pub gps_history: Vec<GpsPosition>,
    pub active_time: std::time::Duration,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct FahrtInfo {
    pub paused: bool,
    pub schlag_count: u32,
    pub distance_traveled: f32,
    pub active_time: std::time::Duration,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct BatteryState {
    pub percentage: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CurrentStats {
    pub bpm: f32,
    pub velocity: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct GpsPosition {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct SateliteCount {
    pub count: u8,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct CurrentBpm {
    pub bpm: f32,
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