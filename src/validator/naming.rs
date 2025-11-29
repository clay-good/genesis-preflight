//! Filename validation

use crate::types::{FileInfo, ValidationResult};
use std::collections::{HashMap, HashSet};

/// Check naming conventions for files
///
/// Validates filenames follow best practices for scientific data.
pub fn check_naming_conventions(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Check for spaces in filenames
    results.extend(check_no_spaces(files));

    // Check for special characters
    results.extend(check_special_characters(files));

    // Check for lowercase preference
    results.extend(check_lowercase_preference(files));

    // Check for duplicate filenames (case-insensitive)
    results.extend(check_duplicates(files));

    results
}

/// Check for spaces in filenames
fn check_no_spaces(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    for file in files {
        if let Some(name) = file.file_name() {
            if name.contains(' ') {
                results.push(
                    ValidationResult::warning(
                        "NAME-001",
                        format!("Filename contains spaces: {}", name),
                        format!("Rename to use underscores or hyphens: {}", name.replace(' ', "_")),
                    )
                    .with_file(file.relative_path.clone()),
                );
            }
        }
    }

    results
}

/// Check for special characters in filenames
fn check_special_characters(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    for file in files {
        if let Some(name) = file.file_name() {
            let invalid_chars: Vec<char> = name
                .chars()
                .filter(|c| !is_valid_filename_char(*c))
                .collect();

            if !invalid_chars.is_empty() {
                let chars_str: String = invalid_chars.iter().collect();
                results.push(
                    ValidationResult::warning(
                        "NAME-002",
                        format!("Filename contains special characters: {}", chars_str),
                        "Use only letters, numbers, hyphens, underscores, and dots",
                    )
                    .with_file(file.relative_path.clone()),
                );
            }
        }
    }

    results
}

/// Check if character is valid in a filename
fn is_valid_filename_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'
}

/// Check for lowercase preference
fn check_lowercase_preference(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let mut uppercase_count = 0;
    let mut total_count = 0;

    for file in files {
        if let Some(name) = file.file_name() {
            // Exclude common uppercase files (README, LICENSE, etc.)
            let upper = name.to_uppercase();
            if upper.starts_with("README")
                || upper.starts_with("LICENSE")
                || upper.starts_with("CONTRIBUTING")
                || upper.starts_with("CHANGELOG")
            {
                continue;
            }

            total_count += 1;

            // Check if filename (without extension) has uppercase letters
            let name_without_ext = if let Some(pos) = name.rfind('.') {
                &name[..pos]
            } else {
                name
            };

            if name_without_ext.chars().any(|c| c.is_uppercase()) {
                uppercase_count += 1;
            }
        }
    }

    // If more than 30% of files use uppercase, suggest consistency
    if total_count > 5 && uppercase_count as f32 / total_count as f32 > 0.3 {
        results.push(ValidationResult::info(
            "NAME-003",
            format!(
                "Mixed case filenames detected ({} of {} files)",
                uppercase_count, total_count
            ),
            "Consider using consistent lowercase naming for better compatibility",
        ));
    }

    results
}

/// Check for duplicate filenames (case-insensitive)
fn check_duplicates(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let mut seen: HashMap<String, Vec<&FileInfo>> = HashMap::new();

    for file in files {
        if let Some(name) = file.file_name() {
            let lower_name = name.to_lowercase();
            seen.entry(lower_name).or_default().push(file);
        }
    }

    for (name, file_list) in seen {
        if file_list.len() > 1 {
            // Check if they're actually different (different paths)
            let unique_paths: HashSet<_> = file_list
                .iter()
                .map(|f| f.relative_path.to_string_lossy())
                .collect();

            if unique_paths.len() > 1 {
                results.push(ValidationResult::warning(
                    "NAME-004",
                    format!(
                        "Duplicate filename (case-insensitive): {} appears {} times",
                        name,
                        file_list.len()
                    ),
                    "Rename files to have unique names across all directories",
                ));
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
    fn test_check_no_spaces() {
        let files = vec![
            FileInfo::new(
                PathBuf::from("file with spaces.txt"),
                PathBuf::from("file with spaces.txt"),
            ),
            FileInfo::new(
                PathBuf::from("good_file.txt"),
                PathBuf::from("good_file.txt"),
            ),
        ];
        let results = check_no_spaces(&files);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "NAME-001");
    }

    #[test]
    fn test_check_special_characters() {
        let files = vec![
            FileInfo::new(
                PathBuf::from("file@name!.txt"),
                PathBuf::from("file@name!.txt"),
            ),
            FileInfo::new(
                PathBuf::from("good-file_name.txt"),
                PathBuf::from("good-file_name.txt"),
            ),
        ];
        let results = check_special_characters(&files);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "NAME-002");
    }

    #[test]
    fn test_is_valid_filename_char() {
        assert!(is_valid_filename_char('a'));
        assert!(is_valid_filename_char('Z'));
        assert!(is_valid_filename_char('0'));
        assert!(is_valid_filename_char('-'));
        assert!(is_valid_filename_char('_'));
        assert!(is_valid_filename_char('.'));
        assert!(!is_valid_filename_char(' '));
        assert!(!is_valid_filename_char('!'));
        assert!(!is_valid_filename_char('@'));
    }

    #[test]
    fn test_check_lowercase_preference() {
        let files: Vec<FileInfo> = (0..10)
            .map(|i| {
                let name = if i < 7 {
                    format!("File{}.txt", i)
                } else {
                    format!("file{}.txt", i)
                };
                FileInfo::new(PathBuf::from(&name), PathBuf::from(&name))
            })
            .collect();

        let results = check_lowercase_preference(&files);
        // Should warn about mixed case
        assert!(!results.is_empty());
    }

    #[test]
    fn test_check_duplicates() {
        let files = vec![
            FileInfo::new(
                PathBuf::from("dir1/file.txt"),
                PathBuf::from("dir1/file.txt"),
            ),
            FileInfo::new(
                PathBuf::from("dir2/FILE.txt"),
                PathBuf::from("dir2/FILE.txt"),
            ),
        ];
        let results = check_duplicates(&files);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "NAME-004");
    }

    #[test]
    fn test_readme_excluded_from_case_check() {
        let files = vec![
            FileInfo::new(
                PathBuf::from("README.md"),
                PathBuf::from("README.md"),
            ),
            FileInfo::new(
                PathBuf::from("LICENSE"),
                PathBuf::from("LICENSE"),
            ),
        ];
        let results = check_lowercase_preference(&files);
        // README and LICENSE should be excluded
        assert_eq!(results.len(), 0);
    }
}
