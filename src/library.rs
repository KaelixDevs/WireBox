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

/// Where an application currently stands: the executable WireBox found
/// for it (if any) inside the shared hub prefix.
#[derive(Debug, Clone)]
pub struct AppState {
    pub application: Application,
    pub executable: Option<PathBuf>,
}

impl AppState {
    pub fn is_installed(&self) -> bool {
        self.executable.is_some()
    }
}

/// WireBox's Wine environment.
///
/// Earlier versions of this gave every application its own isolated
/// prefix. That fell apart in practice: IK Product Manager is one shared
/// gateway to a single IK Multimedia account, and TONEX/AmpliTube 5 both
/// get installed *through* it - isolating each target app meant running
/// (and logging into) a separate copy of Product Manager per app, which
/// is exactly the redundant, confusing flow this exists to avoid. So this
/// is now one shared prefix: Product Manager lives here once, and
/// whichever apps you install through it land here too. The trade-off is
/// real (TONEX and AmpliTube now share one Wine environment instead of
/// being fully isolated from each other) and is being made deliberately,
/// not quietly.
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

    /// The single Wine prefix everything lives in.
    pub fn prefix_path(&self) -> PathBuf {
        self.root.join("hub")
    }

    pub fn ensure_ready(&self) -> Result<()> {
        let path = self.prefix_path();

        fs::create_dir_all(&path).map_err(|source| WireBoxError::CreateDir { path, source })
    }

    /// Inspects disk to determine whether `application` is installed.
    pub fn state(&self, application: Application) -> AppState {
        let executable = find_executable(&self.prefix_path(), application.executable_names());

        AppState { application, executable }
    }

    pub fn all_states(&self) -> Vec<AppState> {
        Application::ALL.into_iter().map(|app| self.state(app)).collect()
    }

    /// Whether IK Product Manager itself is installed in the hub prefix.
    pub fn product_manager_installed(&self) -> bool {
        find_by_substring(&self.prefix_path(), "product manager").is_some()
    }

    /// Launches an already-installed application. Returns an error if it
    /// isn't installed or Wine isn't available.
    pub fn launch(&self, application: Application) -> Result<()> {
        let state = self.state(application);

        let executable = state
            .executable
            .ok_or_else(|| WireBoxError::NotFound(self.prefix_path()))?;

        let wine = Wine::detect()?;
        let prefix = wine.prefix(self.prefix_path());

        prefix.spawn_app(&executable)?;

        Ok(())
    }

    /// Makes sure IK Product Manager is installed in the shared hub
    /// prefix, then opens it. No installer file needed from the user -
    /// WireBox downloads Product Manager itself (the one piece of IK
    /// Multimedia software available without an account) and runs it.
    ///
    /// This does **not** install TONEX or AmpliTube by itself: the user
    /// still has to log into their own IK Multimedia account and pick
    /// which product(s) to install from inside Product Manager's own
    /// window, which WireBox can't (and shouldn't) automate. Poll
    /// `state()` afterward - e.g. on a timer - to notice once that
    /// finishes for a given application.
    ///
    /// Blocks while Product Manager's own setup wizard runs (the first
    /// time only), so call this from a background thread in any UI
    /// context.
    pub fn install_product_manager(&self) -> Result<()> {
        let path = self.prefix_path();

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

        // Already installed from a previous attempt - just reopen it
        // instead of downloading/running the setup again.
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

    /// Registers WineASIO in the shared hub prefix so anything running
    /// there can see a real, low-latency audio device. Blocks on
    /// winetricks, so call this from a background thread in any UI
    /// context.
    pub fn set_up_audio(&self) -> Result<()> {
        let wine = Wine::detect()?;
        let prefix = wine.prefix(self.prefix_path());

        prefix.ensure_initialized()?;

        dependencies::ensure_asio_bridge(prefix.path())
    }

    /// Deletes and recreates the shared hub prefix from scratch. This is
    /// the blunt-instrument fix for a prefix that's gotten into a bad
    /// state (corrupted registry, half-finished install, etc.) - it wipes
    /// Product Manager, TONEX, and AmpliTube all together, since they now
    /// all live in the same place. Everything will need reinstalling
    /// afterward.
    pub fn reset(&self) -> Result<()> {
        let path = self.prefix_path();

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
