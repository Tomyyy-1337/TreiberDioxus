use dioxus::prelude::*;

const DOUBLE_TAP_MS: i64 = 400;

/// Records a tap and returns true when it completes a double tap. A single tap shows the hint.
pub fn register_tap(last_tap: Signal<Option<i64>>, mut hint: Signal<bool>) -> bool {
    let mut last_tap = last_tap;
    let now = chrono::Utc::now().timestamp_millis();
    let is_double = matches!(*last_tap.peek(), Some(prev) if now - prev <= DOUBLE_TAP_MS);
    if is_double {
        last_tap.set(None);
        hint.set(false);
        return true;
    }
    last_tap.set(Some(now));
    hint.set(true);
    spawn(async move {
        let mut timer = document::eval(&format!("setTimeout(() => dioxus.send(true), {DOUBLE_TAP_MS});"));
        let _ = timer.recv::<bool>().await;
        if *last_tap.peek() == Some(now) {
            hint.set(false);
        }
    });
    false
}
