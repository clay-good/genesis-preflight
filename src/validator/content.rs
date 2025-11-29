//! Content validation utilities
//!
//! Validates the actual content of documentation files, not just their presence.
//! This provides genuine FAIR compliance checking beyond simple file existence.

use crate::types::{ValidationResult, ValidationSeverity};
use std::fs;
use std::path::Path;

/// Location of a TODO marker in a file
#[derive(Debug, Clone)]
pub struct TodoLocation {
    /// Line number (1-indexed)
    pub line: usize,
    /// The TODO text found
    pub text: String,
}

/// Recognized license types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum LicenseType {
    MIT,
    Apache2,
    BSD3,
    GPL3,
    LGPL3,
    CC0,
    CCBY4,
    CCBYSA4,
    PublicDomain,
    Unknown,
}

impl LicenseType {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            LicenseType::MIT => "MIT",
            LicenseType::Apache2 => "Apache-2.0",
            LicenseType::BSD3 => "BSD-3-Clause",
            LicenseType::GPL3 => "GPL-3.0",
            LicenseType::LGPL3 => "LGPL-3.0",
            LicenseType::CC0 => "CC0-1.0",
            LicenseType::CCBY4 => "CC-BY-4.0",
            LicenseType::CCBYSA4 => "CC-BY-SA-4.0",
            LicenseType::PublicDomain => "Public Domain",
            LicenseType::Unknown => "Unknown",
        }
    }
}

/// Check if text contains substantive content (not just templates/placeholders)
pub fn is_substantive_content(text: &str, min_length: usize) -> bool {
    // Remove TODO markers and common placeholder text
    let cleaned = text
        .lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            !lower.contains("[todo]")
                && !lower.contains("todo:")
                && !lower.contains("fixme")
                && !lower.contains("xxx")
                && !lower.trim().is_empty()
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Check if remaining content is substantial
    let content_len = cleaned
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .count();

    content_len >= min_length
}

/// Detect TODO markers in a file
pub fn detect_todo_markers(path: &Path) -> Vec<TodoLocation> {
    let mut locations = Vec::new();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return locations,
    };

    let patterns = ["[TODO]", "[TODO:", "TODO:", "FIXME:", "FIXME", "XXX"];

    for (line_num, line) in content.lines().enumerate() {
        let upper = line.to_uppercase();
        for pattern in &patterns {
            if upper.contains(pattern) {
                locations.push(TodoLocation {
                    line: line_num + 1,
                    text: line.trim().to_string(),
                });
                break; // Only count once per line
            }
        }
    }

    locations
}

