use dioxus::prelude::*;
#[cfg(feature = "server")]
use dioxus::server::axum::Extension;
#[cfg(feature = "server")]
use tokio::sync::watch;

use crate::{FahrtInfo, FinishedFahrt, GpsPosition};

#[post("/api/toggle_fahrt", fahrt: Extension<watch::Sender<Option<Fahrt>>>, finished_fahrt: Extension<watch::Sender<FinishedFahrt>>)]
pub async fn toggle_fahrt(client_time: String) -> Result<()> {
    fahrt.send_modify(|fahrt| {
        *fahrt = match fahrt.take() {
            Some(fahrt) => {
                let _ = finished_fahrt.send(fahrt.finish());
                None
            },
            None => Some(Fahrt::new(client_time))
        }
    });
    return Ok(());
}

pub struct Fahrt {
    pub client_start_time: String,
    pub active_time: std::time::Duration,
    pub last_gps_update: std::time::Instant,
    pub paused: bool,
    pub schlag_count: u32,
    pub distance_traveled: f32,
    pub gps_history: Vec<GpsPosition>,
    pub last_history_update: std::time::Instant,
    pub last_position: Option<GpsPosition>,
}

impl Fahrt {
    pub fn new(client_start_time: String) -> Self {
        Self {
            client_start_time,
            active_time: std::time::Duration::from_secs(0),
            last_gps_update: std::time::Instant::now(),
            paused: true,
            schlag_count: 0,
            distance_traveled: 0.0,
            gps_history: Vec::new(),
            last_history_update: std::time::Instant::now() - std::time::Duration::from_secs(60),
            last_position: None,
        }
    }

    pub fn get_info(&self) -> FahrtInfo {
        FahrtInfo {
            paused: self.paused,
            schlag_count: self.schlag_count,
            distance_traveled: self.distance_traveled,
            active_time: self.active_time,
        }
    }

    pub fn finish(self) -> FinishedFahrt {
        FinishedFahrt {
            client_start_time: self.client_start_time,
            schlag_count: self.schlag_count,
            distance_traveled: self.distance_traveled,
            active_time: self.active_time,
            // gps_history: self.gps_history,
            gps_history: vec![
                GpsPosition { latitude: 47.789, longitude: 11.123 },
                GpsPosition { latitude: 47.790, longitude: 11.124 },
                GpsPosition { latitude: 47.791, longitude: 11.123 },
            ]
        }
    }

    pub fn increment_schläge(&mut self, increment: u32) {
        self.schlag_count += increment;
    }

    pub fn no_satellites(&mut self) {
        self.paused = true;
        self.last_position = None;
    }

    pub fn add_gps_data(&mut self, latitude: f64, longitude: f64, velocity: f32) {
        let now = std::time::Instant::now();
        let delta = now - self.last_gps_update;
        self.last_gps_update = now;

        self.paused = velocity <= 0.1;
        if self.paused {
            self.last_position = None;
            return;
        } 

        self.active_time += delta; 

        // Add GPS position to history 
        if self.last_history_update.elapsed() >= std::time::Duration::from_secs(15) {
            self.gps_history.push(GpsPosition { latitude, longitude });
            self.last_history_update = std::time::Instant::now();
        }

        // Add distance traveled since the last GPS position
        if let Some(last_position) = self.last_position {
            self.distance_traveled += Self::calculate_distance(last_position.latitude,last_position.longitude,latitude,longitude);
        }
        self.last_position = Some(GpsPosition { latitude, longitude });        
    }

    pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f32 {
        let r = 6371.0; 
        let dlat = (lat2 - lat1).to_radians();
        let dlon = (lon2 - lon1).to_radians();
        let a = (dlat / 2.0).sin().powi(2) + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        (r * c) as f32 
    }
        
}