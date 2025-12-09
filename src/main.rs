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

                    match state.borrow_mut().load_from_file(path) {
                        Ok(_) => {
                            // Update window title
                            let title = state.borrow().get_window_title();
                            main_window.set_window_title(SharedString::from(title));

                            // Update UI with loaded data
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

            match state.borrow_mut().save() {
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

                    match state.borrow_mut().save_as(path) {
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

            state.borrow_mut().create_new();

            let title = state.borrow().get_window_title();
            main_window.set_window_title(SharedString::from(title));

            update_ui_from_state(&main_window, &state.borrow());

            main_window.set_status_message(SharedString::from("New taxonomy created"));
        });
    }

    main_window.run().unwrap();
}

/// Update the UI from the current application state
fn update_ui_from_state(main_window: &MainWindow, state: &AppState) {
    if let Some(ref taxonomy) = state.taxonomy {
        // Update taxonomy description
        let description = taxonomy.taxonomy_description
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("");
        main_window.set_taxonomy_description(SharedString::from(description));

        // Update hierarchy root
        main_window.set_hierarchy_root(SharedString::from(&taxonomy.classical_hierarchy.root));

        // Update items list
        if let Some(ref items) = taxonomy.example_items {
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
        let empty_model = Rc::new(VecModel::<StandardListViewItem>::default());
        main_window.set_items_list(empty_model.into());
    }
}