/// Validate metadata.json content
///
/// Checks that required fields exist and have non-empty values.
pub fn validate_metadata_content(path: &Path) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            results.push(
                ValidationResult::new(
                    ValidationSeverity::Critical,
                    "CONTENT-001".to_string(),
                    format!("Cannot read metadata file: {}", path.display()),
                )
                .with_suggestion("Ensure the file exists and is readable.".to_string()),
            );
            return results;
        }
    };

    // Parse JSON (simple check without full parser)
    let check_field = |field: &str, content: &str| -> bool {
        let pattern = format!("\"{}\"", field);
        if !content.contains(&pattern) {
            return false;
        }
        // Check if value is non-empty (not just "")
        // Look for pattern: "field": "value" where value is not empty
        let field_pattern = format!("\"{}\"", field);
        if let Some(pos) = content.find(&field_pattern) {
            let after = &content[pos + field_pattern.len()..];
            // Skip to value
            if let Some(colon_pos) = after.find(':') {
                let value_part = after[colon_pos + 1..].trim_start();
                // Check if it's not empty string or TODO
                if value_part.starts_with("\"\"")
                    || value_part.starts_with("\"[TODO]\"")
                    || value_part.starts_with("\"[TODO")
                {
                    return false;
                }
                // Check for empty array
                if value_part.starts_with("[]") {
                    return false;
                }
                return true;
            }
        }
        false
    };

    // Check required fields
    let required_fields = [
        ("title", "FAIR-F101", "Dataset title is required for findability"),
        ("description", "FAIR-F102", "Dataset description is required for findability"),
    ];

    for (field, code, message) in required_fields {
        if !check_field(field, &content) {
            results.push(
                ValidationResult::new(
                    ValidationSeverity::Critical,
                    code.to_string(),
                    format!("{}: '{}' field missing or empty", message, field),
                )
                .with_suggestion(format!(
                    "Add a meaningful '{}' field to metadata.json",
                    field
                )),
            );
        }
    }

    // Check recommended fields
    let recommended_fields = [
        ("keywords", "FAIR-F103", "Keywords help others discover your dataset"),
        ("creator", "FAIR-F104", "Creator/author information aids attribution"),
        ("license", "FAIR-A101", "License information is required for accessibility"),
    ];

    for (field, code, message) in recommended_fields {
        if !check_field(field, &content) {
            results.push(
                ValidationResult::new(
                    ValidationSeverity::Warning,
                    code.to_string(),
                    format!("{}: '{}' field missing or empty", message, field),
                )
                .with_suggestion(format!(
                    "Add a '{}' field to metadata.json",
                    field
                )),
            );
        }
    }

    // Check for TODO markers
    let todos = detect_todo_markers(path);
    if !todos.is_empty() {
        results.push(
            ValidationResult::new(
                ValidationSeverity::Warning,
                "CONTENT-002".to_string(),
                format!(
                    "metadata.json contains {} TODO marker(s) that need completion",
                    todos.len()
                ),
            )
            .with_suggestion(
                "Complete all TODO sections in metadata.json before submission.".to_string(),
            ),
        );
    }

    results
}

/// Validate README content
///
/// Checks that README has substantive content with proper sections.
pub fn validate_readme_content(path: &Path) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            results.push(
                ValidationResult::new(
                    ValidationSeverity::Critical,
                    "CONTENT-010".to_string(),
                    format!("Cannot read README file: {}", path.display()),
                )
                .with_suggestion("Ensure the file exists and is readable.".to_string()),
            );
            return results;
        }
    };

    // Check minimum content length (200 chars of actual content)
    if !is_substantive_content(&content, 200) {
        results.push(
            ValidationResult::new(
                ValidationSeverity::Warning,
                "FAIR-F201".to_string(),
                "README lacks substantive content (< 200 characters of real content)".to_string(),
            )
            .with_suggestion(
                "Add meaningful documentation to help others understand your dataset.".to_string(),
            ),
        );
    }

    // Check for sections (markdown headers)
    let section_count = content.lines().filter(|l| l.starts_with('#')).count();
    if section_count < 2 {
        results.push(
            ValidationResult::new(
                ValidationSeverity::Info,
                "FAIR-F202".to_string(),
                format!(
                    "README has only {} section header(s); consider adding more structure",
                    section_count
                ),
            )
            .with_suggestion(
                "Add sections like Description, Data Files, Usage, Citation, License.".to_string(),
            ),
        );
    }

    // Check for TODO markers
    let todos = detect_todo_markers(path);
    if !todos.is_empty() {
        results.push(
            ValidationResult::new(
                ValidationSeverity::Warning,
                "CONTENT-011".to_string(),
                format!(
                    "README contains {} TODO marker(s) that need completion",
                    todos.len()
                ),
            )
            .with_suggestion(
                "Complete all TODO sections in README before submission.".to_string(),
            ),
        );
    }

    // Check for citation information
    let lower_content = content.to_lowercase();
    if !lower_content.contains("citation")
        && !lower_content.contains("cite")
        && !lower_content.contains("reference")
    {
        results.push(
            ValidationResult::new(
                ValidationSeverity::Info,
                "FAIR-R201".to_string(),
                "README does not include citation information".to_string(),
            )
            .with_suggestion(
                "Add a Citation section explaining how to cite this dataset.".to_string(),
            ),
        );
    }

    results
}

