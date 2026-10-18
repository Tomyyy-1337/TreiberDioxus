use dioxus::prelude::*;

#[cfg(feature = "server")]
const MAX_RANGE_LENGTH: u64 = 4 * 1024 * 1024;

#[post("/api/map_range")]
pub async fn map_range(offset: u64, length: u64) -> Result<Vec<u8>> {
    use std::io::SeekFrom;
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    if length > MAX_RANGE_LENGTH {
        return Err(dioxus::CapturedError::from_display("requested range too large"));
    }
    let mut file = tokio::fs::File::open("maps/germany.pmtiles").await?;
    file.seek(SeekFrom::Start(offset)).await?;
    let mut buf = vec![0u8; length as usize];
    file.read_exact(&mut buf).await?;
    Ok(buf)
}
