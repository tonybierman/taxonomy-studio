mod errors;
mod operations;
mod state;
mod ui;

use clap::Parser;
use slint::{ComponentHandle, Model, SharedString, StandardListViewItem, VecModel};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use taxstud_core::*;

use errors::{map_file_load_error, map_file_save_error, map_revert_error};
use operations::{collect_facets, validate_item_input, FileOperations};
use state::{AppState, UiState, PendingAction, SimpleConfirmationAction};
use ui::{
    create_facet_inputs, format_facets, hide_confirmation, hide_error, hide_simple_confirmation,
    refresh_ui_after_state_change, set_status, show_confirmation, show_error,
    show_simple_confirmation, update_ui_from_state,
};

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
                update_ui_from_state(&main_window, &app_state.borrow());

                set_status(&main_window, format!("Loaded: {}", file_path.display()), StatusLevel::Success);
            }
            Err(e) => {
                set_status(&main_window, format!("Error loading file: {}", e), StatusLevel::Danger);
            }
        }
    }

    // Item selection handler
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();

        main_window.on_item_selected(move |index| {
            let main_window = main_window_weak.unwrap();
            let state_borrow = app_state.borrow();

            if let Some(ref taxonomy) = state_borrow.taxonomy {
                if let Some(ref items) = taxonomy.example_items {
                    if index >= 0 && (index as usize) < items.len() {
                        let item = &items[index as usize];

                        // Update selected item properties
                        main_window.set_selected_item_name(SharedString::from(&item.name));
                        main_window.set_selected_item_path(SharedString::from(
                            item.classical_path.join(" → ")
                        ));

                        // Format facets
                        let facets_text = format_facets(&item.facets);
                        main_window.set_selected_item_facets(SharedString::from(facets_text));
                    }
                }
            }
        });
    }

    // File -> Open
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();
        let ui_state = ui_state.clone();

        main_window.on_file_open(move || {
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
                }).unwrap();
            }
        });
    }

    // File -> Save
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();

        main_window.on_file_save(move || {
            let main_window = main_window_weak.unwrap();

            // Use FileOperations for saving
            let ops = FileOperations::new(&app_state, &main_window);
            let _ = ops.save();
        });
    }

    // File -> Save As
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();

        main_window.on_file_save_as(move || {
            let main_window = main_window_weak.unwrap();
            let app_state = app_state.clone();
            let main_window_clone = main_window.clone_strong();

            slint::spawn_local(async move {
                let ops = FileOperations::new(&app_state, &main_window_clone);
                ops.save_as().await;
            }).unwrap();
        });
    }

    // File -> New
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();
        let ui_state = ui_state.clone();

        main_window.on_file_new(move || {
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

                // Now borrow immutably
                let title = app_state.borrow().get_window_title();
                main_window.set_window_title(SharedString::from(title));

                update_ui_from_state(&main_window, &app_state.borrow());

                set_status(&main_window, "New taxonomy created", StatusLevel::Success);
            }
        });
    }

    // File -> Revert to Saved
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();
        let ui_state = ui_state.clone();

        main_window.on_file_revert(move || {
            let main_window = main_window_weak.unwrap();

            // Check if we have a file to revert to
            let can_revert = {
                let state_borrow = app_state.borrow();
                state_borrow.current_file.is_some() && state_borrow.dirty
            };

            if can_revert {
                // Show confirmation dialog
                ui_state.borrow_mut().simple_confirmation_action = Some(SimpleConfirmationAction::Revert);
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
                    set_status(&main_window, "No file to revert to", StatusLevel::Warning);
                } else {
                    set_status(&main_window, "No unsaved changes to revert", StatusLevel::Info);
                }
            }
        });
    }

    // Sort -> By Name
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();

        main_window.on_sort_by_name(move || {
            let main_window = main_window_weak.unwrap();
            let state_borrow = app_state.borrow();

            if let Some(ref taxonomy) = state_borrow.taxonomy {
                if let Some(ref items) = taxonomy.example_items {
                    // Clone items for sorting (don't modify the taxonomy)
                    let mut sorted_items = items.clone();

                    // Sort by name using taxstud-core
                    sort_items(&mut sorted_items, "name");

                    // Update UI with sorted items
                    let items_model = Rc::new(VecModel::from(
                        sorted_items.iter().map(|item| {
                            StandardListViewItem::from(SharedString::from(&item.name))
                        }).collect::<Vec<_>>()
                    ));
                    main_window.set_items_list(items_model.into());
                    set_status(&main_window, "Items sorted by name", StatusLevel::Info);
                }
            }
        });
    }

    // Apply Filters
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();

        main_window.on_apply_filters(move || {
            let main_window = main_window_weak.unwrap();

            // Get the genus filter text
            let genus_text = main_window.get_genus_filter_text().to_string();

            // Parse comma-separated genera
            let genera: Vec<String> = genus_text
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            // Get the facet filter text and parse it
            let facet_text = main_window.get_facet_filter_text().to_string();
            let facet_strings: Vec<String> = facet_text
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let facet_map = parse_facet_filters(&facet_strings);

            // Update state filters
            {
                let mut state_mut = app_state.borrow_mut();
                state_mut.filters.genera = genera.clone();
                state_mut.filters.facets = facet_map.clone();
            }

            // Apply filters
            let state_borrow = app_state.borrow();
            if let Some(ref taxonomy) = state_borrow.taxonomy {
                if let Some(ref items) = taxonomy.example_items {
                    let filtered_items: Vec<_> = items
                        .iter()
                        .filter(|item| matches_filters(item, &state_borrow.filters))
                        .cloned()
                        .collect();

                    // Update UI with filtered items
                    let items_model = Rc::new(VecModel::from(
                        filtered_items.iter().map(|item| {
                            StandardListViewItem::from(SharedString::from(&item.name))
                        }).collect::<Vec<_>>()
                    ));
                    main_window.set_items_list(items_model.into());

                    // Update active filters text
                    let mut filter_parts = Vec::new();
                    if !genera.is_empty() {
                        filter_parts.push(format!("Genus: {}", genera.join(" OR ")));
                    }
                    for (facet_name, values) in &facet_map {
                        filter_parts.push(format!("{}: {}", facet_name, values.join(" OR ")));
                    }
                    let filters_text = if filter_parts.is_empty() {
                        String::new()
                    } else {
                        filter_parts.join("; ")
                    };
                    main_window.set_active_filters_text(SharedString::from(filters_text));

                    set_status(&main_window, format!("Filters applied: {} items match", filtered_items.len()), StatusLevel::Info);
                }
            }
        });
    }

    // Clear Filters
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();

        main_window.on_clear_filters(move || {
            let main_window = main_window_weak.unwrap();

            // Clear filter inputs
            main_window.set_genus_filter_text(SharedString::from(""));
            main_window.set_facet_filter_text(SharedString::from(""));
            main_window.set_active_filters_text(SharedString::from(""));

            // Clear state filters
            app_state.borrow_mut().filters = Filters {
                genera: Vec::new(),
                facets: std::collections::HashMap::new(),
            };

            // Reset UI to show all items
            update_ui_from_state(&main_window, &app_state.borrow());

            set_status(&main_window, "Filters cleared", StatusLevel::Info);
        });
    }

    // Start Edit
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();

        main_window.on_start_edit(move || {
            let main_window = main_window_weak.unwrap();
            let state_borrow = app_state.borrow();

            // Get the currently selected item
            if let Some(ref taxonomy) = state_borrow.taxonomy {
                if let Some(ref items) = taxonomy.example_items {
                    let selected_idx = main_window.get_selected_item_index();
                    if selected_idx >= 0 && (selected_idx as usize) < items.len() {
                        let item = &items[selected_idx as usize];

                        // Populate edit fields
                        main_window.set_edit_item_name(SharedString::from(&item.name));
                        main_window.set_edit_item_path(SharedString::from(
                            item.classical_path.join(", ")
                        ));

                        // Populate facet inputs based on taxonomy dimensions
                        let facet_inputs = create_facet_inputs(&taxonomy.faceted_dimensions, &item.facets);
                        let facet_inputs_model = Rc::new(VecModel::from(facet_inputs));
                        main_window.set_edit_facet_inputs(facet_inputs_model.into());

                        // Enter edit mode
                        main_window.set_is_editing(true);
                        main_window.set_validation_error(SharedString::from(""));
                        set_status(&main_window, "Editing item...", StatusLevel::Info);
                    }
                }
            }
        });
    }

    // Save Edit
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();

        main_window.on_save_edit(move || {
            let main_window = main_window_weak.unwrap();

            // Get edited values
            let new_name = main_window.get_edit_item_name().to_string();
            let new_path = main_window.get_edit_item_path().to_string();
            let facet_inputs = main_window.get_edit_facet_inputs();

            // Validate inputs using validation module
            let (validated_name, classical_path) = match validate_item_input(&new_name, &new_path) {
                Ok(result) => result,
                Err(e) => {
                    main_window.set_validation_error(SharedString::from(e.message));
                    return;
                }
            };

            // Collect facets from inputs using validation module
            let facets_map = collect_facets(&facet_inputs);

            // Update the item in the taxonomy
            let mut state_mut = app_state.borrow_mut();
            if let Some(ref mut taxonomy) = state_mut.taxonomy {
                if let Some(ref mut items) = taxonomy.example_items {
                    let selected_idx = main_window.get_selected_item_index();
                    if selected_idx >= 0 && (selected_idx as usize) < items.len() {
                        let item = &mut items[selected_idx as usize];
                        item.name = validated_name.clone();
                        item.classical_path = classical_path;
                        item.facets = facets_map;

                        // Mark as dirty
                        state_mut.mark_dirty();

                        // Exit edit mode
                        drop(state_mut);
                        main_window.set_is_editing(false);

                        // Update window title
                        let title = app_state.borrow().get_window_title();
                        main_window.set_window_title(SharedString::from(title));

                        // Refresh the UI
                        update_ui_from_state(&main_window, &app_state.borrow());

                        // Re-select the edited item
                        main_window.set_selected_item_index(selected_idx);

                        // Trigger item selection to update details panel
                        let state_borrow = app_state.borrow();
                        if let Some(ref taxonomy) = state_borrow.taxonomy {
                            if let Some(ref items) = taxonomy.example_items {
                                if selected_idx >= 0 && (selected_idx as usize) < items.len() {
                                    let item = &items[selected_idx as usize];
                                    main_window.set_selected_item_name(SharedString::from(&item.name));
                                    main_window.set_selected_item_path(SharedString::from(
                                        item.classical_path.join(" → ")
                                    ));
                                    let facets_text = format_facets(&item.facets);
                                    main_window.set_selected_item_facets(SharedString::from(facets_text));
                                }
                            }
                        }

                        set_status(&main_window, "Item saved successfully", StatusLevel::Success);
                    }
                }
            }
        });
    }

    // Cancel Edit
    {
        let main_window_weak = main_window.as_weak();

        main_window.on_cancel_edit(move || {
            let main_window = main_window_weak.unwrap();

            // Exit edit mode without saving
            main_window.set_is_editing(false);
            main_window.set_validation_error(SharedString::from(""));
            set_status(&main_window, "Edit cancelled", StatusLevel::Info);
        });
    }

    // Start Create Item
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();

        main_window.on_start_create_item(move || {
            let main_window = main_window_weak.unwrap();
            let state_borrow = app_state.borrow();

            // Clear form fields
            main_window.set_new_item_name(SharedString::from(""));
            main_window.set_new_item_path(SharedString::from(""));
            main_window.set_validation_error(SharedString::from(""));

            // Populate facet inputs based on taxonomy dimensions
            if let Some(ref taxonomy) = state_borrow.taxonomy {
                let empty_facets = std::collections::HashMap::new();
                let facet_inputs = create_facet_inputs(&taxonomy.faceted_dimensions, &empty_facets);
                let facet_inputs_model = Rc::new(VecModel::from(facet_inputs));
                main_window.set_create_facet_inputs(facet_inputs_model.into());
            }

            // Enter create mode
            main_window.set_is_creating(true);
            set_status(&main_window, "Creating new item...", StatusLevel::Info);
        });
    }

    // Save New Item
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();

        main_window.on_save_new_item(move || {
            let main_window = main_window_weak.unwrap();

            // Get form values
            let new_name = main_window.get_new_item_name().to_string();
            let new_path = main_window.get_new_item_path().to_string();
            let facet_inputs = main_window.get_create_facet_inputs();

            // Validate inputs using validation module
            let (validated_name, classical_path) = match validate_item_input(&new_name, &new_path) {
                Ok(result) => result,
                Err(e) => {
                    main_window.set_validation_error(SharedString::from(e.message));
                    return;
                }
            };

            // Collect facets from inputs using validation module
            let facets_map = collect_facets(&facet_inputs);

            // Create new item
            let new_item = Item {
                name: validated_name.clone(),
                classical_path,
                facets: facets_map,
                extra: std::collections::HashMap::new(),
            };

            // Add to taxonomy
            let mut state_mut = app_state.borrow_mut();
            if let Some(ref mut taxonomy) = state_mut.taxonomy {
                if let Some(ref mut items) = taxonomy.example_items {
                    items.push(new_item);
                } else {
                    taxonomy.example_items = Some(vec![new_item]);
                }

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

    // Cancel Create Item
    {
        let main_window_weak = main_window.as_weak();

        main_window.on_cancel_create_item(move || {
            let main_window = main_window_weak.unwrap();

            // Exit create mode without saving
            main_window.set_is_creating(false);
            main_window.set_validation_error(SharedString::from(""));
            set_status(&main_window, "Create cancelled", StatusLevel::Info);
        });
    }

    // About
    {
        let main_window_weak = main_window.as_weak();

        main_window.on_about(move || {
            let main_window = main_window_weak.unwrap();

            // Exit create mode without saving
            set_status(&main_window, "Taxonomy Studio 0.1.0 by Tony Bierman", StatusLevel::Info);
        });
    }

    // Theme Toggle
    {
        let main_window_weak = main_window.as_weak();

        main_window.on_toggle_theme(move || {
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

    // Update Edit Facet Value
    {
        let main_window_weak = main_window.as_weak();

        main_window.on_update_edit_facet(move |index, value| {
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

    // Update Create Facet Value
    {
        let main_window_weak = main_window.as_weak();

        main_window.on_update_create_facet(move |index, value| {
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

    // Delete Item
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();

        main_window.on_delete_item(move || {
            let main_window = main_window_weak.unwrap();

            let selected_idx = main_window.get_selected_item_index();
            if selected_idx < 0 {
                return;
            }

            // Get item name for confirmation message
            let item_name = {
                let state_borrow = app_state.borrow();
                if let Some(ref taxonomy) = state_borrow.taxonomy {
                    if let Some(ref items) = taxonomy.example_items {
                        if (selected_idx as usize) < items.len() {
                            items[selected_idx as usize].name.clone()
                        } else {
                            return;
                        }
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            };

            // For now, delete without confirmation (we can add a dialog later)
            // In a real app, you'd use a confirmation dialog here
            let mut state_mut = app_state.borrow_mut();
            if let Some(ref mut taxonomy) = state_mut.taxonomy {
                if let Some(ref mut items) = taxonomy.example_items {
                    if (selected_idx as usize) < items.len() {
                        items.remove(selected_idx as usize);

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
            }
        });
    }

    // Confirmation Dialog - Save
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();
        let ui_state = ui_state.clone();

        main_window.on_confirmation_save(move || {
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
                        match action {
                            PendingAction::Open => {
                                // Trigger file open using FileOperations
                                let app_state = app_state.clone();
                                let main_window = main_window.clone_strong();
                                slint::spawn_local(async move {
                                    let ops = FileOperations::new(&app_state, &main_window);
                                    ops.open_file_dialog_and_load().await;
                                }).unwrap();
                            }
                            PendingAction::New => {
                                // Create new taxonomy
                                app_state.borrow_mut().create_new();
                                let title = app_state.borrow().get_window_title();
                                main_window.set_window_title(SharedString::from(title));
                                update_ui_from_state(&main_window, &app_state.borrow());
                                set_status(&main_window, "New taxonomy created", StatusLevel::Success);
                            }
                        }
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

    // Confirmation Dialog - Don't Save
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();
        let ui_state = ui_state.clone();

        main_window.on_confirmation_dont_save(move || {
            let main_window = main_window_weak.unwrap();

            // Hide confirmation dialog
            hide_confirmation(&main_window);

            // Proceed with the pending action without saving
            if let Some(action) = ui_state.borrow_mut().pending_action.take() {
                match action {
                    PendingAction::Open => {
                        // Trigger file open using FileOperations
                        let app_state = app_state.clone();
                        let main_window = main_window.clone_strong();
                        slint::spawn_local(async move {
                            let ops = FileOperations::new(&app_state, &main_window);
                            ops.open_file_dialog_and_load().await;
                        }).unwrap();
                    }
                    PendingAction::New => {
                        // Create new taxonomy
                        app_state.borrow_mut().create_new();
                        let title = app_state.borrow().get_window_title();
                        main_window.set_window_title(SharedString::from(title));
                        update_ui_from_state(&main_window, &app_state.borrow());
                        set_status(&main_window, "New taxonomy created", StatusLevel::Success);
                    }
                }
            }
        });
    }

    // Confirmation Dialog - Cancel
    {
        let main_window_weak = main_window.as_weak();
        let _app_state = app_state.clone();
        let ui_state = ui_state.clone();

        main_window.on_confirmation_cancel(move || {
            let main_window = main_window_weak.unwrap();

            // Hide confirmation dialog
            hide_confirmation(&main_window);

            // Clear pending action
            ui_state.borrow_mut().pending_action = None;

            set_status(&main_window, "Action cancelled", StatusLevel::Info);
        });
    }

    // Error Dialog - Close
    {
        let main_window_weak = main_window.as_weak();

        main_window.on_error_dialog_close(move || {
            let main_window = main_window_weak.unwrap();

            // Hide error dialog
            hide_error(&main_window);
        });
    }

    // Simple Confirmation - OK
    {
        let main_window_weak = main_window.as_weak();
        let app_state = app_state.clone();
        let ui_state = ui_state.clone();

        main_window.on_simple_confirmation_ok(move || {
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
                        }).unwrap();
                    }
                }
            }
        });
    }

    // Simple Confirmation - Cancel
    {
        let main_window_weak = main_window.as_weak();
        let _app_state = app_state.clone();
        let ui_state = ui_state.clone();

        main_window.on_simple_confirmation_cancel(move || {
            let main_window = main_window_weak.unwrap();

            // Hide confirmation dialog
            hide_simple_confirmation(&main_window);

            // Clear action
            ui_state.borrow_mut().simple_confirmation_action = None;

            set_status(&main_window, "Action cancelled", StatusLevel::Info);
        });
    }

    main_window.run().unwrap();
}
