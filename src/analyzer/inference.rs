//! Type inference for column data

use crate::types::{ColumnType, InferredType};

/// Infer the type of a column from sample values
///
/// Examines a collection of string values and determines the most likely
/// data type based on pattern matching and semantic analysis.
///
/// # Arguments
///
/// * `values` - Sample values from the column (non-empty values only)
/// * `column_name` - Optional column name for semantic inference
///
/// # Returns
///
/// An InferredType with the detected type and confidence score.
pub fn infer_column_type(values: &[&str], column_name: Option<&str>) -> InferredType {
    if values.is_empty() {
        return InferredType::default();
    }

    // Try semantic inference from column name first
    if let Some(name) = column_name {
        if let Some(semantic_type) = infer_from_name(name) {
            // Verify values match semantic type
            let matches = values.iter().filter(|v| matches_type(v, semantic_type)).count();
            let confidence = matches as f32 / values.len() as f32;

            if confidence > 0.7 {
                return InferredType::new(semantic_type, confidence);
            }
        }
    }

    // Count how many values match each type
    let mut type_counts = [(ColumnType::Boolean, count_matches(values, is_boolean)),
        (ColumnType::Integer, count_matches(values, is_integer)),
        (ColumnType::Float, count_matches(values, is_float)),
        (ColumnType::Timestamp, count_matches(values, is_timestamp)),
        (ColumnType::Date, count_matches(values, is_date)),
        (ColumnType::Time, count_matches(values, is_time))];

    // Sort by match count (descending)
    type_counts.sort_by(|a, b| b.1.cmp(&a.1));

    // Get the best match
    let (best_type, best_count) = type_counts[0];
    let confidence = best_count as f32 / values.len() as f32;

    // If confidence is too low, default to String
    if confidence < 0.8 {
        InferredType::new(ColumnType::String, 1.0)
    } else {
        InferredType::new(best_type, confidence)
    }
}

/// Count how many values match a predicate
fn count_matches<F>(values: &[&str], predicate: F) -> usize
where
    F: Fn(&str) -> bool,
{
    values.iter().filter(|v| predicate(v)).count()
}

/// Check if value matches a type
fn matches_type(value: &str, col_type: ColumnType) -> bool {
    match col_type {
        ColumnType::Boolean => is_boolean(value),
        ColumnType::Integer => is_integer(value),
        ColumnType::Float => is_float(value),
        ColumnType::Timestamp => is_timestamp(value),
        ColumnType::Date => is_date(value),
        ColumnType::Time => is_time(value),
        ColumnType::Identifier => is_identifier(value),
        _ => true,
    }
}

/// Infer type from column name
fn infer_from_name(name: &str) -> Option<ColumnType> {
    let lower = name.to_lowercase();

    if lower.contains("id") && !lower.contains("valid") && !lower.contains("solid") {
        return Some(ColumnType::Identifier);
    }

    if lower.contains("timestamp") || lower == "ts" {
        return Some(ColumnType::Timestamp);
    }

    if lower.contains("time") && !lower.contains("timestamp") {
        return Some(ColumnType::Time);
    }

    if lower.contains("date") {
        return Some(ColumnType::Date);
    }

    if lower.contains("bool") || lower.contains("flag") || lower.starts_with("is_") || lower.starts_with("has_") {
        return Some(ColumnType::Boolean);
    }

    None
}

/// Check if string represents a boolean value
pub fn is_boolean(s: &str) -> bool {
    matches!(
        s.to_lowercase().as_str(),
        "true" | "false" | "yes" | "no" | "y" | "n" | "1" | "0" | "t" | "f"
    )
}

/// Check if string represents an integer
pub fn is_integer(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let s = s.trim();

    // Handle negative numbers
    let s = if let Some(stripped) = s.strip_prefix('-') {
        stripped
    } else {
        s
    };

    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// Check if string represents a floating-point number
pub fn is_float(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let s = s.trim();

    // Try to parse as f64
    s.parse::<f64>().is_ok()
}

/// Check if string represents a timestamp
pub fn is_timestamp(s: &str) -> bool {
    let s = s.trim();

    // ISO 8601 with time: YYYY-MM-DDTHH:MM:SS or similar
    if s.len() >= 19 && (s.contains('T') || s.contains(' ')) && s.contains(':') {
        // Basic validation: check for date-like pattern followed by time-like pattern
        let parts: Vec<&str> = if s.contains('T') {
            s.split('T').collect()
        } else {
            s.split(' ').collect()
        };

        if parts.len() >= 2 {
            return is_date(parts[0]) && is_time(parts[1].split('+').next().unwrap_or("").split('Z').next().unwrap_or(""));
        }
    }

    // Unix timestamp (10 digits for seconds, 13 for milliseconds)
    if s.len() >= 10 && s.len() <= 13 && s.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(num) = s.parse::<i64>() {
            // Reasonable range: 2000-01-01 to 2100-01-01
            return (946684800..=4102444800000).contains(&num);
        }
    }

    false
}

