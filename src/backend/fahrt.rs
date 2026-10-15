use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[post("/api/toggle_fahrt", fahrt: dioxus::server::axum::Extension<tokio::sync::watch::Sender<Option<Fahrt>>>)]
pub async fn toggle_fahrt(client_time: String) -> Result<()> {
    fahrt.send_modify(|fahrt| {
        if fahrt.is_some() {
            *fahrt = None;
        } else {
            *fahrt = Some(Fahrt::new(client_time));
        }
    });
    
    return Ok(());
}

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
    pub fn new(client_time: String) -> Self {
        Self {
            client_time,
            active_time: std::time::Duration::new(0, 0),
            last_update: std::time::Instant::now(),
            paused: false,
            schlag_count: 0,
            distance_traveled: 0.0,
            gps_history: Vec::new(),
        }
    }

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