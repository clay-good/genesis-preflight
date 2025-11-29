//! Metadata file validation

use crate::types::{FileInfo, ValidationResult};
use std::fs;

/// Minimum length for substantive README content
const MIN_README_LENGTH: usize = 100;

/// Validate metadata files
///
/// Checks README content quality and metadata.json structure.
pub fn validate_metadata(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Validate README content
    results.extend(validate_readme_content(files));

    // Validate metadata.json structure
    results.extend(validate_metadata_json(files));

    results
}

/// Validate README file has substantive content
fn validate_readme_content(files: &[FileInfo]) -> Vec<ValidationResult> {
    let readme = files.iter().find(|f| {
        f.file_name()
            .map(|name| name.to_uppercase().starts_with("README"))
            .unwrap_or(false)
    });

    if let Some(readme_file) = readme {
        // Read README content
        match fs::read_to_string(&readme_file.full_path) {
            Ok(content) => {
                if content.trim().len() < MIN_README_LENGTH {
                    vec![ValidationResult::warning(
                        "META-001",
                        format!(
                            "README is too short ({} characters)",
                            content.trim().len()
                        ),
                        "Expand README to at least 100 characters with meaningful description",
                    )
                    .with_file(readme_file.relative_path.clone())]
                } else {
                    vec![]
                }
            }
            Err(_) => {
                vec![ValidationResult::warning(
                    "META-002",
                    "Cannot read README file",
                    "Ensure README file is readable and contains valid text",
                )
                .with_file(readme_file.relative_path.clone())]
            }
        }
    } else {
        // No README found - this is handled by structure checks
        vec![]
    }
}

/// Validate metadata.json file structure
fn validate_metadata_json(files: &[FileInfo]) -> Vec<ValidationResult> {
    let metadata_file = files.iter().find(|f| {
        f.file_name()
            .map(|name| name == "metadata.json")
            .unwrap_or(false)
    });

    if let Some(meta_file) = metadata_file {
        // Read and parse JSON
        match fs::read_to_string(&meta_file.full_path) {
            Ok(content) => validate_json_structure(&content, &meta_file.relative_path),
            Err(_) => {
                vec![ValidationResult::critical(
                    "META-003",
                    "Cannot read metadata.json file",
                    "Ensure metadata.json file is readable",
                )
                .with_file(meta_file.relative_path.clone())]
            }
        }
    } else {
        // No metadata.json - this is handled by structure checks
        vec![]
    }
}

/// Validate JSON structure has required fields
fn validate_json_structure(
    content: &str,
    file_path: &std::path::Path,
) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Try to parse as JSON (simple check)
    let trimmed = content.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return vec![ValidationResult::critical(
            "META-004",
            "metadata.json is not valid JSON",
            "Ensure metadata.json contains valid JSON object",
        )
        .with_file(file_path.to_path_buf())];
    }

    // Check for required fields (simple string matching)
    let required_fields = [
        ("title", "META-005", "Add 'title' field to metadata.json"),
        (
            "description",
            "META-006",
            "Add 'description' field to metadata.json",
        ),
        (
            "creator",
            "META-007",
            "Add 'creator' or 'author' field to metadata.json",
        ),
        ("date", "META-008", "Add 'date' field to metadata.json"),
        (
            "license",
            "META-009",
            "Add 'license' field to metadata.json",
        ),
    ];

    for (field, code, suggestion) in &required_fields {
        // Simple check: look for "field": in the JSON
        // For "creator", also accept "author"
        let has_field = if *field == "creator" {
            content.contains("\"creator\"") || content.contains("\"author\"")
        } else if *field == "date" {
            content.contains("\"date\"") || content.contains("\"created\"")
        } else {
            content.contains(&format!("\"{}\"", field))
        };

        if !has_field {
            results.push(
                ValidationResult::warning(*code, format!("Missing {} field", field), *suggestion)
                    .with_file(file_path.to_path_buf()),
            );
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    #[test]
    fn test_validate_readme_too_short() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_meta_readme_short");
        fs::create_dir_all(&temp_dir).unwrap();

        let readme_path = temp_dir.join("README.md");
        fs::write(&readme_path, "Short").unwrap();

        let files = vec![FileInfo::new(readme_path.clone(), PathBuf::from("README.md"))];
        let results = validate_readme_content(&files);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "META-001");

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_validate_readme_sufficient() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_meta_readme_good");
        fs::create_dir_all(&temp_dir).unwrap();

        let readme_path = temp_dir.join("README.md");
        fs::write(
            &readme_path,
            "This is a comprehensive README with more than 100 characters of content describing the dataset in detail.",
        )
        .unwrap();

        let files = vec![FileInfo::new(readme_path.clone(), PathBuf::from("README.md"))];
        let results = validate_readme_content(&files);
        assert_eq!(results.len(), 0);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_validate_metadata_json_valid() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_meta_json_valid");
        fs::create_dir_all(&temp_dir).unwrap();

        let meta_path = temp_dir.join("metadata.json");
        fs::write(
            &meta_path,
            r#"{
                "title": "Test Dataset",
                "description": "A test dataset",
                "creator": "Test User",
                "date": "2024-01-01",
                "license": "MIT"
            }"#,
        )
        .unwrap();

        let files = vec![FileInfo::new(
            meta_path.clone(),
            PathBuf::from("metadata.json"),
        )];
        let results = validate_metadata_json(&files);
        assert_eq!(results.len(), 0);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_validate_metadata_json_missing_fields() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_meta_json_missing");
        fs::create_dir_all(&temp_dir).unwrap();

        let meta_path = temp_dir.join("metadata.json");
        fs::write(
            &meta_path,
            r#"{
                "title": "Test Dataset"
            }"#,
        )
        .unwrap();

        let files = vec![FileInfo::new(
            meta_path.clone(),
            PathBuf::from("metadata.json"),
        )];
        let results = validate_metadata_json(&files);

        // Should flag missing description, creator, date, license
        assert!(results.len() >= 4);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_validate_metadata_json_invalid() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_meta_json_invalid");
        fs::create_dir_all(&temp_dir).unwrap();

        let meta_path = temp_dir.join("metadata.json");
        fs::write(&meta_path, "not valid json").unwrap();

        let files = vec![FileInfo::new(
            meta_path.clone(),
            PathBuf::from("metadata.json"),
        )];
        let results = validate_metadata_json(&files);
        assert!(results.iter().any(|r| r.code == "META-004"));

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_validate_metadata_author_alias() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_meta_author");
        fs::create_dir_all(&temp_dir).unwrap();

        let meta_path = temp_dir.join("metadata.json");
        fs::write(
            &meta_path,
            r#"{
                "title": "Test",
                "description": "Test",
                "author": "Test User",
                "date": "2024-01-01",
                "license": "MIT"
            }"#,
        )
        .unwrap();

        let files = vec![FileInfo::new(
            meta_path.clone(),
            PathBuf::from("metadata.json"),
        )];
        let results = validate_metadata_json(&files);

        // Should accept "author" instead of "creator"
        assert!(!results.iter().any(|r| r.code == "META-007"));

        fs::remove_dir_all(temp_dir).ok();
    }
}
