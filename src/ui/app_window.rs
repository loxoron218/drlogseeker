use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use glib::clone;
use gtk4::{Adjustment, Box, Button, Label, Orientation::Vertical, ProgressBar, ScrolledWindow, Viewport};
use gtk4::Align::{Fill, Start};
use gtk4::PolicyType::{Automatic, Never};
use libadwaita::{Application, ApplicationWindow, HeaderBar};
use libadwaita::prelude::{AdwApplicationWindowExt, BoxExt, ButtonExt, GtkWindowExt, WidgetExt};

use crate::data::app_state::AppState;
use crate::ui::column_view::create_column_view;
use crate::ui::header_bar::setup_button_actions;
use crate::ui::settings_dialog::show_settings_dialog;
use crate::utils::event_handlers::{setup_keyboard_controls, setup_mouse_controls};

/// Builds the main application window and all its UI components.
///
/// This function sets up the `ApplicationWindow`, `HeaderBar`, `ColumnView` for results,
/// and all associated buttons and event handlers. It initializes the shared application state
/// and connects all the pieces together.
pub fn build_ui(app: &Application) {

    // Create the main application window.
    let window = ApplicationWindow::new(app);
    window.set_title(Some("drlogseeker"));
    window.set_icon_name(Some("com.loxoron218.drlogseeker"));
    window.set_resizable(true);
    window.set_default_size(1000, 600);

    // Create the header bar with action buttons.
    let header_bar = HeaderBar::new();
    let open_button = Button::from_icon_name("list-add-symbolic");
    open_button.set_tooltip_text(Some("Select Directory"));    
    let scan_button = Button::from_icon_name("view-refresh-symbolic");
    scan_button.set_tooltip_text(Some("Scan Files"));
    let clear_button = Button::from_icon_name("process-stop-symbolic");
    clear_button.set_tooltip_text(Some("Clear List"));
    
    // Add a settings button with a gear icon.
    let settings_button = Button::from_icon_name("open-menu-symbolic");
    settings_button.set_tooltip_text(Some("Settings"));
    
    // Buttons start disabled until a directory is selected.
    scan_button.set_sensitive(false);
    clear_button.set_sensitive(false);
    scan_button.add_css_class("suggested-action");

    // Pack buttons into the header bar.
    header_bar.pack_start(&open_button);
    header_bar.pack_start(&clear_button);
    header_bar.pack_end(&settings_button);
    header_bar.pack_end(&scan_button);

    // Create the main vertical layout.
    let vbox = Box::new(Vertical, 0);
    vbox.append(&header_bar);

    // Create the progress bar, initially hidden.
    let progress_bar = ProgressBar::new();
    progress_bar.set_visible(false);
    vbox.append(&progress_bar);

    // Create a scrollable area for the results view.
    let scrolled = ScrolledWindow::new();
    scrolled.set_hexpand(true);
    scrolled.set_vexpand(true);
    scrolled.set_policy(Never, Automatic);
    scrolled.set_propagate_natural_height(true);
    scrolled.set_valign(Fill);
    scrolled.set_size_request(-1, 400); // Set a minimum height for the scrolled window.

    // Create the column view for displaying results.
    let (column_view, list_store, selection_model) = create_column_view();
    let viewport = Viewport::new(None::<&Adjustment>, None::<&Adjustment>);
    viewport.set_hexpand(true);
    viewport.set_vexpand(true);
    viewport.set_valign(Fill);
    viewport.set_child(Some(&column_view));
    scrolled.set_child(Some(&viewport));
    vbox.append(&scrolled);

    // Create a label to display the file count.
    let file_count_label = Label::new(Some("Files: 0"));
    file_count_label.set_halign(Start);
    file_count_label.set_margin_start(10);
    file_count_label.set_margin_end(10);
    file_count_label.set_margin_bottom(5);
    vbox.append(&file_count_label);
    window.set_content(Some(&vbox));

    // Initialize the shared application state, protected by a Mutex for thread safety.
    let app_state = Arc::new(Mutex::new(AppState { 
        results: Vec::new(),
        delete_files: false,  // Default to not deleting files.
        delete_folders: false, // Default to not deleting folders.
    }));
    let selected_path = Arc::new(Mutex::new(None::<PathBuf>));

    // Set up event handlers for keyboard, mouse, and button clicks.
    setup_keyboard_controls(&window, &selection_model, &list_store, &app_state);
    setup_mouse_controls(&column_view, &window, &selection_model);
    setup_button_actions(&window, &open_button, &scan_button, &clear_button, &selected_path, &list_store, &app_state, &progress_bar, &file_count_label);

    // Connect the settings button to show the settings dialog.
    settings_button.connect_clicked(clone!(@weak window, @strong app_state => move |_| {
        show_settings_dialog(&window, &app_state);
    }));

    // Present the window to the user.
    window.present();
}
