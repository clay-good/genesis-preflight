//! Documentation generation module
//!
//! This module generates missing documentation files for datasets,
//! including README, metadata.json, schema files, and manifests.

mod datacard;
mod manifest;
mod metadata_json;
mod readme;
mod schema;

use crate::types::{AnalysisResult, Config, DatasetSummary, FileInfo, ValidationResult};
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

pub use datacard::generate_datacard;
pub use manifest::generate_manifest;
pub use metadata_json::generate_metadata;
pub use readme::generate_readme;
pub use schema::generate_schema;

/// Represents a generated file
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    /// Path where the file was written
    pub path: PathBuf,
    /// Content of the generated file
    pub content: String,
    /// Whether the file was created (true) or skipped because it exists (false)
    pub was_created: bool,
}

impl GeneratedFile {
    /// Create a GeneratedFile that was created
    pub fn created(path: PathBuf) -> Self {
        GeneratedFile {
            path,
            content: String::new(),
            was_created: true,
        }
    }

    /// Create a GeneratedFile that was skipped (already exists)
    pub fn skipped(path: PathBuf) -> Self {
        GeneratedFile {
            path,
            content: String::new(),
            was_created: false,
        }
    }
}

/// Errors that can occur during generation
#[derive(Debug)]
pub enum GenerationError {
    /// IO error occurred
    Io(io::Error),
    /// File already exists and cannot be overwritten
    FileExists(PathBuf),
    /// Invalid output directory
    InvalidOutputDir(String),
}

impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerationError::Io(e) => write!(f, "IO error: {}", e),
            GenerationError::FileExists(p) => write!(f, "File exists: {}", p.display()),
            GenerationError::InvalidOutputDir(s) => write!(f, "Invalid output directory: {}", s),
        }
    }
}

impl From<io::Error> for GenerationError {
    fn from(error: io::Error) -> Self {
        GenerationError::Io(error)
    }
}

impl std::error::Error for GenerationError {}

/// Generate documentation files for a dataset
///
/// Creates missing documentation files based on validation results and analysis.
/// Never overwrites existing files.
///
/// # Arguments
///
/// * `files` - List of files in the dataset
/// * `analyses` - Analysis results for each file
/// * `validation` - Validation results
/// * `config` - Configuration options
///
/// # Returns
///
/// A vector of GeneratedFile structs describing what was created.
pub fn generate_documentation(
    files: &[FileInfo],
    analyses: &[AnalysisResult],
    _validation: &[ValidationResult],
    config: &Config,
) -> Result<Vec<GeneratedFile>, GenerationError> {
    let mut generated = Vec::new();
    let output_dir = config.get_output_dir();

    // Create dataset summary
    let summary = create_dataset_summary(files);

    // Generate README if missing
    if !has_readme(files) {
        let content = readme::generate_readme(&summary);
        let path = output_dir.join("README.md");
        generated.push(write_file(&path, &content, config)?);
    }

    // Generate metadata.json if missing
    if !has_metadata_json(files) {
        let content = metadata_json::generate_metadata(&summary);
        let path = output_dir.join("metadata.json");
        generated.push(write_file(&path, &content, config)?);
    }

    // Generate MANIFEST.sha256 if files have hashes
    if files.iter().any(|f| f.sha256_hash.is_some()) {
        let content = manifest::generate_manifest(files);
        let path = output_dir.join("MANIFEST.sha256");
        generated.push(write_file(&path, &content, config)?);
    }

    // Generate DATACARD.md if missing
    if !has_datacard(files) {
        let content = datacard::generate_datacard(&summary);
        let path = output_dir.join("DATACARD.md");
        generated.push(write_file(&path, &content, config)?);
    }

    // Generate schema files for CSV files
    for (file, analysis) in files.iter().zip(analyses.iter()) {
        if let AnalysisResult::Csv(csv_analysis) = analysis {
            if let Some(filename) = file.file_name() {
                let schema_name = format!("{}.schema.json", filename.trim_end_matches(".csv"));
                let schema_path = output_dir.join(&schema_name);

                if !schema_path.exists() {
                    let content = schema::generate_schema(csv_analysis, filename);
                    generated.push(write_file(&schema_path, &content, config)?);
                }
            }
        }
    }

    Ok(generated)
}

/// Check if dataset has a README file
fn has_readme(files: &[FileInfo]) -> bool {
    files.iter().any(|f| {
        f.file_name()
            .map(|name| name.to_uppercase().starts_with("README"))
            .unwrap_or(false)
    })
}

