use slint::SharedString;

use super::types::{StatusLevel, StatusMessage};
use crate::MainWindow;

/// Helper function to set status message with semantic level
pub fn set_status(window: &MainWindow, text: impl Into<SharedString>, level: StatusLevel) {
    window.set_status(StatusMessage {
        text: text.into(),
        level,
    });
}

/// Helper function to show confirmation dialog
pub fn show_confirmation(window: &MainWindow, message: impl Into<SharedString>) {
    window.set_confirmation_message(message.into());
    window.set_show_confirmation_dialog(true);
}

/// Helper function to hide confirmation dialog
pub fn hide_confirmation(window: &MainWindow) {
    window.set_show_confirmation_dialog(false);
}

/// Helper function to show error dialog
pub fn show_error(
    window: &MainWindow,
    title: impl Into<SharedString>,
    message: impl Into<SharedString>,
    details: impl Into<SharedString>,
) {
    window.set_error_title(title.into());
    window.set_error_message(message.into());
    window.set_error_details(details.into());
    window.set_show_error_dialog(true);
}

/// Helper function to hide error dialog
pub fn hide_error(window: &MainWindow) {
    window.set_show_error_dialog(false);
}

/// Helper function to show simple confirmation dialog
pub fn show_simple_confirmation(
    window: &MainWindow,
    title: impl Into<SharedString>,
    message: impl Into<SharedString>,
    button_text: impl Into<SharedString>,
) {
    window.set_simple_confirmation_title(title.into());
    window.set_simple_confirmation_message(message.into());
    window.set_simple_confirmation_button(button_text.into());
    window.set_show_simple_confirmation(true);
}

/// Helper function to hide simple confirmation dialog
pub fn hide_simple_confirmation(window: &MainWindow) {
    window.set_show_simple_confirmation(false);
}