/// Validate LICENSE file content
///
/// Checks that LICENSE contains recognized license text.
pub fn validate_license_content(path: &Path) -> (Option<LicenseType>, Vec<ValidationResult>) {
    let mut results = Vec::new();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            results.push(
                ValidationResult::new(
                    ValidationSeverity::Critical,
                    "CONTENT-020".to_string(),
                    format!("Cannot read LICENSE file: {}", path.display()),
                )
                .with_suggestion("Ensure the file exists and is readable.".to_string()),
            );
            return (None, results);
        }
    };

    // Detect license type
    let license_type = detect_license_type(&content);

    if license_type == LicenseType::Unknown {
        results.push(
            ValidationResult::new(
                ValidationSeverity::Warning,
                "FAIR-A201".to_string(),
                "LICENSE file does not contain recognized license text".to_string(),
            )
            .with_suggestion(
                "Use a standard license (MIT, Apache-2.0, CC-BY-4.0) for clarity.".to_string(),
            ),
        );
    }

    // Check for TODO markers
    let todos = detect_todo_markers(path);
    if !todos.is_empty() {
        results.push(
            ValidationResult::new(
                ValidationSeverity::Warning,
                "CONTENT-021".to_string(),
                "LICENSE contains TODO markers - license may be incomplete".to_string(),
            )
            .with_suggestion("Complete all placeholders in LICENSE file.".to_string()),
        );
    }

    (Some(license_type), results)
}

/// Detect the type of license from content
fn detect_license_type(content: &str) -> LicenseType {
    let lower = content.to_lowercase();

    // MIT License
    if lower.contains("mit license") || lower.contains("permission is hereby granted, free of charge") {
        return LicenseType::MIT;
    }

    // Apache 2.0
    if lower.contains("apache license") && lower.contains("version 2.0") {
        return LicenseType::Apache2;
    }

    // BSD 3-Clause
    if lower.contains("bsd") && lower.contains("redistributions of source code") {
        return LicenseType::BSD3;
    }

    // GPL 3.0
    if lower.contains("gnu general public license") && lower.contains("version 3") {
        return LicenseType::GPL3;
    }

    // LGPL 3.0
    if lower.contains("gnu lesser general public license") {
        return LicenseType::LGPL3;
    }

    // Creative Commons CC0
    if lower.contains("cc0") || lower.contains("creative commons zero") {
        return LicenseType::CC0;
    }

    // Creative Commons BY 4.0
    if lower.contains("creative commons") && lower.contains("attribution") && lower.contains("4.0") {
        if lower.contains("sharealike") {
            return LicenseType::CCBYSA4;
        }
        return LicenseType::CCBY4;
    }

    // Public Domain
    if lower.contains("public domain") || lower.contains("no copyright") {
        return LicenseType::PublicDomain;
    }

    LicenseType::Unknown
}

/// Validate DATACARD.md content
///
/// Checks that provenance sections are filled in.
pub fn validate_datacard_content(path: &Path) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            return results; // DATACARD is optional
        }
    };

    // Check for TODO markers
    let todos = detect_todo_markers(path);
    if !todos.is_empty() {
        results.push(
            ValidationResult::new(
                ValidationSeverity::Warning,
                "CONTENT-030".to_string(),
                format!(
                    "DATACARD.md contains {} TODO marker(s) - provenance documentation incomplete",
                    todos.len()
                ),
            )
            .with_suggestion("Complete all TODO sections in DATACARD.md for full provenance.".to_string()),
        );
    }

    // Check for key sections with content
    let lower = content.to_lowercase();
    let sections = [
        ("provenance", "FAIR-R301", "Provenance section"),
        ("methodology", "FAIR-R302", "Methodology section"),
        ("data collection", "FAIR-R303", "Data collection section"),
    ];

    for (section, code, name) in sections {
        // Check if section exists and has content after it
        if let Some(pos) = lower.find(section) {
            // Look for content after the section header
            let after = &content[pos..];
            let section_content: String = after
                .lines()
                .skip(1) // Skip header line
                .take_while(|l| !l.starts_with('#')) // Until next section
                .collect::<Vec<_>>()
                .join("\n");

            if !is_substantive_content(&section_content, 50) {
                results.push(
                    ValidationResult::new(
                        ValidationSeverity::Info,
                        code.to_string(),
                        format!("{} exists but lacks substantive content", name),
                    )
                    .with_suggestion(format!("Add detailed information to the {} section.", name)),
                );
            }
        }
    }

    results
}

