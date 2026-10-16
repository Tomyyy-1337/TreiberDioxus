use std::time::Duration;

use tokio::{sync::watch, time::MissedTickBehavior};

use crate::CurrentBpm;


/// Read current bpm and velocity(TODO) and update the current stats sender once per second.
pub async fn send_current_stats_task(
    bpm_receiver: watch::Receiver<CurrentBpm>,
    current_stats_sender: watch::Sender<crate::CurrentStats>,
) {
    let mut timer = tokio::time::interval(Duration::from_secs(1));
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        let bpm = bpm_receiver.borrow().bpm;
        let velocity = 0.0; // TODO Replace with actual velocity 

        let current_states = crate::CurrentStats { bpm, velocity };
        let _ = current_stats_sender.send_if_modified(|current| {
            let changed = *current != current_states;
            if changed {
                *current = current_states;
            }
            changed
        });

        timer.tick().await;
    }
}