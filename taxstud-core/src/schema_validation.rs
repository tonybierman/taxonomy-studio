use serde_json::Value;

/// Validate data against JSON Schema
/// Returns Ok(()) if valid, Err with list of validation errors if invalid
pub fn validate_against_schema(schema: &Value, data: &Value) -> Result<(), Vec<String>> {
    // Compile the JSON Schema
    let compiled = jsonschema::validator_for(schema)
        .map_err(|e| vec![format!("Schema compilation error: {}", e)])?;

    // Validate the data - returns Result<(), ValidationError>
    match compiled.validate(data) {
        Ok(()) => Ok(()),
        Err(error) => {
            // Format validation error with path
            let path_str = error.instance_path.to_string();
            let location = if path_str.is_empty() {
                "root".to_string()
            } else {
                path_str
            };
            Err(vec![format!("{} at {}", error, location)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_data_passes() {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            },
            "required": ["name"]
        });

        let data = json!({
            "name": "Alice",
            "age": 30
        });

        let result = validate_against_schema(&schema, &data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_data_fails() {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            },
            "required": ["name"]
        });

        let data = json!({
            "age": 30
        });

        let result = validate_against_schema(&schema, &data);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_wrong_type_fails() {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "age": {"type": "number"}
            }
        });

        let data = json!({
            "age": "not a number"
        });

        let result = validate_against_schema(&schema, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_enum_validation() {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "color": {
                    "type": "string",
                    "enum": ["red", "green", "blue"]
                }
            }
        });

        let valid_data = json!({"color": "red"});
        assert!(validate_against_schema(&schema, &valid_data).is_ok());

        let invalid_data = json!({"color": "yellow"});
        assert!(validate_against_schema(&schema, &invalid_data).is_err());
    }

    #[test]
    fn test_array_validation() {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "items": {"type": "string"},
                    "minItems": 1
                }
            }
        });

        let valid_data = json!({"items": ["a", "b", "c"]});
        assert!(validate_against_schema(&schema, &valid_data).is_ok());

        let invalid_data = json!({"items": []});
        assert!(validate_against_schema(&schema, &invalid_data).is_err());
    }
}
