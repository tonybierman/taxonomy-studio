use crate::models::{HybridTaxonomy, HierarchyNode, Item};
use std::collections::{HashMap, HashSet};

/// Validate the hybrid taxonomy schema
/// Returns Ok(()) if valid, or Err(Vec<String>) with validation errors
pub fn validate_taxonomy(taxonomy: &HybridTaxonomy) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate classical hierarchy
    if taxonomy.classical_hierarchy.root.trim().is_empty() {
        errors.push("Classical hierarchy root cannot be empty".to_string());
    }

    if let Some(children) = &taxonomy.classical_hierarchy.children {
        validate_hierarchy_nodes(children, &taxonomy.classical_hierarchy.root, &mut errors);
    }

    // Validate faceted dimensions
    if taxonomy.faceted_dimensions.is_empty() {
        errors.push("At least one faceted dimension must be defined".to_string());
    }

    for (facet_name, values) in &taxonomy.faceted_dimensions {
        if facet_name.trim().is_empty() {
            errors.push("Facet names cannot be empty".to_string());
        }

        if values.is_empty() {
            errors.push(format!("Facet '{}' must have at least one value", facet_name));
        }

        // Check for duplicate values within a facet
        let mut seen = HashSet::new();
        for value in values {
            if value.trim().is_empty() {
                errors.push(format!("Facet '{}' contains empty value", facet_name));
            }
            if !seen.insert(value) {
                errors.push(format!("Facet '{}' has duplicate value: '{}'", facet_name, value));
            }
        }
    }

    // Validate example items
    if let Some(items) = &taxonomy.example_items {
        validate_items(items, taxonomy, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_hierarchy_nodes(nodes: &[HierarchyNode], parent: &str, errors: &mut Vec<String>) {
    for node in nodes {
        // Validate required fields are not empty
        if node.genus.trim().is_empty() {
            errors.push("Hierarchy node genus cannot be empty".to_string());
        }
        if node.species.trim().is_empty() {
            errors.push("Hierarchy node species cannot be empty".to_string());
        }
        if node.differentia.trim().is_empty() {
            errors.push(format!("Species '{}' must have non-empty differentia", node.species));
        }

        // Validate genus matches parent
        if node.genus != parent {
            errors.push(format!(
                "Species '{}' has genus '{}', expected '{}' (parent species)",
                node.species, node.genus, parent
            ));
        }

        // Recursively validate children
        if let Some(children) = &node.children {
            validate_hierarchy_nodes(children, &node.species, errors);
        }
    }
}

pub fn validate_items(items: &[Item], taxonomy: &HybridTaxonomy, errors: &mut Vec<String>) {
    let defined_facets: HashSet<_> = taxonomy.faceted_dimensions.keys().collect();
    let mut item_names = HashSet::new();

    for (idx, item) in items.iter().enumerate() {
        let item_ref = format!("Item #{} ('{}')", idx + 1, item.name);

        // Validate name is not empty
        if item.name.trim().is_empty() {
            errors.push(format!("{}: name cannot be empty", item_ref));
        }

        // Check for duplicate names
        if !item_names.insert(&item.name) {
            errors.push(format!("{}: duplicate item name", item_ref));
        }

        // Validate classical path
        if item.classical_path.is_empty() {
            errors.push(format!("{}: classical_path cannot be empty", item_ref));
        } else {
            // First element should be root
            if item.classical_path[0] != taxonomy.classical_hierarchy.root {
                errors.push(format!(
                    "{}: classical_path must start with root '{}', found '{}'",
                    item_ref, taxonomy.classical_hierarchy.root, item.classical_path[0]
                ));
            }

            // Validate path forms valid parent-child relationships
            validate_classical_path(item, taxonomy, &item_ref, errors);
        }

        // Validate facets
        if item.facets.is_empty() {
            errors.push(format!("{}: must have at least one facet", item_ref));
        }

        for (facet_name, facet_value) in &item.facets {
            // Check facet is defined in taxonomy
            if !defined_facets.contains(facet_name) {
                errors.push(format!(
                    "{}: uses undefined facet '{}'",
                    item_ref, facet_name
                ));
                continue;
            }

            // Get allowed values for this facet
            if let Some(allowed_values) = taxonomy.faceted_dimensions.get(facet_name) {
                match facet_value {
                    serde_json::Value::String(s) => {
                        if !allowed_values.contains(s) {
                            errors.push(format!(
                                "{}: facet '{}' has invalid value '{}' (not in allowed values)",
                                item_ref, facet_name, s
                            ));
                        }
                    }
                    serde_json::Value::Array(arr) => {
                        if arr.is_empty() {
                            errors.push(format!(
                                "{}: facet '{}' has empty array",
                                item_ref, facet_name
                            ));
                        }
                        for val in arr {
                            if let Some(s) = val.as_str() {
                                if !allowed_values.contains(&s.to_string()) {
                                    errors.push(format!(
                                        "{}: facet '{}' has invalid value '{}' (not in allowed values)",
                                        item_ref, facet_name, s
                                    ));
                                }
                            } else {
                                errors.push(format!(
                                    "{}: facet '{}' array contains non-string value",
                                    item_ref, facet_name
                                ));
                            }
                        }
                    }
                    _ => {
                        errors.push(format!(
                            "{}: facet '{}' must be a string or array of strings",
                            item_ref, facet_name
                        ));
                    }
                }
            }
        }
    }
}

pub fn validate_classical_path(item: &Item, taxonomy: &HybridTaxonomy, item_ref: &str, errors: &mut Vec<String>) {
    if item.classical_path.len() < 2 {
        return; // Root only is valid
    }

    // Build a map of all valid parent-child relationships
    let mut valid_paths = HashMap::new();
    build_valid_paths(&taxonomy.classical_hierarchy.root, &taxonomy.classical_hierarchy.children, &mut valid_paths);

    // Validate each step in the path
    for i in 0..item.classical_path.len() - 1 {
        let parent = &item.classical_path[i];
        let child = &item.classical_path[i + 1];

        if let Some(valid_children) = valid_paths.get(parent) {
            if !valid_children.contains(child) {
                errors.push(format!(
                    "{}: invalid classical_path - '{}' is not a valid child of '{}'",
                    item_ref, child, parent
                ));
            }
        } else {
            errors.push(format!(
                "{}: invalid classical_path - '{}' has no defined children",
                item_ref, parent
            ));
        }
    }
}

fn build_valid_paths(
    parent: &str,
    children: &Option<Vec<HierarchyNode>>,
    map: &mut HashMap<String, Vec<String>>
) {
    if let Some(nodes) = children {
        let mut child_names = Vec::new();
        for node in nodes {
            child_names.push(node.species.clone());
            build_valid_paths(&node.species, &node.children, map);
        }
        if !child_names.is_empty() {
            map.insert(parent.to_string(), child_names);
        }
    }
}
