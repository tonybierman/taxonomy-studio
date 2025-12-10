// Public modules
pub mod models;
pub mod validation;
pub mod filtering;
pub mod sorting;
pub mod grouping;
pub mod io;

// Re-export commonly used types for convenience
pub use models::{HybridTaxonomy, ClassicalHierarchy, HierarchyNode, Item, Filters};
pub use validation::validate_taxonomy;
pub use filtering::{apply_filters, matches_filters, has_filters, parse_facet_filters};
pub use sorting::{sort_items, normalize_for_sorting, strip_leading_articles};
pub use grouping::{group_items_by_facet, get_sorted_group_names};
pub use io::{load_taxonomy, save_taxonomy};
