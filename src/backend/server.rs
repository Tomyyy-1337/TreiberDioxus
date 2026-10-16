use std::net::SocketAddr;

use dioxus::{server::{DioxusRouterExt, axum::{self, Extension}}};
use tokio::{runtime::LocalOptions, sync::watch};

use crate::{BatteryState, CurrentBpm, CurrentStats, backend::{camera_interface::{CameraInterface, update_camera}, fahrt::{Fahrt, FahrtInfo, FinishedFahrt}, gps_receiver::{SateliteCount, update_gps}, hardware_interface::i2c_interface::I2cInterface, tasks::{battery_reader::update_battery_state, current_stats::current_stats, fahrt_info_creator::fahrt_info_creator_task, read_accelerometer::read_accelerometer_task}}, frontend::app::App};

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
            tokio::spawn(update_battery_state(battery_sender.clone()));

            let (bpm_sender, bpm_receiver) = watch::channel(CurrentBpm { bpm: 0.0 });
            let (fahrt_sender, fahrt_receiver) = watch::channel(None::<Fahrt>);
            tokio::spawn(read_accelerometer_task(fahrt_sender.clone(), bpm_sender));

            let (fahrt_info_sender, fahrt_info_receiver) = watch::channel(None::<FahrtInfo>);
            tokio::spawn(fahrt_info_creator_task(fahrt_receiver, fahrt_info_sender));
            
            let (current_stats_sender, current_stats_receiver) = watch::channel(CurrentStats { bpm: 0.0, velocity: 0.0 });
            tokio::spawn(current_stats(bpm_receiver, current_stats_sender));
            
            let (gps_count_sender, gps_count_receiver) = watch::channel(SateliteCount { count: 0 });
            tokio::spawn(update_gps(gps_count_sender.clone()));
            
            let (camera_interface_sender, camera_interface_receiver) = watch::channel(CameraInterface::new());
            tokio::spawn(update_camera(camera_interface_receiver.clone()));

            let (finished_fahrt_sender, finished_fahrt_receiver) = watch::channel(FinishedFahrt {});
            
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
                .layer(Extension(fahrt_sender))
                .layer(Extension(finished_fahrt_sender))
                .layer(Extension(finished_fahrt_receiver))
                .layer(Extension(current_stats_receiver));

            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            
            axum::serve(listener, router).await.unwrap()
        });
}