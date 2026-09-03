use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    bootstrap,
    catalog::Application,
    dependencies,
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

    /// Makes sure IK Product Manager is installed inside `application`'s
    /// dedicated prefix, then opens it. No installer file needed from the
    /// user - WireBox downloads Product Manager itself (the one piece of
    /// IK Multimedia software available without an account) and runs it.
    ///
    /// This does **not** install the target application by itself: the
    /// user still has to log into their own IK Multimedia account and
    /// click install inside Product Manager's window, which WireBox can't
    /// (and shouldn't) automate. Poll `state()` afterward - e.g. on a
    /// timer - to notice once that finishes.
    ///
    /// Blocks while Product Manager's own setup wizard runs (the first
    /// time only), so call this from a background thread in any UI
    /// context.
    pub fn install(&self, application: Application) -> Result<()> {
        let path = self.prefix_path(application);

        fs::create_dir_all(&path).map_err(|source| WireBoxError::CreateDir {
            path: path.clone(),
            source,
        })?;

        let wine = Wine::detect()?;
        let prefix = wine.prefix(path);

        prefix.ensure_initialized()?;

        // Best-effort: a lot of IK Multimedia's installers quietly expect
        // these to already be present. If winetricks isn't installed, or
        // this step fails for some other reason, don't abort the whole
        // install over it - the target app may still work fine without
        // it, and the alternative (hard-failing here) would block every
        // install on a component that's genuinely optional in many cases.
        if let Err(error) = dependencies::ensure_base_dependencies(prefix.path()) {
            eprintln!(
                "Warning: couldn't install base Windows runtime components ({error}). Continuing anyway - some installers may fail or behave oddly without them."
            );
        }

        // Already installed in this prefix from a previous attempt - just
        // reopen it instead of downloading/running the setup again.
        if let Some(product_manager) = find_by_substring(prefix.path(), "product manager") {
            prefix.spawn_app(&product_manager)?;
            return Ok(());
        }

        let setup = bootstrap::cached_installer()?;

        prefix.run_to_completion(&setup)?;

        let product_manager = find_by_substring(prefix.path(), "product manager")
            .ok_or(WireBoxError::ProductManagerMissing)?;

        prefix.spawn_app(&product_manager)?;

        Ok(())
    }

    /// Registers WineASIO inside `application`'s prefix so it can see a
    /// real, low-latency audio device. Meant to be triggered explicitly
    /// once the application is actually installed - not part of
    /// `install()` itself, since it's specifically about audio quality
    /// rather than getting the app to run at all. Blocks on winetricks,
    /// so call this from a background thread in any UI context.
    pub fn set_up_audio(&self, application: Application) -> Result<()> {
        let wine = Wine::detect()?;
        let prefix = wine.prefix(self.prefix_path(application));

        prefix.ensure_initialized()?;

        dependencies::ensure_asio_bridge(prefix.path())
    }

    /// Deletes and recreates `application`'s prefix from scratch. This is
    /// the blunt-instrument fix for a prefix that's gotten into a bad
    /// state (corrupted registry, half-finished install, etc.) - it wipes
    /// everything installed for that application, including TONEX or
    /// AmpliTube itself, which will need reinstalling afterward.
    pub fn reset(&self, application: Application) -> Result<()> {
        let path = self.prefix_path(application);

        if path.is_dir() {
            fs::remove_dir_all(&path).map_err(|source| WireBoxError::RemoveDir {
                path: path.clone(),
                source,
            })?;
        }

        fs::create_dir_all(&path).map_err(|source| WireBoxError::CreateDir { path, source })?;

        Ok(())
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

    search(&drive_c, 0, &|name| {
        candidates.iter().any(|candidate| name.eq_ignore_ascii_case(candidate))
    })
}

fn find_by_substring(prefix: &Path, needle: &str) -> Option<PathBuf> {
    let drive_c = prefix.join("drive_c");

    if !drive_c.is_dir() {
        return None;
    }

    let needle = needle.to_ascii_lowercase();

    search(&drive_c, 0, &|name| {
        let name = name.to_ascii_lowercase();
        name.ends_with(".exe") && name.contains(&needle)
    })
}

fn search(directory: &Path, depth: usize, matches: &dyn Fn(&str) -> bool) -> Option<PathBuf> {
    if depth > 8 {
        return None;
    }

    let entries = fs::read_dir(directory).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                if matches(name) {
                    return Some(path);
                }
            }
        } else if path.is_dir() {
            let Some(dir_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            // Wine's internal Windows system directory holds tens of
            // thousands of files and never contains what we're looking
            // for - skip it so searches stay fast.
            if dir_name.eq_ignore_ascii_case("windows") {
                continue;
            }

            if let Some(found) = search(&path, depth + 1, matches) {
                return Some(found);
            }
        }
    }

    None
}
