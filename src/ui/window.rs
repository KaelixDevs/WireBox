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
        .default_height(640)
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

    let applications = PreferencesGroup::builder()
        .title("Applications")
        .description("Each app runs in its own isolated Wine environment")
        .build();

    let tonex_row = build_app_row(
        &window,
        Arc::clone(&library),
        &toast_overlay,
        WireApp::Tonex,
        "audio-input-microphone-symbolic",
    );
    applications.add(&tonex_row);

    let amplitube_row = build_app_row(
        &window,
        Arc::clone(&library),
        &toast_overlay,
        WireApp::Amplitube5,
        "audio-speakers-symbolic",
    );
    applications.add(&amplitube_row);

    content.append(&applications);
    content.append(&Separator::new(Orientation::Horizontal));

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
        .comments("Run IK Multimedia TONEX and AmpliTube 5 natively on Linux, via an isolated Wine environment per application.")
        .website(REPO_URL)
        .issue_url(&format!("{REPO_URL}/issues"))
        .build();

    about.present(Some(window));
}

fn build_app_row(
    window: &ApplicationWindow,
    library: Arc<Library>,
    toast_overlay: &ToastOverlay,
    application: WireApp,
    icon_name: &str,
) -> ActionRow {
    let row = ActionRow::builder().title(application.name()).build();
    row.add_prefix(&Image::from_icon_name(icon_name));

    let spinner = Spinner::new();
    spinner.set_visible(false);
    row.add_suffix(&spinner);

    let button = Button::new();
    button.set_valign(Align::Center);
    row.add_suffix(&button);

    let audio_button = Button::from_icon_name("audio-volume-high-symbolic");
    audio_button.set_valign(Align::Center);
    audio_button.set_tooltip_text(Some("Set up low-latency audio (WineASIO)"));
    audio_button.set_visible(false);
    row.add_suffix(&audio_button);

    refresh_row(&library, application, &row, &button, &audio_button);

    let window = window.clone();
    let toast_overlay = toast_overlay.clone();

    let row_for_click = row.clone();
    let button_for_click = button.clone();
    let spinner_for_click = spinner.clone();
    let audio_button_for_install = audio_button.clone();

    let audio_library = Arc::clone(&library);
    let audio_toast_overlay = toast_overlay.clone();
    let audio_row = row.clone();

    audio_button.connect_clicked(move |button| {
        start_audio_setup(
            Arc::clone(&audio_library),
            &audio_toast_overlay,
            application,
            audio_row.clone(),
            button.clone(),
        );
    });

    button.connect_clicked(move |_| {
        let state = library.state(application);

        if state.is_installed() {
            match library.launch(application) {
                Ok(()) => {
                    show_toast(&toast_overlay, &format!("{} launched", application.name()));
                }
                Err(error) => {
                    row_for_click.set_subtitle(&format!("Launch failed — {error}"));
                    show_toast(&toast_overlay, &format!("Couldn't launch {}: {error}", application.name()));
                }
            }

            return;
        }

        start_setup(
            &window,
            Arc::clone(&library),
            &toast_overlay,
            application,
            row_for_click.clone(),
            button_for_click.clone(),
            spinner_for_click.clone(),
            audio_button_for_install.clone(),
        );
    });

    row
}

/// Kicks off the Product Manager bootstrap on a background thread (it
/// blocks the first time, while PM's own setup wizard runs), then hands
/// off to `poll_for_completion` once Product Manager's window is open.
fn start_setup(
    _window: &ApplicationWindow,
    library: Arc<Library>,
    toast_overlay: &ToastOverlay,
    application: WireApp,
    row: ActionRow,
    button: Button,
    spinner: Spinner,
    audio_button: Button,
) {
    button.set_sensitive(false);
    button.set_label("Setting up…");
    spinner.set_visible(true);
    spinner.start();
    row.set_subtitle("Downloading IK Product Manager (first time only)…");

    let (sender, receiver) = mpsc::channel::<wirebox::Result<()>>();

    let install_library = Arc::clone(&library);

    std::thread::spawn(move || {
        let result = install_library.install(application);
        let _ = sender.send(result);
    });

    let toast_overlay = toast_overlay.clone();

    glib::timeout_add_local(Duration::from_millis(150), move || {
        match receiver.try_recv() {
            Ok(Ok(())) => {
                row.set_subtitle(&format!(
                    "IK Product Manager is open — log in and install {} from there.",
                    application.name()
                ));
                button.set_label("Waiting for install…");

                poll_for_completion(
                    Arc::clone(&library),
                    toast_overlay.clone(),
                    application,
                    row.clone(),
                    button.clone(),
                    spinner.clone(),
                    audio_button.clone(),
                );

                glib::ControlFlow::Break
            }

            Ok(Err(error)) => {
                spinner.stop();
                spinner.set_visible(false);
                button.set_sensitive(true);
                button.set_label(&format!("Install {}", application.name()));
                row.set_subtitle(&format!("Setup failed — {error}"));
                show_toast(&toast_overlay, &format!("Couldn't set up {}: {error}", application.name()));

                glib::ControlFlow::Break
            }

            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,

            Err(mpsc::TryRecvError::Disconnected) => {
                spinner.stop();
                spinner.set_visible(false);
                button.set_sensitive(true);
                button.set_label(&format!("Install {}", application.name()));
                row.set_subtitle("Setup thread ended unexpectedly.");

                glib::ControlFlow::Break
            }
        }
    });
}

