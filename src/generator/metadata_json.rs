//! metadata.json generation

use crate::types::DatasetSummary;

/// Generate a metadata.json file for a dataset
///
/// Creates a JSON metadata file following common scientific metadata standards.
pub fn generate_metadata(summary: &DatasetSummary) -> String {
    let mut json = String::new();

    json.push_str("{\n");
    json.push_str("  \"title\": \"[TODO: Dataset Title]\",\n");
    json.push_str("  \"description\": \"[TODO: Provide a comprehensive description of this dataset]\",\n");
    json.push_str("  \"creator\": \"[TODO: Name of dataset creator or organization]\",\n");
    json.push_str("  \"date\": \"[TODO: YYYY-MM-DD]\",\n");
    json.push_str("  \"license\": \"[TODO: License identifier, e.g., MIT, CC-BY-4.0]\",\n");
    json.push_str("  \"keywords\": [\n");
    json.push_str("    \"[TODO: keyword1]\",\n");
    json.push_str("    \"[TODO: keyword2]\"\n");
    json.push_str("  ],\n");
    json.push_str("  \"contact\": {\n");
    json.push_str("    \"name\": \"[TODO: Contact name]\",\n");
    json.push_str("    \"email\": \"[TODO: contact@example.com]\"\n");
    json.push_str("  },\n");

    // Auto-populated file list
    json.push_str("  \"files\": [\n");
    json.push_str("    {\n");
    json.push_str(&format!("      \"count\": {},\n", summary.total_files));
    json.push_str(&format!("      \"total_size_bytes\": {},\n", summary.total_size));
    json.push_str("      \"types\": {\n");

    let type_count = summary.file_type_counts.len();
    for (idx, (file_type, count)) in summary.file_type_counts.iter().enumerate() {
        let comma = if idx < type_count - 1 { "," } else { "" };
        json.push_str(&format!("        \"{}\": {}{}\n", file_type, count, comma));
    }

    json.push_str("      }\n");
    json.push_str("    }\n");
    json.push_str("  ],\n");

    // Genesis preflight metadata
    json.push_str("  \"genesis_preflight\": {\n");
    json.push_str("    \"version\": \"0.1.0\",\n");
    json.push_str(&format!("    \"generated\": \"{}\",\n", summary.scan_timestamp));
    json.push_str("    \"note\": \"Review and complete all [TODO] fields before publication\"\n");
    json.push_str("  }\n");

    json.push_str("}\n");

    json
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileType;

    #[test]
    fn test_generate_metadata() {
        let mut summary = DatasetSummary::new();
        summary.total_files = 10;
        summary.total_size = 5000000;
        summary.file_type_counts = vec![(FileType::Csv, 8), (FileType::Json, 2)];
        summary.scan_timestamp = "2024-01-15 12:00:00 UTC".to_string();

        let content = generate_metadata(&summary);

        assert!(content.contains("\"title\":"));
        assert!(content.contains("\"description\":"));
        assert!(content.contains("\"creator\":"));
        assert!(content.contains("\"license\":"));
        assert!(content.contains("[TODO"));
        assert!(content.contains("\"genesis_preflight\":"));
        assert!(content.contains("\"count\": 10"));
    }

    #[test]
    fn test_metadata_valid_json_structure() {
        let summary = DatasetSummary::new();
        let content = generate_metadata(&summary);

        // Basic JSON structure validation
        assert!(content.starts_with('{'));
        assert!(content.ends_with("}\n"));
        assert!(content.contains("\"title\":"));
    }

    #[test]
    fn test_metadata_includes_todos() {
        let summary = DatasetSummary::new();
        let content = generate_metadata(&summary);

        // Verify TODO markers are present
        let todo_count = content.matches("[TODO").count();
        assert!(todo_count >= 5);
    }
}
