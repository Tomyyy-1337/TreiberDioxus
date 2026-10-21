use chrono::{DateTime, Datelike, Local, Timelike};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct FinishedFahrt {
    pub client_start_time: SerializableDateTime,
    pub schlag_count: u32,
    pub distance_traveled: f32,
    pub gps_history: Vec<GpsPosition>,
    pub active_time: std::time::Duration,
}

impl FinishedFahrt {
    pub fn active_time_string(&self) -> String {
        let seconds = self.active_time.as_secs();
        let seconds = seconds % 60;
        let minutes = (self.active_time.as_secs() / 60) % 60;
        let hours = self.active_time.as_secs() / 3600;
        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WlanSettings {
    pub ssid: String,
    pub password: String,
}

impl Default for WlanSettings {
    fn default() -> Self {
        WlanSettings {
            ssid: "RuderCam".to_string(),
            password: "1234".to_string()
        }
    }
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


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SerializableDateTime {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl SerializableDateTime {
    pub fn date_string(&self) -> String {
        format!("{:02}.{:02}.{:04}", self.day, self.month, self.year)
    }

    pub fn time_string(&self) -> String {
        format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }

    pub fn hour_minute_string(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }

    pub fn year_month(&self) -> (u16, u8) {
        (self.year, self.month)
    }

    pub fn month_label(&self) -> String {
        const MONTHS: [&str; 12] = [
            "Januar", "Februar", "März", "April", "Mai", "Juni",
            "Juli", "August", "September", "Oktober", "November", "Dezember",
        ];
        let name = MONTHS.get(self.month.wrapping_sub(1) as usize).copied().unwrap_or("?");
        format!("{} {}", name, self.year)
    }
}

impl From<DateTime<Local>> for SerializableDateTime { 
    fn from(dt: DateTime<Local>) -> Self {
        SerializableDateTime {
            year: dt.year() as u16,
            month: dt.month() as u8,
            day: dt.day() as u8,
            hour: dt.hour() as u8,
            minute: dt.minute() as u8,
            second: dt.second() as u8,
        }
    }
}