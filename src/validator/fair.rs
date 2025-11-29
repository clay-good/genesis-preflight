//! FAIR principles validation
//!
//! Validates datasets against FAIR principles:
//! - Findable: Has metadata, identifiers, keywords
//! - Accessible: Has license, contact info, standard formats
//! - Interoperable: Has schema, uses standard formats
//! - Reusable: Has documentation, provenance, citation info

use crate::types::{AnalysisResult, FileInfo, FileType, ValidationResult};

/// Calculate FAIR compliance scores
///
/// Returns validation results for FAIR principle violations.
/// The actual scoring is done by the reporter module.
pub fn calculate_fair_scores(
    files: &[FileInfo],
    _analyses: &[AnalysisResult],
) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Findable checks
    results.extend(check_findable(files));

    // Accessible checks
    results.extend(check_accessible(files));

    // Interoperable checks
    results.extend(check_interoperable(files));

    // Reusable checks
    results.extend(check_reusable(files));

    results
}

/// Check Findable criteria
fn check_findable(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Check for metadata
    let has_metadata = files.iter().any(|f| {
        f.file_name()
            .map(|name| name == "metadata.json")
            .unwrap_or(false)
    });

    if !has_metadata {
        results.push(ValidationResult::warning(
            "FAIR-F001",
            "No metadata.json for findability",
            "Create metadata.json with title, description, keywords, and identifiers",
        ));
    }

    // Check for keywords (if metadata exists, this is checked elsewhere)
    // This is a placeholder for dataset-level findability

    results
}

/// Check Accessible criteria
fn check_accessible(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Check for license
    let has_license = files.iter().any(|f| {
        f.file_name()
            .map(|name| {
                let upper = name.to_uppercase();
                upper.starts_with("LICENSE")
            })
            .unwrap_or(false)
    });

    if !has_license {
        results.push(ValidationResult::critical(
            "FAIR-A001",
            "No LICENSE file for accessibility",
            "Add LICENSE file specifying usage rights and permissions",
        ));
    }

    // Check for standard formats
    let non_standard_formats = files
        .iter()
        .filter(|f| matches!(f.file_type, FileType::Unknown | FileType::Binary))
        .count();

    if non_standard_formats > 0 && non_standard_formats as f32 / files.len() as f32 > 0.1 {
        results.push(ValidationResult::info(
            "FAIR-A002",
            format!(
                "{} files use non-standard or unknown formats",
                non_standard_formats
            ),
            "Consider converting to standard formats (CSV, JSON, HDF5, NetCDF)",
        ));
    }

    results
}

/// Check Interoperable criteria
fn check_interoperable(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Check for schema files for structured data
    let has_csv = files.iter().any(|f| matches!(f.file_type, FileType::Csv));
    let has_schema = files.iter().any(|f| {
        f.file_name()
            .map(|name| name.ends_with("schema.json") || name.ends_with(".schema"))
            .unwrap_or(false)
    });

    if has_csv && !has_schema {
        results.push(ValidationResult::info(
            "FAIR-I001",
            "No schema file for CSV data",
            "Create schema.json file(s) describing data structure and types",
        ));
    }

    // Check for documentation of data structure
    let has_readme = files.iter().any(|f| {
        f.file_name()
            .map(|name| name.to_uppercase().starts_with("README"))
            .unwrap_or(false)
    });

    if !has_readme {
        results.push(ValidationResult::critical(
            "FAIR-I002",
            "No README for interoperability",
            "Create README documenting dataset structure and variables",
        ));
    }

    results
}

