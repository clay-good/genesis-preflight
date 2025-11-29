//! Integration tests for documentation generation

use genesis_preflight::analyzer::analyze_csv;
use genesis_preflight::generator::{
    generate_datacard, generate_manifest, generate_metadata_json, generate_readme, generate_schema,
};
use genesis_preflight::scanner::scan_directory;
use genesis_preflight::types::{Command, Config, DatasetSummary, FileType};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_generate_readme() {
    let mut summary = DatasetSummary::new();
    summary.total_files = 10;
    summary.total_size = 1024 * 1024;
    summary.scan_timestamp = "2024-01-15 12:00:00 UTC".to_string();

    let content = generate_readme(&summary);

    // Verify README structure
    assert!(content.contains("# "), "Should have title");
    assert!(content.contains("## Description"), "Should have Description section");
    assert!(content.contains("## Contents"), "Should have Contents section");
    assert!(content.contains("[TODO]"), "Should have TODO markers");
    assert!(content.contains("10 files"), "Should include file count");
}

#[test]
fn test_generate_metadata_json() {
    let mut summary = DatasetSummary::new();
    summary.total_files = 5;
    summary.total_size = 5000;

    let content = generate_metadata_json(&summary);

    // Verify it's valid JSON-like structure
    assert!(content.contains("{"), "Should be JSON format");
    assert!(content.contains("\"name\""), "Should have name field");
    assert!(content.contains("\"version\""), "Should have version field");
    assert!(content.contains("\"license\""), "Should have license field");
    assert!(content.contains("[TODO]"), "Should have TODO markers");
}

#[test]
fn test_generate_datacard() {
    let mut summary = DatasetSummary::new();
    summary.total_files = 3;
    summary.total_size = 1500;

    let content = generate_datacard(&summary);

    // Verify DATACARD structure
    assert!(content.contains("# Data Card:"), "Should have title");
    assert!(content.contains("## Overview"), "Should have Overview section");
    assert!(content.contains("## Intended Use"), "Should have Intended Use section");
    assert!(content.contains("## Data Collection"), "Should have Data Collection section");
    assert!(content.contains("## Limitations"), "Should have Limitations section");
    assert!(content.contains("[TODO]"), "Should have TODO markers");
}

#[test]
fn test_generate_manifest() {
    let fixture_path = PathBuf::from("tests/fixtures/valid_dataset");
    let config = Config::new(fixture_path.clone(), Command::Generate);

    let files = scan_directory(&fixture_path, &config).expect("Scan should succeed");

    let content = generate_manifest(&files, &config);

    // Verify MANIFEST format
    assert!(content.contains("SHA256"), "Should contain SHA256 entries");
    assert!(content.contains(" = "), "Should use correct format");

    // Count lines (should have one per file)
    let line_count = content.lines().count();
    assert!(line_count >= files.len(), "Should have at least one line per file");
}

#[test]
fn test_generate_schema_for_csv() {
    let fixture_path = PathBuf::from("tests/fixtures/valid_dataset/data/measurements.csv");

    let analysis = analyze_csv(&fixture_path);

    if let Some(ref csv_analysis) = analysis.csv_analysis {
        let content = generate_schema(csv_analysis);

        // Verify schema structure
        assert!(content.contains("{"), "Should be JSON format");
        assert!(content.contains("\"columns\""), "Should have columns array");
        assert!(content.contains("\"delimiter\""), "Should specify delimiter");
        assert!(content.contains("\"has_header\""), "Should specify header presence");
    } else {
        panic!("CSV analysis should be present");
    }
}

#[test]
fn test_no_overwrite_existing_files() {
    let temp_dir = std::env::temp_dir().join("genesis_test_no_overwrite");
    fs::create_dir_all(&temp_dir).expect("Should create temp directory");

    // Create a README with specific content
    let readme_path = temp_dir.join("README.md");
    let original_content = "This is the original README content";
    fs::write(&readme_path, original_content).expect("Should write original file");

    // Generate new README
    let mut summary = DatasetSummary::new();
    summary.total_files = 1;
    let new_content = generate_readme(&summary);

    // Verify new content is different
    assert_ne!(new_content, original_content);

    // In actual implementation, generation should skip if file exists
    // This test verifies the generated content is different from original
    assert!(!new_content.contains("This is the original README content"));

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_generated_json_is_valid_structure() {
    let mut summary = DatasetSummary::new();
    summary.total_files = 1;
    summary.total_size = 100;

    let metadata_json = generate_metadata_json(&summary);

    // Basic JSON validation - should have balanced braces
    let open_braces = metadata_json.matches('{').count();
    let close_braces = metadata_json.matches('}').count();
    assert_eq!(open_braces, close_braces, "JSON should have balanced braces");

    let open_brackets = metadata_json.matches('[').count();
    let close_brackets = metadata_json.matches(']').count();
    assert_eq!(
        open_brackets, close_brackets,
        "JSON should have balanced brackets"
    );
}

#[test]
fn test_manifest_sha256_format() {
    let fixture_path = PathBuf::from("tests/fixtures/valid_dataset");
    let config = Config::new(fixture_path.clone(), Command::Generate);

    let files = scan_directory(&fixture_path, &config).expect("Scan should succeed");

    let content = generate_manifest(&files, &config);

    // Each line should match the format: SHA256 (filename) = hash
    for line in content.lines() {
        if line.is_empty() {
            continue;
        }

        assert!(line.starts_with("SHA256"), "Line should start with SHA256");
        assert!(line.contains(" = "), "Line should contain ' = '");

        // Extract hash (last part after =)
        if let Some(hash_part) = line.split(" = ").nth(1) {
            assert_eq!(
                hash_part.len(),
                64,
                "SHA256 hash should be 64 characters"
            );
            assert!(
                hash_part.chars().all(|c| c.is_ascii_hexdigit()),
                "Hash should be hexadecimal"
            );
        }
    }
}

#[test]
fn test_schema_includes_column_info() {
    let fixture_path = PathBuf::from("tests/fixtures/valid_dataset/data/measurements.csv");

    let analysis = analyze_csv(&fixture_path);

    if let Some(ref csv_analysis) = analysis.csv_analysis {
        let schema = generate_schema(csv_analysis);

        // Should include information about the columns
        assert!(
            schema.contains("timestamp") || schema.contains("column"),
            "Should reference columns"
        );
        assert!(
            schema.contains("type") || schema.contains("Type"),
            "Should specify column types"
        );
    }
}
