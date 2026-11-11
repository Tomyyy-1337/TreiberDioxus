use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{
    Router,
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

type StepResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const OUTPUTS: [&str; 2] = ["build/server", "build/public"];
const HASH_HEADER: &str = "x-sha256";
const STAGING_DIR_NAME: &str = ".update_staging";
const DEFAULT_TREIBER_DIR: &str = "treiber";
const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:9000";
const MAX_UPLOAD_BYTES: usize = 1024 * 1024 * 1024;
const BACKEND_SERVICE: &str = "backend.service";

#[derive(Clone)]
struct AppState {
    treiber_dir: Arc<PathBuf>,
    update_lock: Arc<Mutex<()>>,
}

fn hash_matches(data: &[u8], expected_hex: &str) -> bool {
    let digest = Sha256::digest(data);
    let actual: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    actual.eq_ignore_ascii_case(expected_hex.trim())
}

fn remove_path(path: &Path) -> StepResult<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn extract_archive(data: &[u8], staging: &Path) -> StepResult<()> {
    fs::create_dir_all(staging)?;
    let mut archive = tar::Archive::new(GzDecoder::new(data));
    archive.unpack(staging)?;
    Ok(())
}

fn replace_outputs(staging: &Path, treiber_dir: &Path) -> StepResult<()> {
    for rel in OUTPUTS {
        if !staging.join(rel).exists() {
            return Err(format!("archive is missing {rel}").into());
        }
    }
    for rel in OUTPUTS {
        let new_path = staging.join(rel);
        let old_path = treiber_dir.join(rel);
        remove_path(&old_path)?;
        if let Some(parent) = old_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&new_path, &old_path)?;
    }
    Ok(())
}

fn systemctl(action: &str) -> StepResult<()> {
    let status = std::process::Command::new("systemctl")
        .arg(action)
        .arg(BACKEND_SERVICE)
        .status()?;
    if !status.success() {
        return Err(format!("systemctl {action} {BACKEND_SERVICE} failed: {status}").into());
    }
    Ok(())
}

fn apply_update(data: &[u8], treiber_dir: &Path) -> StepResult<()> {
    let staging = treiber_dir.with_file_name(STAGING_DIR_NAME);
    remove_path(&staging)?;
    let result = extract_archive(data, &staging)
        .and_then(|_| replace_outputs(&staging, treiber_dir));
    let cleanup = remove_path(&staging);
    result.and(cleanup)
}

async fn upload_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> (StatusCode, String) {
    let Some(expected) = headers.get(HASH_HEADER).and_then(|v| v.to_str().ok()) else {
        return (StatusCode::BAD_REQUEST, format!("missing {HASH_HEADER} header"));
    };
    let expected = expected.to_owned();
    let Ok(_guard) = state.update_lock.try_lock() else {
        return (StatusCode::CONFLICT, "update already in progress".into());
    };

    let treiber_dir = state.treiber_dir.clone();
    let result = tokio::task::spawn_blocking(move || -> StepResult<bool> {
        if !hash_matches(&body, &expected) {
            return Ok(false);
        }
        let _ = systemctl("stop");
        let result = apply_update(&body, &treiber_dir);
        let started = systemctl("start");
        result?;
        started?;
        Ok(true)
    })
    .await;

    match result {
        Ok(Ok(true)) => (StatusCode::OK, "update applied".into()),
        Ok(Ok(false)) => (StatusCode::BAD_REQUEST, "hash mismatch".into()),
        Ok(Err(err)) => (StatusCode::INTERNAL_SERVER_ERROR, format!("update failed: {err}")),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("update task failed: {err}")),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let treiber_dir = std::env::var("TREIBER_DIR").unwrap_or_else(|_| DEFAULT_TREIBER_DIR.into());
    let listen_addr = std::env::var("UPDATE_LISTEN_ADDR").unwrap_or_else(|_| DEFAULT_LISTEN_ADDR.into());

    let state = AppState {
        treiber_dir: Arc::new(PathBuf::from(treiber_dir)),
        update_lock: Arc::new(Mutex::new(())),
    };

    let app = Router::new()
        .route("/update", post(upload_update))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    axum::serve(listener, app).await
}
