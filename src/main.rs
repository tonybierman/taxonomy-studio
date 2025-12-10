use slint::{ComponentHandle, Model, SharedString, StandardListViewItem, VecModel};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use taxstud_core::*;

slint::slint!(export { MainWindow } from "ui/app-window.slint";);

/// Application state management
#[derive(Debug)]
struct AppState {
    /// Currently loaded taxonomy
    taxonomy: Option<HybridTaxonomy>,
    /// Path to current file
    current_file: Option<PathBuf>,
    /// Whether there are unsaved changes
    dirty: bool,
    /// Currently selected hierarchy node (species name)
    selected_hierarchy_node: Option<String>,
    /// Currently selected item index
    selected_item: Option<usize>,
    /// Active filters
    filters: Filters,
}

impl AppState {
    fn new() -> Self {
        Self {
            taxonomy: None,
            current_file: None,
            dirty: false,
            selected_hierarchy_node: None,
            selected_item: None,
            filters: Filters {
                genera: Vec::new(),
                facets: std::collections::HashMap::new(),
            },
        }
    }

    /// Load a taxonomy from file
    fn load_from_file(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let taxonomy = load_taxonomy(&path)?;

        // Validate the taxonomy
        validate_taxonomy(&taxonomy).map_err(|errors| {
            format!("Validation failed:\n{}", errors.join("\n"))
        })?;

        self.taxonomy = Some(taxonomy);
        self.current_file = Some(path);
        self.dirty = false;
        self.selected_hierarchy_node = None;
        self.selected_item = None;

        Ok(())
    }

    /// Save taxonomy to current file
    fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref taxonomy) = self.taxonomy {
            if let Some(ref path) = self.current_file {
                save_taxonomy(taxonomy, path)?;
                self.dirty = false;
                Ok(())
            } else {
                Err("No file path set".into())
            }
        } else {
            Err("No taxonomy to save".into())
        }
    }

    /// Save taxonomy to a new file
    fn save_as(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref taxonomy) = self.taxonomy {
            save_taxonomy(taxonomy, &path)?;
            self.current_file = Some(path);
            self.dirty = false;
            Ok(())
        } else {
            Err("No taxonomy to save".into())
        }
    }

    /// Create a new empty taxonomy
    fn create_new(&mut self) {
        let new_taxonomy = HybridTaxonomy {
            taxonomy_description: Some("New Taxonomy".to_string()),
            classical_hierarchy: ClassicalHierarchy {
                root: "Root".to_string(),
                children: None,
            },
            faceted_dimensions: std::collections::HashMap::from([
                ("category".to_string(), vec!["uncategorized".to_string()]),
            ]),
            example_items: Some(Vec::new()),
            extra: std::collections::HashMap::new(),
        };

        self.taxonomy = Some(new_taxonomy);
        self.current_file = None;
        self.dirty = true;
        self.selected_hierarchy_node = None;
        self.selected_item = None;
    }

    /// Mark state as modified
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Get window title with file name and dirty indicator
    fn get_window_title(&self) -> String {
        let file_name = self.current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        let dirty_marker = if self.dirty { "*" } else { "" };

        format!("Taxonomy Studio - {}{}", file_name, dirty_marker)
    }
}

