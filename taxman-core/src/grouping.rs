use crate::models::Item;
use std::collections::HashMap;

/// Group items by a facet name
/// Items with multiple values for the facet appear in multiple groups
/// Returns a HashMap where keys are group names and values are vectors of items
pub fn group_items_by_facet(items: &[Item], group_field: &str) -> HashMap<String, Vec<Item>> {
    let mut groups: HashMap<String, Vec<Item>> = HashMap::new();

    for item in items {
        if let Some(facet_value) = item.facets.get(group_field) {
            match facet_value {
                serde_json::Value::String(s) => {
                    groups.entry(s.clone()).or_insert_with(Vec::new).push(item.clone());
                }
                serde_json::Value::Array(arr) => {
                    // Items with multiple values appear in multiple groups
                    for val in arr {
                        if let Some(s) = val.as_str() {
                            groups
                                .entry(s.to_string())
                                .or_insert_with(Vec::new)
                                .push(item.clone());
                        }
                    }
                }
                _ => {}
            }
        } else {
            // Items without this facet go to "unspecified" group
            groups
                .entry("_unspecified_".to_string())
                .or_insert_with(Vec::new)
                .push(item.clone());
        }
    }

    groups
}

/// Get sorted group names from a grouped items map
pub fn get_sorted_group_names(groups: &HashMap<String, Vec<Item>>) -> Vec<String> {
    let mut group_names: Vec<String> = groups.keys().cloned().collect();
    group_names.sort();
    group_names
}
