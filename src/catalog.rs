/// The Windows applications WireBox knows how to run. Pure data - no
/// filesystem or Wine logic lives here, that's `library`'s job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Application {
    Tonex,
    Amplitube5,
}

impl Application {
    pub const ALL: [Application; 2] = [Application::Tonex, Application::Amplitube5];

    /// Human-readable name for UI/log output.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Tonex => "TONEX",
            Self::Amplitube5 => "AmpliTube 5",
        }
    }

    /// Filesystem-safe identifier, also used as the CLI argument.
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Tonex => "tonex",
            Self::Amplitube5 => "amplitube5",
        }
    }

    /// Filenames WireBox looks for once an install finishes. Matched
    /// case-insensitively, since Wine/NTFS-style installs are inconsistent
    /// about casing.
    pub fn executable_names(&self) -> &'static [&'static str] {
        match self {
            Self::Tonex => &["TONEX.exe", "TONEXApp.exe"],
            Self::Amplitube5 => &["AmpliTube 5.exe", "AmpliTube.exe", "AmpliTube5.exe"],
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|app| app.slug() == slug)
    }
}
