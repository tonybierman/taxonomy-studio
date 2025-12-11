mod errors;
mod handlers;
mod operations;
mod state;
mod ui;

use clap::Parser;
use slint::{ComponentHandle, SharedString};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use handlers::*;
use state::{AppState, UiState};
use ui::{set_status, update_ui_from_state};

slint::slint!(export { MainWindow } from "ui/app-window.slint";);

/// TaxStud - Hybrid Taxonomy Management System
#[derive(Parser, Debug)]
#[command(name = "taxstud")]
#[command(author, version, about = "Hybrid Taxonomy Management System", long_about = None)]
struct Args {
    /// Path to taxonomy JSON file to load on startup
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
}

pub fn main() {
    let args = Args::parse();

    let main_window = MainWindow::new().unwrap();
    let app_state = Rc::new(RefCell::new(AppState::new()));
    let ui_state = Rc::new(RefCell::new(UiState::new()));

    // Set initial window title
    main_window.set_window_title(SharedString::from("Taxonomy Studio - No file loaded"));

    // Load file from command line if provided
    if let Some(file_path) = args.file {
        let load_result = app_state.borrow_mut().load_from_file(file_path.clone());

        match load_result {
            Ok(_) => {
                // Update window title
                let title = app_state.borrow().get_window_title();
                main_window.set_window_title(SharedString::from(title));

                // Update UI with loaded data
                update_ui_from_state(&main_window, &app_state);

                set_status(
                    &main_window,
                    format!("Loaded: {}", file_path.display()),
                    StatusLevel::Success,
                );
            }
            Err(e) => {
                set_status(
                    &main_window,
                    format!("Error loading file: {}", e),
                    StatusLevel::Danger,
                );
            }
        }
    }

    // Register all handlers
    register_item_handlers(&main_window, &app_state);
    register_file_handlers(&main_window, &app_state, &ui_state);
    register_filter_handlers(&main_window, &app_state);
    register_dialog_handlers(&main_window, &app_state, &ui_state);
    register_ui_handlers(&main_window);

    main_window.run().unwrap();
}
