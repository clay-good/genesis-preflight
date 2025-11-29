//! Integration tests for Genesis Preflight
//!
//! These tests verify the complete workflow of scanning, validating,
//! and generating documentation for datasets.

use std::fs;
use std::path::PathBuf;

/// Helper to get the test data directory
fn test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test")
}

/// Helper to create a temporary test directory
fn create_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("genesis_preflight_test_{}", name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("Failed to create temp directory");
    dir
}

/// Helper to clean up a temporary directory
fn cleanup_temp_dir(dir: &PathBuf) {
    let _ = fs::remove_dir_all(dir);
}

mod csv_analysis {
    use super::*;
    use genesis_preflight::analyzer::analyze_csv;

    #[test]
    fn test_analyze_temperature_readings() {
        let path = test_data_dir().join("valid_dataset/temperature_readings.csv");
        if !path.exists() {
            eprintln!("Skipping test: test data not found at {:?}", path);
            return;
        }

        let result = analyze_csv(&path);
        assert!(result.is_ok(), "Failed to analyze CSV: {:?}", result.err());

        let analysis = result.unwrap();
        assert!(analysis.has_header, "Should detect header row");
        assert_eq!(analysis.delimiter, ',', "Should detect comma delimiter");
        assert_eq!(analysis.column_count, 6, "Should have 6 columns");
        assert!(analysis.row_count >= 10, "Should have at least 10 data rows");
    }

    #[test]
    fn test_analyze_quoted_fields() {
        let path = test_data_dir().join("valid_dataset/quoted_fields.csv");
        if !path.exists() {
            return;
        }

        let result = analyze_csv(&path);
        assert!(result.is_ok(), "Failed to analyze CSV with quoted fields");

        let analysis = result.unwrap();
        assert_eq!(analysis.column_count, 4);
    }

    #[test]
    fn test_analyze_tab_delimited() {
        let path = test_data_dir().join("edge_cases/tab_delimited.tsv");
        if !path.exists() {
            return;
        }

        let result = analyze_csv(&path);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.delimiter, '\t', "Should detect tab delimiter");
    }

    #[test]
    fn test_analyze_semicolon_delimited() {
        let path = test_data_dir().join("edge_cases/semicolon_delimited.csv");
        if !path.exists() {
            return;
        }

        let result = analyze_csv(&path);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.delimiter, ';', "Should detect semicolon delimiter");
    }

    #[test]
    fn test_analyze_empty_values() {
        let path = test_data_dir().join("edge_cases/empty_values.csv");
        if !path.exists() {
            return;
        }

        let result = analyze_csv(&path);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        // Check that null counts are tracked
        assert!(
            analysis.columns.iter().any(|c| c.null_count > 0),
            "Should detect null/empty values"
        );
    }
}

mod json_analysis {
    use super::*;
    use genesis_preflight::analyzer::analyze_json;

    #[test]
    fn test_analyze_metadata_json() {
        let path = test_data_dir().join("valid_dataset/metadata.json");
        if !path.exists() {
            return;
        }

        let result = analyze_json(&path);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.is_valid, "JSON should be valid");
    }

    #[test]
    fn test_analyze_nested_json() {
        let path = test_data_dir().join("edge_cases/nested_data.json");
        if !path.exists() {
            return;
        }

        let result = analyze_json(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_array_root_json() {
        let path = test_data_dir().join("edge_cases/array_root.json");
        if !path.exists() {
            return;
        }

        let result = analyze_json(&path);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.is_valid);
    }
}

mod validation {
    use super::*;
    use genesis_preflight::types::{FileInfo, ValidationSeverity};
    use genesis_preflight::validator::{check_structure, validate_metadata};

