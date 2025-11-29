//! CSV file analysis
//!
//! Implements RFC 4180 compliant CSV parsing with streaming full-file analysis.
//! All rows are processed for accurate type inference and statistics.

use super::inference::{is_float, is_integer};
use super::AnalysisError;
use crate::types::{ColumnInfo, ColumnType, CsvAnalysis};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Maximum number of lines to sample for delimiter detection
const DELIMITER_SAMPLE_SIZE: usize = 10;

/// Maximum unique values to track per column (memory limit)
const MAX_UNIQUE_SAMPLES: usize = 1000;

/// Maximum sample values to store for display
const MAX_DISPLAY_SAMPLES: usize = 5;

/// Streaming column statistics for memory-efficient analysis
#[derive(Debug)]
struct StreamingColumnStats {
    /// Total non-null values seen
    total_count: u64,
    /// Count of empty/null values
    null_count: u64,
    /// Type occurrence counts for inference
    type_counts: TypeCounts,
    /// Unique values (capped at MAX_UNIQUE_SAMPLES)
    unique_values: HashSet<String>,
    /// Sample values for display (capped at MAX_DISPLAY_SAMPLES)
    sample_values: Vec<String>,
    /// Whether unique tracking is saturated
    unique_saturated: bool,
    /// Numeric statistics
    numeric_min: Option<f64>,
    numeric_max: Option<f64>,
    numeric_sum: f64,
}

/// Counts of each detected type for a column
#[derive(Debug, Default)]
struct TypeCounts {
    integer: u64,
    float: u64,
    boolean: u64,
    timestamp: u64,
    date: u64,
    string: u64,
}

impl StreamingColumnStats {
    fn new() -> Self {
        Self {
            total_count: 0,
            null_count: 0,
            type_counts: TypeCounts::default(),
            unique_values: HashSet::new(),
            sample_values: Vec::new(),
            unique_saturated: false,
            numeric_min: None,
            numeric_max: None,
            numeric_sum: 0.0,
        }
    }

    /// Update statistics with a new value
    fn update(&mut self, value: &str) {
        if value.is_empty() {
            self.null_count += 1;
            return;
        }

        self.total_count += 1;

        // Track unique values (with cap)
        if !self.unique_saturated {
            if self.unique_values.len() < MAX_UNIQUE_SAMPLES {
                self.unique_values.insert(value.to_string());
            } else {
                self.unique_saturated = true;
            }
        }

        // Track sample values for display
        if self.sample_values.len() < MAX_DISPLAY_SAMPLES
            && !self.sample_values.contains(&value.to_string())
        {
            self.sample_values.push(value.to_string());
        }

        // Determine and count type
        let detected_type = detect_value_type(value);
        match detected_type {
            ColumnType::Integer => {
                self.type_counts.integer += 1;
                if let Ok(n) = value.parse::<i64>() {
                    let f = n as f64;
                    self.update_numeric_stats(f);
                }
            }
            ColumnType::Float => {
                self.type_counts.float += 1;
                if let Ok(f) = value.parse::<f64>() {
                    self.update_numeric_stats(f);
                }
            }
            ColumnType::Boolean => self.type_counts.boolean += 1,
            ColumnType::Timestamp => self.type_counts.timestamp += 1,
            ColumnType::Date => self.type_counts.date += 1,
            _ => self.type_counts.string += 1,
        }
    }

    fn update_numeric_stats(&mut self, value: f64) {
        self.numeric_sum += value;
        self.numeric_min = Some(self.numeric_min.map_or(value, |min| min.min(value)));
        self.numeric_max = Some(self.numeric_max.map_or(value, |max| max.max(value)));
    }

    /// Determine the final inferred type based on all observed values
    fn infer_final_type(&self, column_name: Option<&str>) -> ColumnType {
        if self.total_count == 0 {
            return ColumnType::String;
        }

        let total = self.total_count as f64;
        let threshold = 0.8; // 80% of values must match

        // Check semantic hints from column name first
        if let Some(name) = column_name {
            let name_lower = name.to_lowercase();
            if (name_lower.contains("timestamp") || name_lower.contains("datetime"))
                && (self.type_counts.timestamp as f64 / total) > 0.5 {
                    return ColumnType::Timestamp;
                }
            if name_lower.contains("date") && !name_lower.contains("update")
                && (self.type_counts.date as f64 / total) > 0.5 {
                    return ColumnType::Date;
                }
            if name_lower.contains("id") || name_lower.ends_with("_id") {
                return ColumnType::Identifier;
            }
        }

        // Check type distributions
        if (self.type_counts.integer as f64 / total) >= threshold {
            return ColumnType::Integer;
        }
        if ((self.type_counts.integer + self.type_counts.float) as f64 / total) >= threshold {
            return ColumnType::Float;
        }
        if (self.type_counts.boolean as f64 / total) >= threshold {
            return ColumnType::Boolean;
        }
        if (self.type_counts.timestamp as f64 / total) >= threshold {
            return ColumnType::Timestamp;
        }
        if (self.type_counts.date as f64 / total) >= threshold {
            return ColumnType::Date;
        }

        ColumnType::String
    }
}

