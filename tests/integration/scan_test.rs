//! Integration tests for scanning functionality

use genesis_preflight::scanner::scan_directory;
use genesis_preflight::types::{Command, Config, FileType};
use std::path::PathBuf;

#[test]
fn test_scan_valid_dataset() {
    let fixture_path = PathBuf::from("tests/fixtures/valid_dataset");
    let config = Config::new(fixture_path.clone(), Command::Scan);

    let result = scan_directory(&fixture_path, &config);
    assert!(result.is_ok(), "Scan should succeed on valid dataset");

    let files = result.unwrap();

    // Should find README, LICENSE, metadata.json, and measurements.csv
    assert!(files.len() >= 4, "Should find at least 4 files");

    // Verify file types are detected correctly
    let csv_files: Vec<_> = files.iter().filter(|f| f.file_type == FileType::CSV).collect();
    assert!(!csv_files.is_empty(), "Should detect CSV files");

    let json_files: Vec<_> = files.iter().filter(|f| f.file_type == FileType::JSON).collect();
    assert!(!json_files.is_empty(), "Should detect JSON files");

    // Verify files have hashes
    for file in &files {
        assert!(!file.sha256_hash.is_empty(), "Files should have SHA-256 hashes");
        assert_eq!(file.sha256_hash.len(), 64, "SHA-256 hash should be 64 characters");
    }
}

#[test]
fn test_scan_invalid_dataset() {
    let fixture_path = PathBuf::from("tests/fixtures/invalid_dataset");
    let config = Config::new(fixture_path.clone(), Command::Scan);

    let result = scan_directory(&fixture_path, &config);
    assert!(result.is_ok(), "Scan should succeed even on invalid dataset");

    let files = result.unwrap();

    // Should find at least the CSV file and readme.txt
    assert!(files.len() >= 2, "Should find files");

    // Check that bad filename is present
    let bad_file = files.iter().find(|f| f.relative_path.to_string_lossy().contains("Bad File Name"));
    assert!(bad_file.is_some(), "Should find file with spaces in name");
}

#[test]
fn test_scan_partial_dataset() {
    let fixture_path = PathBuf::from("tests/fixtures/partial_dataset");
    let config = Config::new(fixture_path.clone(), Command::Scan);

    let result = scan_directory(&fixture_path, &config);
    assert!(result.is_ok());

    let files = result.unwrap();

    // Should find README and data.csv
    assert!(files.len() >= 2);

    // Verify README is detected as text
    let readme = files.iter().find(|f| f.relative_path.to_string_lossy().contains("README"));
    assert!(readme.is_some());
}

#[test]
fn test_scan_no_hash_flag() {
    let fixture_path = PathBuf::from("tests/fixtures/valid_dataset");
    let mut config = Config::new(fixture_path.clone(), Command::Scan);
    config.skip_hash = true;

    let result = scan_directory(&fixture_path, &config);
    assert!(result.is_ok());

    let files = result.unwrap();

    // When skip_hash is true, hashes should be empty
    for file in &files {
        assert!(file.sha256_hash.is_empty(), "Hashes should be empty with --no-hash flag");
    }
}

#[test]
fn test_scan_file_sizes() {
    let fixture_path = PathBuf::from("tests/fixtures/valid_dataset");
    let config = Config::new(fixture_path.clone(), Command::Scan);

    let result = scan_directory(&fixture_path, &config);
    assert!(result.is_ok());

    let files = result.unwrap();

    // All files should have non-zero size
    for file in &files {
        assert!(file.size_bytes > 0, "Files should have non-zero size: {:?}", file.relative_path);
    }
}

#[test]
fn test_scan_nonexistent_directory() {
    let fixture_path = PathBuf::from("tests/fixtures/nonexistent");
    let config = Config::new(fixture_path.clone(), Command::Scan);

    let result = scan_directory(&fixture_path, &config);
    assert!(result.is_err(), "Should error on nonexistent directory");
}