    #[test]
    fn test_validate_complete_dataset() {
        let valid_dir = test_data_dir().join("valid_dataset");
        if !valid_dir.exists() {
            return;
        }

        // Create FileInfo for each file
        let files: Vec<FileInfo> = fs::read_dir(&valid_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .map(|e| {
                FileInfo::new(
                    e.path(),
                    e.path().file_name().unwrap().into(),
                )
            })
            .collect();

        // Check structure
        let structure_results = check_structure(&files);

        // Should not have critical issues for missing README/LICENSE
        let critical_count = structure_results
            .iter()
            .filter(|r| r.severity == ValidationSeverity::Critical)
            .count();

        assert_eq!(
            critical_count, 0,
            "Valid dataset should have no critical structure issues"
        );
    }

    #[test]
    fn test_validate_incomplete_dataset() {
        let incomplete_dir = test_data_dir().join("incomplete_dataset");
        if !incomplete_dir.exists() {
            return;
        }

        let files: Vec<FileInfo> = fs::read_dir(&incomplete_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .map(|e| {
                FileInfo::new(
                    e.path(),
                    e.path().file_name().unwrap().into(),
                )
            })
            .collect();

        let structure_results = check_structure(&files);

        // Should have critical issue for missing LICENSE
        let has_license_issue = structure_results
            .iter()
            .any(|r| r.message.to_lowercase().contains("license"));

        assert!(
            has_license_issue,
            "Incomplete dataset should flag missing LICENSE"
        );
    }
}

mod integrity {
    use super::*;
    use genesis_preflight::types::FileInfo;
    use genesis_preflight::validator::check_integrity;

    #[test]
    fn test_manifest_integrity_valid() {
        let valid_dir = test_data_dir().join("valid_dataset");
        if !valid_dir.exists() || !valid_dir.join("MANIFEST.txt").exists() {
            return;
        }

        let files: Vec<FileInfo> = fs::read_dir(&valid_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .map(|e| {
                FileInfo::new(
                    e.path(),
                    e.path().file_name().unwrap().into(),
                )
            })
            .collect();

        let integrity_results = check_integrity(&files, &valid_dir);

        // Should have no critical integrity issues if files match manifest
        let critical_integrity = integrity_results
            .iter()
            .filter(|r| r.code.starts_with("INTEGRITY-001") || r.code.starts_with("INTEGRITY-002"))
            .count();

        assert_eq!(
            critical_integrity, 0,
            "Valid dataset should pass integrity check"
        );
    }
}

mod content_validation {
    use super::*;
    use genesis_preflight::validator::detect_todo_markers;

    #[test]
    fn test_detect_todo_in_incomplete_readme() {
        let readme_path = test_data_dir().join("incomplete_dataset/README.md");
        if !readme_path.exists() {
            return;
        }

        let todos = detect_todo_markers(&readme_path);
        assert!(
            !todos.is_empty(),
            "Should detect TODO markers in incomplete README"
        );
    }

    #[test]
    fn test_no_todo_in_complete_readme() {
        let readme_path = test_data_dir().join("valid_dataset/README.md");
        if !readme_path.exists() {
            return;
        }

        let todos = detect_todo_markers(&readme_path);
        assert!(
            todos.is_empty(),
            "Should not find TODO markers in complete README"
        );
    }
}

mod generator {
    use super::*;
    use genesis_preflight::generator::{generate_readme, generate_metadata};
    use genesis_preflight::types::{DatasetSummary, FileType};

    #[test]
    fn test_generate_readme() {
        let mut summary = DatasetSummary::new();
        summary.total_files = 5;
        summary.total_size = 10240;
        summary.file_type_counts = vec![
            (FileType::Csv, 3),
            (FileType::Json, 2),
        ];

        let readme = generate_readme(&summary);

        assert!(readme.contains("# [TODO:"), "README should have title with TODO marker");
        assert!(readme.contains("[TODO]"), "Generated README should have TODO markers");
    }

    #[test]
    fn test_generate_metadata() {
        let mut summary = DatasetSummary::new();
        summary.total_files = 10;
        summary.total_size = 1048576;
        summary.scan_timestamp = "2024-01-15T12:00:00Z".to_string();

        let metadata = generate_metadata(&summary);

        assert!(metadata.contains("\"title\""), "Metadata should have title field");
        assert!(metadata.contains("\"description\""), "Metadata should have description field");
    }
}

mod crypto {
    use super::*;
    use genesis_preflight::crypto::sha256_file;

    #[test]
    fn test_sha256_consistency() {
        let temp_dir = create_temp_dir("sha256_test");
        let test_file = temp_dir.join("test.txt");

        fs::write(&test_file, "Hello, World!").unwrap();

        let hash1 = sha256_file(&test_file).unwrap();
        let hash2 = sha256_file(&test_file).unwrap();

        assert_eq!(hash1, hash2, "SHA-256 should be deterministic");
        assert_eq!(hash1.len(), 64, "SHA-256 hash should be 64 hex characters");

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_sha256_known_value() {
        let temp_dir = create_temp_dir("sha256_known");
        let test_file = temp_dir.join("empty.txt");

        // Empty file has known SHA-256
        fs::write(&test_file, "").unwrap();

        let hash = sha256_file(&test_file).unwrap();
        assert_eq!(
            hash.to_lowercase(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "Empty file should have known SHA-256"
        );

        cleanup_temp_dir(&temp_dir);
    }
}

mod scoring {
    use genesis_preflight::types::{ValidationResult, ValidationSeverity};
    use genesis_preflight::reporter::calculate_score;

    #[test]
    fn test_perfect_score() {
        let results: Vec<ValidationResult> = vec![];
        let score = calculate_score(&results);

        assert_eq!(score.total, 100, "No issues should give perfect score");
    }

    #[test]
    fn test_critical_deduction() {
        let results = vec![
            ValidationResult::critical("TEST-001", "Critical issue", "Fix it"),
        ];
        let score = calculate_score(&results);

        assert!(score.total < 100, "Critical issue should reduce score");
        assert_eq!(score.critical_count, 1);
    }

    #[test]
    fn test_warning_deduction() {
        let results = vec![
            ValidationResult::warning("TEST-001", "Warning issue", "Consider fixing"),
        ];
        let score = calculate_score(&results);

        assert!(score.total < 100, "Warning should reduce score");
        assert_eq!(score.warning_count, 1);
    }
}

mod edge_case_csv {
    use super::*;
    use genesis_preflight::analyzer::analyze_csv;

    #[test]
    fn test_single_column_csv() {
        let path = test_data_dir().join("edge_cases/single_column.csv");
        if !path.exists() {
            return;
        }

        let result = analyze_csv(&path);
        assert!(result.is_ok(), "Should parse single column CSV");

        let analysis = result.unwrap();
        assert_eq!(analysis.column_count, 1, "Should have exactly 1 column");
    }

    #[test]
    fn test_wide_table_csv() {
        let path = test_data_dir().join("edge_cases/wide_table.csv");
        if !path.exists() {
            return;
        }

        let result = analyze_csv(&path);
        assert!(result.is_ok(), "Should parse wide CSV");

        let analysis = result.unwrap();
        assert_eq!(analysis.column_count, 20, "Should have 20 columns");
    }

    #[test]
    fn test_special_characters_csv() {
        let path = test_data_dir().join("edge_cases/special_characters.csv");
        if !path.exists() {
            return;
        }

        let result = analyze_csv(&path);
        assert!(result.is_ok(), "Should handle special characters in CSV");
    }

    #[test]
    fn test_boolean_variations_csv() {
        let path = test_data_dir().join("edge_cases/boolean_variations.csv");
        if !path.exists() {
            return;
        }

        let result = analyze_csv(&path);
        assert!(result.is_ok(), "Should parse boolean variations");
    }

    #[test]
    fn test_date_formats_csv() {
        let path = test_data_dir().join("edge_cases/date_formats.csv");
        if !path.exists() {
            return;
        }

        let result = analyze_csv(&path);
        assert!(result.is_ok(), "Should parse various date formats");
    }

    #[test]
    fn test_numeric_extremes_csv() {
        let path = test_data_dir().join("edge_cases/numeric_extremes.csv");
        if !path.exists() {
            return;
        }

        let result = analyze_csv(&path);
        assert!(result.is_ok(), "Should handle numeric extremes");
    }
}

mod edge_case_json {
    use super::*;
    use genesis_preflight::analyzer::analyze_json;

    #[test]
    fn test_minimal_json() {
        let path = test_data_dir().join("edge_cases/minimal.json");
        if !path.exists() {
            return;
        }

        let result = analyze_json(&path);
        assert!(result.is_ok(), "Should parse minimal JSON");

        let analysis = result.unwrap();
        assert!(analysis.is_valid, "Empty object should be valid JSON");
    }

    #[test]
    fn test_complex_nested_json() {
        let path = test_data_dir().join("edge_cases/complex_nested.json");
        if !path.exists() {
            return;
        }

        let result = analyze_json(&path);
        assert!(result.is_ok(), "Should parse complex nested JSON");

        let analysis = result.unwrap();
        assert!(analysis.is_valid, "Complex nested JSON should be valid");
    }
}

mod file_type_detection {
    use genesis_preflight::types::FileType;
    use std::path::Path;

    #[test]
    fn test_csv_detection() {
        assert_eq!(FileType::from_path(Path::new("data.csv")), FileType::Csv);
        assert_eq!(FileType::from_path(Path::new("DATA.CSV")), FileType::Csv);
    }

    #[test]
    fn test_tsv_detection() {
        assert_eq!(FileType::from_path(Path::new("data.tsv")), FileType::Tsv);
    }

    #[test]
    fn test_json_detection() {
        assert_eq!(FileType::from_path(Path::new("config.json")), FileType::Json);
    }

    #[test]
    fn test_markdown_detection() {
        assert_eq!(FileType::from_path(Path::new("README.md")), FileType::Markdown);
    }

    #[test]
    fn test_binary_detection() {
        assert_eq!(FileType::from_path(Path::new("image.png")), FileType::Binary);
        assert_eq!(FileType::from_path(Path::new("photo.jpg")), FileType::Binary);
        assert_eq!(FileType::from_path(Path::new("doc.pdf")), FileType::Binary);
        assert_eq!(FileType::from_path(Path::new("data.hdf5")), FileType::Binary);
    }
}

mod scanner {
    use super::*;
    use genesis_preflight::scanner::scan_directory;
    use genesis_preflight::types::{Config, Command};

    #[test]
    fn test_scan_valid_dataset() {
        let valid_dir = test_data_dir().join("valid_dataset");
        if !valid_dir.exists() {
            return;
        }

        let config = Config::new(valid_dir.clone(), Command::Scan);
        let result = scan_directory(&valid_dir, &config);

        assert!(result.is_ok(), "Should scan valid dataset");
        let files = result.unwrap();
        assert!(files.len() >= 5, "Should find multiple files");
    }

    #[test]
    fn test_scan_edge_cases() {
        let edge_dir = test_data_dir().join("edge_cases");
        if !edge_dir.exists() {
            return;
        }

        let config = Config::new(edge_dir.clone(), Command::Scan);
        let result = scan_directory(&edge_dir, &config);

        assert!(result.is_ok(), "Should scan edge cases directory");
    }
}
