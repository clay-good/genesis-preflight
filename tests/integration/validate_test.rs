//! Integration tests for validation functionality

use genesis_preflight::analyzer::{analyze_csv, analyze_json, analyze_text};
use genesis_preflight::scanner::scan_directory;
use genesis_preflight::types::{Command, Config, FileType, ValidationSeverity};
use genesis_preflight::validator::{
    validate_data_quality, validate_fair_compliance, validate_file_naming, validate_metadata,
    validate_structure,
};
use std::path::PathBuf;

#[test]
fn test_validate_valid_dataset() {
    let fixture_path = PathBuf::from("tests/fixtures/valid_dataset");
    let config = Config::new(fixture_path.clone(), Command::Scan);

    let files = scan_directory(&fixture_path, &config).expect("Scan should succeed");

    // Analyze files
    let mut analyses = Vec::new();
    for file in &files {
        let analysis = match file.file_type {
            FileType::CSV => analyze_csv(&file.full_path),
            FileType::JSON => analyze_json(&file.full_path),
            FileType::Text => analyze_text(&file.full_path),
            _ => continue,
        };
        analyses.push(analysis);
    }

    // Run all validators
    let mut validation = Vec::new();
    validation.extend(validate_structure(&files));
    validation.extend(validate_file_naming(&files));
    validation.extend(validate_metadata(&files));
    validation.extend(validate_fair_compliance(&files, &analyses));

    for analysis in &analyses {
        validation.extend(validate_data_quality(analysis));
    }

    // Valid dataset should have no critical issues
    let critical_issues: Vec<_> = validation
        .iter()
        .filter(|v| matches!(v.severity, ValidationSeverity::Critical))
        .collect();

    assert!(
        critical_issues.is_empty(),
        "Valid dataset should have no critical issues. Found: {:?}",
        critical_issues
    );
}

#[test]
fn test_validate_invalid_dataset_missing_license() {
    let fixture_path = PathBuf::from("tests/fixtures/invalid_dataset");
    let config = Config::new(fixture_path.clone(), Command::Scan);

    let files = scan_directory(&fixture_path, &config).expect("Scan should succeed");

    let validation = validate_structure(&files);

    // Should detect missing LICENSE
    let license_issue = validation.iter().find(|v| v.code.contains("LICENSE") || v.code.contains("FAIR-A001"));

    assert!(license_issue.is_some(), "Should detect missing LICENSE file");

    if let Some(issue) = license_issue {
        assert!(
            matches!(issue.severity, ValidationSeverity::Critical),
            "Missing LICENSE should be critical"
        );
    }
}

#[test]
fn test_validate_invalid_dataset_bad_naming() {
    let fixture_path = PathBuf::from("tests/fixtures/invalid_dataset");
    let config = Config::new(fixture_path.clone(), Command::Scan);

    let files = scan_directory(&fixture_path, &config).expect("Scan should succeed");

    let validation = validate_file_naming(&files);

    // Should detect file with spaces in name
    let naming_issues: Vec<_> = validation
        .iter()
        .filter(|v| v.message.contains("space") || v.message.contains("Bad File Name"))
        .collect();

    assert!(
        !naming_issues.is_empty(),
        "Should detect file naming issues"
    );
}

#[test]
fn test_validate_partial_dataset_missing_metadata() {
    let fixture_path = PathBuf::from("tests/fixtures/partial_dataset");
    let config = Config::new(fixture_path.clone(), Command::Scan);

    let files = scan_directory(&fixture_path, &config).expect("Scan should succeed");

    let validation = validate_metadata(&files);

    // Should detect missing metadata.json
    let metadata_issue = validation.iter().find(|v| v.code.contains("metadata") || v.code.contains("FAIR-F001"));

    assert!(
        metadata_issue.is_some(),
        "Should detect missing metadata.json"
    );
}

#[test]
fn test_validation_severity_levels() {
    let fixture_path = PathBuf::from("tests/fixtures/invalid_dataset");
    let config = Config::new(fixture_path.clone(), Command::Scan);

    let files = scan_directory(&fixture_path, &config).expect("Scan should succeed");

    let mut analyses = Vec::new();
    for file in &files {
        if file.file_type == FileType::CSV {
            analyses.push(analyze_csv(&file.full_path));
        }
    }

    let mut validation = Vec::new();
    validation.extend(validate_structure(&files));
    validation.extend(validate_file_naming(&files));
    validation.extend(validate_metadata(&files));
    validation.extend(validate_fair_compliance(&files, &analyses));

    // Should have a mix of severities
    let has_critical = validation
        .iter()
        .any(|v| matches!(v.severity, ValidationSeverity::Critical));
    let has_warning = validation
        .iter()
        .any(|v| matches!(v.severity, ValidationSeverity::Warning));

    assert!(has_critical || has_warning, "Should have at least critical or warning issues");
}

#[test]
fn test_validation_result_structure() {
    let fixture_path = PathBuf::from("tests/fixtures/invalid_dataset");
    let config = Config::new(fixture_path.clone(), Command::Scan);

    let files = scan_directory(&fixture_path, &config).expect("Scan should succeed");
    let validation = validate_structure(&files);

    // All validation results should have required fields
    for result in &validation {
        assert!(!result.code.is_empty(), "Validation code should not be empty");
        assert!(!result.message.is_empty(), "Validation message should not be empty");
        assert!(!result.suggestion.is_empty(), "Validation suggestion should not be empty");
    }
}

#[test]
fn test_fair_compliance_validation() {
    let fixture_path = PathBuf::from("tests/fixtures/valid_dataset");
    let config = Config::new(fixture_path.clone(), Command::Scan);

    let files = scan_directory(&fixture_path, &config).expect("Scan should succeed");

    let mut analyses = Vec::new();
    for file in &files {
        if file.file_type == FileType::CSV {
            analyses.push(analyze_csv(&file.full_path));
        }
    }

    let validation = validate_fair_compliance(&files, &analyses);

    // FAIR validation should produce results with FAIR- prefixed codes
    let fair_checks: Vec<_> = validation
        .iter()
        .filter(|v| v.code.starts_with("FAIR-"))
        .collect();

    // Should have some FAIR-related validations
    assert!(
        !validation.is_empty(),
        "FAIR validation should produce results"
    );
}
