use std::process::Command;

/// Whether the system's audio server identifies itself as PipeWire, which
/// is what the WineASIO bridge (see `dependencies::ensure_asio_bridge`)
/// actually routes audio through via its JACK compatibility layer.
///
/// This is a best-effort system check via `pactl info`, not a guarantee -
/// it doesn't confirm `pipewire-jack` specifically is installed (that
/// varies more by distro packaging), just that PipeWire is the active
/// audio server at all.
pub fn is_pipewire_active() -> bool {
    Command::new("pactl")
        .arg("info")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).contains("PipeWire"))
        .unwrap_or(false)
}
