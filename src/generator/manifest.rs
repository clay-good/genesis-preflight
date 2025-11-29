//! MANIFEST.sha256 generation

use crate::types::FileInfo;

/// Generate a MANIFEST.sha256 file
///
/// Creates a manifest file listing SHA-256 hashes for all files in the dataset.
/// Format is compatible with `sha256sum -c` command.
pub fn generate_manifest(files: &[FileInfo]) -> String {
    let mut manifest = String::new();

    // Collect files with hashes
    let mut files_with_hashes: Vec<&FileInfo> = files
        .iter()
        .filter(|f| f.sha256_hash.is_some())
        .collect();

    // Sort by relative path for consistency
    files_with_hashes.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    for file in files_with_hashes {
        if let Some(hash) = &file.sha256_hash {
            let path_str = file.relative_path.to_string_lossy();
            manifest.push_str(&format!("{}  {}\n", hash, path_str));
        }
    }

    manifest
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generate_manifest() {
        let files = vec![
            FileInfo::new(
                PathBuf::from("file1.txt"),
                PathBuf::from("file1.txt"),
            )
            .with_hash("abc123".to_string()),
            FileInfo::new(
                PathBuf::from("file2.txt"),
                PathBuf::from("file2.txt"),
            )
            .with_hash("def456".to_string()),
        ];

        let manifest = generate_manifest(&files);

        assert!(manifest.contains("abc123  file1.txt"));
        assert!(manifest.contains("def456  file2.txt"));
    }

    #[test]
    fn test_manifest_sorted() {
        let files = vec![
            FileInfo::new(
                PathBuf::from("zebra.txt"),
                PathBuf::from("zebra.txt"),
            )
            .with_hash("zzz".to_string()),
            FileInfo::new(
                PathBuf::from("apple.txt"),
                PathBuf::from("apple.txt"),
            )
            .with_hash("aaa".to_string()),
        ];

        let manifest = generate_manifest(&files);

        // apple should come before zebra
        let apple_pos = manifest.find("aaa").unwrap();
        let zebra_pos = manifest.find("zzz").unwrap();
        assert!(apple_pos < zebra_pos);
    }

    #[test]
    fn test_manifest_skips_files_without_hash() {
        let files = vec![
            FileInfo::new(
                PathBuf::from("with_hash.txt"),
                PathBuf::from("with_hash.txt"),
            )
            .with_hash("abc123".to_string()),
            FileInfo::new(
                PathBuf::from("no_hash.txt"),
                PathBuf::from("no_hash.txt"),
            ),
        ];

        let manifest = generate_manifest(&files);

        assert!(manifest.contains("with_hash.txt"));
        assert!(!manifest.contains("no_hash.txt"));
    }

    #[test]
    fn test_manifest_format() {
        let files = vec![FileInfo::new(
            PathBuf::from("test.txt"),
            PathBuf::from("test.txt"),
        )
        .with_hash("1234567890abcdef".to_string())];

        let manifest = generate_manifest(&files);

        // Format should be: hash<two spaces>filename
        assert!(manifest.contains("1234567890abcdef  test.txt"));
    }
}
