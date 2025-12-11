use crate::models::{Filters, Item};
use std::collections::HashMap;

/// Parse facet filter strings in the format "key=value" into a filter map
/// Multiple values for the same key are collected into a vector
pub fn parse_facet_filters(facet_strings: &[String]) -> HashMap<String, Vec<String>> {
    let mut facet_map = HashMap::new();

    for facet_str in facet_strings {
        if let Some((key, value)) = facet_str.split_once('=') {
            facet_map
                .entry(key.trim().to_string())
                .or_insert_with(Vec::new)
                .push(value.trim().to_string());
        }
    }

    facet_map
}

/// Apply filters to a list of items, returning only those that match
pub fn apply_filters(items: &[Item], filters: &Filters) -> Vec<Item> {
    items
        .iter()
        .filter(|item| matches_filters(item, filters))
        .cloned()
        .collect()
}

/// Check if an item matches the given filters
/// AND logic between different filter types (genus AND facets)
pub fn matches_filters(item: &Item, filters: &Filters) -> bool {
    // Check genus filter (OR within genera)
    if !filters.genera.is_empty() {
        let matches_genus = filters.genera.iter().any(|genus| {
            item.classical_path
                .iter()
                .any(|path_item| path_item == genus)
        });

        if !matches_genus {
            return false;
        }
    }

    // Check facet filters (AND between different facet names, OR within same facet name)
    for (facet_name, required_values) in &filters.facets {
        let item_values = item.get_facet_as_vec(facet_name);

        if item_values.is_empty() {
            // Item doesn't have this facet at all
            return false;
        }

        // Check if any required value matches any item value (OR within same facet name)
        let matches = required_values
            .iter()
            .any(|rv| item_values.iter().any(|iv| iv == rv));

        if !matches {
            return false;
        }
    }

    true
}

/// Check if filters are empty
pub fn has_filters(filters: &Filters) -> bool {
    !filters.genera.is_empty() || !filters.facets.is_empty()
}
