//! Dataset validation module
//!
//! This module validates datasets against FAIR principles and best practices
//! for scientific data management.

mod content;
mod data_quality;
mod fair;
mod integrity;
mod metadata;
mod naming;
mod structure;

use crate::types::{AnalysisResult, FileInfo, ValidationResult};

pub use content::{validate_all_content, detect_todo_markers, TodoLocation};
pub use data_quality::check_data_quality;
pub use fair::calculate_fair_scores;
pub use integrity::check_integrity;
pub use metadata::validate_metadata;
pub use naming::check_naming_conventions;
pub use structure::check_structure;

/// Validate a dataset for FAIR compliance and quality
///
/// Runs all validation checks and returns a list of issues found.
/// Validation results are sorted by severity (Critical first).
///
/// # Arguments
///
/// * `files` - List of files in the dataset
/// * `analyses` - Analysis results for each file
///
/// # Returns
///
/// A vector of ValidationResult items describing all issues found.
///
/// # Examples
///
/// ```no_run
/// use genesis_preflight::validator::validate_dataset;
/// use genesis_preflight::types::{FileInfo, AnalysisResult};
///
/// let files: Vec<FileInfo> = vec![];
/// let analyses: Vec<AnalysisResult> = vec![];
/// let results = validate_dataset(&files, &analyses);
/// println!("Found {} issues", results.len());
/// ```
pub fn validate_dataset(
    files: &[FileInfo],
    analyses: &[AnalysisResult],
) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Check directory structure and required files
    results.extend(structure::check_structure(files));

    // Check naming conventions
    results.extend(naming::check_naming_conventions(files));

    // Check metadata files
    results.extend(metadata::validate_metadata(files));

    // Check data quality
    results.extend(data_quality::check_data_quality(files, analyses));

    // Calculate FAIR scores (adds validation results for missing elements)
    results.extend(fair::calculate_fair_scores(files, analyses));

    // Sort by severity (Critical first)
    results.sort();

    results
}

/// Validate a dataset including integrity checking
///
/// This is an extended version that also validates manifest integrity.
///
/// # Arguments
///
/// * `files` - List of files in the dataset
/// * `analyses` - Analysis results for each file
/// * `base_path` - Base path of the dataset (for finding MANIFEST.txt)
///
/// # Returns
///
/// A vector of ValidationResult items describing all issues found.
pub fn validate_dataset_with_integrity(
    files: &[FileInfo],
    analyses: &[AnalysisResult],
    base_path: &std::path::Path,
) -> Vec<ValidationResult> {
    let mut results = validate_dataset(files, analyses);

    // Check manifest integrity if MANIFEST.txt exists
    results.extend(integrity::check_integrity(files, base_path));

    // Re-sort after adding integrity results
    results.sort();

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileType;
    use std::path::PathBuf;

    #[test]
    fn test_validate_empty_dataset() {
        let files = vec![];
        let analyses = vec![];
        let results = validate_dataset(&files, &analyses);

        // Should have issues for missing README, LICENSE, etc.
        assert!(!results.is_empty());
    }

    #[test]
    fn test_validate_minimal_dataset() {
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
                PathBuf::from("data.csv"),
                PathBuf::from("data.csv"),
            ),
        ];
        let analyses = vec![
            AnalysisResult::NotAnalyzed,
            AnalysisResult::NotAnalyzed,
            AnalysisResult::NotAnalyzed,
        ];

        let results = validate_dataset(&files, &analyses);

        // Should pass basic structure checks but may have other issues
        assert!(results.iter().all(|r| {
            !r.message.contains("Missing README") && !r.message.contains("Missing LICENSE")
        }));
    }

    #[test]
    fn test_validation_results_sorted_by_severity() {
        let files = vec![];
        let analyses = vec![];
        let results = validate_dataset(&files, &analyses);

        // Verify results are sorted (Critical first, then Warning, then Info)
        for i in 1..results.len() {
            assert!(results[i - 1].severity >= results[i].severity);
        }
    }
}
