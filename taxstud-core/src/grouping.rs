use crate::models::Item;
use std::collections::HashMap;

/// Group items by a facet name
/// Items with multiple values for the facet appear in multiple groups
/// Returns a HashMap where keys are group names and values are vectors of items
pub fn group_items_by_facet(items: &[Item], group_field: &str) -> HashMap<String, Vec<Item>> {
    let mut groups: HashMap<String, Vec<Item>> = HashMap::new();

    for item in items {
        let facet_values = item.get_facet_as_vec(group_field);

        if facet_values.is_empty() {
            // Items without this facet go to "unspecified" group
            groups
                .entry("_unspecified_".to_string())
                .or_default()
                .push(item.clone());
        } else {
            // Items with multiple values appear in multiple groups
            for value in facet_values {
                groups.entry(value).or_default().push(item.clone());
            }
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
