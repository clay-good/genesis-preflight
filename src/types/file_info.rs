use std::path::PathBuf;
use std::time::SystemTime;

use super::FileType;

/// Information about a file in the dataset
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Absolute path to the file
    pub full_path: PathBuf,
    /// Path relative to dataset root
    pub relative_path: PathBuf,
    /// File size in bytes
    pub size_bytes: u64,
    /// Last modification time
    pub modified: Option<SystemTime>,
    /// Detected file type
    pub file_type: FileType,
    /// SHA-256 hash of file contents (if calculated)
    pub sha256_hash: Option<String>,
    /// Whether this is a hidden file
    pub is_hidden: bool,
}

impl FileInfo {
    /// Create a new FileInfo
    pub fn new(full_path: PathBuf, relative_path: PathBuf) -> Self {
        let file_type = FileType::from_path(&full_path);
        let is_hidden = relative_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false);

        FileInfo {
            full_path,
            relative_path,
            size_bytes: 0,
            modified: None,
            file_type,
            sha256_hash: None,
            is_hidden,
        }
    }

    /// Get the path (alias for full_path for compatibility)
    pub fn path(&self) -> &PathBuf {
        &self.full_path
    }

    /// Set the file size
    pub fn with_size(mut self, size_bytes: u64) -> Self {
        self.size_bytes = size_bytes;
        self
    }

    /// Set the modification time
    pub fn with_modified(mut self, modified: SystemTime) -> Self {
        self.modified = Some(modified);
        self
    }

    /// Set the SHA-256 hash
    pub fn with_hash(mut self, hash: String) -> Self {
        self.sha256_hash = Some(hash);
        self
    }

    /// Set the file type
    pub fn with_type(mut self, file_type: FileType) -> Self {
        self.file_type = file_type;
        self
    }

    /// Get the file name
    pub fn file_name(&self) -> Option<&str> {
        self.relative_path.file_name()?.to_str()
    }

    /// Get the file extension
    pub fn extension(&self) -> Option<&str> {
        self.relative_path.extension()?.to_str()
    }

    /// Check if this file is likely a documentation file
    pub fn is_documentation(&self) -> bool {
        matches!(self.file_type, FileType::Markdown | FileType::Text)
            || self
                .file_name()
                .map(|n| {
                    n.to_uppercase().starts_with("README")
                        || n.to_uppercase().starts_with("LICENSE")
                        || n.to_uppercase().starts_with("CONTRIBUTING")
                })
                .unwrap_or(false)
    }

    /// Check if this file is likely a data file
    pub fn is_data(&self) -> bool {
        matches!(
            self.file_type,
            FileType::Csv | FileType::Tsv | FileType::Json | FileType::Binary
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_info_creation() {
        let info = FileInfo::new(
            PathBuf::from("/data/test.csv"),
            PathBuf::from("test.csv"),
        );
        assert_eq!(info.file_type, FileType::Csv);
        assert!(!info.is_hidden);
    }

    #[test]
    fn test_hidden_file_detection() {
        let info = FileInfo::new(
            PathBuf::from("/data/.hidden"),
            PathBuf::from(".hidden"),
        );
        assert!(info.is_hidden);
    }

    #[test]
    fn test_documentation_detection() {
        let readme = FileInfo::new(
            PathBuf::from("/data/README.md"),
            PathBuf::from("README.md"),
        );
        assert!(readme.is_documentation());

        let csv = FileInfo::new(
            PathBuf::from("/data/data.csv"),
            PathBuf::from("data.csv"),
        );
        assert!(!csv.is_documentation());
    }

    #[test]
    fn test_data_detection() {
        let csv = FileInfo::new(
            PathBuf::from("/data/data.csv"),
            PathBuf::from("data.csv"),
        );
        assert!(csv.is_data());

        let readme = FileInfo::new(
            PathBuf::from("/data/README.md"),
            PathBuf::from("README.md"),
        );
        assert!(!readme.is_data());
    }

    #[test]
    fn test_builder_pattern() {
        let info = FileInfo::new(
            PathBuf::from("/data/test.csv"),
            PathBuf::from("test.csv"),
        )
        .with_size(1024)
        .with_hash("abc123".to_string());

        assert_eq!(info.size_bytes, 1024);
        assert_eq!(info.sha256_hash, Some("abc123".to_string()));
    }
}
