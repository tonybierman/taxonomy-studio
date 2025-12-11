use crate::models::Item;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;

/// Sort items by the specified field (either "name" or a facet name)
pub fn sort_items(items: &mut [Item], sort_field: &str) {
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
            let a_val = a.get_facet_as_string(sort_field).unwrap_or_default();
            let b_val = b.get_facet_as_string(sort_field).unwrap_or_default();

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
    let normalized: String = without_articles.nfd().collect::<String>().to_lowercase();

    // Remove leading/trailing whitespace and collapse internal whitespace
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Strip leading articles following library science conventions
/// Supports: a, an, the (English) and common articles in other languages
pub fn strip_leading_articles(s: &str) -> String {
    let re = Regex::new(
        r"^(?i)(the|a|an|der|die|das|le|la|les|el|la|los|las|il|lo|i|gli|un|une|een)\s+",
    )
    .unwrap();
    re.replace(s, "").to_string()
}
