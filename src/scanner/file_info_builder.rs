//! File metadata extraction and FileInfo construction

use crate::crypto::sha256_file;
use crate::types::{Config, FileInfo};
#[cfg(test)]
use crate::types::FileType;
use std::fs;
use std::io;
use std::path::Path;

/// Build a FileInfo struct for a given file path
///
/// Extracts file metadata including size, modification time, file type,
/// and optionally SHA-256 hash.
///
/// # Arguments
///
/// * `path` - Absolute path to the file
/// * `root` - Root directory of the scan (for computing relative path)
/// * `config` - Configuration controlling hash calculation
///
/// # Returns
///
/// A FileInfo struct populated with file metadata, or an IO error.
pub fn build_file_info(path: &Path, root: &Path, config: &Config) -> io::Result<FileInfo> {
    // Get metadata
    let metadata = fs::metadata(path)?;

    // Calculate relative path
    let relative_path = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_path_buf();

    // Create base FileInfo
    let mut info = FileInfo::new(path.to_path_buf(), relative_path);

    // Set size
    info = info.with_size(metadata.len());

    // Set modification time
    if let Ok(modified) = metadata.modified() {
        info = info.with_modified(modified);
    }

    // Calculate SHA-256 hash if enabled
    if !config.skip_hash {
        match sha256_file(path) {
            Ok(hash) => {
                info = info.with_hash(hash);
            }
            Err(e) => {
                if config.verbose {
                    eprintln!(
                        "Warning: Cannot hash file {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        }
    }

    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Command;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_build_file_info() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_build_info");
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.txt");
        {
            let mut file = fs::File::create(&file_path).unwrap();
            file.write_all(b"test content").unwrap();
        }

        let config = Config::new(temp_dir.clone(), Command::Scan);
        let info = build_file_info(&file_path, &temp_dir, &config).unwrap();

        assert_eq!(info.file_name(), Some("test.txt"));
        assert_eq!(info.size_bytes, 12);
        assert_eq!(info.file_type, FileType::Text);
        assert!(info.modified.is_some());

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_build_file_info_with_hash() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_build_info_hash");
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.csv");
        {
            let mut file = fs::File::create(&file_path).unwrap();
            file.write_all(b"a,b,c\n1,2,3").unwrap();
        }

        let config = Config::new(temp_dir.clone(), Command::Scan);
        let info = build_file_info(&file_path, &temp_dir, &config).unwrap();

        assert_eq!(info.file_type, FileType::Csv);
        assert!(info.sha256_hash.is_some());
        assert_eq!(info.sha256_hash.as_ref().unwrap().len(), 64);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_build_file_info_skip_hash() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_build_info_no_hash");
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.txt");
        fs::write(&file_path, "test").unwrap();

        let config = Config::new(temp_dir.clone(), Command::Scan)
            .with_skip_hash(true);
        let info = build_file_info(&file_path, &temp_dir, &config).unwrap();

        assert!(info.sha256_hash.is_none());

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_relative_path_calculation() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_relative_path");
        fs::create_dir_all(temp_dir.join("subdir")).unwrap();

        let file_path = temp_dir.join("subdir/nested.txt");
        fs::write(&file_path, "test").unwrap();

        let config = Config::new(temp_dir.clone(), Command::Scan);
        let info = build_file_info(&file_path, &temp_dir, &config).unwrap();

        assert_eq!(info.relative_path.to_str().unwrap(), "subdir/nested.txt");

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_file_type_detection() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_file_types");
        fs::create_dir_all(&temp_dir).unwrap();

        let test_files = vec![
            ("data.csv", FileType::Csv),
            ("data.json", FileType::Json),
            ("README.md", FileType::Markdown),
            ("notes.txt", FileType::Text),
        ];

        let config = Config::new(temp_dir.clone(), Command::Scan)
            .with_skip_hash(true);

        for (filename, expected_type) in test_files {
            let file_path = temp_dir.join(filename);
            fs::write(&file_path, "test").unwrap();

            let info = build_file_info(&file_path, &temp_dir, &config).unwrap();
            assert_eq!(
                info.file_type, expected_type,
                "Failed for file: {}",
                filename
            );
        }

        fs::remove_dir_all(temp_dir).ok();
    }
}
