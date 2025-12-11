use slint::{Model, ModelRc};
use std::collections::HashMap;

use crate::FacetInput;

/// Validation error with field and message
#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Validate item input (name and classification path)
pub fn validate_item_input(
    name: &str,
    path_str: &str,
) -> Result<(String, Vec<String>), ValidationError> {
    // Validate name
    if name.trim().is_empty() {
        return Err(ValidationError {
            field: "name".to_string(),
            message: "Name cannot be empty".to_string(),
        });
    }

    // Parse and validate path
    let path = parse_classification_path(path_str)?;

    Ok((name.trim().to_string(), path))
}

/// Parse classification path from comma-separated string
pub fn parse_classification_path(path_str: &str) -> Result<Vec<String>, ValidationError> {
    let path: Vec<String> = path_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if path.is_empty() {
        return Err(ValidationError {
            field: "path".to_string(),
            message: "Classification path cannot be empty".to_string(),
        });
    }

    Ok(path)
}

/// Collect facets from Slint FacetInput model
pub fn collect_facets(
    facet_inputs: &ModelRc<FacetInput>,
) -> HashMap<String, serde_json::Value> {
    let mut facets_map = HashMap::new();

    for facet_input in facet_inputs.iter() {
        let value = facet_input.value.to_string();
        if !value.trim().is_empty() {
            facets_map.insert(
                facet_input.name.to_string(),
                serde_json::Value::String(value.trim().to_string()),
            );
        }
    }

    facets_map
}
