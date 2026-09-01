use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::error::{Result, WireBoxError};

/// IK Multimedia's public, no-account-required download for their Product
/// Manager app - the one piece of software in this ecosystem that isn't
/// gated behind a login. Product Manager is what actually knows how to
/// register, authorize, and download TONEX/AmpliTube 5 against the user's
/// own account, so WireBox bootstraps it instead of asking the user for
/// an installer file.
const PRODUCT_MANAGER_URL: &str =
    "https://g1.ikmultimedia.com/plugins/ProductManager/ik_product_manager_1.1.12.zip";

/// Downloads (once) and extracts IK Product Manager into a shared cache,
/// returning the path to its setup `.exe`. Safe to call repeatedly - it
/// only hits the network the first time.
pub fn cached_installer() -> Result<PathBuf> {
    let cache_dir = cache_directory();

    fs::create_dir_all(&cache_dir).map_err(|source| WireBoxError::CreateDir {
        path: cache_dir.clone(),
        source,
    })?;

    let archive = cache_dir.join("ik_product_manager.zip");

    if !archive.is_file() {
        download(PRODUCT_MANAGER_URL, &archive)?;
    }

    let extracted = cache_dir.join("extracted");

    if !extracted.is_dir() {
        if let Err(error) = extract(&archive, &extracted) {
            // Don't leave a half-extracted directory around to be
            // mistaken for a good cache on the next run.
            let _ = fs::remove_dir_all(&extracted);
            return Err(error);
        }
    }

    find_exe(&extracted).ok_or_else(|| WireBoxError::NoExecutableFound(extracted))
}

fn cache_directory() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("wirebox")
        .join("product-manager")
}

fn download(url: &str, destination: &Path) -> Result<()> {
    let status = Command::new("curl")
        .args(["-L", "--fail", "-o"])
        .arg(destination)
        .arg(url)
        .status()
        .map_err(|source| WireBoxError::Spawn {
            command: "curl".to_string(),
            source,
        })?;

    if !status.success() {
        let _ = fs::remove_file(destination);

        return Err(WireBoxError::NonZeroExit {
            command: format!("curl -L {url}"),
            status,
        });
    }

    Ok(())
}

fn extract(archive: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).map_err(|source| WireBoxError::CreateDir {
        path: destination.to_path_buf(),
        source,
    })?;

    // bsdtar (from libarchive) reads zip natively and is already a pacman
    // dependency on Arch-based systems, so this doesn't add a new package
    // requirement in practice.
    let status = Command::new("bsdtar")
        .arg("-xf")
        .arg(archive)
        .arg("-C")
        .arg(destination)
        .status()
        .map_err(|source| WireBoxError::Spawn {
            command: "bsdtar".to_string(),
            source,
        })?;

    if !status.success() {
        return Err(WireBoxError::NonZeroExit {
            command: "bsdtar".to_string(),
            status,
        });
    }

    Ok(())
}

fn find_exe(dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() {
            let is_exe = path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));

            if is_exe {
                return Some(path);
            }
        } else if path.is_dir() {
            if let Some(found) = find_exe(&path) {
                return Some(found);
            }
        }
    }

    None
}