/// Detect the type of a single value
fn detect_value_type(value: &str) -> ColumnType {
    let trimmed = value.trim();

    // Check boolean
    match trimmed.to_lowercase().as_str() {
        "true" | "false" | "yes" | "no" | "y" | "n" | "1" | "0" => {
            // Only "1"/"0" if it's the only content
            if trimmed == "1" || trimmed == "0" {
                // Could be integer or boolean - prefer integer
                return ColumnType::Integer;
            }
            return ColumnType::Boolean;
        }
        _ => {}
    }

    // Check integer
    if is_integer(trimmed) {
        return ColumnType::Integer;
    }

    // Check float
    if is_float(trimmed) {
        return ColumnType::Float;
    }

    // Check timestamp (ISO 8601 with time)
    if is_timestamp(trimmed) {
        return ColumnType::Timestamp;
    }

    // Check date only
    if is_date_only(trimmed) {
        return ColumnType::Date;
    }

    ColumnType::String
}

/// Check if value looks like an ISO 8601 timestamp
fn is_timestamp(s: &str) -> bool {
    // Match patterns like: 2024-01-15T10:30:00, 2024-01-15 10:30:00, etc.
    let len = s.len();
    if len < 19 {
        return false;
    }

    let bytes = s.as_bytes();

    // Check YYYY-MM-DD prefix
    if !is_date_prefix(bytes) {
        return false;
    }

    // Check separator (T or space)
    if len > 10 && (bytes[10] == b'T' || bytes[10] == b' ') {
        // Check HH:MM:SS
        if len >= 19 {
            return bytes[13] == b':' && bytes[16] == b':';
        }
    }

    false
}

/// Check if value is date-only (no time component)
fn is_date_only(s: &str) -> bool {
    let len = s.len();

    // ISO format: YYYY-MM-DD
    if len == 10 {
        let bytes = s.as_bytes();
        return is_date_prefix(bytes);
    }

    // US format: MM/DD/YYYY
    if len == 10 && s.contains('/') {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 3 {
            return parts[0].len() == 2 && parts[1].len() == 2 && parts[2].len() == 4;
        }
    }

    false
}

/// Check if first 10 bytes match YYYY-MM-DD pattern
fn is_date_prefix(bytes: &[u8]) -> bool {
    if bytes.len() < 10 {
        return false;
    }
    bytes[4] == b'-' && bytes[7] == b'-'
        && bytes[0..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit())
}

