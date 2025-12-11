use crate::models::{ClassicalHierarchy, TaxonomySchema};
use serde_json::Value;
use std::collections::HashMap;

/// Extract classical hierarchy from JSON Schema
/// Looks for "classical_hierarchy" top-level property
pub fn extract_classical_hierarchy(json_schema: &Value) -> Result<ClassicalHierarchy, String> {
    let hierarchy_value = json_schema
        .get("classical_hierarchy")
        .ok_or("JSON Schema missing 'classical_hierarchy' property")?;

    serde_json::from_value(hierarchy_value.clone())
        .map_err(|e| format!("Failed to parse classical_hierarchy: {}", e))
}

/// Extract faceted dimensions from JSON Schema
/// Looks for "faceted_dimensions" top-level property
pub fn extract_faceted_dimensions(
    json_schema: &Value,
) -> Result<HashMap<String, Vec<String>>, String> {
    let dimensions_value = json_schema
        .get("faceted_dimensions")
        .ok_or("JSON Schema missing 'faceted_dimensions' property")?;

    serde_json::from_value(dimensions_value.clone())
        .map_err(|e| format!("Failed to parse faceted_dimensions: {}", e))
}

/// Build TaxonomySchema from a JSON Schema file
pub fn build_schema_from_json(json_schema: Value) -> Result<TaxonomySchema, String> {
    // Extract schema metadata
    let schema_id = json_schema
        .get("$id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let title = json_schema
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled Taxonomy")
        .to_string();

    let description = json_schema
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Extract classical hierarchy and faceted dimensions
    let classical_hierarchy = extract_classical_hierarchy(&json_schema)?;
    let faceted_dimensions = extract_faceted_dimensions(&json_schema)?;

    Ok(TaxonomySchema {
        schema_id,
        title,
        description,
        classical_hierarchy,
        faceted_dimensions,
        json_schema: Some(json_schema),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_classical_hierarchy() {
        let schema = json!({
            "classical_hierarchy": {
                "root": "TestRoot",
                "children": [{
                    "genus": "TestRoot",
                    "species": "TestSpecies",
                    "differentia": "test differentia"
                }]
            }
        });

        let hierarchy = extract_classical_hierarchy(&schema).unwrap();
        assert_eq!(hierarchy.root, "TestRoot");
        assert!(hierarchy.children.is_some());
        assert_eq!(hierarchy.children.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_extract_faceted_dimensions() {
        let schema = json!({
            "faceted_dimensions": {
                "color": ["red", "green", "blue"],
                "size": ["small", "medium", "large"]
            }
        });

        let dimensions = extract_faceted_dimensions(&schema).unwrap();
        assert_eq!(dimensions.len(), 2);
        assert_eq!(dimensions.get("color").unwrap().len(), 3);
        assert_eq!(dimensions.get("size").unwrap().len(), 3);
    }

    #[test]
    fn test_build_schema_from_json() {
        let json_schema = json!({
            "$id": "test-schema",
            "title": "Test Schema",
            "description": "A test schema",
            "classical_hierarchy": {
                "root": "TestRoot"
            },
            "faceted_dimensions": {
                "color": ["red", "blue"]
            }
        });

        let schema = build_schema_from_json(json_schema).unwrap();
        assert_eq!(schema.schema_id, "test-schema");
        assert_eq!(schema.title, "Test Schema");
        assert_eq!(schema.description, Some("A test schema".to_string()));
        assert_eq!(schema.classical_hierarchy.root, "TestRoot");
        assert_eq!(schema.faceted_dimensions.len(), 1);
    }

    #[test]
    fn test_missing_classical_hierarchy() {
        let schema = json!({
            "faceted_dimensions": {}
        });

        let result = extract_classical_hierarchy(&schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_faceted_dimensions() {
        let schema = json!({
            "classical_hierarchy": {"root": "Test"}
        });

        let result = extract_faceted_dimensions(&schema);
        assert!(result.is_err());
    }
}
