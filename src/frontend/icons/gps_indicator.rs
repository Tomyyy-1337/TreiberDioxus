use dioxus::prelude::*;

use crate::{
	backend::gps_receiver::current_satelite_count,
	frontend::app::{GPS_COUNT, WEBSOCKET_DISCONNECTED},
};

#[component]
pub fn GpsIndicator() -> Element {
	#[cfg(feature = "server")]
	let initial_value = use_server_future(current_satelite_count)?.unwrap()?;
	#[cfg(not(feature = "server"))]
	let initial_value = 0;

	let display_value = use_memo(move || match GPS_COUNT() {
		Some(value) => value,
		None => initial_value,
	});

	let color = use_memo(move || match display_value() {
		0..=3 => "text-red-500",
		4..=7 => "text-yellow-500",
		_ => "text-green-500",
	});

	rsx! {
		if !WEBSOCKET_DISCONNECTED() {
			div {
				class: format!("flex items-center gap-1 font-bold leading-none {}", color()),
				aria_label: format!("Satelliten: {}", display_value()),
				div {
					class: "relative h-[1.1rem] w-[1.1rem] shrink-0",
					aria_hidden: "true",
					div {
						class: "absolute left-1/2 top-1/2 h-[0.38rem] w-[0.38rem] -translate-x-1/2 -translate-y-1/2 rotate-45 rounded-[0.1rem] bg-current",
					}
					div {
						class: "absolute left-[0.04rem] top-1/2 h-[0.76rem] w-[0.28rem] -translate-y-1/2 rounded-[0.08rem] bg-current opacity-90",
					}
					div {
						class: "absolute right-[0.04rem] top-1/2 h-[0.76rem] w-[0.28rem] -translate-y-1/2 rounded-[0.08rem] bg-current opacity-90",
					}
				}
				span {
					class: "inline-flex min-w-[2ch] items-center justify-center text-center text-[0.85rem]",
					"{display_value()}"
				}
			}
		}
	}
}
