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
}

pub type Result<T> = std::result::Result<T, WireBoxError>;