/// Check if string represents a date
pub fn is_date(s: &str) -> bool {
    let s = s.trim();

    // YYYY-MM-DD
    if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() == 3 {
            return parts[0].len() == 4 && parts[0].chars().all(|c| c.is_ascii_digit())
                && parts[1].len() == 2 && parts[1].chars().all(|c| c.is_ascii_digit())
                && parts[2].len() == 2 && parts[2].chars().all(|c| c.is_ascii_digit());
        }
    }

    // MM/DD/YYYY or DD/MM/YYYY
    if s.len() == 10 && s.chars().nth(2) == Some('/') && s.chars().nth(5) == Some('/') {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 3 {
            return parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()));
        }
    }

    // YYYYMMDD
    if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    false
}

/// Check if string represents a time
pub fn is_time(s: &str) -> bool {
    let s = s.trim();

    // HH:MM or HH:MM:SS
    if s.contains(':') {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() >= 2 && parts.len() <= 3 {
            return parts.iter().all(|p| {
                p.split('.').next().unwrap_or("").chars().all(|c| c.is_ascii_digit())
            });
        }
    }

    false
}

/// Check if string looks like an identifier
pub fn is_identifier(s: &str) -> bool {
    // Identifiers are often alphanumeric with underscores or hyphens
    // This is a loose check
    !s.is_empty() && s.len() < 100 && s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_boolean() {
        assert!(is_boolean("true"));
        assert!(is_boolean("false"));
        assert!(is_boolean("yes"));
        assert!(is_boolean("no"));
        assert!(is_boolean("1"));
        assert!(is_boolean("0"));
        assert!(!is_boolean("maybe"));
        assert!(!is_boolean("123"));
    }

    #[test]
    fn test_is_integer() {
        assert!(is_integer("123"));
        assert!(is_integer("-456"));
        assert!(is_integer("0"));
        assert!(!is_integer("12.34"));
        assert!(!is_integer("abc"));
        assert!(!is_integer(""));
    }

    #[test]
    fn test_is_float() {
        assert!(is_float("123.456"));
        assert!(is_float("-0.5"));
        assert!(is_float("2.7e-3"));
        assert!(is_float("123"));
        assert!(!is_float("abc"));
        assert!(!is_float(""));
    }

    #[test]
    fn test_is_timestamp() {
        assert!(is_timestamp("2024-01-15T10:30:00"));
        assert!(is_timestamp("2024-01-15 10:30:00"));
        assert!(is_timestamp("2024-01-15T10:30:00Z"));
        assert!(is_timestamp("1609459200")); // Unix timestamp
        assert!(!is_timestamp("2024-01-15"));
        assert!(!is_timestamp("10:30:00"));
    }

    #[test]
    fn test_is_date() {
        assert!(is_date("2024-01-15"));
        assert!(is_date("01/15/2024"));
        assert!(is_date("20240115"));
        assert!(!is_date("2024-1-15"));
        assert!(!is_date("not-a-date"));
    }

    #[test]
    fn test_is_time() {
        assert!(is_time("10:30"));
        assert!(is_time("10:30:45"));
        assert!(is_time("23:59:59"));
        // Note: is_time() does basic format checking, not value validation
        // "25:00" has valid format (HH:MM) even if hour is out of range
        assert!(is_time("25:00")); // Valid format, even if semantically invalid
        assert!(!is_time("not-a-time"));
        assert!(!is_time("abc:def"));
    }

    #[test]
    fn test_infer_column_type_integers() {
        let values = vec!["1", "2", "3", "4", "5"];
        let result = infer_column_type(&values, None);
        assert_eq!(result.column_type, ColumnType::Integer);
        assert!(result.confidence > 0.8);
    }

    #[test]
    fn test_infer_column_type_floats() {
        let values = vec!["1.5", "2.7", "3.14", "4.0", "5.5"];
        let result = infer_column_type(&values, None);
        assert_eq!(result.column_type, ColumnType::Float);
    }

    #[test]
    fn test_infer_column_type_strings() {
        let values = vec!["apple", "banana", "cherry", "date", "elderberry"];
        let result = infer_column_type(&values, None);
        assert_eq!(result.column_type, ColumnType::String);
    }

    #[test]
    fn test_infer_from_name() {
        assert_eq!(infer_from_name("user_id"), Some(ColumnType::Identifier));
        assert_eq!(infer_from_name("timestamp"), Some(ColumnType::Timestamp));
        assert_eq!(infer_from_name("created_date"), Some(ColumnType::Date));
        assert_eq!(infer_from_name("is_active"), Some(ColumnType::Boolean));
        assert_eq!(infer_from_name("temperature"), None);
    }

    #[test]
    fn test_infer_with_semantic_hint() {
        let values = vec!["1", "2", "3"];
        let result = infer_column_type(&values, Some("user_id"));
        assert_eq!(result.column_type, ColumnType::Identifier);
    }
}
