// Public modules
pub mod filtering;
pub mod grouping;
pub mod io;
pub mod models;
pub mod sorting;
pub mod validation;

// Re-export commonly used types for convenience
pub use filtering::{apply_filters, has_filters, matches_filters, parse_facet_filters};
pub use grouping::{get_sorted_group_names, group_items_by_facet};
pub use io::{load_taxonomy, save_taxonomy};
pub use models::{ClassicalHierarchy, Filters, HierarchyNode, HybridTaxonomy, Item};
pub use sorting::{normalize_for_sorting, sort_items, strip_leading_articles};
pub use validation::validate_taxonomy;
