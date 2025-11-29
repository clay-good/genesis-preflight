//! Directory scanning and file metadata extraction module
//!
//! This module handles traversing dataset directories and collecting
//! metadata about all files found.

mod directory;
mod file_info_builder;

use crate::types::{Config, FileInfo};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// Errors that can occur during directory scanning
#[derive(Debug)]
pub enum ScanError {
    /// IO error occurred
    Io(io::Error),
    /// Path does not exist
    PathNotFound(PathBuf),
    /// Path is not a directory
    NotADirectory(PathBuf),
    /// Permission denied
    PermissionDenied(PathBuf),
    /// Invalid path
    InvalidPath(String),
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::Io(e) => write!(f, "IO error: {}", e),
            ScanError::PathNotFound(p) => write!(f, "Path not found: {}", p.display()),
            ScanError::NotADirectory(p) => write!(f, "Not a directory: {}", p.display()),
            ScanError::PermissionDenied(p) => write!(f, "Permission denied: {}", p.display()),
            ScanError::InvalidPath(s) => write!(f, "Invalid path: {}", s),
        }
    }
}

impl From<io::Error> for ScanError {
    fn from(error: io::Error) -> Self {
        ScanError::Io(error)
    }
}

impl std::error::Error for ScanError {}

/// Scan a directory and collect file information
///
/// Recursively traverses the directory tree starting at `path` and collects
/// metadata about all files found. Hidden files and common build directories
/// are excluded by default.
///
/// # Arguments
///
/// * `path` - Root directory to scan
/// * `config` - Configuration options controlling scan behavior
///
/// # Returns
///
/// A vector of FileInfo structs sorted by relative path, or a ScanError if
/// the scan cannot be performed.
///
/// # Examples
///
/// ```no_run
/// use genesis_preflight::scanner::scan_directory;
/// use genesis_preflight::types::{Config, Command};
/// use std::path::PathBuf;
///
/// let config = Config::new(PathBuf::from("/data"), Command::Scan);
/// let files = scan_directory(&PathBuf::from("/data"), &config).unwrap();
/// println!("Found {} files", files.len());
/// ```
pub fn scan_directory(path: &Path, config: &Config) -> Result<Vec<FileInfo>, ScanError> {
    // Validate path exists
    if !path.exists() {
        return Err(ScanError::PathNotFound(path.to_path_buf()));
    }

    // Validate path is a directory
    if !path.is_dir() {
        return Err(ScanError::NotADirectory(path.to_path_buf()));
    }

    // Scan directory
    let mut files = directory::walk_directory(path, config)?;

    // Sort by relative path
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Command;
    use std::fs;

    #[test]
    fn test_scan_nonexistent_path() {
        let config = Config::new(PathBuf::from("/nonexistent"), Command::Scan);
        let result = scan_directory(&PathBuf::from("/nonexistent"), &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_test_empty");
        fs::create_dir_all(&temp_dir).unwrap();

        let config = Config::new(temp_dir.clone(), Command::Scan);
        let result = scan_directory(&temp_dir, &config).unwrap();
        assert_eq!(result.len(), 0);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_scan_with_files() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_test_files");
        fs::create_dir_all(&temp_dir).unwrap();

        // Create test files
        fs::write(temp_dir.join("file1.txt"), "test").unwrap();
        fs::write(temp_dir.join("file2.csv"), "a,b,c").unwrap();

        let config = Config::new(temp_dir.clone(), Command::Scan);
        let result = scan_directory(&temp_dir, &config).unwrap();
        assert_eq!(result.len(), 2);

        // Check sorted order
        assert_eq!(result[0].file_name(), Some("file1.txt"));
        assert_eq!(result[1].file_name(), Some("file2.csv"));

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_scan_excludes_hidden() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_test_hidden");
        fs::create_dir_all(&temp_dir).unwrap();

        fs::write(temp_dir.join("visible.txt"), "test").unwrap();
        fs::write(temp_dir.join(".hidden"), "hidden").unwrap();

        let config = Config::new(temp_dir.clone(), Command::Scan);
        let result = scan_directory(&temp_dir, &config).unwrap();

        // Should only find visible file
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_name(), Some("visible.txt"));

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_scan_nested_directories() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_test_nested");
        fs::create_dir_all(temp_dir.join("subdir")).unwrap();

        fs::write(temp_dir.join("root.txt"), "test").unwrap();
        fs::write(temp_dir.join("subdir/nested.txt"), "nested").unwrap();

        let config = Config::new(temp_dir.clone(), Command::Scan);
        let result = scan_directory(&temp_dir, &config).unwrap();

        assert_eq!(result.len(), 2);

        fs::remove_dir_all(temp_dir).ok();
    }
}
