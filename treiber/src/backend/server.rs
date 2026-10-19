use std::net::SocketAddr;

use dioxus::{server::{DioxusRouterExt, axum::{self, Extension}}};
use tokio::{runtime::LocalOptions, sync::watch};
use tower_http::{CompressionLevel, compression::{CompressionLayer, predicate::SizeAbove}, cors::{Any, CorsLayer}, services::ServeDir};

use crate::{BatteryState, CameraInterface, CurrentBpm, CurrentStats, FahrtInfo, FinishedFahrt, SateliteCount, WlanSettings, backend::{api::toggle_fahrt::Fahrt, hardware_interface::i2c_interface::I2cInterface, tasks::{reader_accelerometer::read_accelerometer_task, reader_battery::read_battery_task, reader_gps::{Velocity, read_gps_task}, save_config::save_config_task, send_current_stats::send_current_stats_task, send_fahrt_info::send_fahrt_task, update_camera::update_camera_task}}, frontend::app::App};

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
            let (bpm_sender, bpm_receiver) = watch::channel(CurrentBpm { bpm: 0.0 });
            let (fahrt_sender, fahrt_receiver) = watch::channel(None::<Fahrt>);
            let (fahrt_info_sender, fahrt_info_receiver) = watch::channel(None::<FahrtInfo>);
            let (current_stats_sender, current_stats_receiver) = watch::channel(CurrentStats { bpm: 0.0, velocity: 0.0 });
            let (gps_count_sender, gps_count_receiver) = watch::channel(SateliteCount { count: 0 });
            let (camera_interface_sender, camera_interface_receiver) = watch::channel(CameraInterface::new());
            let (finished_fahrt_sender, finished_fahrt_receiver) = watch::channel(FinishedFahrt::default());
            let (velocity_sender, velocity_receiver) = watch::channel(Velocity(0.0));
            let (wlan_settings_sender, wlan_settings_receiver) = watch::channel(WlanSettings::default());
                
            tokio::spawn(read_battery_task(battery_sender));
            tokio::spawn(read_accelerometer_task(fahrt_sender.clone(), bpm_sender));
            tokio::spawn(send_fahrt_task(fahrt_receiver, fahrt_info_sender));
            tokio::spawn(send_current_stats_task(bpm_receiver, velocity_receiver, current_stats_sender));
            tokio::spawn(read_gps_task(gps_count_sender, velocity_sender, fahrt_sender.clone()));
            tokio::spawn(update_camera_task(camera_interface_receiver.clone()));
            tokio::spawn(save_config_task(camera_interface_receiver.clone(), wlan_settings_receiver));

            // Nedded only for dioxus cli commands
            // let addr = dioxus::cli_config::fullstack_address_or_localhost();
            let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
            let router = axum::Router::new()
                .serve_dioxus_application(
                    dioxus::server::ServeConfig::builder(),
                    App
                )
                .nest_service("/maps", ServeDir::new("maps"))
                .layer(CompressionLayer::new().gzip(true).quality(CompressionLevel::Default).compress_when(SizeAbove::new(1024)))
                // used by websocket
                .layer(Extension(battery_receiver))
                // used by websocket
                .layer(Extension(gps_count_receiver))
                // used by api::camera_settigns
                .layer(Extension(camera_interface_sender))
                // used by websocket and tasks::update_camera_task
                .layer(Extension(camera_interface_receiver))
                // used by websocket
                .layer(Extension(fahrt_info_receiver))
                // used by api::toggle_fahrt
                .layer(Extension(fahrt_sender))
                // used by api::finished_fahrt
                .layer(Extension(finished_fahrt_sender))
                // used by websocket
                .layer(Extension(finished_fahrt_receiver))
                // used by websocket
                .layer(Extension(current_stats_receiver));

            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            
            axum::serve(listener, router).await.unwrap()
        });
}