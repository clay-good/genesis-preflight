//! Manifest integrity validation
//!
//! Validates dataset file integrity by comparing current file hashes
//! against a previously generated MANIFEST.txt file.

use crate::crypto::sha256_file;
use crate::types::{FileInfo, ValidationResult, ValidationSeverity};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Result of an integrity check
#[derive(Debug)]
pub struct IntegrityIssue {
    /// Type of integrity issue
    pub kind: IntegrityIssueKind,
    /// Relative path of the affected file
    pub path: String,
    /// Expected hash from manifest (if applicable)
    pub expected_hash: Option<String>,
    /// Actual hash of current file (if applicable)
    pub actual_hash: Option<String>,
}

/// Types of integrity issues
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityIssueKind {
    /// File has been modified since manifest was created
    Modified,
    /// File exists in manifest but is missing from filesystem
    Missing,
    /// File exists in filesystem but not in manifest
    New,
    /// Manifest file could not be parsed
    ManifestError,
}

impl IntegrityIssue {
    /// Create a new integrity issue for a modified file
    pub fn modified(path: String, expected: String, actual: String) -> Self {
        Self {
            kind: IntegrityIssueKind::Modified,
            path,
            expected_hash: Some(expected),
            actual_hash: Some(actual),
        }
    }

    /// Create a new integrity issue for a missing file
    pub fn missing(path: String, expected: String) -> Self {
        Self {
            kind: IntegrityIssueKind::Missing,
            path,
            expected_hash: Some(expected),
            actual_hash: None,
        }
    }

    /// Create a new integrity issue for a new file
    pub fn new_file(path: String, actual: String) -> Self {
        Self {
            kind: IntegrityIssueKind::New,
            path,
            expected_hash: None,
            actual_hash: Some(actual),
        }
    }

    /// Create an issue for manifest parsing errors
    pub fn manifest_error(message: String) -> Self {
        Self {
            kind: IntegrityIssueKind::ManifestError,
            path: message,
            expected_hash: None,
            actual_hash: None,
        }
    }
}

/// Parse a MANIFEST.txt file into a hash map of path -> hash
///
/// Expected format: "sha256_hash  relative/path/to/file"
/// (hash followed by two spaces and the path)
fn parse_manifest(manifest_path: &Path) -> Result<HashMap<String, String>, String> {
    let file = File::open(manifest_path)
        .map_err(|e| format!("Failed to open manifest: {}", e))?;
    let reader = BufReader::new(file);

    let mut entries = HashMap::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result
            .map_err(|e| format!("Failed to read manifest line {}: {}", line_num + 1, e))?;

        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse format: "hash  path" (hash, two spaces, path)
        // Compatible with sha256sum -c format
        if let Some(idx) = line.find("  ") {
            let hash = &line[..idx];
            let path = &line[idx + 2..];

            // Validate hash looks like SHA-256 (64 hex chars)
            if hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
                entries.insert(path.to_string(), hash.to_lowercase());
            } else {
                return Err(format!(
                    "Invalid hash on line {}: expected 64 hex characters",
                    line_num + 1
                ));
            }
        } else {
            // Try alternative format: "hash path" (single space)
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let hash = parts[0];
                let path = parts[1];

                if hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    entries.insert(path.to_string(), hash.to_lowercase());
                } else {
                    return Err(format!(
                        "Invalid format on line {}: {}",
                        line_num + 1,
                        line
                    ));
                }
            } else {
                return Err(format!(
                    "Invalid format on line {}: expected 'hash  path'",
                    line_num + 1
                ));
            }
        }
    }

    Ok(entries)
}

/// Verify dataset integrity against a manifest file
///
/// Compares the SHA-256 hashes of current files against those stored
/// in the manifest, reporting any discrepancies.
///
/// # Arguments
///
/// * `manifest_path` - Path to the MANIFEST.txt file
/// * `files` - Current list of files in the dataset
/// * `base_path` - Base path for computing relative paths
///
/// # Returns
///
/// A vector of IntegrityIssue items describing any discrepancies found.
pub fn verify_manifest(
    manifest_path: &Path,
    files: &[FileInfo],
    _base_path: &Path,
) -> Vec<IntegrityIssue> {
    let mut issues = Vec::new();

    // Parse the manifest
    let manifest_entries = match parse_manifest(manifest_path) {
        Ok(entries) => entries,
        Err(e) => {
            issues.push(IntegrityIssue::manifest_error(e));
            return issues;
        }
    };

    // Build a map of current files
    let mut current_files: HashMap<String, &FileInfo> = HashMap::new();
    for file in files {
        let rel_path = file.relative_path.to_string_lossy().to_string();
        current_files.insert(rel_path, file);
    }

    // Check each manifest entry against current files
    for (manifest_path, expected_hash) in &manifest_entries {
        // Skip the manifest file itself
        if manifest_path == "MANIFEST.txt" || manifest_path.ends_with("/MANIFEST.txt") {
            continue;
        }

        if let Some(file_info) = current_files.get(manifest_path) {
            // File exists - check hash
            if let Some(ref actual_hash) = file_info.sha256_hash {
                if actual_hash.to_lowercase() != *expected_hash {
                    issues.push(IntegrityIssue::modified(
                        manifest_path.clone(),
                        expected_hash.clone(),
                        actual_hash.clone(),
                    ));
                }
            } else {
                // Hash not computed - compute it now
                match sha256_file(&file_info.full_path) {
                    Ok(actual_hash) => {
                        if actual_hash.to_lowercase() != *expected_hash {
                            issues.push(IntegrityIssue::modified(
                                manifest_path.clone(),
                                expected_hash.clone(),
                                actual_hash,
                            ));
                        }
                    }
                    Err(_) => {
                        // Can't read file - treat as missing
                        issues.push(IntegrityIssue::missing(
                            manifest_path.clone(),
                            expected_hash.clone(),
                        ));
                    }
                }
            }

            // Remove from current files map (we've processed it)
            current_files.remove(manifest_path);
        } else {
            // File in manifest but not on disk
            issues.push(IntegrityIssue::missing(
                manifest_path.clone(),
                expected_hash.clone(),
            ));
        }
    }

    // Check for new files (in current but not in manifest)
    for (path, file_info) in current_files {
        // Skip documentation files that might have been generated after manifest
        if path == "MANIFEST.txt"
            || path.ends_with("/MANIFEST.txt")
            || path == "README.md"
            || path == "metadata.json"
            || path == "DATACARD.md"
            || path.ends_with(".schema.json")
        {
            continue;
        }

        // Compute hash for new file
        let actual_hash = if let Some(ref hash) = file_info.sha256_hash {
            hash.clone()
        } else {
            sha256_file(&file_info.full_path).unwrap_or_else(|_| "unknown".to_string())
        };

        issues.push(IntegrityIssue::new_file(path, actual_hash));
    }

    issues
}

