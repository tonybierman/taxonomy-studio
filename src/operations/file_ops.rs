use slint::{ComponentHandle, SharedString};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use taxstud_core::*;

use crate::errors::{map_file_load_error, map_file_save_error, map_revert_error};
use crate::state::AppState;
use crate::ui::{set_status, show_error, update_ui_from_state};
use crate::{MainWindow, StatusLevel};

/// File operations orchestration
/// Handles all file I/O with proper error handling and UI updates
pub struct FileOperations<'a> {
    state: &'a Rc<RefCell<AppState>>,
    window: &'a MainWindow,
}

impl<'a> FileOperations<'a> {
    pub fn new(state: &'a Rc<RefCell<AppState>>, window: &'a MainWindow) -> Self {
        Self { state, window }
    }

    /// Open file dialog and load the selected taxonomy file
    pub async fn open_file_dialog_and_load(&self) {
        if let Some(file) = rfd::AsyncFileDialog::new()
            .add_filter("JSON", &["json"])
            .set_title("Open Taxonomy File")
            .pick_file()
            .await
        {
            self.load_file(file.path()).await;
        }
    }

    /// Load a taxonomy file from the given path
    pub async fn load_file(&self, path: &Path) {
        // Load the file (borrow mutably, then drop the borrow)
        let load_result = self.state.borrow_mut().load_from_file(path.to_path_buf());

        match load_result {
            Ok(_) => {
                // Update window title (borrow immutably)
                let title = self.state.borrow().get_window_title();
                self.window.set_window_title(SharedString::from(title));

                // Update UI with loaded data (borrow immutably)
                update_ui_from_state(self.window, &self.state.borrow());

                set_status(self.window, "File loaded successfully", StatusLevel::Success);
            }
            Err(e) => {
                // Show enhanced error dialog using error mapper
                let (title, message, details) = map_file_load_error(&*e, path);
                show_error(self.window, title, message, details);
            }
        }
    }

    /// Save the current taxonomy to its current file
    pub fn save(&self) -> Result<(), String> {
        let save_result = self.state.borrow_mut().save();

        match save_result {
            Ok(_) => {
                // Update window title
                let title = self.state.borrow().get_window_title();
                self.window.set_window_title(SharedString::from(title));

                set_status(self.window, "File saved successfully", StatusLevel::Success);
                Ok(())
            }
            Err(e) => {
                let path = self.state.borrow().current_file.clone();
                if let Some(file_path) = path {
                    let (title, message, details) = map_file_save_error(&*e, Some(&file_path));
                    show_error(self.window, title, message, details);
                } else {
                    show_error(
                        self.window,
                        "Save Error",
                        "Cannot save: no file path set",
                        e.to_string(),
                    );
                }
                Err(e.to_string())
            }
        }
    }

    /// Save the current taxonomy to a new file (async for file dialog)
    pub async fn save_as(&self) {
        if let Some(file) = rfd::AsyncFileDialog::new()
            .add_filter("JSON", &["json"])
            .set_title("Save Taxonomy As")
            .save_file()
            .await
        {
            let path = file.path().to_path_buf();
            let save_result = self.state.borrow_mut().save_as(path.clone());

            match save_result {
                Ok(_) => {
                    // Update window title
                    let title = self.state.borrow().get_window_title();
                    self.window.set_window_title(SharedString::from(title));

                    set_status(self.window, "File saved successfully", StatusLevel::Success);
                }
                Err(e) => {
                    let (title, message, details) = map_file_save_error(&*e, Some(&path));
                    show_error(self.window, title, message, details);
                }
            }
        }
    }

    /// Revert to the last saved version of the file
    pub async fn revert(&self) {
        let path = self.state.borrow().current_file.clone();

        if let Some(file_path) = path {
            // Load the file again
            let load_result = self.state.borrow_mut().load_from_file(file_path.clone());

            match load_result {
                Ok(_) => {
                    // Update window title
                    let title = self.state.borrow().get_window_title();
                    self.window.set_window_title(SharedString::from(title));

                    // Update UI with loaded data
                    update_ui_from_state(self.window, &self.state.borrow());

                    set_status(self.window, "Reverted to saved version", StatusLevel::Success);
                }
                Err(e) => {
                    let (title, message, details) = map_revert_error(&*e, &file_path);
                    show_error(self.window, title, message, details);
                }
            }
        }
    }
}