/// Check if a filename is descriptive (not generic)
pub fn is_descriptive_filename(filename: &str) -> bool {
    let _lower = filename.to_lowercase();
    let stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Generic/non-descriptive names
    let generic_names = [
        "data", "data1", "data2", "data3",
        "file", "file1", "file2",
        "test", "test1", "test2",
        "temp", "tmp",
        "new", "new1",
        "untitled",
        "document",
        "copy",
    ];

    // Check if stem is a generic name
    if generic_names.contains(&stem.as_str()) {
        return false;
    }

    // Check if it's just numbers
    if stem.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Check minimum length
    if stem.len() < 3 {
        return false;
    }

    true
}

/// Validate all documentation content in a dataset
pub fn validate_all_content(
    files: &[crate::types::FileInfo],
    _base_path: &Path,
) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Find and validate specific files
    for file in files {
        let filename = file
            .relative_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let filename_upper = filename.to_uppercase();

        // Validate metadata.json
        if filename == "metadata.json" {
            results.extend(validate_metadata_content(&file.full_path));
        }

        // Validate README
        if filename_upper.starts_with("README") {
            results.extend(validate_readme_content(&file.full_path));
        }

        // Validate LICENSE
        if filename_upper.starts_with("LICENSE") {
            let (_, license_results) = validate_license_content(&file.full_path);
            results.extend(license_results);
        }

        // Validate DATACARD
        if filename_upper == "DATACARD.MD" {
            results.extend(validate_datacard_content(&file.full_path));
        }

        // Check for non-descriptive data filenames
        let ext = file
            .relative_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if ["csv", "json", "txt", "dat", "tsv"].contains(&ext.to_lowercase().as_str())
            && !is_descriptive_filename(filename) {
                results.push(
                    ValidationResult::new(
                        ValidationSeverity::Info,
                        "FAIR-F301".to_string(),
                        format!("Data file '{}' has a non-descriptive name", filename),
                    )
                    .with_suggestion(
                        "Use descriptive filenames that indicate the content (e.g., 'temperature_readings.csv')."
                            .to_string(),
                    ),
                );
            }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_is_substantive_content() {
        assert!(is_substantive_content("This is a meaningful description of the dataset that provides useful information.", 50));
        assert!(!is_substantive_content("[TODO] Fill this in later", 50));
        assert!(!is_substantive_content("", 50));
    }

    #[test]
    fn test_detect_todo_markers() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_todo_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.md");
        {
            let mut file = std::fs::File::create(&file_path).unwrap();
            writeln!(file, "# Title").unwrap();
            writeln!(file, "[TODO] Add description").unwrap();
            writeln!(file, "Some content").unwrap();
            writeln!(file, "FIXME: Fix this").unwrap();
        }

        let todos = detect_todo_markers(&file_path);
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].line, 2);
        assert_eq!(todos[1].line, 4);

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_detect_license_type() {
        assert_eq!(
            detect_license_type("MIT License\n\nPermission is hereby granted, free of charge"),
            LicenseType::MIT
        );
        assert_eq!(
            detect_license_type("Apache License Version 2.0"),
            LicenseType::Apache2
        );
        assert_eq!(
            detect_license_type("Creative Commons Attribution 4.0"),
            LicenseType::CCBY4
        );
        assert_eq!(
            detect_license_type("Some custom license text"),
            LicenseType::Unknown
        );
    }

    #[test]
    fn test_is_descriptive_filename() {
        assert!(is_descriptive_filename("temperature_readings.csv"));
        assert!(is_descriptive_filename("experiment_results_2024.json"));
        assert!(!is_descriptive_filename("data.csv"));
        assert!(!is_descriptive_filename("data1.csv"));
        assert!(!is_descriptive_filename("test.csv"));
        assert!(!is_descriptive_filename("123.csv"));
    }
}
