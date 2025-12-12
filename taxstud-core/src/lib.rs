// Public modules
pub mod filtering;
pub mod grouping;
pub mod io;
pub mod models;
pub mod schema;
pub mod schema_validation;
pub mod sorting;
pub mod validation;

// Re-export commonly used types for convenience
pub use filtering::{apply_filters, has_filters, matches_filters, parse_facet_filters};
pub use grouping::{get_sorted_group_names, group_items_by_facet};
pub use io::{load_data_with_auto_schema, load_data_with_schema, load_schema, save_data};
pub use models::{
    ClassicalHierarchy, Filters, HierarchyNode, HybridTaxonomy, Item, TaxonomyData, TaxonomySchema,
};
pub use schema::{build_schema_from_json, extract_classical_hierarchy, extract_faceted_dimensions};
pub use schema_validation::validate_against_schema;
pub use sorting::{normalize_for_sorting, sort_items, strip_leading_articles};
pub use validation::{validate_path_exists, validate_taxonomy};