pub fn main() {
    let main_window = MainWindow::new().unwrap();
    let state = Rc::new(RefCell::new(AppState::new()));

    // Set initial window title
    main_window.set_window_title(SharedString::from("Taxonomy Studio - No file loaded"));

    // Item selection handler
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_item_selected(move |index| {
            let main_window = main_window_weak.unwrap();
            let state_borrow = state.borrow();

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
        let state = state.clone();

        main_window.on_file_open(move || {
            let main_window = main_window_weak.unwrap();
            let state = state.clone();

            slint::spawn_local(async move {
                if let Some(file) = rfd::AsyncFileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_title("Open Taxonomy File")
                    .pick_file()
                    .await
                {
                    let path = file.path().to_path_buf();

                    // Load the file (borrow mutably, then drop the borrow)
                    let load_result = state.borrow_mut().load_from_file(path);

                    match load_result {
                        Ok(_) => {
                            // Update window title (borrow immutably)
                            let title = state.borrow().get_window_title();
                            main_window.set_window_title(SharedString::from(title));

                            // Update UI with loaded data (borrow immutably)
                            update_ui_from_state(&main_window, &state.borrow());

                            main_window.set_status_message(SharedString::from("File loaded successfully"));
                        }
                        Err(e) => {
                            main_window.set_status_message(SharedString::from(
                                format!("Error loading file: {}", e)
                            ));
                        }
                    }
                }
            }).unwrap();
        });
    }

    // File -> Save
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_file_save(move || {
            let main_window = main_window_weak.unwrap();

            let save_result = state.borrow_mut().save();

            match save_result {
                Ok(_) => {
                    let title = state.borrow().get_window_title();
                    main_window.set_window_title(SharedString::from(title));
                    main_window.set_status_message(SharedString::from("File saved successfully"));
                }
                Err(e) => {
                    main_window.set_status_message(SharedString::from(
                        format!("Error saving file: {}", e)
                    ));
                }
            }
        });
    }

    // File -> Save As
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_file_save_as(move || {
            let main_window = main_window_weak.unwrap();
            let state = state.clone();

            slint::spawn_local(async move {
                if let Some(file) = rfd::AsyncFileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_title("Save Taxonomy As")
                    .save_file()
                    .await
                {
                    let path = file.path().to_path_buf();

                    let save_result = state.borrow_mut().save_as(path);

                    match save_result {
                        Ok(_) => {
                            let title = state.borrow().get_window_title();
                            main_window.set_window_title(SharedString::from(title));
                            main_window.set_status_message(SharedString::from("File saved successfully"));
                        }
                        Err(e) => {
                            main_window.set_status_message(SharedString::from(
                                format!("Error saving file: {}", e)
                            ));
                        }
                    }
                }
            }).unwrap();
        });
    }

    // File -> New
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_file_new(move || {
            let main_window = main_window_weak.unwrap();

            // Create new (drops mutable borrow immediately)
            state.borrow_mut().create_new();

            // Now borrow immutably
            let title = state.borrow().get_window_title();
            main_window.set_window_title(SharedString::from(title));

            update_ui_from_state(&main_window, &state.borrow());

            main_window.set_status_message(SharedString::from("New taxonomy created"));
        });
    }

    // Sort -> By Name
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_sort_by_name(move || {
            let main_window = main_window_weak.unwrap();
            let state_borrow = state.borrow();

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
                    main_window.set_status_message(SharedString::from("Items sorted by name"));
                }
            }
        });
    }

    // Apply Filters
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

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

            // Get the facet filter text
            let facet_text = main_window.get_facet_filter_text().to_string();

            // Parse facet filters (format: "name=value, name2=value2")
            let mut facet_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
            for facet_str in facet_text.split(',') {
                if let Some((key, value)) = facet_str.split_once('=') {
                    facet_map
                        .entry(key.trim().to_string())
                        .or_insert_with(Vec::new)
                        .push(value.trim().to_string());
                }
            }

            // Update state filters
            {
                let mut state_mut = state.borrow_mut();
                state_mut.filters.genera = genera.clone();
                state_mut.filters.facets = facet_map.clone();
            }

            // Apply filters
            let state_borrow = state.borrow();
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

                    main_window.set_status_message(SharedString::from(
                        format!("Filters applied: {} items match", filtered_items.len())
                    ));
                }
            }
        });
    }

    // Clear Filters
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_clear_filters(move || {
            let main_window = main_window_weak.unwrap();

            // Clear filter inputs
            main_window.set_genus_filter_text(SharedString::from(""));
            main_window.set_facet_filter_text(SharedString::from(""));
            main_window.set_active_filters_text(SharedString::from(""));

            // Clear state filters
            state.borrow_mut().filters = Filters {
                genera: Vec::new(),
                facets: std::collections::HashMap::new(),
            };

            // Reset UI to show all items
            update_ui_from_state(&main_window, &state.borrow());

            main_window.set_status_message(SharedString::from("Filters cleared"));
        });
    }

    // Start Edit
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_start_edit(move || {
            let main_window = main_window_weak.unwrap();
            let state_borrow = state.borrow();

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
                        main_window.set_status_message(SharedString::from("Editing item..."));
                    }
                }
            }
        });
    }

    // Save Edit
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_save_edit(move || {
            let main_window = main_window_weak.unwrap();

            // Get edited values
            let new_name = main_window.get_edit_item_name().to_string();
            let new_path = main_window.get_edit_item_path().to_string();
            let facet_inputs = main_window.get_edit_facet_inputs();

            // Validate inputs
            if new_name.trim().is_empty() {
                main_window.set_validation_error(SharedString::from("Name cannot be empty"));
                return;
            }

            // Parse classification path (comma-separated)
            let classical_path: Vec<String> = new_path
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if classical_path.is_empty() {
                main_window.set_validation_error(SharedString::from("Classification path cannot be empty"));
                return;
            }

            // Collect facets from inputs
            let mut facets_map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
            for facet_input in facet_inputs.iter() {
                let value = facet_input.value.to_string();
                if !value.trim().is_empty() {
                    facets_map.insert(
                        facet_input.name.to_string(),
                        serde_json::Value::String(value.trim().to_string())
                    );
                }
            }

            // Update the item in the taxonomy
            let mut state_mut = state.borrow_mut();
            if let Some(ref mut taxonomy) = state_mut.taxonomy {
                if let Some(ref mut items) = taxonomy.example_items {
                    let selected_idx = main_window.get_selected_item_index();
                    if selected_idx >= 0 && (selected_idx as usize) < items.len() {
                        let item = &mut items[selected_idx as usize];
                        item.name = new_name.clone();
                        item.classical_path = classical_path;
                        item.facets = facets_map;

                        // Mark as dirty
                        state_mut.mark_dirty();

                        // Exit edit mode
                        drop(state_mut);
                        main_window.set_is_editing(false);

                        // Update window title
                        let title = state.borrow().get_window_title();
                        main_window.set_window_title(SharedString::from(title));

                        // Refresh the UI
                        update_ui_from_state(&main_window, &state.borrow());

                        // Re-select the edited item
                        main_window.set_selected_item_index(selected_idx);

                        // Trigger item selection to update details panel
                        let state_borrow = state.borrow();
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

                        main_window.set_status_message(SharedString::from("Item saved successfully"));
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
            main_window.set_status_message(SharedString::from("Edit cancelled"));
        });
    }

    // Start Create Item
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_start_create_item(move || {
            let main_window = main_window_weak.unwrap();
            let state_borrow = state.borrow();

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
            main_window.set_status_message(SharedString::from("Creating new item..."));
        });
    }

    // Save New Item
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_save_new_item(move || {
            let main_window = main_window_weak.unwrap();

            // Get form values
            let new_name = main_window.get_new_item_name().to_string();
            let new_path = main_window.get_new_item_path().to_string();
            let facet_inputs = main_window.get_create_facet_inputs();

            // Validate inputs
            if new_name.trim().is_empty() {
                main_window.set_validation_error(SharedString::from("Name cannot be empty"));
                return;
            }

            // Parse classification path (comma-separated)
            let classical_path: Vec<String> = new_path
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if classical_path.is_empty() {
                main_window.set_validation_error(SharedString::from("Classification path cannot be empty"));
                return;
            }

            // Collect facets from inputs
            let mut facets_map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
            for facet_input in facet_inputs.iter() {
                let value = facet_input.value.to_string();
                if !value.trim().is_empty() {
                    facets_map.insert(
                        facet_input.name.to_string(),
                        serde_json::Value::String(value.trim().to_string())
                    );
                }
            }

            // Create new item
            let new_item = Item {
                name: new_name.clone(),
                classical_path,
                facets: facets_map,
                extra: std::collections::HashMap::new(),
            };

            // Add to taxonomy
            let mut state_mut = state.borrow_mut();
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

                // Update window title
                let title = state.borrow().get_window_title();
                main_window.set_window_title(SharedString::from(title));

                // Refresh the UI
                update_ui_from_state(&main_window, &state.borrow());

                main_window.set_status_message(SharedString::from(
                    format!("Item '{}' created successfully", new_name)
                ));
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
            main_window.set_status_message(SharedString::from("Create cancelled"));
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
        let state = state.clone();

        main_window.on_delete_item(move || {
            let main_window = main_window_weak.unwrap();

            let selected_idx = main_window.get_selected_item_index();
            if selected_idx < 0 {
                return;
            }

            // Get item name for confirmation message
            let item_name = {
                let state_borrow = state.borrow();
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
            let mut state_mut = state.borrow_mut();
            if let Some(ref mut taxonomy) = state_mut.taxonomy {
                if let Some(ref mut items) = taxonomy.example_items {
                    if (selected_idx as usize) < items.len() {
                        items.remove(selected_idx as usize);

                        // Mark as dirty
                        state_mut.mark_dirty();

                        // Exit and update
                        drop(state_mut);

                        // Update window title
                        let title = state.borrow().get_window_title();
                        main_window.set_window_title(SharedString::from(title));

                        // Refresh the UI
                        update_ui_from_state(&main_window, &state.borrow());

                        main_window.set_status_message(SharedString::from(
                            format!("Item '{}' deleted", item_name)
                        ));
                    }
                }
            }
        });
    }

    main_window.run().unwrap();
}

/// Update the UI from the current application state
fn update_ui_from_state(main_window: &MainWindow, state: &AppState) {
    // Clear selected item
    main_window.set_selected_item_index(-1);
    main_window.set_selected_item_name(SharedString::from(""));
    main_window.set_selected_item_path(SharedString::from(""));
    main_window.set_selected_item_facets(SharedString::from(""));

    if let Some(ref taxonomy) = state.taxonomy {
        // Update taxonomy description
        let description = taxonomy.taxonomy_description
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("");

        eprintln!("DEBUG: Setting taxonomy description: '{}'", description);
        main_window.set_taxonomy_description(SharedString::from(description));

        // Update hierarchy root
        eprintln!("DEBUG: Setting hierarchy root: '{}'", taxonomy.classical_hierarchy.root);
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
                items.iter().map(|item| {
                    StandardListViewItem::from(SharedString::from(&item.name))
                }).collect::<Vec<_>>()
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

/// Format facets HashMap into a displayable string
fn format_facets(facets: &std::collections::HashMap<String, serde_json::Value>) -> String {
    let mut facet_lines: Vec<String> = facets
        .iter()
        .map(|(key, value)| {
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Array(arr) => {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
                _ => value.to_string(),
            };
            format!("• {}: {}", key, value_str)
        })
        .collect();

    facet_lines.sort();
    facet_lines.join("\n")
}

/// Format facet dimensions into a displayable string
fn format_facet_dimensions(dimensions: &std::collections::HashMap<String, Vec<String>>) -> String {
    let mut dim_lines: Vec<String> = dimensions
        .iter()
        .map(|(key, values)| {
            format!("{}: {}", key, values.join(", "))
        })
        .collect();

    dim_lines.sort();
    dim_lines.join(" • ")
}

/// Create facet input list from taxonomy dimensions and current facet values
fn create_facet_inputs(
    dimensions: &std::collections::HashMap<String, Vec<String>>,
    facets: &std::collections::HashMap<String, serde_json::Value>
) -> Vec<FacetInput> {
    let mut facet_inputs: Vec<FacetInput> = dimensions
        .keys()
        .map(|key| {
            let value = facets.get(key).map(|v| {
                match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Array(arr) => {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                    _ => v.to_string(),
                }
            }).unwrap_or_default();

            FacetInput {
                name: SharedString::from(key.as_str()),
                value: SharedString::from(value),
            }
        })
        .collect();

    facet_inputs.sort_by(|a, b| a.name.cmp(&b.name));
    facet_inputs
}

/// Flatten hierarchy tree into a list of tree nodes with indentation levels
fn flatten_hierarchy(hierarchy: &ClassicalHierarchy) -> Vec<TreeNode> {
    let mut nodes = Vec::new();

    if let Some(ref children) = hierarchy.children {
        for child in children {
            flatten_node(child, 0, &mut nodes);
        }
    }

    nodes
}

/// Recursively flatten a hierarchy node and its children
fn flatten_node(node: &HierarchyNode, indent_level: i32, nodes: &mut Vec<TreeNode>) {
    // Format: "species (differentia)"
    let label = if node.differentia.is_empty() {
        node.species.clone()
    } else {
        format!("{} ({})", node.species, node.differentia)
    };

    nodes.push(TreeNode {
        label: SharedString::from(label),
        indent_level,
    });

    if let Some(ref children) = node.children {
        for child in children {
            flatten_node(child, indent_level + 1, nodes);
        }
    }
}