/// After Product Manager is open, WireBox has no way to know when the
/// user finishes logging in and clicking install inside it - so it just
/// checks disk periodically until the application's executable shows up.
fn poll_for_completion(
    library: Arc<Library>,
    toast_overlay: ToastOverlay,
    application: WireApp,
    row: ActionRow,
    button: Button,
    spinner: Spinner,
    audio_button: Button,
) {
    glib::timeout_add_local(Duration::from_secs(2), move || {
        let state = library.state(application);

        if let Some(executable) = state.executable {
            spinner.stop();
            spinner.set_visible(false);
            button.set_sensitive(true);
            button.set_label(&format!("Launch {}", application.name()));
            row.set_subtitle(&format!("Installed — {}", executable.display()));
            audio_button.set_visible(true);
            show_toast(&toast_overlay, &format!("{} is ready", application.name()));

            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

fn refresh_row(
    library: &Library,
    application: WireApp,
    row: &ActionRow,
    button: &Button,
    audio_button: &Button,
) {
    let state = library.state(application);

    match state.executable {
        Some(executable) => {
            button.set_label(&format!("Launch {}", application.name()));
            row.set_subtitle(&format!("Installed — {}", executable.display()));
            audio_button.set_visible(true);
        }
        None => {
            button.set_label(&format!("Install {}", application.name()));
            row.set_subtitle("Not installed");
            audio_button.set_visible(false);
        }
    }
}

fn show_toast(overlay: &ToastOverlay, message: &str) {
    let toast = adw::Toast::builder().title(message).timeout(4).build();
    overlay.add_toast(toast);
}

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

/// Registers WineASIO inside `application`'s prefix via `winetricks`, on
/// a background thread so the UI stays responsive while it downloads and
/// builds the driver (can take a while the first time).
fn start_audio_setup(
    library: Arc<Library>,
    toast_overlay: &ToastOverlay,
    application: WireApp,
    row: ActionRow,
    audio_button: Button,
) {
    audio_button.set_sensitive(false);

    // Rebuild the "installed" subtitle ourselves rather than reading it
    // back off the widget - GTK's string-property getters vary between
    // returning `GString` and `Option<GString>` across binding versions,
    // and we already know the state that produced it (this button is
    // only visible once `state.executable` is `Some`).
    let installed_subtitle = library
        .state(application)
        .executable
        .map(|executable| format!("Installed — {}", executable.display()))
        .unwrap_or_else(|| "Installed".to_string());

    row.set_subtitle("Setting up low-latency audio (WineASIO via winetricks)…");

    let (sender, receiver) = mpsc::channel::<wirebox::Result<()>>();

    let audio_library = Arc::clone(&library);

    std::thread::spawn(move || {
        let result = audio_library.set_up_audio(application);
        let _ = sender.send(result);
    });

    let toast_overlay = toast_overlay.clone();

    glib::timeout_add_local(Duration::from_millis(150), move || {
        match receiver.try_recv() {
            Ok(Ok(())) => {
                audio_button.set_sensitive(true);
                row.set_subtitle(&format!("{installed_subtitle} — WineASIO is registered"));
                show_toast(&toast_overlay, &format!("Audio set up for {}", application.name()));

                glib::ControlFlow::Break
            }

            Ok(Err(error)) => {
                audio_button.set_sensitive(true);
                row.set_subtitle(&installed_subtitle);
                show_toast(&toast_overlay, &format!("Audio setup failed: {error}"));

                glib::ControlFlow::Break
            }

            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,

            Err(mpsc::TryRecvError::Disconnected) => {
                audio_button.set_sensitive(true);
                show_toast(&toast_overlay, "Audio setup thread ended unexpectedly.");

                glib::ControlFlow::Break
            }
        }
    });
}

