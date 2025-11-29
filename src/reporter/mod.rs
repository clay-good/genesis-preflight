//! Report generation module
//!
//! This module produces compliance reports in various formats.

mod json_report;
mod score;
mod terminal;

use crate::generator::GeneratedFile;
use crate::types::{Config, FileInfo, ValidationResult};

pub use json_report::generate_json_report;
pub use score::{calculate_score, ComplianceScore};
pub use terminal::print_terminal_report;

/// Complete report data structure
#[derive(Debug, Clone)]
pub struct Report {
    /// Dataset path
    pub dataset_path: String,
    /// Scan timestamp
    pub scan_timestamp: String,
    /// Files scanned
    pub files: Vec<FileInfo>,
    /// Validation results
    pub validation_results: Vec<ValidationResult>,
    /// Generated files
    pub generated_files: Vec<GeneratedFile>,
    /// Compliance score
    pub score: ComplianceScore,
}

impl Report {
    /// Get exit code based on score and issues
    pub fn exit_code(&self) -> i32 {
        if self.score.critical_count > 0 || self.score.total < 50 {
            2
        } else if self.score.total < 80 || self.score.warning_count > 0 {
            1
        } else {
            0
        }
    }
}

/// Generate a complete report
///
/// Creates a report structure containing all scan and validation results.
///
/// # Arguments
///
/// * `files` - List of files scanned
/// * `validation` - Validation results
/// * `generated` - Files generated during the run
/// * `config` - Configuration options
///
/// # Returns
///
/// A Report struct containing all report data.
pub fn generate_report(
    files: &[FileInfo],
    validation: &[ValidationResult],
    generated: &[GeneratedFile],
    config: &Config,
) -> Report {
    let score = calculate_score(validation);

    Report {
        dataset_path: config.target_path.to_string_lossy().to_string(),
        scan_timestamp: get_current_timestamp(),
        files: files.to_vec(),
        validation_results: validation.to_vec(),
        generated_files: generated.to_vec(),
        score,
    }
}

/// Get current timestamp as string
fn get_current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
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

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        y, m, d, hour, minute, second
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Command;
    use std::path::PathBuf;

    #[test]
    fn test_generate_report() {
        let config = Config::new(PathBuf::from("/test"), Command::Scan);
        let files = vec![];
        let validation = vec![];
        let generated = vec![];

        let report = generate_report(&files, &validation, &generated, &config);

        assert_eq!(report.files.len(), 0);
        assert_eq!(report.validation_results.len(), 0);
        assert_eq!(report.score.total, 100);
    }

    #[test]
    fn test_exit_code_perfect() {
        let mut report = Report {
            dataset_path: "/test".to_string(),
            scan_timestamp: "2024-01-15".to_string(),
            files: vec![],
            validation_results: vec![],
            generated_files: vec![],
            score: ComplianceScore {
                total: 100,
                findable: 25,
                accessible: 25,
                interoperable: 25,
                reusable: 25,
                critical_count: 0,
                warning_count: 0,
                info_count: 0,
            },
        };

        assert_eq!(report.exit_code(), 0);
    }

    #[test]
    fn test_exit_code_with_warnings() {
        let mut report = Report {
            dataset_path: "/test".to_string(),
            scan_timestamp: "2024-01-15".to_string(),
            files: vec![],
            validation_results: vec![],
            generated_files: vec![],
            score: ComplianceScore {
                total: 85,
                findable: 25,
                accessible: 20,
                interoperable: 20,
                reusable: 20,
                critical_count: 0,
                warning_count: 3,
                info_count: 2,
            },
        };

        assert_eq!(report.exit_code(), 1);
    }

    #[test]
    fn test_exit_code_with_critical() {
        let mut report = Report {
            dataset_path: "/test".to_string(),
            scan_timestamp: "2024-01-15".to_string(),
            files: vec![],
            validation_results: vec![],
            generated_files: vec![],
            score: ComplianceScore {
                total: 60,
                findable: 15,
                accessible: 15,
                interoperable: 15,
                reusable: 15,
                critical_count: 2,
                warning_count: 3,
                info_count: 1,
            },
        };

        assert_eq!(report.exit_code(), 2);
    }
}
