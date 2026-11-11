use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use sha2::{Digest, Sha256};

pub type StepResult<T> = Result<T, Box<dyn Error>>;

const WEB_OUTPUT_REL: &str = "build/public";
const SERVER_OUTPUT_REL: &str = "build/server";
const PACKAGE_STAGING_REL: &str = "build/package";
const ARCHIVE_ROOT: &str = "treiber";
const ARCHIVE_NAME: &str = "treiber_update.tar.gz";
const HASH_FILE_SUFFIX: &str = ".sha256";
const LAST_WIFI_FILE_NAME: &str = "last_wifi.txt";
const UPDATE_HOST: &str = "192.168.50.1";
const UPDATE_PORT: u16 = 9000;
const UPLOAD_TIMEOUT: Duration = Duration::from_secs(600);

pub struct Paths {
    pub treiber_dir: PathBuf,
    pub updates_dir: PathBuf,
}

impl Paths {
    pub fn from_workspace() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        Self {
            treiber_dir: root.join("treiber"),
            updates_dir: root.join("webspace_config").join("updates"),
        }
    }
}

fn run_command(program: &str, args: &[&str], cwd: &Path) -> StepResult<()> {
    let status = Command::new(program).args(args).current_dir(cwd).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`{program} {}` failed with {status}", args.join(" ")).into())
    }
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Step 1: Remove previous build outputs.
pub fn clean_build_outputs(paths: &Paths) -> StepResult<()> {
    for rel in [WEB_OUTPUT_REL, SERVER_OUTPUT_REL, PACKAGE_STAGING_REL] {
        let path = paths.treiber_dir.join(rel);
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else if path.exists() {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

/// Step 2: Build the frontend and the Raspberry Pi server via Docker.
pub fn build_server(paths: &Paths) -> StepResult<()> {
    run_command(
        "docker",
        &["build", "--target", "artifacts", "-o", "type=local,dest=./build", "."],
        &paths.treiber_dir,
    )
}

/// Step 4: Package the frontend and server artifacts into a tar.gz archive.
pub fn package_artifacts(paths: &Paths) -> StepResult<PathBuf> {
    fs::create_dir_all(&paths.updates_dir)?;
    let archive = paths.updates_dir.join(ARCHIVE_NAME);
    let archive_str = archive.to_str().ok_or("archive path is not valid UTF-8")?;
    // Archive entries must sit under treiber/ to match the update service.
    let staging = paths.treiber_dir.join(PACKAGE_STAGING_REL);
    let staged_root = staging.join(ARCHIVE_ROOT);
    fs::create_dir_all(&staged_root)?;
    for rel in [WEB_OUTPUT_REL, SERVER_OUTPUT_REL] {
        let name = Path::new(rel).file_name().ok_or("output path has no name")?;
        fs::rename(paths.treiber_dir.join(rel), staged_root.join(name))?;
    }
    run_command("tar", &["-czf", archive_str, ARCHIVE_ROOT], &staging)?;
    fs::remove_dir_all(&staging)?;
    Ok(archive)
}

/// Step 5: Compute the SHA-256 hash of a file.
pub fn generate_hash(file: &Path) -> StepResult<String> {
    let mut reader = File::open(file)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(to_hex(&hasher.finalize()))
}

/// Step 6: Write the hash to a `.sha256` file next to the archive.
pub fn write_hash_file(archive: &Path, hash: &str) -> StepResult<PathBuf> {
    let file_name = archive
        .file_name()
        .ok_or("archive has no file name")?
        .to_string_lossy()
        .into_owned();
    let hash_path = archive.with_file_name(format!("{file_name}{HASH_FILE_SUFFIX}"));
    fs::write(&hash_path, format!("{hash}  {file_name}\n"))?;
    Ok(hash_path)
}

/// Step 7: Recompute the archive hash and compare it with the stored hash file.
pub fn verify_integrity(archive: &Path, hash_file: &Path) -> StepResult<bool> {
    let stored = fs::read_to_string(hash_file)?;
    let expected = stored
        .split_whitespace()
        .next()
        .ok_or("hash file is empty")?;
    let actual = generate_hash(archive)?;
    Ok(actual.eq_ignore_ascii_case(expected))
}

fn wait_for_enter() -> StepResult<()> {
    print!("Press Enter to upload the update to {UPDATE_HOST}...");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(())
}

fn run_netsh(args: &[&str]) -> StepResult<String> {
    let output = Command::new("netsh").args(args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    if output.status.success() {
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("`netsh {}` failed: {stdout}{stderr}", args.join(" ")).into())
    }
}

fn wait_for_wifi(ssid: &str) -> StepResult<()> {
    for _ in 0..30 {
        let output = run_netsh(&["wlan", "show", "interfaces"])?;
        let mut current_ssid = None;
        let mut connected = false;
        for line in output.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim();
                if key == "ssid" {
                    current_ssid = Some(value.to_owned());
                } else if matches!(key.as_str(), "state" | "status" | "zustand") {
                    let state = value.to_lowercase();
                    connected = state.starts_with("connect") || state.starts_with("verbunden");
                }
            }
        }
        if current_ssid.as_deref() == Some(ssid) && connected {
            return Ok(());
        }
        thread::sleep(Duration::from_secs(1));
    }
    Err(format!("could not connect to {ssid}").into())
}

fn list_known_wifis() -> StepResult<Vec<String>> {
    let output = run_netsh(&["wlan", "show", "profiles"])?;
    let profiles: Vec<String> = output
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(key, _)| key.to_lowercase().contains("profil"))
        .map(|(_, value)| value.trim().to_owned())
        .filter(|name| !name.is_empty())
        .collect();
    if profiles.is_empty() {
        return Err(format!("no Wi-Fi profiles parsed from netsh output:\n{output}").into());
    }
    Ok(profiles)
}

