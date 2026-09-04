use std::{
    sync::{mpsc, Arc},
    time::Duration,
};

use adw::{
    prelude::*, AboutDialog, ActionRow, Application, ApplicationWindow, Clamp, HeaderBar,
    PreferencesGroup, StatusPage, ToastOverlay, ToolbarView, WindowTitle,
};

use gtk::{glib, Align, Box, Button, Image, Orientation, Separator, Spinner};

use wirebox::{Application as WireApp, Library};

const REPO_URL: &str = "https://github.com/KaelixDevs/WireBox";

pub fn build_window(app: &Application) {
    let library = Arc::new(Library::new());

    if let Err(error) = library.ensure_ready() {
        eprintln!("Failed to prepare WireBox's storage: {error}");
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("WireBox")
        .default_width(760)
        .default_height(680)
        .build();

    let toast_overlay = ToastOverlay::new();

    // ---------------------------------------------------------
    // Header bar
    // ---------------------------------------------------------

    let header = HeaderBar::new();

    let title = WindowTitle::new("WireBox", "TONEX + AmpliTube on Linux");
    header.set_title_widget(Some(&title));

    let about_button = Button::from_icon_name("help-about-symbolic");
    about_button.set_tooltip_text(Some("About WireBox"));

    let window_for_about = window.clone();
    about_button.connect_clicked(move |_| show_about(&window_for_about));

    header.pack_end(&about_button);

    // ---------------------------------------------------------
    // Content
    // ---------------------------------------------------------

    let content = Box::new(Orientation::Vertical, 24);
    content.set_margin_top(24);
    content.set_margin_bottom(32);
    content.set_margin_start(16);
    content.set_margin_end(16);

    let intro = StatusPage::builder()
        .title("Your Rig")
        .description("TONEX and AmpliTube 5, running natively on your Linux desktop")
        .icon_name("audio-x-generic-symbolic")
        .build();
    intro.set_vexpand(false);

    content.append(&intro);

    // ---------------------------------------------------------
    // IK Product Manager - the one shared gateway to installing
    // either (or both) application.
    // ---------------------------------------------------------

    let product_manager_group = PreferencesGroup::builder()
        .title("IK Product Manager")
        .description("Your IK Multimedia account, shared by both applications below")
        .build();

    let product_manager_row = build_product_manager_row(Arc::clone(&library), &toast_overlay);
    product_manager_group.add(&product_manager_row);
    content.append(&product_manager_group);

    // ---------------------------------------------------------
    // Applications - install through Product Manager above, then
    // launch from here once WireBox detects them.
    // ---------------------------------------------------------

    let applications = PreferencesGroup::builder()
        .title("Applications")
        .description("Installed and launched from the same shared Wine environment")
        .build();

    let tonex_row = build_launch_row(&library, &toast_overlay, WireApp::Tonex, "audio-input-microphone-symbolic");
    applications.add(&tonex_row.row);

    let amplitube_row = build_launch_row(&library, &toast_overlay, WireApp::Amplitube5, "audio-speakers-symbolic");
    applications.add(&amplitube_row.row);

    content.append(&applications);
    content.append(&Separator::new(Orientation::Horizontal));

    // A single periodic check keeps both Launch rows in sync with
    // whatever the user does inside Product Manager's own window,
    // without needing bespoke polling wired to any one click.
    start_periodic_refresh(Arc::clone(&library), vec![tonex_row, amplitube_row]);

    // ---------------------------------------------------------
    // Runtime
    // ---------------------------------------------------------

    let runtime = PreferencesGroup::builder()
        .title("Runtime")
        .description("The compatibility layer WireBox runs everything through")
        .build();

    let wine_row = ActionRow::builder().title("Wine").build();
    wine_row.add_prefix(&Image::from_icon_name("application-x-executable-symbolic"));

    match wirebox::Wine::detect() {
        Ok(wine) => {
            wine_row.set_subtitle(&format!("Available — {}", wine.executable().display()));
        }
        Err(error) => wine_row.set_subtitle(&format!("Not available — {error}")),
    }

    runtime.add(&wine_row);

    let pipewire_row = ActionRow::builder().title("PipeWire").build();
    pipewire_row.add_prefix(&Image::from_icon_name("audio-volume-high-symbolic"));
    pipewire_row.set_subtitle(if wirebox::audio::is_pipewire_active() {
        "Active — WineASIO audio setup is available"
    } else {
        "Not detected — WineASIO audio setup will likely fail"
    });
    runtime.add(&pipewire_row);

    let audio_setup_row = build_audio_setup_row(Arc::clone(&library), &toast_overlay);
    runtime.add(&audio_setup_row);

    let library_row = ActionRow::builder().title("Library Location").build();
    library_row.add_prefix(&Image::from_icon_name("folder-symbolic"));
    library_row.set_subtitle(&library.root().display().to_string());
    runtime.add(&library_row);

    content.append(&runtime);

    let audio_devices = PreferencesGroup::builder()
        .title("Audio Devices")
        .description("What PipeWire currently sees on this system")
        .build();

    let audio_placeholder = ActionRow::builder().title("Scanning for devices…").build();
    audio_devices.add(&audio_placeholder);
    content.append(&audio_devices);

    load_audio_devices(&audio_devices, audio_placeholder);

    // ---------------------------------------------------------
    // Assemble
    // ---------------------------------------------------------

    let clamp = Clamp::builder().maximum_size(640).child(&content).build();

    let scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&clamp)
        .vexpand(true)
        .build();

    let toolbar_view = ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&scroller));

    toast_overlay.set_child(Some(&toolbar_view));

    window.set_content(Some(&toast_overlay));
    window.present();
}

