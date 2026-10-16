use crate::CurrentBpm;


/// Read current bpm and velocity(TODO) and update the current stats sender once per second.
pub async fn current_stats(
    bpm_receiver: tokio::sync::watch::Receiver<CurrentBpm>,
    current_stats_sender: tokio::sync::watch::Sender<crate::CurrentStats>,
) {
    let mut timer = tokio::time::interval(std::time::Duration::from_secs(1));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        let bpm = bpm_receiver.borrow().bpm;
        let velocity = 0.0; // TODO Replace with actual velocity 

        let current_states = crate::CurrentStats { bpm, velocity };
        let _ = current_stats_sender.send(current_states);

        timer.tick().await;
    }
}