/// Check if dataset has a metadata.json file
fn has_metadata_json(files: &[FileInfo]) -> bool {
    files.iter().any(|f| {
        f.file_name()
            .map(|name| name == "metadata.json")
            .unwrap_or(false)
    })
}

/// Check if dataset has a DATACARD file
fn has_datacard(files: &[FileInfo]) -> bool {
    files.iter().any(|f| {
        f.file_name()
            .map(|name| name.to_uppercase().starts_with("DATACARD"))
            .unwrap_or(false)
    })
}

/// Create a dataset summary from file list
fn create_dataset_summary(files: &[FileInfo]) -> DatasetSummary {
    use crate::types::FileType;
    use std::collections::HashMap;

    let mut summary = DatasetSummary::new();
    summary.total_files = files.len();
    summary.total_size = files.iter().map(|f| f.size_bytes).sum();

    // Count file types
    let mut type_counts: HashMap<FileType, usize> = HashMap::new();
    for file in files {
        *type_counts.entry(file.file_type).or_insert(0) += 1;
    }

    summary.file_type_counts = type_counts.into_iter().collect();
    summary.file_type_counts.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    // Set timestamp
    summary.scan_timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

    summary
}

/// Write a file, never overwriting existing files
fn write_file(
    path: &PathBuf,
    content: &str,
    config: &Config,
) -> Result<GeneratedFile, GenerationError> {
    if path.exists() {
        if config.verbose {
            println!("Skipping existing file: {}", path.display());
        }
        Ok(GeneratedFile {
            path: path.clone(),
            content: content.to_string(),
            was_created: false,
        })
    } else {
        fs::write(path, content)?;
        if config.verbose {
            println!("Created: {}", path.display());
        }
        Ok(GeneratedFile {
            path: path.clone(),
            content: content.to_string(),
            was_created: true,
        })
    }
}

// Stub for chrono-like functionality using only std
mod chrono {
    use std::time::SystemTime;

    pub struct Utc;

    impl Utc {
        pub fn now() -> DateTime {
            DateTime {
                time: SystemTime::now(),
            }
        }
    }

    pub struct DateTime {
        time: SystemTime,
    }

    impl DateTime {
        pub fn format(&self, _fmt: &str) -> FormattedDateTime {
            FormattedDateTime {
                time: self.time,
            }
        }
    }

    #[allow(dead_code)]
    pub struct FormattedDateTime {
        time: SystemTime,
    }

    impl std::fmt::Display for FormattedDateTime {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use std::time::UNIX_EPOCH;

            let duration = self.time
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            let secs = duration.as_secs();

            // Convert seconds since epoch to UTC datetime components
            // Algorithm based on Howard Hinnant's date algorithms
            let days_since_epoch = (secs / 86400) as i64;
            let time_of_day = secs % 86400;

            // Convert days since 1970-01-01 to year/month/day
            let z = days_since_epoch + 719468;
            let era = if z >= 0 { z } else { z - 146096 } / 146097;
            let doe = (z - era * 146097) as u32;
            let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
            let y = yoe as i64 + era * 400;
            let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
            let mp = (5 * doy + 2) / 153;
            let d = doy - (153 * mp + 2) / 5 + 1;
            let m = if mp < 10 { mp + 3 } else { mp - 9 };
            let y = if m <= 2 { y + 1 } else { y };

            let hour = time_of_day / 3600;
            let minute = (time_of_day % 3600) / 60;
            let second = time_of_day % 60;

            write!(
                f,
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
                y, m, d, hour, minute, second
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Command;
    use std::path::PathBuf;

    #[test]
    fn test_has_readme() {
        let files = vec![
            FileInfo::new(
                PathBuf::from("README.md"),
                PathBuf::from("README.md"),
            ),
        ];
        assert!(has_readme(&files));
    }

    #[test]
    fn test_has_no_readme() {
        let files = vec![
            FileInfo::new(
                PathBuf::from("data.csv"),
                PathBuf::from("data.csv"),
            ),
        ];
        assert!(!has_readme(&files));
    }

    #[test]
    fn test_create_dataset_summary() {
        let files = vec![
            FileInfo::new(
                PathBuf::from("data.csv"),
                PathBuf::from("data.csv"),
            )
            .with_size(1024),
            FileInfo::new(
                PathBuf::from("info.txt"),
                PathBuf::from("info.txt"),
            )
            .with_size(512),
        ];

        let summary = create_dataset_summary(&files);
        assert_eq!(summary.total_files, 2);
        assert_eq!(summary.total_size, 1536);
    }
}
