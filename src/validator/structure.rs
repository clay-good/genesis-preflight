//! Directory structure validation

use crate::types::{FileInfo, ValidationResult};

/// Maximum reasonable directory depth
const MAX_DEPTH: usize = 10;

/// Maximum reasonable filename length
const MAX_FILENAME_LENGTH: usize = 255;

/// Check dataset directory structure
///
/// Validates presence of essential files and reasonable directory organization.
pub fn check_structure(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Check for README
    results.extend(check_readme(files));

    // Check for LICENSE
    results.extend(check_license(files));

    // Check for metadata.json
    results.extend(check_metadata_json(files));

    // Check directory depth
    results.extend(check_directory_depth(files));

    // Check filename length
    results.extend(check_filename_length(files));

    results
}

/// Check for README file presence
fn check_readme(files: &[FileInfo]) -> Vec<ValidationResult> {
    let has_readme = files.iter().any(|f| {
        f.file_name()
            .map(|name| {
                let upper = name.to_uppercase();
                upper.starts_with("README")
            })
            .unwrap_or(false)
    });

    if !has_readme {
        vec![ValidationResult::critical(
            "STRUCT-001",
            "Missing README file",
            "Create a README.md file describing your dataset",
        )]
    } else {
        vec![]
    }
}

/// Check for LICENSE file presence
fn check_license(files: &[FileInfo]) -> Vec<ValidationResult> {
    let has_license = files.iter().any(|f| {
        f.file_name()
            .map(|name| {
                let upper = name.to_uppercase();
                upper.starts_with("LICENSE") || upper.starts_with("LICENCE")
            })
            .unwrap_or(false)
    });

    if !has_license {
        vec![ValidationResult::critical(
            "STRUCT-002",
            "Missing LICENSE file",
            "Add a LICENSE file specifying usage terms and permissions",
        )]
    } else {
        vec![]
    }
}

/// Check for metadata.json file presence
fn check_metadata_json(files: &[FileInfo]) -> Vec<ValidationResult> {
    let has_metadata = files.iter().any(|f| {
        f.file_name()
            .map(|name| name == "metadata.json")
            .unwrap_or(false)
    });

    if !has_metadata {
        vec![ValidationResult::warning(
            "STRUCT-003",
            "Missing metadata.json file",
            "Create a metadata.json file with dataset description and provenance",
        )]
    } else {
        vec![]
    }
}

/// Check for excessive directory nesting
fn check_directory_depth(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    for file in files {
        let depth = file.relative_path.components().count();
        if depth > MAX_DEPTH {
            results.push(
                ValidationResult::warning(
                    "STRUCT-004",
                    format!(
                        "Deeply nested file: {} levels deep",
                        depth
                    ),
                    "Consider flattening directory structure for better accessibility",
                )
                .with_file(file.relative_path.clone()),
            );
        }
    }

    results
}

/// Check for excessively long filenames
fn check_filename_length(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    for file in files {
        if let Some(name) = file.file_name() {
            if name.len() > MAX_FILENAME_LENGTH {
                results.push(
                    ValidationResult::warning(
                        "STRUCT-005",
                        format!(
                            "Filename too long: {} characters",
                            name.len()
                        ),
                        "Shorten filename to under 255 characters",
                    )
                    .with_file(file.relative_path.clone()),
                );
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_check_readme_missing() {
        let files = vec![FileInfo::new(
            PathBuf::from("data.csv"),
            PathBuf::from("data.csv"),
        )];
        let results = check_readme(&files);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "STRUCT-001");
    }

    #[test]
    fn test_check_readme_present() {
        let files = vec![FileInfo::new(
            PathBuf::from("README.md"),
            PathBuf::from("README.md"),
        )];
        let results = check_readme(&files);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_check_license_missing() {
        let files = vec![FileInfo::new(
            PathBuf::from("data.csv"),
            PathBuf::from("data.csv"),
        )];
        let results = check_license(&files);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "STRUCT-002");
    }

    #[test]
    fn test_check_license_present() {
        let files = vec![FileInfo::new(
            PathBuf::from("LICENSE"),
            PathBuf::from("LICENSE"),
        )];
        let results = check_license(&files);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_check_metadata_json() {
        let files = vec![FileInfo::new(
            PathBuf::from("metadata.json"),
            PathBuf::from("metadata.json"),
        )];
        let results = check_metadata_json(&files);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_check_directory_depth() {
        let deep_path = PathBuf::from("a/b/c/d/e/f/g/h/i/j/k/file.txt");
        let files = vec![FileInfo::new(deep_path.clone(), deep_path)];
        let results = check_directory_depth(&files);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "STRUCT-004");
    }

    #[test]
    fn test_check_filename_length() {
        let long_name = "a".repeat(300);
        let files = vec![FileInfo::new(
            PathBuf::from(&long_name),
            PathBuf::from(&long_name),
        )];
        let results = check_filename_length(&files);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "STRUCT-005");
    }
}
