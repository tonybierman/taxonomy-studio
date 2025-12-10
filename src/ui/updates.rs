use slint::{SharedString, StandardListViewItem, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

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
    update_ui_from_state(main_window, &state.borrow());

    // Set status
    set_status(main_window, status_message, level);
}

/// Refresh UI after successfully loading a file
/// Convenience function that combines common operations after file load
pub fn refresh_after_file_load(
    main_window: &MainWindow,
    state: &AppState,
    message: impl Into<SharedString>,
) {
    // Update window title
    let title = state.get_window_title();
    main_window.set_window_title(SharedString::from(title));

    // Update UI with loaded data
    update_ui_from_state(main_window, state);

    // Set success status
    set_status(main_window, message, StatusLevel::Success);
}

/// Update the UI from the current application state
pub fn update_ui_from_state(main_window: &MainWindow, state: &AppState) {
    // Clear selected item
    main_window.set_selected_item_index(-1);
    main_window.set_selected_item_name(SharedString::from(""));
    main_window.set_selected_item_path(SharedString::from(""));
    main_window.set_selected_item_facets(SharedString::from(""));

    if let Some(ref taxonomy) = state.taxonomy {
        // Update taxonomy description
        let description = taxonomy.taxonomy_description.as_deref().unwrap_or("");

        eprintln!("DEBUG: Setting taxonomy description: '{}'", description);
        main_window.set_taxonomy_description(SharedString::from(description));

        // Update hierarchy root
        eprintln!(
            "DEBUG: Setting hierarchy root: '{}'",
            taxonomy.classical_hierarchy.root
        );
        main_window.set_hierarchy_root(SharedString::from(&taxonomy.classical_hierarchy.root));

        // Update hierarchy tree
        let tree_nodes = flatten_hierarchy(&taxonomy.classical_hierarchy);
        let tree_model = Rc::new(VecModel::from(tree_nodes));
        main_window.set_hierarchy_tree(tree_model.into());

        // Update facet dimensions
        let facet_dims_text = format_facet_dimensions(&taxonomy.faceted_dimensions);
        main_window.set_facet_dimensions_text(SharedString::from(facet_dims_text));

        // Update items list
        if let Some(ref items) = taxonomy.example_items {
            eprintln!("DEBUG: Loading {} items", items.len());
            for (i, item) in items.iter().enumerate() {
                eprintln!("DEBUG: Item {}: '{}'", i, item.name);
            }

            let items_model = Rc::new(VecModel::from(
                items
                    .iter()
                    .map(|item| StandardListViewItem::from(SharedString::from(&item.name)))
                    .collect::<Vec<_>>(),
            ));
            main_window.set_items_list(items_model.into());
        } else {
            let empty_model = Rc::new(VecModel::<StandardListViewItem>::default());
            main_window.set_items_list(empty_model.into());
        }
    } else {
        // Clear UI
        main_window.set_taxonomy_description(SharedString::from(""));
        main_window.set_hierarchy_root(SharedString::from(""));
        let empty_tree_model = Rc::new(VecModel::<TreeNode>::default());
        main_window.set_hierarchy_tree(empty_tree_model.into());
        main_window.set_facet_dimensions_text(SharedString::from(""));
        let empty_model = Rc::new(VecModel::<StandardListViewItem>::default());
        main_window.set_items_list(empty_model.into());
    }
}
