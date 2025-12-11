use std::path::PathBuf;
use taxstud_core::*;

/// Application state management - Domain state only
/// This struct contains only domain/business logic state
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
}

impl AppState {
    pub fn new() -> Self {
        Self {
            taxonomy: None,
            current_file: None,
            dirty: false,
            selected_item: None,
            filters: Filters {
                genera: Vec::new(),
                facets: std::collections::HashMap::new(),
            },
        }
    }

    /// Load a taxonomy from file
    pub fn load_from_file(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let taxonomy = load_taxonomy(&path)?;

        // Validate the taxonomy
        validate_taxonomy(&taxonomy)
            .map_err(|errors| format!("Validation failed:\n{}", errors.join("\n")))?;

        self.taxonomy = Some(taxonomy);
        self.current_file = Some(path);
        self.dirty = false;
        self.selected_item = None;

        Ok(())
    }

    /// Save taxonomy to current file
    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
    pub fn save_as(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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
    pub fn create_new(&mut self) {
        let new_taxonomy = HybridTaxonomy {
            taxonomy_description: Some("New Taxonomy".to_string()),
            classical_hierarchy: ClassicalHierarchy {
                root: "Root".to_string(),
                children: None,
            },
            faceted_dimensions: std::collections::HashMap::from([(
                "category".to_string(),
                vec!["uncategorized".to_string()],
            )]),
            example_items: Some(Vec::new()),
            extra: std::collections::HashMap::new(),
        };

        self.taxonomy = Some(new_taxonomy);
        self.current_file = None;
        self.dirty = true;
        self.selected_item = None;
    }

    /// Mark state as modified
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Get window title with file name and dirty indicator
    pub fn get_window_title(&self) -> String {
        let file_name = self
            .current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        let dirty_marker = if self.dirty { "*" } else { "" };

        format!("Taxonomy Studio - {}{}", file_name, dirty_marker)
    }

    /// Get a reference to an item by index
    #[allow(dead_code)]
    pub fn get_item(&self, index: i32) -> Option<&Item> {
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
    pub fn get_item_mut(&mut self, index: i32) -> Option<&mut Item> {
        if index < 0 {
            return None;
        }

        self.taxonomy
            .as_mut()
            .and_then(|t| t.example_items.as_mut())
            .and_then(|items| items.get_mut(index as usize))
    }
}