fn show_about(window: &ApplicationWindow) {
    let about = AboutDialog::builder()
        .application_name("WireBox")
        .application_icon("audio-x-generic")
        .version("0.1.0")
        .developer_name("KaelixDevs")
        .comments("Run IK Multimedia TONEX and AmpliTube 5 natively on Linux, through one shared Wine environment.")
        .website(REPO_URL)
        .issue_url(&format!("{REPO_URL}/issues"))
        .build();

    about.present(Some(window));
}

// ===========================================================
// IK Product Manager
// ===========================================================

fn build_product_manager_row(library: Arc<Library>, toast_overlay: &ToastOverlay) -> ActionRow {
    let row = ActionRow::builder().title("IK Product Manager").build();
    row.add_prefix(&Image::from_icon_name("system-software-install-symbolic"));

    let spinner = Spinner::new();
    spinner.set_visible(false);
    row.add_suffix(&spinner);

    let button = Button::new();
    button.set_valign(Align::Center);
    row.add_suffix(&button);

    refresh_product_manager_row(&library, &row, &button);

    let toast_overlay = toast_overlay.clone();
    let row_for_click = row.clone();

    button.connect_clicked(move |button| {
        start_product_manager_setup(
            Arc::clone(&library),
            &toast_overlay,
            row_for_click.clone(),
            button.clone(),
            spinner.clone(),
        );
    });

    row
}

fn refresh_product_manager_row(library: &Library, row: &ActionRow, button: &Button) {
    if library.product_manager_installed() {
        button.set_label("Open IK Product Manager");
        row.set_subtitle("Installed — log in there to install TONEX or AmpliTube 5");
    } else {
        button.set_label("Install IK Product Manager");
        row.set_subtitle("Not installed — WireBox downloads this automatically, no file needed");
    }
}

/// Downloads (first time only) and opens IK Product Manager, on a
/// background thread since the download/first-run setup blocks. Once
/// this returns successfully, Product Manager's own window is open and
/// ready for the user to log in - installing TONEX/AmpliTube 5 from
/// there is picked up by `start_periodic_refresh`, not by this function.
fn start_product_manager_setup(
    library: Arc<Library>,
    toast_overlay: &ToastOverlay,
    row: ActionRow,
    button: Button,
    spinner: Spinner,
) {
    button.set_sensitive(false);
    button.set_label("Setting up…");
    spinner.set_visible(true);
    spinner.start();
    row.set_subtitle("Downloading IK Product Manager (first time only)…");

    let (sender, receiver) = mpsc::channel::<wirebox::Result<()>>();

    let install_library = Arc::clone(&library);

    std::thread::spawn(move || {
        let result = install_library.install_product_manager();
        let _ = sender.send(result);
    });

    let toast_overlay = toast_overlay.clone();

    glib::timeout_add_local(Duration::from_millis(150), move || {
        match receiver.try_recv() {
            Ok(Ok(())) => {
                spinner.stop();
                spinner.set_visible(false);
                button.set_sensitive(true);
                refresh_product_manager_row(&library, &row, &button);
                show_toast(&toast_overlay, "IK Product Manager is open");

                glib::ControlFlow::Break
            }

            Ok(Err(error)) => {
                spinner.stop();
                spinner.set_visible(false);
                button.set_sensitive(true);
                refresh_product_manager_row(&library, &row, &button);
                row.set_subtitle(&format!("Setup failed — {error}"));
                show_toast(&toast_overlay, &format!("Couldn't set up Product Manager: {error}"));

                glib::ControlFlow::Break
            }

            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,

            Err(mpsc::TryRecvError::Disconnected) => {
                spinner.stop();
                spinner.set_visible(false);
                button.set_sensitive(true);
                row.set_subtitle("Setup thread ended unexpectedly.");

                glib::ControlFlow::Break
            }
        }
    });
}

