use slint::{ComponentHandle, SharedString};
use std::cell::RefCell;
use std::rc::Rc;
use taxstud_core::{matches_filters, parse_facet_filters, Filters};

use crate::state::AppState;
use crate::ui::{set_status, update_ui_from_state};
use crate::{MainWindow, StatusLevel};

/// Register all filter and sorting handlers
pub fn register_filter_handlers(window: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    register_sort_by_name(window, app_state);
    register_apply_filters(window, app_state);
    register_clear_filters(window, app_state);
}

/// Register sort by name handler
fn register_sort_by_name(window: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();

    window.on_sort_by_name(move || {
        let main_window = main_window_weak.unwrap();

        // Set sort_by in state
        {
            let mut state_mut = app_state.borrow_mut();
            state_mut.sort_by = Some("name".to_string());
        }

        // Update UI from state (will apply the sort)
        update_ui_from_state(&main_window, &app_state);
        set_status(&main_window, "Items sorted by name", StatusLevel::Info);
    });
}

/// Register apply filters handler
fn register_apply_filters(window: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();

    window.on_apply_filters(move || {
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
        let filtered_count = {
            let mut state_mut = app_state.borrow_mut();
            state_mut.filters.genera = genera.clone();
            state_mut.filters.facets = facet_map.clone();

            // Count filtered items
            if let Some(ref data) = state_mut.data {
                data.items
                    .iter()
                    .filter(|item| matches_filters(item, &state_mut.filters))
                    .count()
            } else {
                0
            }
        };

        // Update UI from state (will apply filters and any active sort)
        update_ui_from_state(&main_window, &app_state);

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

        set_status(
            &main_window,
            format!("Filters applied: {} items match", filtered_count),
            StatusLevel::Info,
        );
    });
}

/// Register clear filters handler
fn register_clear_filters(window: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();

    window.on_clear_filters(move || {
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
        update_ui_from_state(&main_window, &app_state);

        set_status(&main_window, "Filters cleared", StatusLevel::Info);
    });
}
