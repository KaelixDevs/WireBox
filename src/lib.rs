//! WireBox's core engine: Wine prefix management, the application catalog,
//! and install/launch orchestration. Deliberately has no GUI dependency -
//! `main.rs` exposes it as a CLI for now, and the eventual GTK front end
//! will depend on this crate the same way.

pub mod catalog;
pub mod error;
pub mod library;
pub mod wine;

pub use catalog::Application;
pub use error::{Result, WireBoxError};
pub use library::{AppState, Library};
pub use wine::{Wine, WinePrefix};
