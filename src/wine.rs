use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus},
};

use crate::error::{Result, WireBoxError};

/// The system's Wine installation. Detecting Wine and running things
/// inside a prefix are separate concerns - this type only knows "where is
/// the wine binary."
#[derive(Debug, Clone)]
pub struct Wine {
    executable: PathBuf,
}

impl Wine {
    /// Looks for `wine` or `wine64` on `$PATH`.
    pub fn detect() -> Result<Self> {
        for candidate in ["wine", "wine64"] {
            if let Some(executable) = which(candidate) {
                return Ok(Self { executable });
            }
        }

        Err(WireBoxError::WineNotInstalled)
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    /// Opens a handle to an isolated prefix at `path`. Doesn't touch disk -
    /// call `ensure_initialized()` on the result before using it.
    pub fn prefix(&self, path: PathBuf) -> WinePrefix {
        WinePrefix {
            wine: self.executable.clone(),
            path,
        }
    }
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

/// A single, isolated `WINEPREFIX` - the sandboxed "Windows install" a
/// compatible application runs inside. Every application in the catalog
/// gets its own prefix so they can never share (or corrupt) each other's
/// registry, DLL overrides, or installed state.
#[derive(Debug, Clone)]
pub struct WinePrefix {
    wine: PathBuf,
    path: PathBuf,
}

impl WinePrefix {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_initialized(&self) -> bool {
        self.path.join("drive_c").is_dir()
    }

    pub fn ensure_initialized(&self) -> Result<()> {
        if self.is_initialized() {
            return Ok(());
        }

        fs::create_dir_all(&self.path).map_err(|source| WireBoxError::CreateDir {
            path: self.path.clone(),
            source,
        })?;

        let status = self
            .command()
            .arg("wineboot")
            .status()
            .map_err(|source| WireBoxError::Spawn {
                command: "wineboot".to_string(),
                source,
            })?;

        check_status("wineboot", status)
    }

    /// Opens Wine's configuration UI for this prefix (audio/graphics/etc).
    pub fn winecfg(&self) -> Result<()> {
        self.ensure_initialized()?;

        self.command()
            .arg("winecfg")
            .spawn()
            .map_err(|source| WireBoxError::Spawn {
                command: "winecfg".to_string(),
                source,
            })?;

        Ok(())
    }

    /// Launches an executable inside this prefix and returns immediately.
    /// Use this for running the actual application once it's installed.
    pub fn spawn_app(&self, executable: &Path) -> Result<Child> {
        self.require_file(executable)?;
        self.ensure_initialized()?;

        self.command()
            .arg(executable)
            .spawn()
            .map_err(|source| WireBoxError::Spawn {
                command: executable.display().to_string(),
                source,
            })
    }

    /// Runs an executable inside this prefix and blocks until it exits.
    /// Meant for installers, which are interactive and need to finish
    /// before WireBox goes looking for the result. Call this from a
    /// background thread in any UI context.
    pub fn run_to_completion(&self, executable: &Path) -> Result<()> {
        self.require_file(executable)?;
        self.ensure_initialized()?;

        let status = self
            .command()
            .arg(executable)
            .status()
            .map_err(|source| WireBoxError::Spawn {
                command: executable.display().to_string(),
                source,
            })?;

        check_status(&executable.display().to_string(), status)
    }

    fn require_file(&self, executable: &Path) -> Result<()> {
        if executable.is_file() {
            Ok(())
        } else {
            Err(WireBoxError::NotFound(executable.to_path_buf()))
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.wine);

        command
            .env("WINEPREFIX", &self.path)
            .env("WINEARCH", "win64")
            .env("WINEDLLOVERRIDES", "winemenubuilder.exe=d");

        command
    }
}

fn check_status(command: &str, status: ExitStatus) -> Result<()> {
    if status.success() {
        Ok(())
    } else {
        Err(WireBoxError::NonZeroExit {
            command: command.to_string(),
            status,
        })
    }
}
