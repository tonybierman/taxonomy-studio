use crate::models::HybridTaxonomy;
use std::error::Error;
use std::fs;
use std::path::Path;

/// Load a taxonomy from a JSON file
pub fn load_taxonomy<P: AsRef<Path>>(path: P) -> Result<HybridTaxonomy, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let taxonomy: HybridTaxonomy = serde_json::from_str(&contents)?;
    Ok(taxonomy)
}

/// Save a taxonomy to a JSON file with pretty printing
pub fn save_taxonomy<P: AsRef<Path>>(taxonomy: &HybridTaxonomy, path: P) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(taxonomy)?;
    fs::write(path, json)?;
    Ok(())
}
