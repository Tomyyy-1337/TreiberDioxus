use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Optimization {
    Basic,
    Full,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Platform {
    Windows,
    RaspberryPi,
}

const APP_NAME: &str = "ruder_cam_beta";

const FULL_PROFILE: [(&str, &str); 7] = [
    ("CARGO_PROFILE_RELEASE_OPT_LEVEL", "s"),
    ("CARGO_PROFILE_RELEASE_DEBUG", "false"),
    ("CARGO_PROFILE_RELEASE_PANIC", "abort"),
    ("CARGO_PROFILE_RELEASE_STRIP", "true"),
    ("CARGO_PROFILE_RELEASE_INCREMENTAL", "false"),
    ("CARGO_PROFILE_RELEASE_LTO", "true"),
    ("CARGO_PROFILE_RELEASE_CODEGEN_UNITS", "1"),
];

fn profile_env(optimization: Optimization) -> &'static [(&'static str, &'static str)] {
    match optimization {
        Optimization::Full => &FULL_PROFILE,
        Optimization::Basic => &[],
    }
}

fn run(command: &mut Command) -> io::Result<()> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("{command:?} failed: {status}")))
    }
}

/// Builds the server and its `public` dir into `update_service/build`.
pub fn build(platform: Platform, optimization: Optimization) -> io::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project_dir = root.join("../treiber");
    let out_dir = root.join("build");
    let staging = root.join("build.tmp");

    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;

    match platform {
        Platform::Windows => bundle_windows(&project_dir, &staging, optimization)?,
        Platform::RaspberryPi => bundle_pi(&project_dir, &staging, optimization)?,
    }

    // Only replace the previous build once the new one succeeded.
    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir)?;
    }
    std::fs::rename(&staging, &out_dir)
}

fn bundle_windows(project_dir: &Path, out: &Path, optimization: Optimization) -> io::Result<()> {
    let web_out: PathBuf = project_dir.join("target/dx").join(APP_NAME).join("release/web");
    if web_out.exists() {
        std::fs::remove_dir_all(&web_out)?;
    }

    run(Command::new("dx")
        .args(["bundle", "--release", "--web", "@server", "--server"])
        .envs(profile_env(optimization).iter().copied())
        .current_dir(project_dir))?;

    std::fs::copy(web_out.join("server.exe"), out.join("server.exe"))?;
    copy_dir(&web_out.join("public"), &out.join("public"))
}

// The Pi target needs a Linux cross toolchain, so `dx bundle` runs inside Docker.
fn bundle_pi(project_dir: &Path, out: &Path, optimization: Optimization) -> io::Result<()> {
    let mut command = Command::new("docker");
    command
        .args(["build", "--target", "artifacts", "-o"])
        .arg(format!("type=local,dest={}", out.display()));
    for (key, value) in profile_env(optimization) {
        command.arg("--build-arg").arg(format!("{key}={value}"));
    }
    run(command.arg(".").current_dir(project_dir))
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
