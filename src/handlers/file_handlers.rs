use slint::{ComponentHandle, SharedString};
use std::cell::RefCell;
use std::rc::Rc;

use crate::operations::FileOperations;
use crate::state::{AppState, PendingAction, SimpleConfirmationAction, UiState};
use crate::ui::{set_status, show_confirmation, show_simple_confirmation, update_ui_from_state};
use crate::{MainWindow, StatusLevel};

/// Register all file operation handlers
pub fn register_file_handlers(
    window: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    ui_state: &Rc<RefCell<UiState>>,
) {
    register_file_open(window, app_state, ui_state);
    register_file_save(window, app_state);
    register_file_save_as(window, app_state);
    register_file_new(window, app_state, ui_state);
    register_file_revert(window, app_state, ui_state);
    register_file_exit(window, app_state, ui_state);
}

/// Register File -> Open handler
fn register_file_open(
    window: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    ui_state: &Rc<RefCell<UiState>>,
) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();
    let ui_state = ui_state.clone();

    window.on_file_open(move || {
        let main_window = main_window_weak.unwrap();

        // Check for unsaved changes
        if app_state.borrow().dirty {
            // Store pending action and show confirmation dialog
            ui_state.borrow_mut().pending_action = Some(PendingAction::Open);
            show_confirmation(
                &main_window,
                "You have unsaved changes. Do you want to save before opening another file?",
            );
        } else {
            // No unsaved changes, proceed with open using FileOperations
            let app_state = app_state.clone();
            let main_window_clone = main_window.clone_strong();
            slint::spawn_local(async move {
                let ops = FileOperations::new(&app_state, &main_window_clone);
                ops.open_file_dialog_and_load().await;
            })
            .unwrap();
        }
    });
}

/// Register File -> Save handler
fn register_file_save(window: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();

    window.on_file_save(move || {
        let main_window = main_window_weak.unwrap();

        // Use FileOperations for saving
        let ops = FileOperations::new(&app_state, &main_window);
        let _ = ops.save();
    });
}

/// Register File -> Save As handler
fn register_file_save_as(window: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();

    window.on_file_save_as(move || {
        let main_window = main_window_weak.unwrap();
        let app_state = app_state.clone();
        let main_window_clone = main_window.clone_strong();

        slint::spawn_local(async move {
            let ops = FileOperations::new(&app_state, &main_window_clone);
            ops.save_as().await;
        })
        .unwrap();
    });
}

/// Register File -> New handler
fn register_file_new(
    window: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    ui_state: &Rc<RefCell<UiState>>,
) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();
    let ui_state = ui_state.clone();

    window.on_file_new(move || {
        let main_window = main_window_weak.unwrap();

        // Check for unsaved changes
        if app_state.borrow().dirty {
            // Store pending action and show confirmation dialog
            ui_state.borrow_mut().pending_action = Some(PendingAction::New);
            show_confirmation(
                &main_window,
                "You have unsaved changes. Do you want to save before creating a new taxonomy?",
            );
        } else {
            // No unsaved changes, proceed with creating new taxonomy
            // Create new (drops mutable borrow immediately)
            app_state.borrow_mut().create_new();

            // Update window title (borrow immutably)
            let title = app_state.borrow().get_window_title();
            main_window.set_window_title(SharedString::from(title));

            // Update UI with new empty taxonomy (borrow immutably)
            update_ui_from_state(&main_window, &app_state);

            set_status(&main_window, "New taxonomy created", StatusLevel::Success);
        }
    });
}

/// Register File -> Revert to Saved handler
fn register_file_revert(
    window: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    ui_state: &Rc<RefCell<UiState>>,
) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();
    let ui_state = ui_state.clone();

    window.on_file_revert(move || {
        let main_window = main_window_weak.unwrap();

        // Check if we have a file to revert to
        let can_revert = {
            let state_borrow = app_state.borrow();
            state_borrow.current_file.is_some() && state_borrow.dirty
        };

        if can_revert {
            // Show confirmation dialog
            ui_state.borrow_mut().simple_confirmation_action =
                Some(SimpleConfirmationAction::Revert);
            show_simple_confirmation(
                &main_window,
                "Revert to Saved",
                "Are you sure you want to revert to the last saved version? All unsaved changes will be lost.",
                "Revert",
            );
        } else {
            // Either no file or no changes
            let state_borrow = app_state.borrow();
            if state_borrow.current_file.is_none() {
                set_status(
                    &main_window,
                    "No file to revert to",
                    StatusLevel::Warning,
                );
            } else {
                set_status(&main_window, "No unsaved changes", StatusLevel::Info);
            }
        }
    });
}

/// Register File -> Exit handler
fn register_file_exit(
    window: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    ui_state: &Rc<RefCell<UiState>>,
) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();
    let ui_state = ui_state.clone();

    window.on_file_exit(move || {
        let main_window = main_window_weak.unwrap();

        // Check for unsaved changes
        if app_state.borrow().dirty {
            // Store pending action and show confirmation dialog
            ui_state.borrow_mut().pending_action = Some(PendingAction::Exit);
            show_confirmation(
                &main_window,
                "You have unsaved changes. Do you want to save before exiting?",
            );
        } else {
            // No unsaved changes, exit immediately
            let _ = main_window.hide();
            let _ = slint::quit_event_loop();
        }
    });
}
