use std::cmp::Ordering;
use std::fmt;
use std::path::PathBuf;

/// Severity level of a validation issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// Informational message (lowest severity)
    Info,
    /// Warning that should be addressed
    Warning,
    /// Critical issue that must be fixed
    Critical,
}

impl ValidationSeverity {
    /// Get the point deduction for this severity level
    pub fn point_deduction(&self) -> u8 {
        match self {
            ValidationSeverity::Critical => 20,
            ValidationSeverity::Warning => 5,
            ValidationSeverity::Info => 1,
        }
    }
}

impl fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationSeverity::Critical => write!(f, "CRITICAL"),
            ValidationSeverity::Warning => write!(f, "WARNING"),
            ValidationSeverity::Info => write!(f, "INFO"),
        }
    }
}

/// Result of a validation check
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Severity level of this issue
    pub severity: ValidationSeverity,
    /// Unique error code (e.g., "FAIR-F001")
    pub code: String,
    /// Human-readable message describing the issue
    pub message: String,
    /// Optional file path where issue was found
    pub file_path: Option<PathBuf>,
    /// Optional line number within the file
    pub line_number: Option<usize>,
    /// Suggested fix or action to resolve the issue
    pub suggestion: String,
}

impl ValidationResult {
    /// Create a new validation result (builder pattern - use with_suggestion to add suggestion)
    pub fn new(
        severity: ValidationSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        ValidationResult {
            severity,
            code: code.into(),
            message: message.into(),
            file_path: None,
            line_number: None,
            suggestion: String::new(),
        }
    }

    /// Create a new validation result with all fields
    pub fn new_with_suggestion(
        severity: ValidationSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        ValidationResult {
            severity,
            code: code.into(),
            message: message.into(),
            file_path: None,
            line_number: None,
            suggestion: suggestion.into(),
        }
    }

    /// Create a critical validation result
    pub fn critical(
        code: impl Into<String>,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self::new_with_suggestion(ValidationSeverity::Critical, code, message, suggestion)
    }

    /// Create a warning validation result
    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self::new_with_suggestion(ValidationSeverity::Warning, code, message, suggestion)
    }

    /// Create an info validation result
    pub fn info(
        code: impl Into<String>,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self::new_with_suggestion(ValidationSeverity::Info, code, message, suggestion)
    }

    /// Set the suggestion for this validation result
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = suggestion.into();
        self
    }

    /// Set the file path for this validation result
    pub fn with_file(mut self, file_path: PathBuf) -> Self {
        self.file_path = Some(file_path);
        self
    }

    /// Set the line number for this validation result
    pub fn with_line(mut self, line_number: usize) -> Self {
        self.line_number = Some(line_number);
        self
    }
}

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref path) = self.file_path {
            write!(f, " ({})", path.display())?;
            if let Some(line) = self.line_number {
                write!(f, ":{}", line)?;
            }
        }
        write!(f, "\n  -> {}", self.suggestion)
    }
}

impl PartialEq for ValidationResult {
    fn eq(&self, other: &Self) -> bool {
        self.severity == other.severity && self.code == other.code
    }
}

impl Eq for ValidationResult {}

impl PartialOrd for ValidationResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValidationResult {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by severity (Critical first), then by file path, then by code
        match other.severity.cmp(&self.severity) {
            Ordering::Equal => match (&self.file_path, &other.file_path) {
                (Some(a), Some(b)) => match a.cmp(b) {
                    Ordering::Equal => self.code.cmp(&other.code),
                    other => other,
                },
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => self.code.cmp(&other.code),
            },
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(ValidationSeverity::Critical > ValidationSeverity::Warning);
        assert!(ValidationSeverity::Warning > ValidationSeverity::Info);
    }

    #[test]
    fn test_point_deduction() {
        assert_eq!(ValidationSeverity::Critical.point_deduction(), 20);
        assert_eq!(ValidationSeverity::Warning.point_deduction(), 5);
        assert_eq!(ValidationSeverity::Info.point_deduction(), 1);
    }

    #[test]
    fn test_validation_result_creation() {
        let result = ValidationResult::critical(
            "TEST-001",
            "Test message",
            "Fix the issue",
        );
        assert_eq!(result.severity, ValidationSeverity::Critical);
        assert_eq!(result.code, "TEST-001");
    }

    #[test]
    fn test_validation_result_sorting() {
        let critical = ValidationResult::critical("C1", "Critical", "Fix");
        let warning = ValidationResult::warning("W1", "Warning", "Fix");
        let info = ValidationResult::info("I1", "Info", "Fix");

        let mut results = vec![info.clone(), warning.clone(), critical.clone()];
        results.sort();

        assert_eq!(results[0].severity, ValidationSeverity::Critical);
        assert_eq!(results[1].severity, ValidationSeverity::Warning);
        assert_eq!(results[2].severity, ValidationSeverity::Info);
    }
}