// ===========================================================
// Per-application Launch rows
// ===========================================================

struct LaunchRow {
    application: WireApp,
    row: ActionRow,
    button: Button,
}

fn build_launch_row(
    library: &Arc<Library>,
    toast_overlay: &ToastOverlay,
    application: WireApp,
    icon_name: &str,
) -> LaunchRow {
    let row = ActionRow::builder().title(application.name()).build();
    row.add_prefix(&Image::from_icon_name(icon_name));

    let button = Button::with_label("Launch");
    button.set_valign(Align::Center);
    row.add_suffix(&button);

    refresh_launch_row(library, application, &row, &button);

    let library = Arc::clone(library);
    let toast_overlay = toast_overlay.clone();
    let row_for_click = row.clone();

    button.connect_clicked(move |_| {
        match library.launch(application) {
            Ok(()) => show_toast(&toast_overlay, &format!("{} launched", application.name())),
            Err(error) => {
                row_for_click.set_subtitle(&format!("Launch failed — {error}"));
                show_toast(&toast_overlay, &format!("Couldn't launch {}: {error}", application.name()));
            }
        }
    });

    LaunchRow { application, row, button }
}

fn refresh_launch_row(library: &Library, application: WireApp, row: &ActionRow, button: &Button) {
    let state = library.state(application);

    match state.executable {
        Some(executable) => {
            row.set_subtitle(&format!("Installed — {}", executable.display()));
            button.set_sensitive(true);
        }
        None => {
            row.set_subtitle("Not installed yet — install it from IK Product Manager above");
            button.set_sensitive(false);
        }
    }
}

/// Keeps every Launch row's enabled state and subtitle in sync with
/// whatever the user does inside Product Manager's own window. WireBox
/// has no event to hook for "an install finished in some other app's
/// UI," so this just checks disk every couple of seconds for as long as
/// the window is open - cheap enough that a fixed poll is simpler and
/// more robust than trying to wire this to any specific button click.
fn start_periodic_refresh(library: Arc<Library>, rows: Vec<LaunchRow>) {
    glib::timeout_add_local(Duration::from_secs(2), move || {
        for row in &rows {
            refresh_launch_row(&library, row.application, &row.row, &row.button);
        }

        glib::ControlFlow::Continue
    });
}

// ===========================================================
// Audio setup (hub-wide, not per-application)
// ===========================================================

fn build_audio_setup_row(library: Arc<Library>, toast_overlay: &ToastOverlay) -> ActionRow {
    let row = ActionRow::builder().title("Low-Latency Audio").build();
    row.add_prefix(&Image::from_icon_name("audio-card-symbolic"));
    row.set_subtitle("Registers WineASIO via winetricks, for use once an app is installed");

    let spinner = Spinner::new();
    spinner.set_visible(false);
    row.add_suffix(&spinner);

    let button = Button::with_label("Set Up Audio");
    button.set_valign(Align::Center);
    row.add_suffix(&button);

    let toast_overlay = toast_overlay.clone();
    let row_for_click = row.clone();

    button.connect_clicked(move |button| {
        start_audio_setup(
            Arc::clone(&library),
            &toast_overlay,
            row_for_click.clone(),
            button.clone(),
            spinner.clone(),
        );
    });

    row
}

