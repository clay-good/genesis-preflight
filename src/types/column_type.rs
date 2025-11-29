use std::fmt;

/// Enumeration of data types that can be inferred from column values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    /// Integer values (e.g., 42, -17, 0)
    Integer,
    /// Floating-point values (e.g., 3.14, -0.5, 2.7e-3)
    Float,
    /// String values (arbitrary text)
    String,
    /// Boolean values (true/false, yes/no, 0/1)
    Boolean,
    /// Timestamp with date and time (e.g., 2024-01-15T10:30:00Z)
    Timestamp,
    /// Date only (e.g., 2024-01-15)
    Date,
    /// Time only (e.g., 10:30:00)
    Time,
    /// Identifier or key (e.g., ID column)
    Identifier,
    /// Unknown or unable to infer type
    Unknown,
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColumnType::Integer => write!(f, "integer"),
            ColumnType::Float => write!(f, "float"),
            ColumnType::String => write!(f, "string"),
            ColumnType::Boolean => write!(f, "boolean"),
            ColumnType::Timestamp => write!(f, "timestamp"),
            ColumnType::Date => write!(f, "date"),
            ColumnType::Time => write!(f, "time"),
            ColumnType::Identifier => write!(f, "identifier"),
            ColumnType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Inferred type with confidence level
#[derive(Debug, Clone)]
pub struct InferredType {
    /// The inferred column type
    pub column_type: ColumnType,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
}

impl InferredType {
    /// Create a new inferred type with specified confidence
    pub fn new(column_type: ColumnType, confidence: f32) -> Self {
        InferredType {
            column_type,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Create a high-confidence inferred type
    pub fn certain(column_type: ColumnType) -> Self {
        InferredType {
            column_type,
            confidence: 1.0,
        }
    }

    /// Create a low-confidence inferred type
    pub fn uncertain(column_type: ColumnType) -> Self {
        InferredType {
            column_type,
            confidence: 0.5,
        }
    }
}

impl Default for InferredType {
    fn default() -> Self {
        InferredType {
            column_type: ColumnType::Unknown,
            confidence: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", ColumnType::Integer), "integer");
        assert_eq!(format!("{}", ColumnType::Float), "float");
        assert_eq!(format!("{}", ColumnType::Timestamp), "timestamp");
    }

    #[test]
    fn test_inferred_type_confidence_clamping() {
        let inferred = InferredType::new(ColumnType::Integer, 1.5);
        assert_eq!(inferred.confidence, 1.0);

        let inferred = InferredType::new(ColumnType::Integer, -0.5);
        assert_eq!(inferred.confidence, 0.0);
    }

    #[test]
    fn test_inferred_type_certainty() {
        let certain = InferredType::certain(ColumnType::Float);
        assert_eq!(certain.confidence, 1.0);

        let uncertain = InferredType::uncertain(ColumnType::String);
        assert_eq!(uncertain.confidence, 0.5);
    }
}
