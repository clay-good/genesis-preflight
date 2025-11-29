//! JSON report generation

use super::Report;

/// Generate a JSON report
///
/// Creates a machine-readable JSON report containing all scan results.
pub fn generate_json_report(report: &Report) -> String {
    let mut json = String::new();

    json.push_str("{\n");
    json.push_str(&format!("  \"dataset_path\": \"{}\",\n", escape_json(&report.dataset_path)));
    json.push_str(&format!("  \"scan_timestamp\": \"{}\",\n", escape_json(&report.scan_timestamp)));

    // Score
    json.push_str("  \"score\": {\n");
    json.push_str(&format!("    \"total\": {},\n", report.score.total));
    json.push_str(&format!("    \"findable\": {},\n", report.score.findable));
    json.push_str(&format!("    \"accessible\": {},\n", report.score.accessible));
    json.push_str(&format!("    \"interoperable\": {},\n", report.score.interoperable));
    json.push_str(&format!("    \"reusable\": {},\n", report.score.reusable));
    json.push_str(&format!("    \"critical_count\": {},\n", report.score.critical_count));
    json.push_str(&format!("    \"warning_count\": {},\n", report.score.warning_count));
    json.push_str(&format!("    \"info_count\": {}\n", report.score.info_count));
    json.push_str("  },\n");

    // Files
    json.push_str("  \"files\": {\n");
    json.push_str(&format!("    \"count\": {},\n", report.files.len()));

    let total_size: u64 = report.files.iter().map(|f| f.size_bytes).sum();
    json.push_str(&format!("    \"total_size_bytes\": {}\n", total_size));
    json.push_str("  },\n");

    // Validation results
    json.push_str("  \"validation_results\": [\n");
    for (idx, result) in report.validation_results.iter().enumerate() {
        let comma = if idx < report.validation_results.len() - 1 { "," } else { "" };

        json.push_str("    {\n");
        json.push_str(&format!("      \"severity\": \"{}\",\n", result.severity));
        json.push_str(&format!("      \"code\": \"{}\",\n", escape_json(&result.code)));
        json.push_str(&format!("      \"message\": \"{}\",\n", escape_json(&result.message)));
        json.push_str(&format!("      \"suggestion\": \"{}\"", escape_json(&result.suggestion)));

        if let Some(ref path) = result.file_path {
            json.push_str(",\n");
            json.push_str(&format!("      \"file_path\": \"{}\"", escape_json(&path.to_string_lossy())));
        }

        if let Some(line) = result.line_number {
            json.push_str(",\n");
            json.push_str(&format!("      \"line_number\": {}", line));
        }

        json.push('\n');
        json.push_str(&format!("    }}{}\n", comma));
    }
    json.push_str("  ],\n");

    // Generated files
    json.push_str("  \"generated_files\": [\n");
    for (idx, gen_file) in report.generated_files.iter().enumerate() {
        let comma = if idx < report.generated_files.len() - 1 { "," } else { "" };

        json.push_str("    {\n");
        json.push_str(&format!("      \"path\": \"{}\",\n", escape_json(&gen_file.path.to_string_lossy())));
        json.push_str(&format!("      \"was_created\": {}\n", gen_file.was_created));
        json.push_str(&format!("    }}{}\n", comma));
    }
    json.push_str("  ],\n");

    // Exit code
    json.push_str(&format!("  \"exit_code\": {}\n", report.exit_code()));

    json.push_str("}\n");

    json
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
    use crate::generator::GeneratedFile;
    use crate::reporter::ComplianceScore;
    use crate::types::ValidationResult;
    use std::path::PathBuf;

    #[test]
    fn test_generate_json_report() {
        let report = Report {
            dataset_path: "/test/dataset".to_string(),
            scan_timestamp: "2024-01-15 12:00:00 UTC".to_string(),
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

        let json = generate_json_report(&report);

        assert!(json.contains("\"dataset_path\":"));
        assert!(json.contains("\"score\":"));
        assert!(json.contains("\"total\": 100"));
        assert!(json.contains("\"exit_code\":"));
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("hello\"world"), "hello\\\"world");
        assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_json_with_validation_results() {
        let report = Report {
            dataset_path: "/test".to_string(),
            scan_timestamp: "2024-01-15".to_string(),
            files: vec![],
            validation_results: vec![
                ValidationResult::critical("TEST-001", "Test issue", "Fix it"),
            ],
            generated_files: vec![],
            score: ComplianceScore {
                total: 80,
                findable: 20,
                accessible: 20,
                interoperable: 20,
                reusable: 20,
                critical_count: 1,
                warning_count: 0,
                info_count: 0,
            },
        };

        let json = generate_json_report(&report);

        assert!(json.contains("\"validation_results\":"));
        assert!(json.contains("\"code\": \"TEST-001\""));
        assert!(json.contains("\"message\": \"Test issue\""));
    }
}
