//! Compliance score calculation

use crate::types::{ValidationResult, ValidationSeverity};

/// Compliance score breakdown
#[derive(Debug, Clone)]
pub struct ComplianceScore {
    /// Total score (0-100)
    pub total: u8,
    /// Findable score (0-25)
    pub findable: u8,
    /// Accessible score (0-25)
    pub accessible: u8,
    /// Interoperable score (0-25)
    pub interoperable: u8,
    /// Reusable score (0-25)
    pub reusable: u8,
    /// Number of critical issues
    pub critical_count: usize,
    /// Number of warnings
    pub warning_count: usize,
    /// Number of info messages
    pub info_count: usize,
}

/// Calculate compliance score from validation results
///
/// Scoring algorithm:
/// - Start with 100 points
/// - Critical issues: -20 each
/// - Warning issues: -5 each
/// - Info issues: -1 each
/// - Floor at 0
///
/// FAIR sub-scores are calculated based on specific check codes.
pub fn calculate_score(validation: &[ValidationResult]) -> ComplianceScore {
    // Count issues by severity
    let critical_count = validation
        .iter()
        .filter(|v| matches!(v.severity, ValidationSeverity::Critical))
        .count();

    let warning_count = validation
        .iter()
        .filter(|v| matches!(v.severity, ValidationSeverity::Warning))
        .count();

    let info_count = validation
        .iter()
        .filter(|v| matches!(v.severity, ValidationSeverity::Info))
        .count();

    // Calculate total score
    let deductions = (critical_count * 20) + (warning_count * 5) + info_count;
    let total = if deductions > 100 {
        0
    } else {
        (100 - deductions) as u8
    };

    // Calculate FAIR sub-scores
    let findable = calculate_fair_subscore(validation, "FAIR-F");
    let accessible = calculate_fair_subscore(validation, "FAIR-A");
    let interoperable = calculate_fair_subscore(validation, "FAIR-I");
    let reusable = calculate_fair_subscore(validation, "FAIR-R");

    ComplianceScore {
        total,
        findable,
        accessible,
        interoperable,
        reusable,
        critical_count,
        warning_count,
        info_count,
    }
}

/// Calculate FAIR sub-score for a specific dimension
fn calculate_fair_subscore(validation: &[ValidationResult], prefix: &str) -> u8 {
    let issues: Vec<&ValidationResult> = validation
        .iter()
        .filter(|v| v.code.starts_with(prefix))
        .collect();

    if issues.is_empty() {
        return 25;
    }

    // Count deductions for this dimension
    let critical = issues
        .iter()
        .filter(|v| matches!(v.severity, ValidationSeverity::Critical))
        .count();

    let warning = issues
        .iter()
        .filter(|v| matches!(v.severity, ValidationSeverity::Warning))
        .count();

    let info = issues
        .iter()
        .filter(|v| matches!(v.severity, ValidationSeverity::Info))
        .count();

    // Scale to 0-25 range
    let deductions = (critical * 10) + (warning * 3) + info;
    if deductions >= 25 {
        0
    } else {
        (25 - deductions) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_score_perfect() {
        let validation = vec![];
        let score = calculate_score(&validation);

        assert_eq!(score.total, 100);
        assert_eq!(score.findable, 25);
        assert_eq!(score.accessible, 25);
        assert_eq!(score.interoperable, 25);
        assert_eq!(score.reusable, 25);
        assert_eq!(score.critical_count, 0);
        assert_eq!(score.warning_count, 0);
        assert_eq!(score.info_count, 0);
    }

    #[test]
    fn test_calculate_score_with_critical() {
        let validation = vec![
            ValidationResult::critical("TEST-001", "Test critical", "Fix it"),
            ValidationResult::critical("TEST-002", "Test critical 2", "Fix it"),
        ];

        let score = calculate_score(&validation);

        assert_eq!(score.total, 60); // 100 - (2 * 20)
        assert_eq!(score.critical_count, 2);
    }

    #[test]
    fn test_calculate_score_with_warnings() {
        let validation = vec![
            ValidationResult::warning("TEST-001", "Test warning", "Fix it"),
            ValidationResult::warning("TEST-002", "Test warning 2", "Fix it"),
            ValidationResult::warning("TEST-003", "Test warning 3", "Fix it"),
        ];

        let score = calculate_score(&validation);

        assert_eq!(score.total, 85); // 100 - (3 * 5)
        assert_eq!(score.warning_count, 3);
    }

    #[test]
    fn test_calculate_score_floor_at_zero() {
        let mut validation = vec![];
        // Add enough critical issues to exceed 100 points
        for i in 0..10 {
            validation.push(ValidationResult::critical(
                format!("TEST-{:03}", i),
                "Critical",
                "Fix",
            ));
        }

        let score = calculate_score(&validation);
        assert_eq!(score.total, 0);
    }

    #[test]
    fn test_fair_subscore_findable() {
        let validation = vec![
            ValidationResult::warning("FAIR-F001", "Missing metadata", "Add metadata"),
        ];

        let score = calculate_score(&validation);
        assert_eq!(score.findable, 22); // 25 - 3
        assert_eq!(score.accessible, 25); // No FAIR-A issues
    }

    #[test]
    fn test_fair_subscore_multiple_dimensions() {
        let validation = vec![
            ValidationResult::critical("FAIR-F001", "Missing metadata", "Fix"),
            ValidationResult::warning("FAIR-A001", "Missing license", "Fix"),
            ValidationResult::info("FAIR-I001", "No schema", "Fix"),
            ValidationResult::warning("FAIR-R001", "No docs", "Fix"),
        ];

        let score = calculate_score(&validation);

        assert_eq!(score.findable, 15); // 25 - 10 (critical)
        assert_eq!(score.accessible, 22); // 25 - 3 (warning)
        assert_eq!(score.interoperable, 24); // 25 - 1 (info)
        assert_eq!(score.reusable, 22); // 25 - 3 (warning)
    }
}
