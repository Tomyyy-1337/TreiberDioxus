use serde::{Deserialize, Serialize};

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

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct CurrentBpm {
    pub bpm: f32,
}