/// Check Reusable criteria
fn check_reusable(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Check for documentation
    let has_readme = files.iter().any(|f| {
        f.file_name()
            .map(|name| name.to_uppercase().starts_with("README"))
            .unwrap_or(false)
    });

    if !has_readme {
        results.push(ValidationResult::critical(
            "FAIR-R001",
            "No README for reusability",
            "Create README with usage instructions and examples",
        ));
    }

    // Check for provenance information (metadata.json or DATACARD.md)
    let has_provenance = files.iter().any(|f| {
        f.file_name()
            .map(|name| name == "metadata.json" || name.to_uppercase().starts_with("DATACARD"))
            .unwrap_or(false)
    });

    if !has_provenance {
        results.push(ValidationResult::warning(
            "FAIR-R002",
            "No provenance information",
            "Create metadata.json or DATACARD.md documenting data origin and processing",
        ));
    }

    // Check for citation information
    let has_citation = files.iter().any(|f| {
        f.file_name()
            .map(|name| {
                name.to_uppercase().contains("CITATION")
                    || name == "metadata.json"
            })
            .unwrap_or(false)
    });

    if !has_citation {
        results.push(ValidationResult::info(
            "FAIR-R003",
            "No citation information",
            "Add citation information to README or create CITATION file",
        ));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_check_findable_no_metadata() {
        let files = vec![FileInfo::new(
            PathBuf::from("data.csv"),
            PathBuf::from("data.csv"),
        )];
        let results = check_findable(&files);
        assert!(results.iter().any(|r| r.code == "FAIR-F001"));
    }

    #[test]
    fn test_check_findable_has_metadata() {
        let files = vec![FileInfo::new(
            PathBuf::from("metadata.json"),
            PathBuf::from("metadata.json"),
        )];
        let results = check_findable(&files);
        assert!(!results.iter().any(|r| r.code == "FAIR-F001"));
    }

    #[test]
    fn test_check_accessible_no_license() {
        let files = vec![FileInfo::new(
            PathBuf::from("data.csv"),
            PathBuf::from("data.csv"),
        )];
        let results = check_accessible(&files);
        assert!(results.iter().any(|r| r.code == "FAIR-A001"));
    }

    #[test]
    fn test_check_accessible_has_license() {
        let files = vec![FileInfo::new(
            PathBuf::from("LICENSE"),
            PathBuf::from("LICENSE"),
        )];
        let results = check_accessible(&files);
        assert!(!results.iter().any(|r| r.code == "FAIR-A001"));
    }

    #[test]
    fn test_check_interoperable_no_schema() {
        let files = vec![FileInfo::new(
            PathBuf::from("data.csv"),
            PathBuf::from("data.csv"),
        )];
        let results = check_interoperable(&files);
        assert!(results.iter().any(|r| r.code == "FAIR-I001"));
    }

    #[test]
    fn test_check_reusable_complete() {
        let files = vec![
            FileInfo::new(
                PathBuf::from("README.md"),
                PathBuf::from("README.md"),
            ),
            FileInfo::new(
                PathBuf::from("metadata.json"),
                PathBuf::from("metadata.json"),
            ),
        ];
        let results = check_reusable(&files);

        // Should not have critical reusability issues
        assert!(!results.iter().any(|r| r.code == "FAIR-R001"));
        assert!(!results.iter().any(|r| r.code == "FAIR-R002"));
    }

    #[test]
    fn test_calculate_fair_scores_empty() {
        let files = vec![];
        let analyses = vec![];
        let results = calculate_fair_scores(&files, &analyses);

        // Empty dataset should fail multiple FAIR checks
        assert!(!results.is_empty());
    }

    #[test]
    fn test_calculate_fair_scores_minimal() {
        let files = vec![
            FileInfo::new(
                PathBuf::from("README.md"),
                PathBuf::from("README.md"),
            ),
            FileInfo::new(
                PathBuf::from("LICENSE"),
                PathBuf::from("LICENSE"),
            ),
            FileInfo::new(
                PathBuf::from("metadata.json"),
                PathBuf::from("metadata.json"),
            ),
        ];
        let analyses = vec![];
        let results = calculate_fair_scores(&files, &analyses);

        // Minimal compliant dataset should have fewer issues
        let critical_count = results
            .iter()
            .filter(|r| matches!(r.severity, crate::types::ValidationSeverity::Critical))
            .count();

        assert_eq!(critical_count, 0);
    }
}
