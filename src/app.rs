use adw::Application;
use adw::prelude::*;

use crate::ui::window::build_window;

const APP_ID: &str = "io.wirebox.WireBox";

pub fn run() {
    let application = Application::builder().application_id(APP_ID).build();

    application.connect_activate(|app| {
        build_window(app);
    });

    application.run();
}
