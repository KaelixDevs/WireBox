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

    let library_row = ActionRow::builder().title("Library Location").build();
    library_row.add_prefix(&Image::from_icon_name("folder-symbolic"));
    library_row.set_subtitle(&library.root().display().to_string());
    runtime.add(&library_row);

    content.append(&runtime);

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

    refresh_row(&library, application, &row, &button);

    let window = window.clone();
    let toast_overlay = toast_overlay.clone();

    let row_for_click = row.clone();
    let button_for_click = button.clone();
    let spinner_for_click = spinner.clone();

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
) {
    glib::timeout_add_local(Duration::from_secs(2), move || {
        let state = library.state(application);

        if let Some(executable) = state.executable {
            spinner.stop();
            spinner.set_visible(false);
            button.set_sensitive(true);
            button.set_label(&format!("Launch {}", application.name()));
            row.set_subtitle(&format!("Installed — {}", executable.display()));
            show_toast(&toast_overlay, &format!("{} is ready", application.name()));

            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

fn refresh_row(library: &Library, application: WireApp, row: &ActionRow, button: &Button) {
    let state = library.state(application);

    match state.executable {
        Some(executable) => {
            button.set_label(&format!("Launch {}", application.name()));
            row.set_subtitle(&format!("Installed — {}", executable.display()));
        }
        None => {
            button.set_label(&format!("Install {}", application.name()));
            row.set_subtitle("Not installed");
        }
    }
}

fn show_toast(overlay: &ToastOverlay, message: &str) {
    let toast = adw::Toast::builder().title(message).timeout(4).build();
    overlay.add_toast(toast);
}

