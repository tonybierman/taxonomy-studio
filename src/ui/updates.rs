use slint::{SharedString, StandardListViewItem, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use taxstud_core::{matches_filters, sort_items};

use crate::state::AppState;
use crate::ui::dialogs::set_status;
use crate::ui::formatting::{flatten_hierarchy, format_facet_dimensions};
use crate::ui::types::{StatusLevel, TreeNode};
use crate::MainWindow;

/// Refresh UI after a state-changing operation (edit, create, delete)
/// Updates window title, refreshes UI from state, and sets status message
pub fn refresh_ui_after_state_change(
    main_window: &MainWindow,
    state: &Rc<RefCell<AppState>>,
    status_message: &str,
    level: StatusLevel,
) {
    // Update window title
    let title = state.borrow().get_window_title();
    main_window.set_window_title(SharedString::from(title));

    // Refresh the UI
    update_ui_from_state(main_window, state);

    // Set status
    set_status(main_window, status_message, level);
}

/// Update the UI from the current application state
pub fn update_ui_from_state(main_window: &MainWindow, state: &Rc<RefCell<AppState>>) {
    // Clear selected item
    main_window.set_selected_item_index(-1);
    main_window.set_selected_item_name(SharedString::from(""));
    main_window.set_selected_item_path(SharedString::from(""));
    main_window.set_selected_item_facets(SharedString::from(""));

    let state_borrow = state.borrow();

    // Update from schema (if present)
    if let Some(ref schema) = state_borrow.schema {
        // Update taxonomy description
        let description = schema.description.as_deref().unwrap_or("");
        main_window.set_taxonomy_description(SharedString::from(description));

        // Update hierarchy root
        main_window.set_hierarchy_root(SharedString::from(&schema.classical_hierarchy.root));

        // Update hierarchy tree
        let tree_nodes = flatten_hierarchy(&schema.classical_hierarchy);
        let tree_model = Rc::new(VecModel::from(tree_nodes));
        main_window.set_hierarchy_tree(tree_model.into());

        // Update facet dimensions
        let facet_dims_text = format_facet_dimensions(&schema.faceted_dimensions);
        main_window.set_facet_dimensions_text(SharedString::from(facet_dims_text));
    } else {
        // Clear schema-related UI
        main_window.set_taxonomy_description(SharedString::from(""));
        main_window.set_hierarchy_root(SharedString::from(""));
        let empty_tree_model = Rc::new(VecModel::<TreeNode>::default());
        main_window.set_hierarchy_tree(empty_tree_model.into());
        main_window.set_facet_dimensions_text(SharedString::from(""));
    }

    // Update items from data (if present)
    if let Some(ref data) = state_borrow.data {
        // Start with all items
        let mut items = data.items.clone();

        // Apply filters if any are active
        let has_active_filters =
            !state_borrow.filters.genera.is_empty() || !state_borrow.filters.facets.is_empty();
        if has_active_filters {
            items.retain(|item| matches_filters(item, &state_borrow.filters));
        }

        // Apply sorting if active
        if let Some(ref sort_field) = state_borrow.sort_by {
            sort_items(&mut items, sort_field);
        }

        // Store displayed items for index mapping
        drop(state_borrow);
        state.borrow_mut().displayed_items = items.clone();

        // Update UI with processed items
        let items_model = Rc::new(VecModel::from(
            items
                .iter()
                .map(|item| StandardListViewItem::from(SharedString::from(&item.name)))
                .collect::<Vec<_>>(),
        ));
        main_window.set_items_list(items_model.into());
    } else {
        drop(state_borrow);
        state.borrow_mut().displayed_items = Vec::new();

        let empty_model = Rc::new(VecModel::<StandardListViewItem>::default());
        main_window.set_items_list(empty_model.into());
    }
}
