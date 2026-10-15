use dioxus::{server::{DioxusRouterExt, axum::{self, Extension}}};
use tokio::{runtime::LocalOptions, sync::watch};

use crate::{backend::{battery::{BatteryState, update_battery_state}, camera_interface::{CameraInterface, update_camera}, gps_receiver::{SateliteCount, update_gps}}, frontend::app::App};

pub fn start_server() {
    tokio::runtime::Builder::new_current_thread()
        .max_blocking_threads(4)
        .enable_all()
        .build_local(LocalOptions::default())
        .unwrap()
        .block_on(async {
            let(battery_sender, battery_receiver) = watch::channel(BatteryState { percentage: 0 });
            let (gps_count_sender, gps_count_receiver) = watch::channel(SateliteCount { count: 0 });
            let (camera_interface_sender, camera_interface_receiver) = watch::channel(CameraInterface::new());

            tokio::spawn(update_battery_state(battery_sender.clone()));
            tokio::spawn(update_gps(gps_count_sender.clone()));
            tokio::spawn(update_camera(camera_interface_receiver.clone()));

            let addr = dioxus::cli_config::fullstack_address_or_localhost();
            let router = axum::Router::new()
                .serve_dioxus_application(
                    dioxus::server::ServeConfig::builder(),
                    App
                )
                .layer(Extension(battery_receiver))
                .layer(Extension(gps_count_receiver))
                .layer(Extension(camera_interface_sender))
                .layer(Extension(camera_interface_receiver));

            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            
            axum::serve(listener, router).await.unwrap()
        });
}