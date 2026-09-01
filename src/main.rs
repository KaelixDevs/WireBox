//! WireBox's actual application entry point (GTK4 + libadwaita). Thin on
//! purpose - `mod app` wires up the `adw::Application`, `mod ui` builds
//! the window, and all real logic lives in the `wirebox` library crate.

mod app;
mod ui;

fn main() {
    app::run();
}
