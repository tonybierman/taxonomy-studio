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

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassicalHierarchy {
    pub root: String,
    pub children: Option<Vec<HierarchyNode>>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug)]
pub struct Filters {
    pub genera: Vec<String>,
    pub facets: HashMap<String, Vec<String>>,
}
