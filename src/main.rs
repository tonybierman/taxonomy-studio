mod errors;
mod state;
mod ui;

use clap::Parser;
use slint::{ComponentHandle, Model, SharedString, StandardListViewItem, VecModel};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use taxstud_core::*;

use errors::{map_file_load_error, map_file_save_error, map_revert_error};
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

/// Represents a pending action waiting for confirmation
#[derive(Debug, Clone)]
pub enum PendingAction {
    Open,
    New,
}

/// Represents an action for simple confirmation dialog
#[derive(Debug, Clone)]
pub enum SimpleConfirmationAction {
    Revert,
}

/// Application state management
#[derive(Debug)]
pub struct AppState {
    /// Currently loaded taxonomy
    pub taxonomy: Option<HybridTaxonomy>,
    /// Path to current file
    pub current_file: Option<PathBuf>,
    /// Whether there are unsaved changes
    pub dirty: bool,
    /// Currently selected item index
    pub selected_item: Option<usize>,
    /// Active filters
    pub filters: Filters,
    /// Pending action awaiting user confirmation
    pub pending_action: Option<PendingAction>,
    /// Simple confirmation action
    pub simple_confirmation_action: Option<SimpleConfirmationAction>,
}

impl AppState {
    fn new() -> Self {
        Self {
            taxonomy: None,
            current_file: None,
            dirty: false,
            selected_item: None,
            filters: Filters {
                genera: Vec::new(),
                facets: std::collections::HashMap::new(),
            },
            pending_action: None,
            simple_confirmation_action: None,
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

    /// Get a reference to an item by index
    #[allow(dead_code)]
    fn get_item(&self, index: i32) -> Option<&Item> {
        if index < 0 {
            return None;
        }

        self.taxonomy
            .as_ref()
            .and_then(|t| t.example_items.as_ref())
            .and_then(|items| items.get(index as usize))
    }

    /// Get a mutable reference to an item by index
    #[allow(dead_code)]
    fn get_item_mut(&mut self, index: i32) -> Option<&mut Item> {
        if index < 0 {
            return None;
        }

        self.taxonomy
            .as_mut()
            .and_then(|t| t.example_items.as_mut())
            .and_then(|items| items.get_mut(index as usize))
    }
}

pub fn main() {
    let args = Args::parse();

    let main_window = MainWindow::new().unwrap();
    let state = Rc::new(RefCell::new(AppState::new()));

    // Set initial window title
    main_window.set_window_title(SharedString::from("Taxonomy Studio - No file loaded"));

    // Load file from command line if provided
    if let Some(file_path) = args.file {
        let load_result = state.borrow_mut().load_from_file(file_path.clone());

        match load_result {
            Ok(_) => {
                // Update window title
                let title = state.borrow().get_window_title();
                main_window.set_window_title(SharedString::from(title));

                // Update UI with loaded data
                update_ui_from_state(&main_window, &state.borrow());

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

            // Check for unsaved changes
            if state.borrow().dirty {
                // Store pending action and show confirmation dialog
                state.borrow_mut().pending_action = Some(PendingAction::Open);
                show_confirmation(
                    &main_window,
                    "You have unsaved changes. Do you want to save before opening another file?",
                );
            } else {
                // No unsaved changes, proceed with open
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
                        let load_result = state.borrow_mut().load_from_file(path.clone());

                        match load_result {
                            Ok(_) => {
                                // Update window title (borrow immutably)
                                let title = state.borrow().get_window_title();
                                main_window.set_window_title(SharedString::from(title));

                                // Update UI with loaded data (borrow immutably)
                                update_ui_from_state(&main_window, &state.borrow());

                                set_status(&main_window, "File loaded successfully", StatusLevel::Success);
                            }
                            Err(e) => {
                                // Show enhanced error dialog using error mapper
                                let (title, message, details) = map_file_load_error(&*e, &path);
                                show_error(&main_window, title, message, details);
                            }
                        }
                    }
                }).unwrap();
            }
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
                    set_status(&main_window, "File saved successfully", StatusLevel::Success);
                }
                Err(e) => {
                    // Show enhanced error dialog using error mapper
                    let (title, message, details) = map_file_save_error(&*e, None);
                    show_error(&main_window, title, message, details);
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

                    let save_result = state.borrow_mut().save_as(path.clone());

                    match save_result {
                        Ok(_) => {
                            let title = state.borrow().get_window_title();
                            main_window.set_window_title(SharedString::from(title));
                            set_status(&main_window, "File saved successfully", StatusLevel::Success);
                        }
                        Err(e) => {
                            // Show enhanced error dialog using error mapper
                            let (title, message, details) = map_file_save_error(&*e, Some(&path));
                            show_error(&main_window, title, message, details);
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

            // Check for unsaved changes
            if state.borrow().dirty {
                // Store pending action and show confirmation dialog
                state.borrow_mut().pending_action = Some(PendingAction::New);
                show_confirmation(
                    &main_window,
                    "You have unsaved changes. Do you want to save before creating a new taxonomy?",
                );
            } else {
                // No unsaved changes, proceed with creating new taxonomy
                // Create new (drops mutable borrow immediately)
                state.borrow_mut().create_new();

                // Now borrow immutably
                let title = state.borrow().get_window_title();
                main_window.set_window_title(SharedString::from(title));

                update_ui_from_state(&main_window, &state.borrow());

                set_status(&main_window, "New taxonomy created", StatusLevel::Success);
            }
        });
    }

    // File -> Revert to Saved
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_file_revert(move || {
            let main_window = main_window_weak.unwrap();

            // Check if we have a file to revert to
            let can_revert = {
                let state_borrow = state.borrow();
                state_borrow.current_file.is_some() && state_borrow.dirty
            };

            if can_revert {
                // Show confirmation dialog
                state.borrow_mut().simple_confirmation_action = Some(SimpleConfirmationAction::Revert);
                show_simple_confirmation(
                    &main_window,
                    "Revert to Saved",
                    "Are you sure you want to revert to the last saved version? All unsaved changes will be lost.",
                    "Revert",
                );
            } else {
                // Either no file or no changes
                let state_borrow = state.borrow();
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
                    set_status(&main_window, "Items sorted by name", StatusLevel::Info);
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

                    set_status(&main_window, format!("Filters applied: {} items match", filtered_items.len()), StatusLevel::Info);
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

            set_status(&main_window, "Filters cleared", StatusLevel::Info);
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
                        set_status(&main_window, "Editing item...", StatusLevel::Info);
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
            set_status(&main_window, "Creating new item...", StatusLevel::Info);
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

                // Refresh UI and show success message
                refresh_ui_after_state_change(
                    &main_window,
                    &state,
                    &format!("Item '{}' created successfully", new_name),
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

                        // Refresh UI and show success message
                        refresh_ui_after_state_change(
                            &main_window,
                            &state,
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
        let state = state.clone();

        main_window.on_confirmation_save(move || {
            let main_window = main_window_weak.unwrap();

            // Save the file first
            let save_result = state.borrow_mut().save();

            match save_result {
                Ok(_) => {
                    // Update window title
                    let title = state.borrow().get_window_title();
                    main_window.set_window_title(SharedString::from(title));

                    // Hide confirmation dialog
                    hide_confirmation(&main_window);

                    // Now proceed with the pending action
                    if let Some(action) = state.borrow_mut().pending_action.take() {
                        match action {
                            PendingAction::Open => {
                                // Trigger file open
                                let state = state.clone();
                                slint::spawn_local(async move {
                                    if let Some(file) = rfd::AsyncFileDialog::new()
                                        .add_filter("JSON", &["json"])
                                        .set_title("Open Taxonomy File")
                                        .pick_file()
                                        .await
                                    {
                                        let path = file.path().to_path_buf();
                                        let load_result = state.borrow_mut().load_from_file(path.clone());

                                        match load_result {
                                            Ok(_) => {
                                                let title = state.borrow().get_window_title();
                                                main_window.set_window_title(SharedString::from(title));
                                                update_ui_from_state(&main_window, &state.borrow());
                                                set_status(&main_window, "File loaded successfully", StatusLevel::Success);
                                            }
                                            Err(e) => {
                                                let (title, message, details) = map_file_load_error(&*e, &path);
                                                show_error(&main_window, title, message, details);
                                            }
                                        }
                                    }
                                }).unwrap();
                            }
                            PendingAction::New => {
                                // Create new taxonomy
                                state.borrow_mut().create_new();
                                let title = state.borrow().get_window_title();
                                main_window.set_window_title(SharedString::from(title));
                                update_ui_from_state(&main_window, &state.borrow());
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
                    state.borrow_mut().pending_action = None;
                }
            }
        });
    }

    // Confirmation Dialog - Don't Save
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_confirmation_dont_save(move || {
            let main_window = main_window_weak.unwrap();

            // Hide confirmation dialog
            hide_confirmation(&main_window);

            // Proceed with the pending action without saving
            if let Some(action) = state.borrow_mut().pending_action.take() {
                match action {
                    PendingAction::Open => {
                        // Trigger file open
                        let state = state.clone();
                        slint::spawn_local(async move {
                            if let Some(file) = rfd::AsyncFileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_title("Open Taxonomy File")
                                .pick_file()
                                .await
                            {
                                let path = file.path().to_path_buf();
                                let load_result = state.borrow_mut().load_from_file(path.clone());

                                match load_result {
                                    Ok(_) => {
                                        let title = state.borrow().get_window_title();
                                        main_window.set_window_title(SharedString::from(title));
                                        update_ui_from_state(&main_window, &state.borrow());
                                        set_status(&main_window, "File loaded successfully", StatusLevel::Success);
                                    }
                                    Err(e) => {
                                        let (title, message, details) = map_file_load_error(&*e, &path);
                                        show_error(&main_window, title, message, details);
                                    }
                                }
                            }
                        }).unwrap();
                    }
                    PendingAction::New => {
                        // Create new taxonomy
                        state.borrow_mut().create_new();
                        let title = state.borrow().get_window_title();
                        main_window.set_window_title(SharedString::from(title));
                        update_ui_from_state(&main_window, &state.borrow());
                        set_status(&main_window, "New taxonomy created", StatusLevel::Success);
                    }
                }
            }
        });
    }

    // Confirmation Dialog - Cancel
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_confirmation_cancel(move || {
            let main_window = main_window_weak.unwrap();

            // Hide confirmation dialog
            hide_confirmation(&main_window);

            // Clear pending action
            state.borrow_mut().pending_action = None;

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
        let state = state.clone();

        main_window.on_simple_confirmation_ok(move || {
            let main_window = main_window_weak.unwrap();

            // Hide confirmation dialog
            hide_simple_confirmation(&main_window);

            // Get the action and drop the borrow immediately
            let action = state.borrow_mut().simple_confirmation_action.take();

            // Execute the action
            if let Some(action) = action {
                match action {
                    SimpleConfirmationAction::Revert => {
                        // Get the file path (borrow immutably, then drop)
                        let path = state.borrow().current_file.clone();

                        if let Some(file_path) = path {
                            // Now borrow mutably to reload
                            let load_result = state.borrow_mut().load_from_file(file_path.clone());

                            match load_result {
                                Ok(_) => {
                                    // Update window title
                                    let title = state.borrow().get_window_title();
                                    main_window.set_window_title(SharedString::from(title));

                                    // Update UI with loaded data
                                    update_ui_from_state(&main_window, &state.borrow());

                                    set_status(&main_window, "Reverted to saved version", StatusLevel::Success);
                                }
                                Err(e) => {
                                    // Show error using error mapper
                                    let (title, message, details) = map_revert_error(&*e, &file_path);
                                    show_error(&main_window, title, message, details);
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    // Simple Confirmation - Cancel
    {
        let main_window_weak = main_window.as_weak();
        let state = state.clone();

        main_window.on_simple_confirmation_cancel(move || {
            let main_window = main_window_weak.unwrap();

            // Hide confirmation dialog
            hide_simple_confirmation(&main_window);

            // Clear action
            state.borrow_mut().simple_confirmation_action = None;

            set_status(&main_window, "Action cancelled", StatusLevel::Info);
        });
    }

    main_window.run().unwrap();
}
