use crate::models::{Item, Filters};

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
            item.classical_path.iter().any(|path_item| path_item == genus)
        });

        if !matches_genus {
            return false;
        }
    }

    // Check facet filters (AND between different facet names, OR within same facet name)
    for (facet_name, required_values) in &filters.facets {
        if let Some(item_value) = item.facets.get(facet_name) {
            let matches = match item_value {
                serde_json::Value::String(s) => {
                    // OR within same facet name
                    required_values.iter().any(|rv| rv == s)
                }
                serde_json::Value::Array(arr) => {
                    // Check if any required value is in the array
                    required_values.iter().any(|rv| {
                        arr.iter().any(|v| {
                            if let Some(s) = v.as_str() {
                                s == rv
                            } else {
                                false
                            }
                        })
                    })
                }
                _ => false,
            };

            if !matches {
                return false;
            }
        } else {
            // Item doesn't have this facet at all
            return false;
        }
    }

    true
}

/// Check if filters are empty
pub fn has_filters(filters: &Filters) -> bool {
    !filters.genera.is_empty() || !filters.facets.is_empty()
}
