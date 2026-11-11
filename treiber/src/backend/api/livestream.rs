use dioxus::prelude::*;

const WHEP_URL: &str = "http://127.0.0.1:8889/stream/whep";

/// Forwards the browser's SDP offer to MediaMTX and returns the SDP answer.
#[post("/api/livestream/whep")]
pub async fn whep_offer(offer: String) -> Result<String> {
    let response = reqwest::Client::new()
        .post(WHEP_URL)
        .header(reqwest::header::CONTENT_TYPE, "application/sdp")
        .body(offer)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ServerFnError::new(format!("WHEP {}", response.status())).into());
    }

    let sdp = response.text().await.map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(sdp)
}
