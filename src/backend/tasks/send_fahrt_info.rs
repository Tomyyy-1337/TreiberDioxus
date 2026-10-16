use std::time::Duration;

use crate::backend::fahrt::{Fahrt, FahrtInfo};

pub async fn send_fahrt_task(
    mut fahrt_receiver: tokio::sync::watch::Receiver<Option<Fahrt>>,
    fahrt_info_sender: tokio::sync::watch::Sender<Option<FahrtInfo>>,
) {
    let mut rate_limit = tokio::time::interval(Duration::from_millis(1000));
    while fahrt_receiver.changed().await.is_ok() {
        rate_limit.tick().await;
        let fahrt_info = fahrt_receiver.borrow().as_ref().map(|fahrt| fahrt.get_info());
        let _ = fahrt_info_sender.send(fahrt_info);
    }
}