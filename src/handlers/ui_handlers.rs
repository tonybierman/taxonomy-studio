use slint::{ComponentHandle, Model, VecModel};

use crate::ui::set_status;
use crate::{FacetInput, MainWindow, StatusLevel, Theme};

/// Register all UI-related handlers (theme, about, facet updates)
pub fn register_ui_handlers(window: &MainWindow) {
    register_about(window);
    register_toggle_theme(window);
    register_update_edit_facet(window);
    register_update_create_facet(window);
}

/// Register about handler
fn register_about(window: &MainWindow) {
    let main_window_weak = window.as_weak();

    window.on_about(move || {
        let main_window = main_window_weak.unwrap();

        set_status(
            &main_window,
            "Taxonomy Studio 0.1.0 by Tony Bierman",
            StatusLevel::Info,
        );
    });
}

/// Register theme toggle handler
fn register_toggle_theme(window: &MainWindow) {
    let main_window_weak = window.as_weak();

    window.on_toggle_theme(move || {
        let main_window = main_window_weak.unwrap();

        // Toggle between light and dark theme
        let current_theme = main_window.get_theme();
        let new_theme = if current_theme == Theme::Light {
            Theme::Dark
        } else {
            Theme::Light
        };
        main_window.set_theme(new_theme);
    });
}

/// Register edit facet value update handler
fn register_update_edit_facet(window: &MainWindow) {
    let main_window_weak = window.as_weak();

    window.on_update_edit_facet(move |index, value| {
        let main_window = main_window_weak.unwrap();
        let facet_inputs = main_window.get_edit_facet_inputs();
        if let Some(model) = facet_inputs.as_any().downcast_ref::<VecModel<FacetInput>>() {
            if (index as usize) < model.row_count() {
                let mut item = model.row_data(index as usize).unwrap();
                item.value = value;
                model.set_row_data(index as usize, item);
            }
        }
    });
}

/// Register create facet value update handler
fn register_update_create_facet(window: &MainWindow) {
    let main_window_weak = window.as_weak();

    window.on_update_create_facet(move |index, value| {
        let main_window = main_window_weak.unwrap();
        let facet_inputs = main_window.get_create_facet_inputs();
        if let Some(model) = facet_inputs.as_any().downcast_ref::<VecModel<FacetInput>>() {
            if (index as usize) < model.row_count() {
                let mut item = model.row_data(index as usize).unwrap();
                item.value = value;
                model.set_row_data(index as usize, item);
            }
        }
    });
}
