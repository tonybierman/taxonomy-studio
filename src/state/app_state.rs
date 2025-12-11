use std::collections::HashMap;
use std::path::PathBuf;
use taxstud_core::*;

/// Application state management - Domain state only
/// This struct contains only domain/business logic state
#[derive(Debug)]
pub struct AppState {
    /// Currently loaded schema (classical hierarchy + facet dimensions)
    pub schema: Option<TaxonomySchema>,
    /// Currently loaded data (items only)
    pub data: Option<TaxonomyData>,
    /// Path to current data file
    pub current_file: Option<PathBuf>,
    /// Path to current schema file (for reference)
    pub schema_file: Option<PathBuf>,
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
            schema: None,
            data: None,
            current_file: None,
            schema_file: None,
            dirty: false,
            selected_item: None,
            filters: Filters {
                genera: Vec::new(),
                facets: HashMap::new(),
            },
        }
    }

    /// Load a data file with its schema
    pub fn load_from_file(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let (data, schema) = load_data_with_auto_schema(&path)?;

        self.data = Some(data.clone());
        self.schema = Some(schema);
        self.current_file = Some(path.clone());

        // Reconstruct schema_file path
        let data_dir = path.parent().unwrap();
        self.schema_file = Some(data_dir.join(&data.schema));

        self.dirty = false;
        self.selected_item = None;

        Ok(())
    }

    /// Save data to current file
    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref data) = self.data {
            if let Some(ref path) = self.current_file {
                save_data(data, path)?;
                self.dirty = false;
                Ok(())
            } else {
                Err("No file path set".into())
            }
        } else {
            Err("No data to save".into())
        }
    }

    /// Save data to a new file
    pub fn save_as(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref data) = self.data {
            save_data(data, &path)?;
            self.current_file = Some(path);
            self.dirty = false;
            Ok(())
        } else {
            Err("No data to save".into())
        }
    }

    /// Create a new empty taxonomy with default schema
    pub fn create_new(&mut self) {
        let default_schema = TaxonomySchema {
            schema_id: "default".to_string(),
            title: "Default Schema".to_string(),
            description: Some("Default taxonomy schema".to_string()),
            classical_hierarchy: ClassicalHierarchy {
                root: "Root".to_string(),
                children: None,
            },
            faceted_dimensions: HashMap::from([(
                "category".to_string(),
                vec!["uncategorized".to_string()],
            )]),
            json_schema: None,
        };

        let default_data = TaxonomyData {
            schema: "schema.json".to_string(),
            items: Vec::new(),
            extra: HashMap::new(),
        };

        self.schema = Some(default_schema);
        self.data = Some(default_data);
        self.current_file = None;
        self.schema_file = None;
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

    /// Get a reference to items
    pub fn get_items(&self) -> Option<&Vec<Item>> {
        self.data.as_ref().map(|d| &d.items)
    }

    /// Get a mutable reference to items
    pub fn get_items_mut(&mut self) -> Option<&mut Vec<Item>> {
        self.data.as_mut().map(|d| &mut d.items)
    }

    /// Get a reference to the classical hierarchy
    pub fn get_classical_hierarchy(&self) -> Option<&ClassicalHierarchy> {
        self.schema.as_ref().map(|s| &s.classical_hierarchy)
    }

    /// Get a reference to faceted dimensions
    pub fn get_faceted_dimensions(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.schema.as_ref().map(|s| &s.faceted_dimensions)
    }

    /// Get a reference to an item by index
    #[allow(dead_code)]
    pub fn get_item(&self, index: i32) -> Option<&Item> {
        if index < 0 {
            return None;
        }

        self.data
            .as_ref()
            .and_then(|d| d.items.get(index as usize))
    }

    /// Get a mutable reference to an item by index
    #[allow(dead_code)]
    pub fn get_item_mut(&mut self, index: i32) -> Option<&mut Item> {
        if index < 0 {
            return None;
        }

        self.data
            .as_mut()
            .and_then(|d| d.items.get_mut(index as usize))
    }
}
