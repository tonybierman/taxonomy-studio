use crate::models::{TaxonomyData, TaxonomySchema};
use crate::schema::build_schema_from_json;
use crate::schema_validation::validate_against_schema;
use std::error::Error;
use std::fs;
use std::path::Path;

/// Load a JSON Schema file and build TaxonomySchema
pub fn load_schema<P: AsRef<Path>>(path: P) -> Result<TaxonomySchema, Box<dyn Error>> {
    let contents = fs::read_to_string(&path)?;
    let json_value: serde_json::Value = serde_json::from_str(&contents)?;

    let mut schema = build_schema_from_json(json_value.clone())?;
    schema.json_schema = Some(json_value);

    Ok(schema)
}

/// Load a data file and validate it against a provided schema
pub fn load_data_with_schema<P: AsRef<Path>>(
    data_path: P,
    schema: &TaxonomySchema,
) -> Result<TaxonomyData, Box<dyn Error>> {
    let contents = fs::read_to_string(&data_path)?;
    let data_value: serde_json::Value = serde_json::from_str(&contents)?;

    // Validate against JSON Schema if available
    if let Some(ref json_schema) = schema.json_schema {
        validate_against_schema(json_schema, &data_value)
            .map_err(|errors| format!("Validation failed:\n{}", errors.join("\n")))?;
    }

    // Deserialize if validation passed
    let data: TaxonomyData = serde_json::from_value(data_value)?;

    Ok(data)
}

/// Load data file and automatically load its referenced schema
/// Resolves schema path relative to data file directory
pub fn load_data_with_auto_schema<P: AsRef<Path>>(
    data_path: P,
) -> Result<(TaxonomyData, TaxonomySchema), Box<dyn Error>> {
    // First, read just to get the schema reference
    let contents = fs::read_to_string(&data_path)?;
    let data_value: serde_json::Value = serde_json::from_str(&contents)?;

    let schema_ref = data_value
        .get("schema")
        .and_then(|v| v.as_str())
        .ok_or("Data file missing 'schema' field")?;

    // Resolve schema path relative to data file
    let data_dir = data_path
        .as_ref()
        .parent()
        .ok_or("Cannot determine data file directory")?;
    let schema_path = data_dir.join(schema_ref);

    // Load schema
    let schema = load_schema(&schema_path)?;

    // Load and validate data
    let data = load_data_with_schema(&data_path, &schema)?;

    Ok((data, schema))
}

/// Save data to JSON file with pretty printing
pub fn save_data<P: AsRef<Path>>(data: &TaxonomyData, path: P) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, json)?;
    Ok(())
}
