#[cfg(feature = "server")]
use crate::SateliteCount;

#[cfg(feature = "server")]
pub async fn read_gps_task(gps_sender: tokio::sync::watch::Sender<SateliteCount>) {
    loop {
        use crate::SateliteCount;

        let new_count = (gps_sender.borrow().count + 1) % 16;
        let _ = gps_sender.send(SateliteCount { count: new_count });
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
