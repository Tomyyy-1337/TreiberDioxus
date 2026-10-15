#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "server")]
pub mod hardware_interface;
pub mod tasks;

pub mod battery;
pub mod socket;
pub mod gps_receiver;
pub mod camera_interface;
pub mod fahrt;