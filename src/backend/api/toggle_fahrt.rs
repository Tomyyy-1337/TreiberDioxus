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
    pub client_time: String,
    pub active_time: std::time::Duration,
    pub last_update: std::time::Instant,
    pub paused: bool,
    pub schlag_count: u32,
    pub distance_traveled: f64,
    pub gps_history: Vec<GpsPosition>,
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

    pub fn finish(self) -> FinishedFahrt {
        FinishedFahrt {
            
        }
    }

    pub fn increment_schläge(&mut self, increment: u32) {
        self.schlag_count += increment;
    }
}