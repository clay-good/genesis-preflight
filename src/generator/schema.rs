//! JSON Schema generation for CSV files

use crate::types::{ColumnType, CsvAnalysis};

/// Generate a JSON Schema for a CSV file
///
/// Creates a JSON Schema (draft-07) describing the structure and types
/// of columns in the CSV file.
pub fn generate_schema(csv_analysis: &CsvAnalysis, filename: &str) -> String {
    let mut schema = String::new();

    schema.push_str("{\n");
    schema.push_str("  \"$schema\": \"http://json-schema.org/draft-07/schema#\",\n");
    schema.push_str(&format!("  \"title\": \"Schema for {}\",\n", filename));
    schema.push_str("  \"description\": \"Auto-generated schema from CSV analysis\",\n");
    schema.push_str("  \"type\": \"array\",\n");
    schema.push_str("  \"items\": {\n");
    schema.push_str("    \"type\": \"object\",\n");
    schema.push_str("    \"properties\": {\n");

    let col_count = csv_analysis.columns.len();
    for (idx, column) in csv_analysis.columns.iter().enumerate() {
        let default_name = format!("column_{}", idx);
        let col_name = column.name.as_deref().unwrap_or(&default_name);
        let json_type = column_type_to_json_type(column.inferred_type);

        let comma = if idx < col_count - 1 { "," } else { "" };

        schema.push_str(&format!("      \"{}\": {{\n", col_name));
        schema.push_str(&format!("        \"type\": \"{}\",\n", json_type));
        schema.push_str(&format!("        \"description\": \"Column {} (inferred type: {})\"\n", idx, column.inferred_type));

        // Add examples if we have sample values
        if !column.sample_values.is_empty() {
            schema.push_str(",\n");
            schema.push_str("        \"examples\": [\n");
            let sample_count = column.sample_values.len();
            for (i, sample) in column.sample_values.iter().enumerate() {
                let sample_comma = if i < sample_count - 1 { "," } else { "" };
                schema.push_str(&format!("          \"{}\"{}\n", escape_json(sample), sample_comma));
            }
            schema.push_str("        ]\n");
        } else {
            schema.push('\n');
        }

        schema.push_str(&format!("      }}{}\n", comma));
    }

    schema.push_str("    }\n");
    schema.push_str("  }\n");
    schema.push_str("}\n");

    schema
}

/// Convert ColumnType to JSON Schema type
fn column_type_to_json_type(col_type: ColumnType) -> &'static str {
    match col_type {
        ColumnType::Integer => "integer",
        ColumnType::Float => "number",
        ColumnType::Boolean => "boolean",
        ColumnType::String | ColumnType::Identifier => "string",
        ColumnType::Timestamp | ColumnType::Date | ColumnType::Time => "string",
        ColumnType::Unknown => "string",
    }
}

/// Escape string for JSON
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ColumnInfo;

    #[test]
    fn test_generate_schema() {
        let mut analysis = CsvAnalysis::new(',', true);
        analysis.columns = vec![
            ColumnInfo::new(0)
                .with_name("id".to_string())
                .with_type(ColumnType::Integer),
            ColumnInfo::new(1)
                .with_name("name".to_string())
                .with_type(ColumnType::String),
            ColumnInfo::new(2)
                .with_name("value".to_string())
                .with_type(ColumnType::Float),
        ];

        let schema = generate_schema(&analysis, "test.csv");

        assert!(schema.contains("\"$schema\":"));
        assert!(schema.contains("\"title\": \"Schema for test.csv\""));
        assert!(schema.contains("\"id\":"));
        assert!(schema.contains("\"type\": \"integer\""));
        assert!(schema.contains("\"name\":"));
        assert!(schema.contains("\"type\": \"string\""));
        assert!(schema.contains("\"value\":"));
        assert!(schema.contains("\"type\": \"number\""));
    }

    #[test]
    fn test_column_type_to_json_type() {
        assert_eq!(column_type_to_json_type(ColumnType::Integer), "integer");
        assert_eq!(column_type_to_json_type(ColumnType::Float), "number");
        assert_eq!(column_type_to_json_type(ColumnType::Boolean), "boolean");
        assert_eq!(column_type_to_json_type(ColumnType::String), "string");
        assert_eq!(column_type_to_json_type(ColumnType::Timestamp), "string");
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("hello\"world"), "hello\\\"world");
        assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_json("tab\there"), "tab\\there");
    }

    #[test]
    fn test_schema_with_samples() {
        let mut analysis = CsvAnalysis::new(',', true);
        let col = ColumnInfo::new(0)
            .with_name("id".to_string())
            .with_type(ColumnType::Integer)
            .add_sample("1".to_string())
            .add_sample("2".to_string());

        analysis.columns = vec![col];

        let schema = generate_schema(&analysis, "test.csv");
        assert!(schema.contains("\"examples\":"));
        assert!(schema.contains("\"1\""));
        assert!(schema.contains("\"2\""));
    }
}
