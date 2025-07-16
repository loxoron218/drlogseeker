use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gio::ListStore;
use glib::clone;
use glib::Propagation::{Proceed, Stop};
use gtk4::{ColumnView, EventControllerKey, GestureClick, MultiSelection, PropagationPhase::Capture, StringObject};
use gtk4::gdk::{Key, ModifierType};
use libadwaita::{ApplicationWindow};
use libadwaita::prelude::{Cast, EventControllerExt, GestureSingleExt, ListModelExt, SelectionModelExt, WidgetExt};

use crate::data::app_state::AppState;
use crate::file_manager::file_ops::{delete_selected_files, try_open_file};

/// Sets up global keyboard shortcuts for the application window.
///
/// This function attaches an `EventControllerKey` to the main window to handle
/// the following actions:
/// - **Ctrl+A**: Selects all items in the `ColumnView`.
/// - **Delete**: Deletes the selected items, potentially from the filesystem based on settings.
/// - **Enter**: Opens the selected file(s) using the system's default application.
///
/// The controller operates in the `Capture` phase to ensure these shortcuts are
/// handled before any other widget-specific key events.
pub fn setup_keyboard_controls(window: &ApplicationWindow, selection_model: &MultiSelection, list_store: &ListStore, app_state: &Arc<Mutex<AppState>>) {
    let key_controller = EventControllerKey::new();
    key_controller.set_propagation_phase(Capture);
    window.add_controller(key_controller.clone());
    key_controller.connect_key_pressed(clone!(@weak window, @weak selection_model, @weak list_store, @weak app_state => 
        @default-return Proceed, move |_controller, key, _keycode, modifier_state| {
            match key {

                // Ctrl+A: Select all items.
                Key::a | Key::A if modifier_state.bits() & ModifierType::CONTROL_MASK.bits() != 0 => {
                    selection_model.unselect_all();
                    for i in 0..selection_model.n_items() {
                        selection_model.select_item(i, false);
                    }
                    Stop // Stop propagation to prevent other widgets from handling it.
                }

                // Delete: Remove selected items.
                Key::Delete => {
                    delete_selected_files(&window, &selection_model, &list_store, &app_state);
                    Stop
                }

                // Enter: Open selected files.
                Key::Return | Key::KP_Enter | Key::ISO_Enter => {
                    let selected_indices: Vec<u32> = (0..selection_model.n_items())
                        .filter(|&i| selection_model.is_selected(i))
                        .collect();
                    if !selected_indices.is_empty() {
                        for index in selected_indices {
                            if let Some(item) = selection_model.item(index) {
                                if let Some(string_obj) = item.downcast_ref::<StringObject>() {
                                    let path = PathBuf::from(string_obj.string().split('\t').nth(1).unwrap_or(""));
                                    try_open_file(&window, &path);
                                }
                            }
                        }
                        Stop
                    } else {
                        Proceed // Proceed if no items are selected.
                    }
                }
                _ => Proceed, // Allow other keys to be handled normally.
            }
    }));
}

/// Sets up mouse controls for the `ColumnView`, specifically for handling double-clicks.
///
/// This function attaches a `GestureClick` controller to the `ColumnView` to detect
/// a double-click with the left mouse button, which triggers opening the selected file.
pub fn setup_mouse_controls(column_view: &ColumnView, window: &ApplicationWindow, selection_model: &MultiSelection) {
    let gesture_click = GestureClick::new();
    gesture_click.set_button(1); // Primary (left) mouse button.
    column_view.add_controller(gesture_click.clone());
    
    // Use the capture phase to handle the event early.
    gesture_click.set_propagation_phase(Capture);
    gesture_click.connect_released(clone!(@weak window, @weak selection_model => move |gesture, n_press, _x, _y| {

        // Check for a double-click (n_press == 2).
        if gesture.current_button() == 1 && n_press == 2 {

            // Find the first selected item and open it.
            if let Some(index) = (0..selection_model.n_items()).find(|&i| selection_model.is_selected(i)) {
                if let Some(item) = selection_model.item(index) {
                    if let Some(string_obj) = item.downcast_ref::<StringObject>() {
                        let path = PathBuf::from(string_obj.string().split('\t').nth(1).unwrap_or(""));
                        try_open_file(&window, &path);
                    }
                }
            }
        }
    }));
}