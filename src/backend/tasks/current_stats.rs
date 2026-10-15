use serde::{Deserialize, Serialize};

use crate::backend::tasks::read_accelerometer::CurrentBpm;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CurrentStats {
    pub bpm: f32,
    pub velocity: f32,
}

#[cfg(feature = "server")]
pub async fn current_stats(
    bpm_receiver: tokio::sync::watch::Receiver<CurrentBpm>,
    current_stats_sender: tokio::sync::watch::Sender<CurrentStats>,
) {
    let mut timer = tokio::time::interval(std::time::Duration::from_secs(1));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        let bpm = bpm_receiver.borrow().bpm;
        let velocity = 0.0; // TODO Replace with actual velocity 

        let current_states = CurrentStats { bpm, velocity };
        let _ = current_stats_sender.send(current_states);

        timer.tick().await;
    }
}