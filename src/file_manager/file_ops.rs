use std::{io::Result, process::Command};
use std::fs::{read_dir, remove_dir, remove_file}; 
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use gio::ListStore;
use glib::clone;
use gtk4::{ButtonsType::YesNo, DialogFlags, MessageDialog, MessageType::Warning, MultiSelection, ResponseType::Yes, StringObject};
use libadwaita::ApplicationWindow;
use libadwaita::prelude::{Cast, DialogExt, GtkWindowExt, ListModelExt, SelectionModelExt, WidgetExt};

use crate::data::app_state::AppState;
use crate::dr_analyzer::analyzer::update_ui;
use crate::ui::dialogs::show_error_dialog;

/// Attempts to open a file using the system's default application (`xdg-open`).
///
/// If the file does not exist or the open command fails, it displays an error dialog.
pub fn try_open_file(window: &ApplicationWindow, path: &Path) {
    if !path.exists() {
        show_error_dialog(window, &format!("File not found: {}", path.display()));
        return;
    }
    if let Err(err) = Command::new("xdg-open").arg(path).spawn() {
        show_error_dialog(window, &format!("Failed to open file: {}", err));
    }
}

/// Recursively finds all files with `.txt` or `.log` extensions in a given directory.
///
/// The discovered file paths are appended to the `files` vector.
pub fn find_log_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            find_log_files(&path, files)?;
        } else if let Some(ext) = path.extension() {
            if ext == "txt" || ext == "log" {
                files.push(path);
            }
        }
    }
    Ok(())
}

/// Removes the selected files from the UI list and, if configured, from the filesystem.
///
/// This function checks the `app_state` to determine if file deletion is enabled.
/// If `delete_files` is true, it shows a confirmation dialog before permanently
/// deleting the files. If `delete_folders` is also true, it will attempt to remove
/// the parent directory of a deleted file if it becomes empty.
///
/// If `delete_files` is false, it only removes the items from the application's internal list.
pub fn delete_selected_files(window: &ApplicationWindow, selection_model: &MultiSelection, 
                        list_store: &ListStore, app_state: &Arc<Mutex<AppState>>) {
    let selected_items: Vec<_> = (0..selection_model.n_items())
        .filter(|&i| selection_model.is_selected(i))
        .collect();
    if selected_items.is_empty() {
        return;
    }
    let mut paths_to_remove = Vec::new();
    if let Ok(_) = app_state.lock() {
        for &index in &selected_items {
            if let Some(item) = selection_model.item(index) {
                if let Some(string_obj) = item.downcast_ref::<StringObject>() {
                    let path = PathBuf::from(string_obj.string().split('\t').nth(1).unwrap_or(""));
                    paths_to_remove.push(path);
                }
            }
        }
    }
    let should_confirm = if let Ok(state) = app_state.lock() {
        state.delete_files
    } else {
        false
    };
    if should_confirm {
        let dialog = MessageDialog::new(
            Some(window),
            DialogFlags::MODAL,
            Warning,
            YesNo,
            &format!("This will permanently delete {} file(s) from your system{} Continue?", 
                paths_to_remove.len(),
                if let Ok(state) = app_state.lock() {
                    if state.delete_folders { " and their parent folders" } else { "" }
                } else { "" }
            )
        );
        if let Some(button) = dialog.widget_for_response(Yes) {
            button.add_css_class("destructive-action");
        }
        let paths_to_remove_clone = paths_to_remove.clone();
        dialog.connect_response(clone!(@strong app_state, @strong list_store => move |dialog, response| {
            if response == Yes {
                if let Ok(mut state) = app_state.lock() {
                    // Delete files from system
                    for path in &paths_to_remove_clone {
                        if let Err(err) = remove_file(path) {
                            eprintln!("Failed to delete file {}: {}", path.display(), err);
                        } else if state.delete_folders {
                            // Try to remove parent folder if it's empty
                            if let Some(parent) = path.parent() {
                                if let Ok(entries) = read_dir(parent) {
                                    if entries.count() == 0 {
                                        if let Err(err) = remove_dir(parent) {
                                            eprintln!("Failed to delete empty folder {}: {}", parent.display(), err);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    state.results.retain(|result| !paths_to_remove_clone.contains(&result.path));
                    update_ui(&list_store, &state.results);
                }
            }
            dialog.close();
        }));
        dialog.show();
    } else {
        if let Ok(mut state) = app_state.lock() {
            state.results.retain(|result| !paths_to_remove.contains(&result.path));
            update_ui(&list_store, &state.results);
        }
    }
}