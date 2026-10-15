use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GpsPosition {
    pub latitude: f64,
    pub longitude: f64,
}

pub struct Fahrt {
    pub client_time: String,
    pub active_time: std::time::Duration,
    pub last_update: std::time::Instant,
    pub paused: bool,
    pub schlag_count: u32,
    pub distance_traveled: f64,
    pub gps_history: Vec<GpsPosition>,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct FahrtInfo {
    paused: bool,
    schlag_count: u32,
    distance_traveled: f64,
}

impl Fahrt {
    pub fn get_info(&self) -> FahrtInfo {
        FahrtInfo {
            paused: self.paused,
            schlag_count: self.schlag_count,
            distance_traveled: self.distance_traveled,
        }
    }

    pub fn increment_schläge(&mut self, increment: u32) {
        self.schlag_count += increment;
    }
}