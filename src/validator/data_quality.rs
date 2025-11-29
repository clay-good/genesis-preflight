//! Data quality checks

use crate::types::{AnalysisResult, FileInfo, ValidationResult};

/// Threshold for minimum documentation ratio
const MIN_DOC_RATIO: f32 = 0.1;

/// Size threshold for large file warning (1GB)
const LARGE_FILE_THRESHOLD: u64 = 1024 * 1024 * 1024;

/// Check data quality aspects
///
/// Validates data-to-documentation ratio, empty files, and file sizes.
pub fn check_data_quality(
    files: &[FileInfo],
    _analyses: &[AnalysisResult],
) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Check data-to-documentation ratio
    results.extend(check_documentation_ratio(files));

    // Check for empty files
    results.extend(check_empty_files(files));

    // Check for large files
    results.extend(check_large_files(files));

    results
}

/// Check data-to-documentation ratio
fn check_documentation_ratio(files: &[FileInfo]) -> Vec<ValidationResult> {
    if files.is_empty() {
        return vec![];
    }

    let doc_count = files.iter().filter(|f| f.is_documentation()).count();
    let total_count = files.len();
    let ratio = doc_count as f32 / total_count as f32;

    if ratio < MIN_DOC_RATIO {
        vec![ValidationResult::warning(
            "QUALITY-001",
            format!(
                "Low documentation ratio: {:.1}% ({} of {} files)",
                ratio * 100.0,
                doc_count,
                total_count
            ),
            "Add more documentation files (README, guides, data dictionaries)",
        )]
    } else {
        vec![]
    }
}

/// Check for empty files
fn check_empty_files(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    for file in files {
        if file.size_bytes == 0 {
            results.push(
                ValidationResult::warning(
                    "QUALITY-002",
                    "File is empty",
                    "Remove empty file or add content",
                )
                .with_file(file.relative_path.clone()),
            );
        }
    }

    results
}

/// Check for suspiciously large files
fn check_large_files(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    for file in files {
        if file.size_bytes > LARGE_FILE_THRESHOLD {
            let size_gb = file.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            results.push(
                ValidationResult::info(
                    "QUALITY-003",
                    format!("Large file: {:.2} GB", size_gb),
                    "Consider splitting large files for better accessibility and processing",
                )
                .with_file(file.relative_path.clone()),
            );
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_check_documentation_ratio_low() {
        let files = vec![
            FileInfo::new(PathBuf::from("data1.csv"), PathBuf::from("data1.csv")),
            FileInfo::new(PathBuf::from("data2.csv"), PathBuf::from("data2.csv")),
            FileInfo::new(PathBuf::from("data3.csv"), PathBuf::from("data3.csv")),
            FileInfo::new(PathBuf::from("data4.csv"), PathBuf::from("data4.csv")),
            FileInfo::new(PathBuf::from("data5.csv"), PathBuf::from("data5.csv")),
            FileInfo::new(PathBuf::from("data6.csv"), PathBuf::from("data6.csv")),
            FileInfo::new(PathBuf::from("data7.csv"), PathBuf::from("data7.csv")),
            FileInfo::new(PathBuf::from("data8.csv"), PathBuf::from("data8.csv")),
            FileInfo::new(PathBuf::from("data9.csv"), PathBuf::from("data9.csv")),
            FileInfo::new(PathBuf::from("data10.csv"), PathBuf::from("data10.csv")),
        ];

        let results = check_documentation_ratio(&files);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "QUALITY-001");
    }

    #[test]
    fn test_check_documentation_ratio_good() {
        let files = vec![
            FileInfo::new(
                PathBuf::from("README.md"),
                PathBuf::from("README.md"),
            ),
            FileInfo::new(
                PathBuf::from("LICENSE"),
                PathBuf::from("LICENSE"),
            ),
            FileInfo::new(PathBuf::from("data1.csv"), PathBuf::from("data1.csv")),
            FileInfo::new(PathBuf::from("data2.csv"), PathBuf::from("data2.csv")),
        ];

        let results = check_documentation_ratio(&files);
        // 2 doc files out of 4 = 50%, which is good
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_check_empty_files() {
        let files = vec![
            FileInfo::new(PathBuf::from("empty.csv"), PathBuf::from("empty.csv")).with_size(0),
            FileInfo::new(PathBuf::from("data.csv"), PathBuf::from("data.csv")).with_size(1024),
        ];

        let results = check_empty_files(&files);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "QUALITY-002");
    }

    #[test]
    fn test_check_large_files() {
        let large_size = 2 * 1024 * 1024 * 1024; // 2GB
        let files = vec![
            FileInfo::new(
                PathBuf::from("large.bin"),
                PathBuf::from("large.bin"),
            )
            .with_size(large_size),
            FileInfo::new(
                PathBuf::from("small.csv"),
                PathBuf::from("small.csv"),
            )
            .with_size(1024),
        ];

        let results = check_large_files(&files);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, "QUALITY-003");
    }

    #[test]
    fn test_check_data_quality_comprehensive() {
        use crate::types::FileType;

        let files = vec![
            FileInfo::new(PathBuf::from("empty.txt"), PathBuf::from("empty.txt"))
                .with_size(0)
                .with_type(FileType::Text),
            FileInfo::new(
                PathBuf::from("large.dat"),
                PathBuf::from("large.dat"),
            )
            .with_size(2 * 1024 * 1024 * 1024)
            .with_type(FileType::Binary),
            FileInfo::new(PathBuf::from("data.csv"), PathBuf::from("data.csv"))
                .with_size(1024)
                .with_type(FileType::Csv),
            // Add multiple data files to trigger low doc ratio
            FileInfo::new(PathBuf::from("data2.csv"), PathBuf::from("data2.csv"))
                .with_size(1024)
                .with_type(FileType::Csv),
            FileInfo::new(PathBuf::from("data3.csv"), PathBuf::from("data3.csv"))
                .with_size(1024)
                .with_type(FileType::Csv),
            FileInfo::new(PathBuf::from("data4.csv"), PathBuf::from("data4.csv"))
                .with_size(1024)
                .with_type(FileType::Csv),
            FileInfo::new(PathBuf::from("data5.csv"), PathBuf::from("data5.csv"))
                .with_size(1024)
                .with_type(FileType::Csv),
            FileInfo::new(PathBuf::from("data6.csv"), PathBuf::from("data6.csv"))
                .with_size(1024)
                .with_type(FileType::Csv),
            FileInfo::new(PathBuf::from("data7.csv"), PathBuf::from("data7.csv"))
                .with_size(1024)
                .with_type(FileType::Csv),
            FileInfo::new(PathBuf::from("data8.csv"), PathBuf::from("data8.csv"))
                .with_size(1024)
                .with_type(FileType::Csv),
            FileInfo::new(PathBuf::from("data9.csv"), PathBuf::from("data9.csv"))
                .with_size(1024)
                .with_type(FileType::Csv),
            FileInfo::new(PathBuf::from("data10.csv"), PathBuf::from("data10.csv"))
                .with_size(1024)
                .with_type(FileType::Csv),
            // Only 1 doc file out of 13 = 7.7% < 10% threshold
        ];

        let analyses = vec![];
        let results = check_data_quality(&files, &analyses);

        // Should have issues for empty file, large file, and low doc ratio
        assert!(results.iter().any(|r| r.code == "QUALITY-002"), "Expected QUALITY-002 for empty file");
        assert!(results.iter().any(|r| r.code == "QUALITY-003"), "Expected QUALITY-003 for large file");
        assert!(results.iter().any(|r| r.code == "QUALITY-001"), "Expected QUALITY-001 for low doc ratio");
    }
}
