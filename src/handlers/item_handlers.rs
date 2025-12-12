use slint::{ComponentHandle, SharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use taxstud_core::Item;

use crate::operations::{collect_facets, validate_item_input};
use crate::state::AppState;
use crate::ui::{create_facet_inputs, format_facets, refresh_ui_after_state_change, set_status};
use crate::{MainWindow, StatusLevel};

/// Register all item CRUD handlers
pub fn register_item_handlers(window: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    register_item_selected(window, app_state);
    register_start_edit(window, app_state);
    register_save_edit(window, app_state);
    register_cancel_edit(window);
    register_start_create_item(window, app_state);
    register_save_new_item(window, app_state);
    register_cancel_create_item(window);
    register_delete_item(window, app_state);
}

/// Register item selection handler
fn register_item_selected(window: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();

    window.on_item_selected(move |index| {
        let main_window = main_window_weak.unwrap();
        let state_borrow = app_state.borrow();

        // Use displayed_items which reflects the current sort/filter state
        if index >= 0 && (index as usize) < state_borrow.displayed_items.len() {
            let item = &state_borrow.displayed_items[index as usize];

            // Update selected item properties
            main_window.set_selected_item_name(SharedString::from(&item.name));
            main_window.set_selected_item_path(SharedString::from(item.classical_path.join(" → ")));

            // Format facets
            let facets_text = format_facets(&item.facets);
            main_window.set_selected_item_facets(SharedString::from(facets_text));
        }
    });
}

/// Register start edit handler
fn register_start_edit(window: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();

    window.on_start_edit(move || {
        let main_window = main_window_weak.unwrap();
        let state_borrow = app_state.borrow();

        // Get the currently selected item from displayed_items
        if let Some(ref schema) = state_borrow.schema {
            let selected_idx = main_window.get_selected_item_index();
            if selected_idx >= 0 && (selected_idx as usize) < state_borrow.displayed_items.len() {
                let item = &state_borrow.displayed_items[selected_idx as usize];

                // Populate edit fields
                main_window.set_edit_item_name(SharedString::from(&item.name));
                main_window.set_edit_item_path(SharedString::from(item.classical_path.join(", ")));

                // Populate facet inputs based on schema dimensions
                let facet_inputs = create_facet_inputs(&schema.faceted_dimensions, &item.facets);
                let facet_inputs_model = Rc::new(VecModel::from(facet_inputs));
                main_window.set_edit_facet_inputs(facet_inputs_model.into());

                // Enter edit mode
                main_window.set_is_editing(true);
                main_window.set_validation_error(SharedString::from(""));
                set_status(&main_window, "Editing item...", StatusLevel::Info);
            }
        }
    });
}

/// Register save edit handler
fn register_save_edit(window: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();

    window.on_save_edit(move || {
        let main_window = main_window_weak.unwrap();

        // Get the original item name from the selected item (before editing)
        let original_name = main_window.get_selected_item_name().to_string();

        // Get edited values
        let new_name = main_window.get_edit_item_name().to_string();
        let new_path = main_window.get_edit_item_path().to_string();
        let facet_inputs = main_window.get_edit_facet_inputs();

        // Get the classical hierarchy from the schema
        let state_borrow = app_state.borrow();
        let hierarchy = match state_borrow.schema.as_ref() {
            Some(schema) => &schema.classical_hierarchy,
            None => {
                main_window.set_validation_error(SharedString::from("No schema loaded"));
                return;
            }
        };

        // Validate inputs using validation module
        let (validated_name, classical_path) =
            match validate_item_input(&new_name, &new_path, hierarchy) {
                Ok(result) => result,
                Err(e) => {
                    main_window.set_validation_error(SharedString::from(e.message));
                    return;
                }
            };
        drop(state_borrow);

        // Collect facets from inputs using validation module
        let facets_map = collect_facets(&facet_inputs);

        // Find and update the item in the data by original name
        let mut state_mut = app_state.borrow_mut();
        if let Some(ref mut data) = state_mut.data {
            // Find the item by original name
            if let Some(item) = data.items.iter_mut().find(|i| i.name == original_name) {
                item.name = validated_name.clone();
                item.classical_path = classical_path;
                item.facets = facets_map;

                // Mark as dirty
                state_mut.mark_dirty();

                // Exit edit mode
                drop(state_mut);
                main_window.set_is_editing(false);

                // Refresh UI and show success message
                refresh_ui_after_state_change(
                    &main_window,
                    &app_state,
                    "Item saved successfully",
                    StatusLevel::Success,
                );
            }
        }
    });
}

/// Register cancel edit handler
fn register_cancel_edit(window: &MainWindow) {
    let main_window_weak = window.as_weak();

    window.on_cancel_edit(move || {
        let main_window = main_window_weak.unwrap();

        // Exit edit mode without saving
        main_window.set_is_editing(false);
        main_window.set_validation_error(SharedString::from(""));
        set_status(&main_window, "Edit cancelled", StatusLevel::Info);
    });
}

/// Register start create item handler
fn register_start_create_item(window: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();

    window.on_start_create_item(move || {
        let main_window = main_window_weak.unwrap();
        let state_borrow = app_state.borrow();

        // Clear form fields
        main_window.set_new_item_name(SharedString::from(""));
        main_window.set_new_item_path(SharedString::from(""));
        main_window.set_validation_error(SharedString::from(""));

        // Populate facet inputs based on schema dimensions
        if let Some(ref schema) = state_borrow.schema {
            let empty_facets = std::collections::HashMap::new();
            let facet_inputs = create_facet_inputs(&schema.faceted_dimensions, &empty_facets);
            let facet_inputs_model = Rc::new(VecModel::from(facet_inputs));
            main_window.set_create_facet_inputs(facet_inputs_model.into());
        }

        // Enter create mode
        main_window.set_is_creating(true);
        set_status(&main_window, "Creating new item...", StatusLevel::Info);
    });
}

/// Register save new item handler
fn register_save_new_item(window: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();

    window.on_save_new_item(move || {
        let main_window = main_window_weak.unwrap();

        // Get form values
        let new_name = main_window.get_new_item_name().to_string();
        let new_path = main_window.get_new_item_path().to_string();
        let facet_inputs = main_window.get_create_facet_inputs();

        // Get the classical hierarchy from the schema
        let state_borrow = app_state.borrow();
        let hierarchy = match state_borrow.schema.as_ref() {
            Some(schema) => &schema.classical_hierarchy,
            None => {
                main_window.set_validation_error(SharedString::from("No schema loaded"));
                return;
            }
        };

        // Validate inputs using validation module
        let (validated_name, classical_path) =
            match validate_item_input(&new_name, &new_path, hierarchy) {
                Ok(result) => result,
                Err(e) => {
                    main_window.set_validation_error(SharedString::from(e.message));
                    return;
                }
            };
        drop(state_borrow);

        // Collect facets from inputs using validation module
        let facets_map = collect_facets(&facet_inputs);

        // Create new item
        let new_item = Item {
            name: validated_name.clone(),
            classical_path,
            facets: facets_map,
            extra: std::collections::HashMap::new(),
        };

        // Add to data
        let mut state_mut = app_state.borrow_mut();
        if let Some(ref mut data) = state_mut.data {
            data.items.push(new_item);

            // Mark as dirty
            state_mut.mark_dirty();

            // Exit create mode
            drop(state_mut);
            main_window.set_is_creating(false);

            // Refresh UI and show success message
            refresh_ui_after_state_change(
                &main_window,
                &app_state,
                &format!("Item '{}' created successfully", validated_name),
                StatusLevel::Success,
            );
        }
    });
}

/// Register cancel create item handler
fn register_cancel_create_item(window: &MainWindow) {
    let main_window_weak = window.as_weak();

    window.on_cancel_create_item(move || {
        let main_window = main_window_weak.unwrap();

        // Exit create mode without saving
        main_window.set_is_creating(false);
        main_window.set_validation_error(SharedString::from(""));
        set_status(&main_window, "Create cancelled", StatusLevel::Info);
    });
}

/// Register delete item handler
fn register_delete_item(window: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();

    window.on_delete_item(move || {
        let main_window = main_window_weak.unwrap();

        let selected_idx = main_window.get_selected_item_index();
        if selected_idx < 0 {
            return;
        }

        // Get item name from displayed_items
        let item_name = {
            let state_borrow = app_state.borrow();
            if (selected_idx as usize) < state_borrow.displayed_items.len() {
                state_borrow.displayed_items[selected_idx as usize]
                    .name
                    .clone()
            } else {
                return;
            }
        };

        // Find and delete the item from data by name
        let mut state_mut = app_state.borrow_mut();
        if let Some(ref mut data) = state_mut.data {
            // Find the item position by name
            if let Some(pos) = data.items.iter().position(|i| i.name == item_name) {
                data.items.remove(pos);

                // Mark as dirty
                state_mut.mark_dirty();

                // Exit and update
                drop(state_mut);

                // Refresh UI and show success message
                refresh_ui_after_state_change(
                    &main_window,
                    &app_state,
                    &format!("Item '{}' deleted", item_name),
                    StatusLevel::Success,
                );
            }
        }
    });
}
