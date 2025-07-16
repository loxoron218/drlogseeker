use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use gio::ListStore;
use glib::clone;
use gtk4::{Button, FileChooserAction::SelectFolder, FileChooserDialog, ProgressBar};
use gtk4::ResponseType::{Accept, Cancel};
use libadwaita::ApplicationWindow;
use libadwaita::prelude::{ButtonExt, DialogExt, FileChooserExt, FileExt, GtkWindowExt, ListModelExt, WidgetExt};

use crate::data::app_state::{AppState, DRResult};
use crate::dr_analyzer::analyzer::{scan_directory, update_ui};
use crate::file_manager::file_ops::find_log_files;
use crate::ui::dialogs::show_error_dialog;

/// Connects the primary header bar buttons to their respective actions and manages their state.
///
/// This function orchestrates the main user workflow:
/// 1.  **Open Button**: Triggers a `FileChooserDialog` to select a directory. Upon selection,
///     it finds all `.log` and `.txt` files, populates the `list_store` with initial
///     (unscanned) results, and enables the "Scan" and "Clear" buttons.
/// 2.  **Scan Button**: Initiates the asynchronous analysis of the files in the selected directory.
///     It disables itself and the "Clear" button during the scan and shows the progress bar.
/// 3.  **Clear Button**: Removes all items from the `list_store` and clears the internal
///     application state, resetting the UI to its initial state.
///
/// The sensitivity of the "Scan" and "Clear" buttons is automatically managed based on whether
/// the `list_store` contains any items.
pub fn setup_button_actions(window: &ApplicationWindow, open_button: &Button, 
                       scan_button: &Button, clear_button: &Button, 
                       selected_path: &Arc<Mutex<Option<PathBuf>>>, list_store: &ListStore, 
                       app_state: &Arc<Mutex<AppState>>, progress_bar: &ProgressBar) {
    
    // Automatically update button sensitivity when the list store changes.
    list_store.connect_items_changed(clone!(@weak scan_button, @weak clear_button => move |list_store, _, _, _| {
        let has_items = list_store.n_items() > 0;
        clear_button.set_sensitive(has_items);
        scan_button.set_sensitive(has_items);
    }));

    // The "Clear" button resets the application state.
    clear_button.connect_clicked(clone!(@strong list_store, @strong app_state, @strong scan_button, @strong clear_button => move |_| {
        list_store.remove_all();
        if let Ok(mut state) = app_state.lock() {
            state.results.clear();
        }
        scan_button.set_sensitive(false);
        clear_button.set_sensitive(false);
    }));

    // The "Open" button shows a directory selection dialog.
    open_button.connect_clicked(clone!(@strong window, @strong scan_button, @strong clear_button, @strong selected_path, @strong list_store, @strong app_state => move |_| {
        let dialog = FileChooserDialog::new(
            Some("Select Directory"),
            Some(&window),
            SelectFolder,
            &[("Cancel", Cancel), ("Open", Accept)]
        );
        dialog.connect_response(clone!(@strong window, @strong scan_button, @strong clear_button, @strong selected_path, @strong list_store, @strong app_state => move |dialog, response| {
            if response == Accept {
                if let Some(path) = dialog.file().and_then(|f| f.path()) {

                    // Store the selected path and find all log files within it.
                    *selected_path.lock().unwrap() = Some(path.clone());
                    let mut files = Vec::new();
                    if let Err(err) = find_log_files(&path, &mut files) {
                        show_error_dialog(&window, &format!("Error reading directory: {}", err));
                        return;
                    }

                    // Create an initial list of results with a "pending" state.
                    let initial_results: Vec<DRResult> = files.iter().map(|path: &PathBuf| DRResult {
                        filename: path.file_name().unwrap().to_string_lossy().into_owned(),
                        path: path.clone(),
                        dr_value: None,
                        scanned: false, // Mark as unscanned initially.
                    }).collect();

                    // Update the application state and the UI.
                    if let Ok(mut state) = app_state.lock() {
                        state.results = initial_results;
                        update_ui(&list_store, &state.results);
                        let has_items = !state.results.is_empty();
                        scan_button.set_sensitive(has_items);
                        clear_button.set_sensitive(has_items);
                    }
                }
            }
            dialog.close();
        }));
        dialog.show();
    }));

    // The "Scan" button initiates the DR value analysis.
    scan_button.connect_clicked(clone!(@strong app_state, @strong progress_bar, @strong list_store, @strong selected_path, @strong clear_button => move |button| {
        if let Some(path) = selected_path.lock().unwrap().clone() {

            // Disable buttons and show progress bar during scan.
            button.set_sensitive(false);
            clear_button.set_sensitive(false);
            progress_bar.set_visible(true);
            progress_bar.set_fraction(0.0);
            
            // Start the asynchronous scan.
            scan_directory(path, app_state.clone(), progress_bar.clone(), list_store.clone(), button.clone(), clear_button.clone());
        }
    }));
}
