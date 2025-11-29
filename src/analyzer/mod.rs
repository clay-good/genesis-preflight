//! File content analysis module
//!
//! This module analyzes file contents to infer structure, data types,
//! and other characteristics needed for validation and documentation.

mod binary;
mod csv;
mod inference;
mod json;
mod text;

use crate::types::{AnalysisResult, FileInfo, FileType};
use std::fmt;
use std::io;

pub use binary::{detect_binary_type, is_binary};
pub use csv::analyze_csv;
pub use inference::infer_column_type;
pub use json::analyze_json;
pub use text::analyze_text;

/// Errors that can occur during file analysis
#[derive(Debug)]
pub enum AnalysisError {
    /// IO error occurred
    Io(io::Error),
    /// File format is invalid
    InvalidFormat(String),
    /// File is too large to analyze
    FileTooLarge,
    /// Analysis not supported for this file type
    UnsupportedType,
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisError::Io(e) => write!(f, "IO error: {}", e),
            AnalysisError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            AnalysisError::FileTooLarge => write!(f, "File too large to analyze"),
            AnalysisError::UnsupportedType => write!(f, "Unsupported file type"),
        }
    }
}

impl From<io::Error> for AnalysisError {
    fn from(error: io::Error) -> Self {
        AnalysisError::Io(error)
    }
}

impl std::error::Error for AnalysisError {}

/// Analyze a file based on its type
///
/// Dispatches to the appropriate analyzer based on the FileInfo's detected type.
/// Returns detailed analysis results or NotAnalyzed if the file type is not supported.
///
/// # Arguments
///
/// * `file_info` - Information about the file to analyze
///
/// # Returns
///
/// An AnalysisResult containing type-specific analysis data.
///
/// # Examples
///
/// ```no_run
/// use genesis_preflight::analyzer::analyze_file;
/// use genesis_preflight::types::{FileInfo, FileType};
/// use std::path::PathBuf;
///
/// let file_info = FileInfo::new(
///     PathBuf::from("data.csv"),
///     PathBuf::from("data.csv")
/// );
/// let result = analyze_file(&file_info);
/// ```
pub fn analyze_file(file_info: &FileInfo) -> AnalysisResult {
    match file_info.file_type {
        FileType::Csv | FileType::Tsv => {
            match csv::analyze_csv(&file_info.full_path) {
                Ok(analysis) => AnalysisResult::Csv(analysis),
                Err(_) => AnalysisResult::NotAnalyzed,
            }
        }
        FileType::Json => {
            match json::analyze_json(&file_info.full_path) {
                Ok(analysis) => AnalysisResult::Json(analysis),
                Err(_) => AnalysisResult::NotAnalyzed,
            }
        }
        FileType::Text | FileType::Markdown => {
            match text::analyze_text(&file_info.full_path) {
                Ok(analysis) => AnalysisResult::Text(analysis),
                Err(_) => AnalysisResult::NotAnalyzed,
            }
        }
        FileType::Binary => {
            let binary_type = binary::detect_binary_type(&file_info.full_path)
                .unwrap_or(crate::types::BinaryType::Unknown);
            AnalysisResult::Binary(crate::types::BinaryAnalysis::new(binary_type))
        }
        FileType::Unknown => AnalysisResult::NotAnalyzed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    #[test]
    fn test_analyze_csv() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_analyze_csv");
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.csv");
        {
            let mut file = fs::File::create(&file_path).unwrap();
            file.write_all(b"a,b,c\n1,2,3\n4,5,6").unwrap();
        }

        let file_info = FileInfo::new(file_path.clone(), PathBuf::from("test.csv"));
        let result = analyze_file(&file_info);

        match result {
            AnalysisResult::Csv(analysis) => {
                assert_eq!(analysis.delimiter, ',');
                assert!(analysis.has_header);
            }
            _ => panic!("Expected CSV analysis result"),
        }

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_analyze_json() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_analyze_json");
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.json");
        fs::write(&file_path, r#"{"key": "value"}"#).unwrap();

        let file_info = FileInfo::new(file_path.clone(), PathBuf::from("test.json"));
        let result = analyze_file(&file_info);

        match result {
            AnalysisResult::Json(analysis) => {
                assert!(analysis.is_valid);
            }
            _ => panic!("Expected JSON analysis result"),
        }

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_analyze_text() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_analyze_text");
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("README.md");
        fs::write(&file_path, "# Title\n\nSome text").unwrap();

        let file_info = FileInfo::new(file_path.clone(), PathBuf::from("README.md"));
        let result = analyze_file(&file_info);

        match result {
            AnalysisResult::Text(analysis) => {
                assert!(analysis.line_count > 0);
            }
            _ => panic!("Expected text analysis result"),
        }

        fs::remove_dir_all(temp_dir).ok();
    }
}
