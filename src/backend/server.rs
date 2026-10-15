use std::net::SocketAddr;

use dioxus::{server::{DioxusRouterExt, axum::{self, Extension}}};
use tokio::{runtime::LocalOptions, sync::watch};

use crate::{backend::{tasks::battery::{BatteryState, update_battery_state}, camera_interface::{CameraInterface, update_camera}, fahrt::{Fahrt, FahrtInfo}, gps_receiver::{SateliteCount, update_gps}, hardware_interface::i2c_interface::I2cInterface, tasks::{fahrt_info_creator::fahrt_info_creator_task, read_accelerometer::{CurrentBpm, read_accelerometer_task}}}, frontend::app::App};

lazy_static::lazy_static! {
    pub static ref I2C_INTERFACE: I2cInterface = I2cInterface::new();
}

pub fn start_server() {
    tokio::runtime::Builder::new_current_thread()
        .max_blocking_threads(4)
        .enable_all()
        .build_local(LocalOptions::default())
        .unwrap()
        .block_on(async {
            let (battery_sender, battery_receiver) = watch::channel(BatteryState { percentage: 0 });
            let (gps_count_sender, gps_count_receiver) = watch::channel(SateliteCount { count: 0 });
            let (camera_interface_sender, camera_interface_receiver) = watch::channel(CameraInterface::new());
            let (fahrt_sender, fahrt_receiver) = watch::channel(None::<Fahrt>);
            let (fahrt_info_sender, fahrt_info_receiver) = watch::channel(None::<FahrtInfo>);

            let (bpm_sender, bpm_receiver) = watch::channel(CurrentBpm { bpm: 0.0 });

            tokio::spawn(update_battery_state(battery_sender.clone()));
            tokio::spawn(update_gps(gps_count_sender.clone()));
            tokio::spawn(update_camera(camera_interface_receiver.clone()));
            tokio::spawn(read_accelerometer_task(fahrt_sender.clone(), bpm_sender));
            tokio::spawn(fahrt_info_creator_task(fahrt_receiver.clone(), fahrt_info_sender.clone()));
            
            // Nedded only for dioxus cli commands
            // let addr = dioxus::cli_config::fullstack_address_or_localhost();
            let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
            let router = axum::Router::new()
                .serve_dioxus_application(
                    dioxus::server::ServeConfig::builder(),
                    App
                )
                .layer(Extension(battery_receiver))
                .layer(Extension(gps_count_receiver))
                .layer(Extension(camera_interface_sender))
                .layer(Extension(camera_interface_receiver))
                .layer(Extension(fahrt_info_receiver))
                .layer(Extension(bpm_receiver));

            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            
            axum::serve(listener, router).await.unwrap()
        });
}