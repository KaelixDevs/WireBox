use std::{
    cell::RefCell,
    process::Command,
    rc::Rc,
};

use pipewire::{context::Context, main_loop::MainLoop, types::ObjectType};

use crate::error::{Result, WireBoxError};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioDirection {
    /// A sink - something audio plays out to (speakers, headphones, an
    /// interface's output).
    Output,
    /// A source - something audio comes in from (a mic, an interface's
    /// input).
    Input,
}

#[derive(Debug, Clone)]
pub struct AudioDevice {
    /// PipeWire's internal node name (stable-ish, used as an identifier).
    pub name: String,
    /// The human-readable name PipeWire's UI tools show for this device.
    pub description: String,
    pub direction: AudioDirection,
}

/// Connects to PipeWire just long enough to collect its current list of
/// audio sink/source nodes, then disconnects.
///
/// NOTE: this is the least-verified function in the whole codebase - it's
/// built directly from pipewire-rs's documented API shape (MainLoop /
/// Context / Core / Registry, with `Core::sync` + the registry's `done`
/// event as the documented way to know its initial burst of objects has
/// finished sending), but there's no way to compile-check it without a
/// working PipeWire dev environment. If `cargo check` flags something
/// here, that's expected - send the exact error and it'll get fixed
/// against the real compiler output instead of more guessing.
///
/// Blocking - call this from a background thread in any UI context.
pub fn list_audio_devices() -> Result<Vec<AudioDevice>> {
    let mainloop = MainLoop::new(None).map_err(pipewire_error)?;
    let context = Context::new(&mainloop).map_err(pipewire_error)?;
    let core = context.connect(None).map_err(pipewire_error)?;
    let registry = core.get_registry().map_err(pipewire_error)?;

    let devices: Rc<RefCell<Vec<AudioDevice>>> = Rc::new(RefCell::new(Vec::new()));
    let devices_for_listener = Rc::clone(&devices);

    let _registry_listener = registry
        .add_listener_local()
        .global(move |global| {
            if global.type_ != ObjectType::Node {
                return;
            }

            let Some(props) = global.props else {
                return;
            };

            let media_class = props.get("media.class").unwrap_or_default();

            let direction = if media_class.contains("Audio/Sink") {
                AudioDirection::Output
            } else if media_class.contains("Audio/Source") {
                AudioDirection::Input
            } else {
                // Not an audio sink/source node (could be a video node,
                // a monitor port, a module, etc.) - not something a
                // WineASIO setup would route to, so skip it.
                return;
            };

            let name = props.get("node.name").unwrap_or("(unnamed)").to_string();
            let description = props.get("node.description").unwrap_or(&name).to_string();

            devices_for_listener.borrow_mut().push(AudioDevice {
                name,
                description,
                direction,
            });
        })
        .register();

    // PipeWire sends its current globals in a burst right after the
    // registry is created. `sync()` asks the server to confirm once
    // everything up to this point has been processed; the matching
    // `done` event is the documented signal that the burst is over, so
    // that's what we use to know it's safe to stop the loop and return.
    let pending = core.sync(0).map_err(pipewire_error)?;

    let quit_loop = mainloop.clone();

    let _core_listener = core
        .add_listener_local()
        .done(move |id, seq| {
            if id == pipewire::core::PW_ID_CORE && seq == pending {
                quit_loop.quit();
            }
        })
        .register();

    mainloop.run();

    let devices = Rc::try_unwrap(devices)
        .map(RefCell::into_inner)
        .unwrap_or_default();

    Ok(devices)
}

fn pipewire_error(error: impl std::fmt::Display) -> WireBoxError {
    WireBoxError::PipeWire(error.to_string())
}
