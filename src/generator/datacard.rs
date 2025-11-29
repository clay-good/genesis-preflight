//! DATACARD.md generation

use crate::types::DatasetSummary;

/// Generate a DATACARD.md file for a dataset
///
/// Creates a data card (model card adapted for datasets) documenting
/// provenance, intended use, and limitations.
pub fn generate_datacard(summary: &DatasetSummary) -> String {
    let mut content = String::new();

    // Title
    content.push_str("# Data Card: [TODO: Dataset Name]\n\n");

    // Overview
    content.push_str("## Overview\n\n");
    content.push_str("[TODO: Provide a high-level overview of this dataset, including:\n");
    content.push_str("- What the data represents\n");
    content.push_str("- Why it was collected\n");
    content.push_str("- Time period covered\n");
    content.push_str("- Geographic or scientific scope]\n\n");

    // Intended Use
    content.push_str("## Intended Use\n\n");
    content.push_str("[TODO: Describe the intended uses of this dataset]\n\n");
    content.push_str("**Primary Uses:**\n");
    content.push_str("- [TODO: List primary intended uses]\n\n");
    content.push_str("**Out-of-Scope Uses:**\n");
    content.push_str("- [TODO: List uses that are not appropriate for this data]\n\n");

    // Data Collection
    content.push_str("## Data Collection\n\n");
    content.push_str("[TODO: Describe how the data was collected]\n\n");
    content.push_str("**Collection Methods:**\n");
    content.push_str("- [TODO: Describe instruments, techniques, or procedures]\n\n");
    content.push_str("**Collection Period:**\n");
    content.push_str("- [TODO: Specify dates or time range]\n\n");
    content.push_str("**Quality Control:**\n");
    content.push_str("- [TODO: Describe validation and quality control procedures]\n\n");

    // Data Format
    content.push_str("## Data Format\n\n");
    content.push_str(&format!(
        "This dataset contains {} files totaling {}.\n\n",
        summary.total_files,
        summary.format_size()
    ));

    content.push_str("**File Types:**\n");
    for (file_type, count) in &summary.file_type_counts {
        content.push_str(&format!("- {}: {} files\n", file_type, count));
    }
    content.push('\n');

    content.push_str("**File Structure:**\n");
    content.push_str("- [TODO: Describe directory structure and organization]\n\n");

    // Limitations
    content.push_str("## Limitations\n\n");
    content.push_str("[TODO: Document known limitations, including:\n");
    content.push_str("- Missing data or gaps in coverage\n");
    content.push_str("- Measurement uncertainties\n");
    content.push_str("- Known biases or systematic errors\n");
    content.push_str("- Constraints on interpretation]\n\n");

    // Provenance
    content.push_str("## Provenance\n\n");
    content.push_str("**Dataset Version:**\n");
    content.push_str("- [TODO: Version number or identifier]\n\n");
    content.push_str("**Previous Versions:**\n");
    content.push_str("- [TODO: List previous versions if applicable]\n\n");
    content.push_str("**Processing History:**\n");
    content.push_str("- [TODO: Describe any processing or transformations applied]\n\n");

    // Maintenance
    content.push_str("## Maintenance\n\n");
    content.push_str("**Update Frequency:**\n");
    content.push_str("- [TODO: How often will this dataset be updated?]\n\n");
    content.push_str("**Retention Policy:**\n");
    content.push_str("- [TODO: How long will this dataset be maintained?]\n\n");

    // Footer
    content.push_str("---\n\n");
    content.push_str(&format!("**Generated:** {}\n", summary.scan_timestamp));
    content.push_str("**Tool:** genesis-preflight v0.1.0\n\n");
    content.push_str("Review and complete all [TODO] sections before publication.\n");

    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_datacard() {
        let mut summary = DatasetSummary::new();
        summary.total_files = 5;
        summary.total_size = 1024 * 1024;
        summary.scan_timestamp = "2024-01-15 12:00:00 UTC".to_string();

        let content = generate_datacard(&summary);

        assert!(content.contains("# Data Card:"));
        assert!(content.contains("## Overview"));
        assert!(content.contains("## Intended Use"));
        assert!(content.contains("## Data Collection"));
        assert!(content.contains("## Data Format"));
        assert!(content.contains("## Limitations"));
        assert!(content.contains("## Provenance"));
        assert!(content.contains("## Maintenance"));
        assert!(content.contains("genesis-preflight"));
    }

    #[test]
    fn test_datacard_has_todos() {
        let summary = DatasetSummary::new();
        let content = generate_datacard(&summary);

        // Verify TODO markers are present
        let todo_count = content.matches("[TODO").count();
        assert!(todo_count >= 10);
    }

    #[test]
    fn test_datacard_includes_summary_info() {
        let mut summary = DatasetSummary::new();
        summary.total_files = 42;
        summary.total_size = 1024 * 1024 * 100; // 100 MB

        let content = generate_datacard(&summary);

        assert!(content.contains("42 files"));
    }
}