fn list_visible_wifis() -> StepResult<Vec<String>> {
    let output = run_netsh(&["wlan", "show", "networks"])?;
    let mut names: Vec<String> = output
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(key, _)| {
            key.trim().to_lowercase().strip_prefix("ssid").is_some_and(|rest| {
                let rest = rest.trim();
                !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
            })
        })
        .map(|(_, value)| value.trim().to_owned())
        .filter(|name| !name.is_empty())
        .collect();
    names.sort();
    names.dedup();
    Ok(names)
}

fn last_wifi_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(LAST_WIFI_FILE_NAME)
}

fn load_last_wifi() -> Option<String> {
    let name = fs::read_to_string(last_wifi_path()).ok()?;
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

fn save_last_wifi(ssid: &str) -> StepResult<()> {
    fs::write(last_wifi_path(), ssid)?;
    Ok(())
}

fn choose_wifi(known: &[String]) -> StepResult<String> {
    if known.is_empty() {
        return Err("no known Wi-Fi network is currently available".into());
    }
    let default_index = load_last_wifi().and_then(|last| known.iter().position(|k| *k == last));
    for (i, name) in known.iter().enumerate() {
        let marker = if Some(i) == default_index { " (last)" } else { "" };
        println!("  {}. {name}{marker}", i + 1);
    }
    match default_index {
        Some(i) => print!("Select Wi-Fi [1-{}, Enter for {}]: ", known.len(), known[i]),
        None => print!("Select Wi-Fi [1-{}]: ", known.len()),
    }
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if input.is_empty() {
        return match default_index {
            Some(i) => Ok(known[i].clone()),
            None => Err("no default Wi-Fi, please choose one".into()),
        };
    }
    let index: usize = input.parse()?;
    index
        .checked_sub(1)
        .and_then(|i| known.get(i))
        .cloned()
        .ok_or_else(|| "invalid Wi-Fi selection".into())
}

fn connect_wifi(ssid: &str) -> StepResult<()> {
    run_netsh(&["wlan", "connect", &format!("name={ssid}")])?;
    wait_for_wifi(ssid)
}

fn upload_update(archive: &Path, hash: &str) -> StepResult<String> {
    let body = fs::read(archive)?;
    let mut stream = TcpStream::connect((UPDATE_HOST, UPDATE_PORT))?;
    stream.set_read_timeout(Some(UPLOAD_TIMEOUT))?;
    stream.set_write_timeout(Some(UPLOAD_TIMEOUT))?;
    let request = format!(
        "POST /update HTTP/1.1\r\nHost: {UPDATE_HOST}:{UPDATE_PORT}\r\nContent-Type: application/gzip\r\nContent-Length: {}\r\nx-sha256: {hash}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(request.as_bytes())?;
    stream.write_all(&body)?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    let (head, payload) = response.split_once("\r\n\r\n").ok_or("malformed response")?;
    let status = head.lines().next().unwrap_or_default();
    if status.split_whitespace().nth(1) == Some("200") {
        Ok(payload.trim().to_owned())
    } else {
        Err(format!("{status}: {}", payload.trim()).into())
    }
}

fn run() -> StepResult<()> {
    let paths = Paths::from_workspace();

    println!("Cleaning previous build outputs...");
    clean_build_outputs(&paths)?;

    println!("Building frontend and server...");
    build_server(&paths)?;

    println!("Packaging artifacts...");
    let archive = package_artifacts(&paths)?;

    println!("Generating hash...");
    let hash = generate_hash(&archive)?;
    let hash_file = write_hash_file(&archive, &hash)?;

    println!("Verifying integrity...");
    if !verify_integrity(&archive, &hash_file)? {
        return Err("integrity verification failed".into());
    }

    println!("Update written to {}", archive.display());

    wait_for_enter()?;
    let known = list_known_wifis()?;
    let visible = list_visible_wifis()?;
    let available: Vec<String> = known.into_iter().filter(|name| visible.contains(name)).collect();
    let ssid = choose_wifi(&available)?;
    save_last_wifi(&ssid)?;
    println!("Connecting to Wi-Fi {ssid}...");
    connect_wifi(&ssid)?;
    println!("Uploading update to {UPDATE_HOST}:{UPDATE_PORT}...");
    let response = upload_update(&archive, &hash)?;
    println!("Update service: {response}");
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Build failed: {err}");
        std::process::exit(1);
    }
}
