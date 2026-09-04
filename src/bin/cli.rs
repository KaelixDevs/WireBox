//! `wirebox-cli` - a debug/dev binary that exercises the engine (`wirebox`,
//! see `lib.rs`) directly from a terminal, no GTK involved. The actual
//! app is the `wirebox` binary (`src/main.rs`); this stays around because
//! it's still the fastest way to sanity-check Wine/install/launch changes.

use wirebox::{Application, Library};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("status") => status(),
        Some("install") => install(),
        Some("launch") => launch(args.get(2)),
        Some("audio") => audio(),
        Some("reset") => reset(),
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("WireBox — engine preview (no GUI yet)");
    println!();
    println!("USAGE:");
    println!("    wirebox-cli status");
    println!("    wirebox-cli install                    (downloads/opens IK Product Manager)");
    println!("    wirebox-cli launch <tonex|amplitube5>");
    println!("    wirebox-cli audio                       (registers WineASIO for low-latency audio)");
    println!("    wirebox-cli reset                       (wipes and recreates the shared prefix)");
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

    println!("Library root:    {}", library.root().display());
    println!("Shared prefix:   {}", library.prefix_path().display());
    println!(
        "Product Manager: {}",
        if library.product_manager_installed() { "installed" } else { "not installed" }
    );
    println!(
        "PipeWire:        {}",
        if wirebox::audio::is_pipewire_active() { "active" } else { "not detected" }
    );
    println!();

    for state in library.all_states() {
        match state.executable {
            Some(executable) => println!("{:<12} installed — {}", state.application.name(), executable.display()),
            None => println!("{:<12} not installed", state.application.name()),
        }
    }
}

fn install() {
    let library = Library::new();

    if let Err(error) = library.ensure_ready() {
        eprintln!("Failed to prepare WireBox's storage: {error}");
        return;
    }

    println!("Setting up IK Product Manager — downloading it if needed, this can take a moment...");

    match library.install_product_manager() {
        Ok(()) => {
            println!("IK Product Manager is open. Log in, then install TONEX and/or AmpliTube 5 from there.");
            println!("Run `wirebox-cli status` afterward to check whether WireBox found them.");
        }
        Err(error) => eprintln!("Setup failed: {error}"),
    }
}

fn launch(app: Option<&String>) {
    let Some(application) = parse_application(app) else {
        eprintln!("Usage: wirebox-cli launch <tonex|amplitube5>");
        return;
    };

    let library = Library::new();

    match library.launch(application) {
        Ok(()) => println!("{} launched.", application.name()),
        Err(error) => eprintln!("Launch failed: {error}"),
    }
}

fn audio() {
    let library = Library::new();

    println!("Registering WineASIO — this needs winetricks installed and can take a moment...");

    match library.set_up_audio() {
        Ok(()) => println!("Done. Apps in the shared prefix should now see a low-latency ASIO audio device."),
        Err(error) => eprintln!("Audio setup failed: {error}"),
    }
}

fn reset() {
    let library = Library::new();

    match library.reset() {
        Ok(()) => println!("The shared prefix has been wiped and recreated. Everything will need reinstalling."),
        Err(error) => eprintln!("Reset failed: {error}"),
    }
}
