//! Engine-preview CLI. No GUI yet on purpose - this exists so the Wine
//! prefix / install / launch logic in `wirebox` (see `lib.rs`) can be
//! exercised and trusted before any UI gets built on top of it.

use std::path::PathBuf;

use wirebox::{Application, Library};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("status") => status(),
        Some("install") => install(args.get(2), args.get(3)),
        Some("launch") => launch(args.get(2)),
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("WireBox — engine preview (no GUI yet)");
    println!();
    println!("USAGE:");
    println!("    wirebox status");
    println!("    wirebox install <tonex|amplitube5> <path-to-installer.exe>");
    println!("    wirebox launch  <tonex|amplitube5>");
}

fn parse_application(arg: Option<&String>) -> Option<Application> {
    Application::from_slug(arg?.as_str())
}

fn status() {
    let library = Library::new();

    if let Err(error) = library.ensure_ready() {
        eprintln!("Failed to prepare WireBox's storage: {error}");
        return;
    }

    println!("Library root: {}", library.root().display());
    println!();

    for state in library.all_states() {
        match state.executable {
            Some(executable) => {
                println!("{:<12} installed — {}", state.application.name(), executable.display());
            }
            None => {
                println!(
                    "{:<12} not installed (prefix: {})",
                    state.application.name(),
                    state.prefix.display()
                );
            }
        }
    }
}

fn install(app: Option<&String>, installer: Option<&String>) {
    let (Some(application), Some(installer)) = (parse_application(app), installer) else {
        eprintln!("Usage: wirebox install <tonex|amplitube5> <path-to-installer.exe>");
        return;
    };

    let library = Library::new();

    if let Err(error) = library.ensure_ready() {
        eprintln!("Failed to prepare WireBox's storage: {error}");
        return;
    }

    println!(
        "Installing {} — this runs the installer's own window inside Wine, so watch for it to pop up.",
        application.name()
    );

    match library.install(application, &PathBuf::from(installer)) {
        Ok(state) => {
            let executable = state.executable.expect("install() guarantees this is Some");
            println!("Installed — {}", executable.display());
        }
        Err(error) => eprintln!("Install failed: {error}"),
    }
}

fn launch(app: Option<&String>) {
    let Some(application) = parse_application(app) else {
        eprintln!("Usage: wirebox launch <tonex|amplitube5>");
        return;
    };

    let library = Library::new();

    match library.launch(application) {
        Ok(()) => println!("{} launched.", application.name()),
        Err(error) => eprintln!("Launch failed: {error}"),
    }
}
