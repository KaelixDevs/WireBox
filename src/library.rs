use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    catalog::Application,
    error::{Result, WireBoxError},
    wine::Wine,
};

/// Where an application currently stands: its dedicated prefix, and the
/// executable WireBox found inside it (if any).
#[derive(Debug, Clone)]
pub struct AppState {
    pub application: Application,
    pub prefix: PathBuf,
    pub executable: Option<PathBuf>,
}

impl AppState {
    pub fn is_installed(&self) -> bool {
        self.executable.is_some()
    }
}

/// WireBox's on-disk library of applications: one isolated Wine prefix per
/// application, all rooted under `~/.local/share/wirebox/applications/`.
#[derive(Debug, Clone)]
pub struct Library {
    root: PathBuf,
}

impl Library {
    pub fn new() -> Self {
        Self { root: default_root() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn prefix_path(&self, application: Application) -> PathBuf {
        self.root.join(application.slug())
    }

    /// Creates the directory for every catalog application. Safe to call
    /// repeatedly (e.g. on every app startup).
    pub fn ensure_ready(&self) -> Result<()> {
        for application in Application::ALL {
            let path = self.prefix_path(application);

            fs::create_dir_all(&path).map_err(|source| WireBoxError::CreateDir { path, source })?;
        }

        Ok(())
    }

    /// Inspects disk to determine whether `application` is installed.
    pub fn state(&self, application: Application) -> AppState {
        let prefix = self.prefix_path(application);
        let executable = find_executable(&prefix, application.executable_names());

        AppState {
            application,
            prefix,
            executable,
        }
    }

    pub fn all_states(&self) -> Vec<AppState> {
        Application::ALL.into_iter().map(|app| self.state(app)).collect()
    }

    /// Launches an already-installed application. Returns an error if it
    /// isn't installed or Wine isn't available.
    pub fn launch(&self, application: Application) -> Result<()> {
        let state = self.state(application);

        let executable = state
            .executable
            .ok_or_else(|| WireBoxError::NotFound(state.prefix.clone()))?;

        let wine = Wine::detect()?;
        let prefix = wine.prefix(state.prefix);

        prefix.spawn_app(&executable)?;

        Ok(())
    }

    /// Runs `installer` inside `application`'s dedicated prefix and blocks
    /// until it exits, then re-checks disk for the result. Installers are
    /// interactive, so call this from a background thread in any UI
    /// context - it will not return until the user finishes clicking
    /// through the installer's own window.
    pub fn install(&self, application: Application, installer: &Path) -> Result<AppState> {
        if !installer.is_file() {
            return Err(WireBoxError::NotFound(installer.to_path_buf()));
        }

        let path = self.prefix_path(application);

        fs::create_dir_all(&path).map_err(|source| WireBoxError::CreateDir { path, source })?;

        let wine = Wine::detect()?;
        let prefix = wine.prefix(self.prefix_path(application));

        prefix.run_to_completion(installer)?;

        let state = self.state(application);

        if state.executable.is_none() {
            return Err(WireBoxError::InstallVerificationFailed {
                application: application.name(),
            });
        }

        Ok(state)
    }
}

impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}

fn default_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local")
        .join("share")
        .join("wirebox")
        .join("applications")
}

fn find_executable(prefix: &Path, candidates: &[&str]) -> Option<PathBuf> {
    let drive_c = prefix.join("drive_c");

    if !drive_c.is_dir() {
        return None;
    }

    search(&drive_c, candidates, 0)
}

fn search(directory: &Path, candidates: &[&str], depth: usize) -> Option<PathBuf> {
    if depth > 8 {
        return None;
    }

    let entries = fs::read_dir(directory).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            if candidates.iter().any(|candidate| name.eq_ignore_ascii_case(candidate)) {
                return Some(path);
            }
        } else if path.is_dir() {
            let Some(dir_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            // Wine's internal Windows system directory holds tens of
            // thousands of files and never contains TONEX/AmpliTube -
            // skip it so installs are found quickly.
            if dir_name.eq_ignore_ascii_case("windows") {
                continue;
            }

            if let Some(found) = search(&path, candidates, depth + 1) {
                return Some(found);
            }
        }
    }

    None
}
