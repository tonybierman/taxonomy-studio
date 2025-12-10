use crate::models::Item;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;

/// Sort items by the specified field (either "name" or a facet name)
pub fn sort_items(items: &mut Vec<Item>, sort_field: &str) {
    items.sort_by(|a, b| {
        if sort_field == "name" {
            // Library science sorting: strip articles, normalize unicode, handle numbers
            let a_key = normalize_for_sorting(&a.name);
            let b_key = normalize_for_sorting(&b.name);

            // Primary sort by normalized name
            match a_key.cmp(&b_key) {
                std::cmp::Ordering::Equal => {
                    // Secondary sort: original name for ties
                    a.name.cmp(&b.name)
                }
                other => other,
            }
        } else {
            // Sort by facet value
            let a_val = get_facet_string(a, sort_field);
            let b_val = get_facet_string(b, sort_field);

            // Normalize facet values for sorting
            let a_key = normalize_for_sorting(&a_val);
            let b_key = normalize_for_sorting(&b_val);

            // Primary sort by normalized facet, secondary by name
            match a_key.cmp(&b_key) {
                std::cmp::Ordering::Equal => {
                    let a_name_key = normalize_for_sorting(&a.name);
                    let b_name_key = normalize_for_sorting(&b.name);
                    a_name_key.cmp(&b_name_key)
                }
                other => other,
            }
        }
    });
}

/// Normalize string for library science sorting
/// - Strip leading articles (a, an, the)
/// - Normalize unicode (NFD then lowercase)
/// - Handle punctuation
/// - Preserve numbers for natural sorting
pub fn normalize_for_sorting(s: &str) -> String {
    // Strip leading articles (case-insensitive)
    let without_articles = strip_leading_articles(s);

    // Unicode normalization (NFD decomposition) and lowercase
    let normalized: String = without_articles
        .nfd()
        .collect::<String>()
        .to_lowercase();

    // Remove leading/trailing whitespace and collapse internal whitespace
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Strip leading articles following library science conventions
/// Supports: a, an, the (English) and common articles in other languages
pub fn strip_leading_articles(s: &str) -> String {
    let re = Regex::new(r"^(?i)(the|a|an|der|die|das|le|la|les|el|la|los|las|il|lo|i|gli|un|une|een)\s+").unwrap();
    re.replace(s, "").to_string()
}

/// Get facet value as a string (handles both single values and arrays)
fn get_facet_string(item: &Item, facet_name: &str) -> String {
    item.facets
        .get(facet_name)
        .and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Array(arr) => {
                let values: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                Some(values.join(", "))
            }
            _ => None,
        })
        .unwrap_or_default()
}
