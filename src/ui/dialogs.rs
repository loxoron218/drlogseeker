use gtk4::{ButtonsType::Ok, DialogFlags, MessageDialog, MessageType::Error};
use libadwaita::ApplicationWindow;
use libadwaita::prelude::{DialogExt, GtkWindowExt, WidgetExt};

// Opens file in system's default application, showing error dialog on failure
pub fn show_error_dialog(window: &ApplicationWindow, message: &str) {
    let dialog = MessageDialog::new(
        Some(window),
        DialogFlags::MODAL,
        Error,
        Ok,
        message
    );

    // Auto-close dialog on response to prevent memory leaks
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.show();
}