/// Analyze a CSV file with streaming full-file analysis
///
/// Processes ALL rows for accurate type inference and statistics.
/// Uses streaming to handle large files efficiently (memory-bounded).
///
/// # Arguments
///
/// * `path` - Path to the CSV file
///
/// # Returns
///
/// A CsvAnalysis struct with detected characteristics, or an error.
pub fn analyze_csv(path: &Path) -> Result<CsvAnalysis, AnalysisError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Read first few lines for delimiter detection
    let mut sample_lines = Vec::new();
    let mut line = String::new();
    for _ in 0..DELIMITER_SAMPLE_SIZE {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        sample_lines.push(line.trim_end().to_string());
    }

    if sample_lines.is_empty() {
        return Ok(CsvAnalysis::new(',', false));
    }

    // Detect delimiter
    let delimiter = detect_delimiter(&sample_lines);

    // Parse first line as potential header
    let first_line_fields = parse_line_rfc4180(&sample_lines[0], delimiter);
    let column_count = first_line_fields.len();

    if column_count == 0 {
        return Ok(CsvAnalysis::new(delimiter, false));
    }

    // Check if first line is a header
    let has_header = detect_header(&sample_lines, delimiter);

    // Initialize streaming stats for each column
    let mut column_stats: Vec<StreamingColumnStats> =
        (0..column_count).map(|_| StreamingColumnStats::new()).collect();

    // Store header names
    let header_names: Vec<Option<String>> = if has_header {
        first_line_fields.iter().map(|s| Some(s.clone())).collect()
    } else {
        (0..column_count).map(|_| None).collect()
    };

    // Stream through entire file for full analysis
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut row_count: usize = 0;
    let mut data_row_count: usize = 0;

    for line_result in reader.lines() {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }

        row_count += 1;

        // Skip header row for data analysis
        if has_header && row_count == 1 {
            continue;
        }

        let fields = parse_line_rfc4180(&line, delimiter);

        // Only process rows with correct column count
        if fields.len() == column_count {
            data_row_count += 1;

            // Update stats for each column
            for (col_idx, value) in fields.iter().enumerate() {
                column_stats[col_idx].update(value);
            }
        }
    }

    // Build column info from streaming stats
    let columns: Vec<ColumnInfo> = column_stats
        .into_iter()
        .enumerate()
        .map(|(idx, stats)| {
            let mut info = ColumnInfo::new(idx);

            // Set name if we have header
            if let Some(Some(name)) = header_names.get(idx) {
                info = info.with_name(name.clone());
            }

            // Infer type from full-file statistics
            let inferred_type = stats.infer_final_type(
                header_names.get(idx).and_then(|n| n.as_deref())
            );
            info = info.with_type(inferred_type);

            // Set null count
            info = info.with_null_count(stats.null_count as usize);

            // Add sample values
            for sample in stats.sample_values {
                info = info.add_sample(sample);
            }

            info
        })
        .collect();

    Ok(CsvAnalysis {
        delimiter,
        has_header,
        column_count,
        row_count: data_row_count,
        columns,
    })
}

/// Detect the delimiter used in a CSV file
fn detect_delimiter(lines: &[String]) -> char {
    let candidates = [',', '\t', ';', '|'];
    let mut best_delimiter = ',';
    let mut best_score = 0.0;

    for &delimiter in &candidates {
        let counts: Vec<usize> = lines
            .iter()
            .map(|line| parse_line_rfc4180(line, delimiter).len())
            .collect();

        if counts.is_empty() {
            continue;
        }

        // Calculate consistency score
        let avg = counts.iter().sum::<usize>() as f32 / counts.len() as f32;
        let variance = counts
            .iter()
            .map(|&c| (c as f32 - avg).powi(2))
            .sum::<f32>()
            / counts.len() as f32;

        // Prefer delimiters with low variance and reasonable field counts
        let score = if variance < 1.0 && avg >= 2.0 {
            avg / (variance + 1.0)
        } else {
            0.0
        };

        if score > best_score {
            best_score = score;
            best_delimiter = delimiter;
        }
    }

    best_delimiter
}

/// Detect if the first line is a header
fn detect_header(lines: &[String], delimiter: char) -> bool {
    if lines.len() < 2 {
        return false;
    }

    let first_fields = parse_line_rfc4180(&lines[0], delimiter);
    let second_fields = parse_line_rfc4180(&lines[1], delimiter);

    if first_fields.len() != second_fields.len() {
        return false;
    }

    // Check if first row looks like column names (non-numeric strings)
    let first_non_numeric = first_fields.iter().all(|f| !is_integer(f) && !is_float(f));

    // Check if second row has at least some numeric values
    let second_has_numbers = second_fields.iter().any(|f| is_integer(f) || is_float(f));

    first_non_numeric && second_has_numbers
}

/// Parse a line into fields following RFC 4180 CSV specification
///
/// Handles:
/// - Quoted fields containing delimiters
/// - Escaped quotes (doubled quotes "" within quoted fields)
/// - Mixed quoted and unquoted fields
///
/// Note: Does NOT handle multiline fields (fields with embedded newlines)
/// as we process line-by-line for streaming.
fn parse_line_rfc4180(line: &str, delimiter: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                // Check for escaped quote (doubled quote)
                if chars.peek() == Some(&'"') {
                    // Escaped quote - consume the second quote and add one quote to field
                    chars.next();
                    current_field.push('"');
                } else {
                    // End of quoted field
                    in_quotes = false;
                }
            } else {
                // Regular character inside quotes
                current_field.push(c);
            }
        } else if c == '"' {
            // Start of quoted field (should be at start of field)
            in_quotes = true;
        } else if c == delimiter {
            // End of field
            fields.push(current_field.trim().to_string());
            current_field = String::new();
        } else {
            // Regular character
            current_field.push(c);
        }
    }

    // Don't forget the last field
    fields.push(current_field.trim().to_string());

    fields
}

