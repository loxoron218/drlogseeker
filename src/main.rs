use std::{error::Error, result::Result};

use gio::ApplicationFlags;
use libadwaita::{Application, init};
use libadwaita::prelude::{ApplicationExt, ApplicationExtManual};
use tokio::main;

use crate::ui::app_window::build_ui;

mod data;
mod dr_analyzer;
mod file_manager;
mod ui;
mod utils;

/// The main entry point for the application.
///
/// This asynchronous function initializes the Tokio runtime and the Libadwaita/GTK application.
/// It sets up the application with a unique ID, connects the `activate` signal to the
/// `build_ui` function (which creates the main window), and then runs the application's
/// main event loop.
#[main]
async fn main() -> Result<(), Box<dyn Error>> {

    // Initialize Libadwaita, which is required for modern GNOME styling and widgets.
    init().expect("Failed to initialize libadwaita");

    // Create a new Libadwaita application instance.
    let application = Application::new(
        Some("com.loxoron218.drlogseeker"), // A unique application ID.
        ApplicationFlags::FLAGS_NONE,
    );

    // Connect the `activate` signal to the `build_ui` function.
    // This signal is emitted when the application is first launched.
    application.connect_activate(build_ui);

    // Run the application's main event loop.
    application.run();
    Ok(())
}
