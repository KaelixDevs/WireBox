//! `wirebox-cli` - a debug/dev binary that exercises the engine (`wirebox`,
//! see `lib.rs`) directly from a terminal, no GTK involved. The actual
//! app is the `wirebox` binary (`src/main.rs`); this stays around because
//! it's still the fastest way to sanity-check Wine/install/launch changes.

use wirebox::{Application, Library};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("status") => status(),
        Some("install") => install(args.get(2)),
        Some("launch") => launch(args.get(2)),
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("WireBox — engine preview (no GUI yet)");
    println!();
    println!("USAGE:");
    println!("    wirebox status");
    println!("    wirebox install <tonex|amplitube5>");
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

fn install(app: Option<&String>) {
    let Some(application) = parse_application(app) else {
        eprintln!("Usage: wirebox install <tonex|amplitube5>");
        return;
    };

    let library = Library::new();

    if let Err(error) = library.ensure_ready() {
        eprintln!("Failed to prepare WireBox's storage: {error}");
        return;
    }

    println!(
        "Setting up {} — downloading IK Product Manager if needed, this can take a moment...",
        application.name()
    );

    match library.install(application) {
        Ok(()) => {
            println!(
                "IK Product Manager is open. Log in, then install {} from there.",
                application.name()
            );
            println!("Run `wirebox status` afterward to check whether WireBox found it.");
        }
        Err(error) => eprintln!("Setup failed: {error}"),
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
