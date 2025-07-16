use std::sync::{Arc, Mutex};

use glib::{clone, idle_add_local_once};
use glib::Propagation::{Proceed, Stop};
use gtk4::{Box, Dialog, Label, Switch};
use gtk4::Orientation::{Horizontal, Vertical};
use libadwaita::{ApplicationWindow};
use libadwaita::prelude::{BoxExt, DialogExt, GtkWindowExt, WidgetExt};

use crate::data::app_state::AppState;

/// Displays a modal dialog for configuring application settings.
///
/// This dialog provides options to control file deletion behavior:
/// 1.  **Delete files from system**: A `Switch` to enable or disable permanent file deletion
///     when items are removed from the list.
/// 2.  **Also delete parent folders**: A dependent `Switch` that, when enabled, also removes
///     a file's parent directory if it becomes empty after the file is deleted. This option
///     is marked as "DANGEROUS" and is only sensitive when the main deletion switch is active.
///
/// The state of the switches is synchronized with the shared `AppState`.
pub fn show_settings_dialog(window: &ApplicationWindow, app_state: &Arc<Mutex<AppState>>) {

    // Create the modal dialog.
    let dialog = Dialog::new();
    dialog.set_title(Some("Settings"));
    dialog.set_transient_for(Some(window));
    dialog.set_modal(true);
    dialog.set_default_size(400, 300);

    // Set up the main content area with padding.
    let content_area = dialog.content_area();
    let vbox = Box::new(Vertical, 10);
    vbox.set_margin_top(10);
    vbox.set_margin_bottom(10);
    vbox.set_margin_start(10);
    vbox.set_margin_end(10);

    // --- "Delete files" setting ---
    let hbox = Box::new(Horizontal, 10);
    let label = Label::new(Some("Delete files from system when removing from list"));
    label.set_hexpand(true);
    label.set_xalign(0.0);
    let switch = Switch::new();
    hbox.append(&label);
    hbox.append(&switch);
    vbox.append(&hbox);

    // --- "Delete parent folders" setting ---
    let folder_hbox = Box::new(Horizontal, 10);
    let folder_label = Label::new(Some("Also delete parent folders (DANGEROUS)"));
    folder_label.set_hexpand(true);
    folder_label.set_xalign(0.0);
    let folder_switch = Switch::new();
    folder_hbox.append(&folder_label);
    folder_hbox.append(&folder_switch);
    vbox.append(&folder_hbox);
    
    // Initialize switch states from the application state.
    if let Ok(state) = app_state.lock() {
        switch.set_active(state.delete_files);
        folder_switch.set_active(state.delete_folders);
        folder_switch.set_sensitive(state.delete_files);
        
        if state.delete_files {
            folder_label.remove_css_class("dim-label");
        } else {
            folder_label.add_css_class("dim-label");
        }
    }

    // Connect the main "Delete files" switch to update state and UI.
    switch.connect_state_set(clone!(@strong app_state, @strong folder_switch, @strong folder_label => move |_, active| {
        if let Ok(mut state) = app_state.lock() {
            state.delete_files = active;
            
            // If the main switch is turned off, also turn off and disable the folder switch.
            if !active {
                state.delete_folders = false;
                
                // Defer UI updates to avoid deadlocks and ensure they run on the main thread.
                idle_add_local_once(clone!(@strong folder_switch, @strong folder_label => move || {
                    folder_switch.set_active(false);
                    folder_switch.set_sensitive(false);
                    folder_label.add_css_class("dim-label");
                }));
            } else {
                // If turned on, just enable the folder switch.
                idle_add_local_once(clone!(@strong folder_switch, @strong folder_label => move || {
                    folder_switch.set_sensitive(true);
                    folder_label.remove_css_class("dim-label");
                }));
            }
        }
        Proceed // Allow the state change to proceed.
    }));

    // Connect the "Delete parent folders" switch.
    folder_switch.connect_state_set(clone!(@strong app_state => move |_, active| {
        if let Ok(mut state) = app_state.lock() {
            // Only allow changing the folder switch if the main delete switch is active.
            if state.delete_files {
                state.delete_folders = active;
                Proceed // Allow the state change.
            } else {
                Stop // Prevent the state change.
            }
        } else {
            Stop // Prevent change if the lock fails.
        }
    }));

    content_area.append(&vbox);
    dialog.show();
}