/// Registers WineASIO in the shared hub prefix via `winetricks`, on a
/// background thread so the UI stays responsive while it downloads and
/// builds the driver (can take a while the first time).
fn start_audio_setup(library: Arc<Library>, toast_overlay: &ToastOverlay, row: ActionRow, button: Button, spinner: Spinner) {
    button.set_sensitive(false);
    spinner.set_visible(true);
    spinner.start();
    row.set_subtitle("Setting up low-latency audio (WineASIO via winetricks)…");

    let (sender, receiver) = mpsc::channel::<wirebox::Result<()>>();

    let audio_library = Arc::clone(&library);

    std::thread::spawn(move || {
        let result = audio_library.set_up_audio();
        let _ = sender.send(result);
    });

    let toast_overlay = toast_overlay.clone();

    glib::timeout_add_local(Duration::from_millis(150), move || {
        match receiver.try_recv() {
            Ok(Ok(())) => {
                spinner.stop();
                spinner.set_visible(false);
                button.set_sensitive(true);
                row.set_subtitle("WineASIO is registered");
                show_toast(&toast_overlay, "Audio set up");

                glib::ControlFlow::Break
            }

            Ok(Err(error)) => {
                spinner.stop();
                spinner.set_visible(false);
                button.set_sensitive(true);
                row.set_subtitle("Registers WineASIO via winetricks, for use once an app is installed");
                show_toast(&toast_overlay, &format!("Audio setup failed: {error}"));

                glib::ControlFlow::Break
            }

            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,

            Err(mpsc::TryRecvError::Disconnected) => {
                spinner.stop();
                spinner.set_visible(false);
                button.set_sensitive(true);
                show_toast(&toast_overlay, "Audio setup thread ended unexpectedly.");

                glib::ControlFlow::Break
            }
        }
    });
}

fn show_toast(overlay: &ToastOverlay, message: &str) {
    let toast = adw::Toast::builder().title(message).timeout(4).build();
    overlay.add_toast(toast);
}

// ===========================================================
// Audio device list
// ===========================================================

/// Scans for PipeWire audio devices on a background thread and replaces
/// `placeholder` with one row per device once done. Each row lets the
/// user mark it as their preferred input/output in `Config` - this is a
/// saved *preference* for WireBox to remember, not live PipeWire routing;
/// actually moving audio between devices is still qpwgraph/helvum/your
/// system's sound settings, same as for any other app.
fn load_audio_devices(group: &PreferencesGroup, placeholder: ActionRow) {
    let (sender, receiver) = mpsc::channel::<wirebox::Result<Vec<wirebox::audio::AudioDevice>>>();

    std::thread::spawn(move || {
        let result = wirebox::audio::list_audio_devices();
        let _ = sender.send(result);
    });

    let group = group.clone();

    glib::timeout_add_local(Duration::from_millis(150), move || {
        match receiver.try_recv() {
            Ok(Ok(devices)) => {
                group.remove(&placeholder);

                if devices.is_empty() {
                    let row = ActionRow::builder()
                        .title("No devices found")
                        .subtitle("PipeWire returned an empty list")
                        .build();
                    group.add(&row);
                } else {
                    let config = wirebox::Config::load().unwrap_or_default();

                    for device in devices {
                        add_device_row(&group, device, &config);
                    }
                }

                glib::ControlFlow::Break
            }

            Ok(Err(error)) => {
                placeholder.set_title("Couldn't scan for audio devices");
                placeholder.set_subtitle(&error.to_string());

                glib::ControlFlow::Break
            }

            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,

            Err(mpsc::TryRecvError::Disconnected) => {
                placeholder.set_title("Audio scan thread ended unexpectedly");

                glib::ControlFlow::Break
            }
        }
    });
}

fn add_device_row(group: &PreferencesGroup, device: wirebox::audio::AudioDevice, config: &wirebox::Config) {
    use wirebox::audio::AudioDirection;

    let (icon_name, kind_label, is_preferred) = match device.direction {
        AudioDirection::Output => (
            "audio-speakers-symbolic",
            "Output",
            config.preferred_output_device.as_deref() == Some(device.name.as_str()),
        ),
        AudioDirection::Input => (
            "audio-input-microphone-symbolic",
            "Input",
            config.preferred_input_device.as_deref() == Some(device.name.as_str()),
        ),
    };

    let row = ActionRow::builder()
        .title(device.description.clone())
        .subtitle(format!("{kind_label} — {}", device.name))
        .build();

    row.add_prefix(&Image::from_icon_name(icon_name));

    let button = Button::with_label(if is_preferred { "Preferred" } else { "Set as preferred" });
    button.set_valign(Align::Center);
    button.set_sensitive(!is_preferred);
    row.add_suffix(&button);

    let device_name = device.name.clone();
    let direction = device.direction;

    button.connect_clicked(move |button| {
        let Ok(mut config) = wirebox::Config::load() else {
            return;
        };

        match direction {
            AudioDirection::Output => config.preferred_output_device = Some(device_name.clone()),
            AudioDirection::Input => config.preferred_input_device = Some(device_name.clone()),
        }

        if config.save().is_ok() {
            button.set_label("Preferred");
            button.set_sensitive(false);
        }
    });

    group.add(&row);
}
