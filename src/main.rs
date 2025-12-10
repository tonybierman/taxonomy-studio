use slint::{ComponentHandle, SharedString, StandardListViewItem, VecModel};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use taxman_core::*;

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

        format!("Taxman - {}{}", file_name, dirty_marker)
    }
}

pub fn main() {
    let main_window = MainWindow::new().unwrap();
    let state = Rc::new(RefCell::new(AppState::new()));

    // Set initial window title
    main_window.set_window_title(SharedString::from("Taxman - No file loaded"));

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

                    // Sort by name using taxman-core
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

                        // Format facets for editing (key=value, comma-separated)
                        let facets_edit = format_facets_for_edit(&item.facets);
                        main_window.set_edit_item_facets(SharedString::from(facets_edit));

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
            let new_facets = main_window.get_edit_item_facets().to_string();

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

            // Parse facets (key=value pairs)
            let mut facets_map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
            for facet_str in new_facets.split(',') {
                if facet_str.trim().is_empty() {
                    continue;
                }
                if let Some((key, value)) = facet_str.split_once('=') {
                    facets_map.insert(
                        key.trim().to_string(),
                        serde_json::Value::String(value.trim().to_string())
                    );
                } else {
                    main_window.set_validation_error(SharedString::from(
                        format!("Invalid facet format: '{}'. Use key=value", facet_str)
                    ));
                    return;
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

/// Format facets for editing (key=value, comma-separated)
fn format_facets_for_edit(facets: &std::collections::HashMap<String, serde_json::Value>) -> String {
    let mut facet_pairs: Vec<String> = facets
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
            format!("{}={}", key, value_str)
        })
        .collect();

    facet_pairs.sort();
    facet_pairs.join(", ")
}
