use slint::{ComponentHandle, SharedString};
use std::cell::RefCell;
use std::rc::Rc;

use crate::errors::map_file_save_error;
use crate::operations::FileOperations;
use crate::state::{AppState, PendingAction, SimpleConfirmationAction, UiState};
use crate::ui::{
    hide_confirmation, hide_error, hide_simple_confirmation, set_status, show_error,
    update_ui_from_state,
};
use crate::{MainWindow, StatusLevel};

/// Register all dialog response handlers
pub fn register_dialog_handlers(
    window: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    ui_state: &Rc<RefCell<UiState>>,
) {
    register_confirmation_save(window, app_state, ui_state);
    register_confirmation_dont_save(window, app_state, ui_state);
    register_confirmation_cancel(window, ui_state);
    register_simple_confirmation_ok(window, app_state, ui_state);
    register_simple_confirmation_cancel(window, ui_state);
    register_error_dialog_close(window);
}

/// Register confirmation dialog "Save" handler
fn register_confirmation_save(
    window: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    ui_state: &Rc<RefCell<UiState>>,
) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();
    let ui_state = ui_state.clone();

    window.on_confirmation_save(move || {
        let main_window = main_window_weak.unwrap();

        // Save the file first
        let save_result = app_state.borrow_mut().save();

        match save_result {
            Ok(_) => {
                // Update window title
                let title = app_state.borrow().get_window_title();
                main_window.set_window_title(SharedString::from(title));

                // Hide confirmation dialog
                hide_confirmation(&main_window);

                // Now proceed with the pending action
                if let Some(action) = ui_state.borrow_mut().pending_action.take() {
                    execute_pending_action(action, &app_state, &main_window);
                }
            }
            Err(e) => {
                // Hide confirmation dialog
                hide_confirmation(&main_window);

                // Show error using error mapper
                let (title, message, details) = map_file_save_error(&*e, None);
                show_error(&main_window, title, message, details);

                // Clear pending action since we couldn't save
                ui_state.borrow_mut().pending_action = None;
            }
        }
    });
}

/// Register confirmation dialog "Don't Save" handler
fn register_confirmation_dont_save(
    window: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    ui_state: &Rc<RefCell<UiState>>,
) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();
    let ui_state = ui_state.clone();

    window.on_confirmation_dont_save(move || {
        let main_window = main_window_weak.unwrap();

        // Hide confirmation dialog
        hide_confirmation(&main_window);

        // Proceed with the pending action without saving
        if let Some(action) = ui_state.borrow_mut().pending_action.take() {
            execute_pending_action(action, &app_state, &main_window);
        }
    });
}

/// Register confirmation dialog "Cancel" handler
fn register_confirmation_cancel(window: &MainWindow, ui_state: &Rc<RefCell<UiState>>) {
    let main_window_weak = window.as_weak();
    let ui_state = ui_state.clone();

    window.on_confirmation_cancel(move || {
        let main_window = main_window_weak.unwrap();

        // Hide confirmation dialog
        hide_confirmation(&main_window);

        // Clear pending action
        ui_state.borrow_mut().pending_action = None;

        set_status(&main_window, "Action cancelled", StatusLevel::Info);
    });
}

/// Register simple confirmation dialog "OK" handler
fn register_simple_confirmation_ok(
    window: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    ui_state: &Rc<RefCell<UiState>>,
) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();
    let ui_state = ui_state.clone();

    window.on_simple_confirmation_ok(move || {
        let main_window = main_window_weak.unwrap();

        // Hide confirmation dialog
        hide_simple_confirmation(&main_window);

        // Get the action and drop the borrow immediately
        let action = ui_state.borrow_mut().simple_confirmation_action.take();

        // Execute the action
        if let Some(action) = action {
            match action {
                SimpleConfirmationAction::Revert => {
                    // Use FileOperations for revert
                    let app_state = app_state.clone();
                    let main_window = main_window.clone_strong();
                    slint::spawn_local(async move {
                        let ops = FileOperations::new(&app_state, &main_window);
                        ops.revert().await;
                    })
                    .unwrap();
                }
            }
        }
    });
}

/// Register simple confirmation dialog "Cancel" handler
fn register_simple_confirmation_cancel(window: &MainWindow, ui_state: &Rc<RefCell<UiState>>) {
    let main_window_weak = window.as_weak();
    let ui_state = ui_state.clone();

    window.on_simple_confirmation_cancel(move || {
        let main_window = main_window_weak.unwrap();

        // Hide confirmation dialog
        hide_simple_confirmation(&main_window);

        // Clear action
        ui_state.borrow_mut().simple_confirmation_action = None;

        set_status(&main_window, "Action cancelled", StatusLevel::Info);
    });
}

/// Register error dialog close handler
fn register_error_dialog_close(window: &MainWindow) {
    let main_window_weak = window.as_weak();

    window.on_error_dialog_close(move || {
        let main_window = main_window_weak.unwrap();

        // Hide error dialog
        hide_error(&main_window);
    });
}

/// Helper function to execute a pending action
fn execute_pending_action(
    action: PendingAction,
    app_state: &Rc<RefCell<AppState>>,
    main_window: &MainWindow,
) {
    match action {
        PendingAction::Open => {
            // Trigger file open using FileOperations
            let app_state = app_state.clone();
            let main_window = main_window.clone_strong();
            slint::spawn_local(async move {
                let ops = FileOperations::new(&app_state, &main_window);
                ops.open_file_dialog_and_load().await;
            })
            .unwrap();
        }
        PendingAction::New => {
            // Create new taxonomy
            app_state.borrow_mut().create_new();
            let title = app_state.borrow().get_window_title();
            main_window.set_window_title(SharedString::from(title));
            update_ui_from_state(main_window, app_state);
            set_status(main_window, "New taxonomy created", StatusLevel::Success);
        }
        PendingAction::Exit => {
            // Exit the application
            let _ = main_window.hide();
            let _ = slint::quit_event_loop();
        }
    }
}