/// Check manifest integrity as part of validation
///
/// If a MANIFEST.txt exists, verify all files match their recorded hashes.
/// Returns validation results for any integrity issues found.
pub fn check_integrity(files: &[FileInfo], base_path: &Path) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Look for manifest file
    let manifest_path = base_path.join("MANIFEST.txt");
    if !manifest_path.exists() {
        // No manifest - nothing to check
        return results;
    }

    // Verify integrity
    let issues = verify_manifest(&manifest_path, files, base_path);

    for issue in issues {
        let result = match issue.kind {
            IntegrityIssueKind::Modified => ValidationResult::new(
                ValidationSeverity::Critical,
                "INTEGRITY-001".to_string(),
                format!(
                    "File has been modified since manifest was created: {}",
                    issue.path
                ),
            )
            .with_suggestion(format!(
                "Expected hash: {}, actual hash: {}. Regenerate manifest if changes are intentional.",
                issue.expected_hash.unwrap_or_default(),
                issue.actual_hash.unwrap_or_default()
            )),

            IntegrityIssueKind::Missing => ValidationResult::new(
                ValidationSeverity::Critical,
                "INTEGRITY-002".to_string(),
                format!(
                    "File listed in manifest is missing: {}",
                    issue.path
                ),
            )
            .with_suggestion(
                "Restore the missing file or regenerate the manifest if removal was intentional."
                    .to_string(),
            ),

            IntegrityIssueKind::New => ValidationResult::new(
                ValidationSeverity::Warning,
                "INTEGRITY-003".to_string(),
                format!(
                    "File not in manifest (added after manifest was created): {}",
                    issue.path
                ),
            )
            .with_suggestion(
                "Regenerate the manifest to include new files.".to_string(),
            ),

            IntegrityIssueKind::ManifestError => ValidationResult::new(
                ValidationSeverity::Warning,
                "INTEGRITY-004".to_string(),
                format!("Could not parse manifest: {}", issue.path),
            )
            .with_suggestion(
                "Ensure MANIFEST.txt follows the format: 'sha256_hash  path'".to_string(),
            ),
        };

        results.push(result);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_manifest_valid() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_manifest_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let manifest_path = temp_dir.join("MANIFEST.txt");
        {
            let mut file = File::create(&manifest_path).unwrap();
            writeln!(
                file,
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  empty.txt"
            )
            .unwrap();
            writeln!(
                file,
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  abc.txt"
            )
            .unwrap();
        }

        let entries = parse_manifest(&manifest_path).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries.get("empty.txt"),
            Some(&"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string())
        );

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_parse_manifest_with_comments() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_manifest_comments");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let manifest_path = temp_dir.join("MANIFEST.txt");
        {
            let mut file = File::create(&manifest_path).unwrap();
            writeln!(file, "# This is a comment").unwrap();
            writeln!(file, "").unwrap();
            writeln!(
                file,
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  file.txt"
            )
            .unwrap();
        }

        let entries = parse_manifest(&manifest_path).unwrap();
        assert_eq!(entries.len(), 1);

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_parse_manifest_invalid_hash() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_manifest_invalid");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let manifest_path = temp_dir.join("MANIFEST.txt");
        {
            let mut file = File::create(&manifest_path).unwrap();
            writeln!(file, "notahash  file.txt").unwrap();
        }

        let result = parse_manifest(&manifest_path);
        assert!(result.is_err());

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_integrity_issue_modified() {
        let issue = IntegrityIssue::modified(
            "data.csv".to_string(),
            "abc123".to_string(),
            "def456".to_string(),
        );
        assert_eq!(issue.kind, IntegrityIssueKind::Modified);
        assert_eq!(issue.expected_hash, Some("abc123".to_string()));
        assert_eq!(issue.actual_hash, Some("def456".to_string()));
    }

    #[test]
    fn test_integrity_issue_missing() {
        let issue = IntegrityIssue::missing("data.csv".to_string(), "abc123".to_string());
        assert_eq!(issue.kind, IntegrityIssueKind::Missing);
        assert!(issue.actual_hash.is_none());
    }

    #[test]
    fn test_integrity_issue_new() {
        let issue = IntegrityIssue::new_file("data.csv".to_string(), "abc123".to_string());
        assert_eq!(issue.kind, IntegrityIssueKind::New);
        assert!(issue.expected_hash.is_none());
    }
}
