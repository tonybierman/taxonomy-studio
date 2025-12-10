use std::path::Path;

/// Map file loading errors to user-friendly messages
/// Returns (title, message, details)
pub fn map_file_load_error(error: &dyn std::error::Error, path: &Path) -> (String, String, String) {
    let error_string = error.to_string();

    if error_string.contains("Validation failed") {
        (
            "Validation Error".to_string(),
            "The taxonomy file has validation errors.".to_string(),
            error_string,
        )
    } else if error_string.contains("No such file") {
        (
            "File Not Found".to_string(),
            "The file could not be found.".to_string(),
            format!(
                "Path: {}\n\nPlease verify the file exists and you have permission to read it.",
                path.display()
            ),
        )
    } else if error_string.contains("Permission denied") {
        (
            "Permission Denied".to_string(),
            "Permission denied.".to_string(),
            format!("You don't have permission to read this file:\n{}", path.display()),
        )
    } else {
        (
            "Error Loading File".to_string(),
            "Failed to load taxonomy file.".to_string(),
            error_string,
        )
    }
}

/// Map file saving errors to user-friendly messages
/// Returns (title, message, details)
pub fn map_file_save_error(error: &dyn std::error::Error, path: Option<&Path>) -> (String, String, String) {
    let error_string = error.to_string();

    if error_string.contains("No file path set") {
        (
            "No File Path".to_string(),
            "No file path is set for this taxonomy.".to_string(),
            "Please use 'Save As...' to choose a location for this file.".to_string(),
        )
    } else if error_string.contains("Permission denied") {
        let details = if let Some(p) = path {
            format!("You don't have permission to write to:\n{}", p.display())
        } else {
            "You don't have permission to write to this file.".to_string()
        };
        (
            "Permission Denied".to_string(),
            "Permission denied.".to_string(),
            details,
        )
    } else if error_string.contains("No space left") {
        (
            "Disk Full".to_string(),
            "Disk full.".to_string(),
            "There is no space left on the device to save the file.".to_string(),
        )
    } else {
        (
            "Error Saving File".to_string(),
            "Failed to save taxonomy file.".to_string(),
            error_string,
        )
    }
}

/// Map revert errors to user-friendly messages
/// Returns (title, message, details)
pub fn map_revert_error(error: &dyn std::error::Error, path: &Path) -> (String, String, String) {
    let error_string = error.to_string();

    if error_string.contains("Validation failed") {
        (
            "Validation Error".to_string(),
            "The taxonomy file has validation errors.".to_string(),
            error_string,
        )
    } else if error_string.contains("No such file") {
        (
            "File Not Found".to_string(),
            "The file could not be found.".to_string(),
            format!(
                "Path: {}\n\nThe file may have been moved or deleted.",
                path.display()
            ),
        )
    } else if error_string.contains("Permission denied") {
        (
            "Permission Denied".to_string(),
            "Permission denied.".to_string(),
            format!("You don't have permission to read this file:\n{}", path.display()),
        )
    } else {
        (
            "Error Reverting File".to_string(),
            "Failed to reload taxonomy file.".to_string(),
            error_string,
        )
    }
}
