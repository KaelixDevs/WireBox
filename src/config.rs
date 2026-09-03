use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, WireBoxError};

/// User-adjustable settings that aren't tied to any one application.
/// Stored as TOML at `~/.config/wirebox/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// ASIO buffer size, in samples. Lower = less latency, more risk of
    /// audio dropouts on marginal hardware.
    pub buffer_size: u32,
    pub sample_rate: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            buffer_size: 128,
            sample_rate: 48_000,
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wirebox")
            .join("config.toml")
    }

    /// Loads the config, or returns defaults if none has been saved yet.
    pub fn load() -> Result<Self> {
        let path = Self::path();

        if !path.is_file() {
            return Ok(Self::default());
        }

        let text = fs::read_to_string(&path).map_err(|source| WireBoxError::ConfigRead {
            path: path.clone(),
            source,
        })?;

        toml::from_str(&text).map_err(|source| WireBoxError::ConfigParse { path, source })
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| WireBoxError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let text = toml::to_string_pretty(self).map_err(|source| WireBoxError::ConfigSerialize { source })?;

        fs::write(&path, text).map_err(|source| WireBoxError::ConfigWrite { path, source })
    }
}
