use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::error::{Result, WireBoxError};

/// Windows runtime components IK Multimedia's installers commonly expect.
/// Best-effort: run this before Product Manager's setup wizard, since
/// missing these is a common (and confusing) cause of installers failing
/// partway through with no clear error.
const BASE_VERBS: &[&str] = &["corefonts", "vcrun2019"];

/// The winetricks verb that registers WineASIO - a low-latency ASIO
/// driver DLL - into a prefix. This is what lets a Wine application see a
/// real, low-latency audio device (via PipeWire's JACK compatibility
/// layer) instead of falling back to Wine's default audio path.
const ASIO_VERB: &str = "wineasio";

/// Installs `BASE_VERBS` into `prefix`. Safe to call repeatedly -
/// winetricks skips verbs that are already installed. Not fatal to the
/// overall install flow if this fails; callers should log a warning and
/// continue rather than aborting, since the target app may still install
/// fine without it.
pub fn ensure_base_dependencies(prefix: &Path) -> Result<()> {
    run_winetricks(prefix, BASE_VERBS)
}

/// Registers WineASIO in `prefix`. This is the audio-bridge step, meant
/// to be triggered explicitly (e.g. from a "Set Up Audio" button) once
/// the target application is actually installed, not bundled silently
/// into the install flow.
pub fn ensure_asio_bridge(prefix: &Path) -> Result<()> {
    run_winetricks(prefix, &[ASIO_VERB])
}

fn run_winetricks(prefix: &Path, verbs: &[&str]) -> Result<()> {
    let winetricks = winetricks_path()?;

    let status = Command::new(&winetricks)
        .env("WINEPREFIX", prefix)
        .env("WINEARCH", "win64")
        .arg("-q") // unattended - don't pop up winetricks' own picker UI
        .args(verbs)
        .status()
        .map_err(|source| WireBoxError::Spawn {
            command: "winetricks".to_string(),
            source,
        })?;

    if !status.success() {
        return Err(WireBoxError::NonZeroExit {
            command: format!("winetricks {}", verbs.join(" ")),
            status,
        });
    }

    Ok(())
}

fn winetricks_path() -> Result<PathBuf> {
    which("winetricks").ok_or(WireBoxError::WinetricksNotInstalled)
}

fn which(binary: &str) -> Option<PathBuf> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {binary}"))
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    (!path.is_empty()).then(|| PathBuf::from(path))
}
