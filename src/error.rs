use std::path::PathBuf;
use std::process::ExitStatus;

use thiserror::Error;

/// Every failure mode the engine can produce. Kept flat and specific on
/// purpose - callers (CLI today, GTK later) can match on variants instead
/// of grepping error strings.
#[derive(Debug, Error)]
pub enum WireBoxError {
    #[error("wine was not found on this system (checked $PATH for `wine` and `wine64`)")]
    WineNotInstalled,

    #[error("failed to run `{command}`: {source}")]
    Spawn {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("`{command}` exited with a non-zero status ({status})")]
    NonZeroExit { command: String, status: ExitStatus },

    #[error("path does not exist: {0}")]
    NotFound(PathBuf),

    #[error("failed to create directory `{path}`: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{application} installer finished, but WireBox couldn't find it afterward")]
    InstallVerificationFailed { application: &'static str },

    #[error("IK Product Manager finished installing, but WireBox couldn't find it inside the prefix afterward")]
    ProductManagerMissing,

    #[error("no .exe found after extracting {0}")]
    NoExecutableFound(PathBuf),

    #[error("failed to remove `{path}`: {source}")]
    RemoveDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("winetricks was not found on this system (needed to set up audio/runtime components)")]
    WinetricksNotInstalled,

    #[error("failed to read config at `{path}`: {source}")]
    ConfigRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse config at `{path}`: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("failed to serialize config: {source}")]
    ConfigSerialize {
        #[source]
        source: toml::ser::Error,
    },

    #[error("failed to write config at `{path}`: {source}")]
    ConfigWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, WireBoxError>;
