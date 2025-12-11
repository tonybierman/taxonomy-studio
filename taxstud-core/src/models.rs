use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct HybridTaxonomy {
    pub taxonomy_description: Option<String>,
    pub classical_hierarchy: ClassicalHierarchy,
    pub faceted_dimensions: HashMap<String, Vec<String>>,
    pub example_items: Option<Vec<Item>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassicalHierarchy {
    pub root: String,
    pub children: Option<Vec<HierarchyNode>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HierarchyNode {
    pub genus: String,
    pub species: String,
    pub differentia: String,
    pub children: Option<Vec<HierarchyNode>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub name: String,
    pub classical_path: Vec<String>,
    pub facets: HashMap<String, serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Item {
    /// Get a facet value as a string (handles both single values and arrays)
    /// For arrays, values are joined with ", "
    pub fn get_facet_as_string(&self, facet_name: &str) -> Option<String> {
        self.facets.get(facet_name).and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Array(arr) => {
                let values: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                if values.is_empty() {
                    None
                } else {
                    Some(values.join(", "))
                }
            }
            _ => None,
        })
    }

    /// Get facet values as a vector (always returns Vec, empty if not found)
    /// For single string values, returns a Vec with one element
    /// For arrays, extracts all string values
    pub fn get_facet_as_vec(&self, facet_name: &str) -> Vec<String> {
        self.facets
            .get(facet_name)
            .map(|v| match v {
                serde_json::Value::String(s) => vec![s.clone()],
                serde_json::Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect(),
                _ => vec![],
            })
            .unwrap_or_default()
    }
}

/// Schema definition - contains classical hierarchy and facet dimensions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxonomySchema {
    pub schema_id: String,
    pub title: String,
    pub description: Option<String>,
    pub classical_hierarchy: ClassicalHierarchy,
    pub faceted_dimensions: HashMap<String, Vec<String>>,
    /// Raw JSON Schema for validation (not serialized)
    #[serde(skip)]
    pub json_schema: Option<serde_json::Value>,
}

/// Data file - references schema and contains items only
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxonomyData {
    pub schema: String,
    pub items: Vec<Item>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub struct Filters {
    pub genera: Vec<String>,
    pub facets: HashMap<String, Vec<String>>,
}