/// Simple non-RFC4180 parsing for backwards compatibility (kept for reference)
#[allow(dead_code)]
fn parse_line_simple(line: &str, delimiter: char) -> Vec<String> {
    line.split(delimiter)
        .map(|s| s.trim().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_detect_delimiter_comma() {
        let lines = vec![
            "a,b,c".to_string(),
            "1,2,3".to_string(),
            "4,5,6".to_string(),
        ];
        assert_eq!(detect_delimiter(&lines), ',');
    }

    #[test]
    fn test_detect_delimiter_tab() {
        let lines = vec![
            "a\tb\tc".to_string(),
            "1\t2\t3".to_string(),
            "4\t5\t6".to_string(),
        ];
        assert_eq!(detect_delimiter(&lines), '\t');
    }

    #[test]
    fn test_detect_header() {
        let lines = vec![
            "name,age,city".to_string(),
            "Alice,30,NYC".to_string(),
            "Bob,25,LA".to_string(),
        ];
        assert!(detect_header(&lines, ','));
    }

    #[test]
    fn test_detect_no_header() {
        let lines = vec![
            "1,2,3".to_string(),
            "4,5,6".to_string(),
            "7,8,9".to_string(),
        ];
        assert!(!detect_header(&lines, ','));
    }

    #[test]
    fn test_parse_line_simple() {
        let line = "a,b,c";
        let fields = parse_line_rfc4180(line, ',');
        assert_eq!(fields, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_parse_line_rfc4180_quoted_field() {
        // Field with embedded comma
        let line = r#"a,"hello, world",c"#;
        let fields = parse_line_rfc4180(line, ',');
        assert_eq!(fields, vec!["a", "hello, world", "c"]);
    }

    #[test]
    fn test_parse_line_rfc4180_escaped_quotes() {
        // Field with escaped quotes (doubled)
        let line = r#"a,"He said ""Hello""",c"#;
        let fields = parse_line_rfc4180(line, ',');
        assert_eq!(fields, vec!["a", r#"He said "Hello""#, "c"]);
    }

    #[test]
    fn test_parse_line_rfc4180_mixed() {
        // Mix of quoted and unquoted fields
        let line = r#"simple,"quoted,with,commas",another"#;
        let fields = parse_line_rfc4180(line, ',');
        assert_eq!(fields, vec!["simple", "quoted,with,commas", "another"]);
    }

    #[test]
    fn test_parse_line_rfc4180_empty_quoted() {
        // Empty quoted field
        let line = r#"a,"",c"#;
        let fields = parse_line_rfc4180(line, ',');
        assert_eq!(fields, vec!["a", "", "c"]);
    }

    #[test]
    fn test_parse_line_rfc4180_tab_delimiter() {
        // Tab delimiter with quoted field containing tabs
        let line = "a\t\"hello\tworld\"\tc";
        let fields = parse_line_rfc4180(line, '\t');
        assert_eq!(fields, vec!["a", "hello\tworld", "c"]);
    }

    #[test]
    fn test_analyze_csv_with_header() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_csv_header");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.csv");
        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "name,age,score").unwrap();
            writeln!(file, "Alice,30,95.5").unwrap();
            writeln!(file, "Bob,25,87.3").unwrap();
        }

        let result = analyze_csv(&file_path).unwrap();
        assert_eq!(result.delimiter, ',');
        assert!(result.has_header);
        assert_eq!(result.column_count, 3);
        assert_eq!(result.row_count, 2);
        assert_eq!(result.columns.len(), 3);

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_analyze_csv_without_header() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_csv_no_header");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.csv");
        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "1,2,3").unwrap();
            writeln!(file, "4,5,6").unwrap();
            writeln!(file, "7,8,9").unwrap();
        }

        let result = analyze_csv(&file_path).unwrap();
        assert_eq!(result.delimiter, ',');
        assert!(!result.has_header);
        assert_eq!(result.row_count, 3);

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_analyze_csv_empty_values() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_csv_empty");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.csv");
        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "a,b,c").unwrap();
            writeln!(file, "1,,3").unwrap();
            writeln!(file, ",5,").unwrap();
        }

        let result = analyze_csv(&file_path).unwrap();
        assert!(result.columns.iter().any(|c| c.null_count > 0));

        std::fs::remove_dir_all(temp_dir).ok();
    }
}
