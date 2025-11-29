//! Directory traversal implementation

use crate::types::{Config, FileInfo};
use super::file_info_builder;
use super::ScanError;
use std::fs;
use std::path::Path;

/// Maximum directory depth to prevent infinite recursion
const MAX_DEPTH: usize = 20;

/// Directories to skip during scanning
const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "__pycache__",
    "target",
    ".svn",
    ".hg",
];

/// Recursively walk a directory tree and collect file information
pub fn walk_directory(root: &Path, config: &Config) -> Result<Vec<FileInfo>, ScanError> {
    let mut files = Vec::new();
    let mut file_count = 0;

    walk_recursive(root, root, &mut files, &mut file_count, 0, config)?;

    Ok(files)
}

/// Recursive helper function for directory walking
fn walk_recursive(
    root: &Path,
    current: &Path,
    files: &mut Vec<FileInfo>,
    file_count: &mut usize,
    depth: usize,
    config: &Config,
) -> Result<(), ScanError> {
    // Check depth limit
    if depth > MAX_DEPTH {
        if config.verbose {
            eprintln!(
                "Warning: Maximum depth exceeded at {}",
                current.display()
            );
        }
        return Ok(());
    }

    // Read directory entries
    let entries = match fs::read_dir(current) {
        Ok(entries) => entries,
        Err(e) => {
            if config.verbose {
                eprintln!(
                    "Warning: Cannot read directory {}: {}",
                    current.display(),
                    e
                );
            }
            return Ok(()); // Continue scanning other directories
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                if config.verbose {
                    eprintln!("Warning: Cannot read directory entry: {}", e);
                }
                continue;
            }
        };

        let path = entry.path();

        // Get file name
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        // Skip hidden files
        if file_name.starts_with('.') {
            continue;
        }

        // Check if it's a directory
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                if config.verbose {
                    eprintln!(
                        "Warning: Cannot read metadata for {}: {}",
                        path.display(),
                        e
                    );
                }
                continue;
            }
        };

        if metadata.is_dir() {
            // Skip common build/version control directories
            if SKIP_DIRS.contains(&file_name) {
                continue;
            }

            // Recurse into subdirectory
            walk_recursive(root, &path, files, file_count, depth + 1, config)?;
        } else if metadata.is_file() {
            // Don't follow symlinks - only process regular files
            if metadata.is_symlink() {
                if config.verbose {
                    eprintln!("Warning: Skipping symlink: {}", path.display());
                }
                continue;
            }

            // Build FileInfo
            match file_info_builder::build_file_info(&path, root, config) {
                Ok(info) => {
                    files.push(info);
                    *file_count += 1;

                    // Progress indication
                    if config.verbose && (*file_count).is_multiple_of(1000) {
                        println!("Scanned {} files...", file_count);
                    }
                }
                Err(e) => {
                    if config.verbose {
                        eprintln!(
                            "Warning: Cannot process file {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Command;
    use std::fs;

    #[test]
    fn test_walk_empty_directory() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_walk_empty");
        fs::create_dir_all(&temp_dir).unwrap();

        let config = Config::new(temp_dir.clone(), Command::Scan);
        let result = walk_directory(&temp_dir, &config).unwrap();
        assert_eq!(result.len(), 0);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_walk_skips_git() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_walk_git");
        fs::create_dir_all(temp_dir.join(".git")).unwrap();

        fs::write(temp_dir.join("visible.txt"), "test").unwrap();
        fs::write(temp_dir.join(".git/config"), "git config").unwrap();

        let config = Config::new(temp_dir.clone(), Command::Scan);
        let result = walk_directory(&temp_dir, &config).unwrap();

        // Should only find visible.txt, not .git/config
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_name(), Some("visible.txt"));

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_walk_nested() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_walk_nested");
        fs::create_dir_all(temp_dir.join("level1/level2")).unwrap();

        fs::write(temp_dir.join("root.txt"), "root").unwrap();
        fs::write(temp_dir.join("level1/file1.txt"), "level1").unwrap();
        fs::write(temp_dir.join("level1/level2/file2.txt"), "level2").unwrap();

        let config = Config::new(temp_dir.clone(), Command::Scan);
        let result = walk_directory(&temp_dir, &config).unwrap();

        assert_eq!(result.len(), 3);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_skip_directories() {
        for skip_dir in SKIP_DIRS {
            let temp_dir = std::env::temp_dir().join(format!("genesis_preflight_skip_{}", skip_dir));
            fs::create_dir_all(temp_dir.join(skip_dir)).unwrap();

            fs::write(temp_dir.join("keep.txt"), "keep").unwrap();
            fs::write(temp_dir.join(skip_dir).join("skip.txt"), "skip").unwrap();

            let config = Config::new(temp_dir.clone(), Command::Scan);
            let result = walk_directory(&temp_dir, &config).unwrap();

            assert_eq!(result.len(), 1, "Failed for skip_dir: {}", skip_dir);
            assert_eq!(result[0].file_name(), Some("keep.txt"));

            fs::remove_dir_all(temp_dir).ok();
        }
    }
}
