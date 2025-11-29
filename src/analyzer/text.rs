//! Text file analysis

use super::AnalysisError;
use crate::types::TextAnalysis;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Analyze a text file
///
/// Counts lines and words, detects if file appears to be documentation,
/// and checks for encoding issues.
///
/// # Arguments
///
/// * `path` - Path to the text file
///
/// # Returns
///
/// A TextAnalysis struct with analysis results.
pub fn analyze_text(path: &Path) -> Result<TextAnalysis, AnalysisError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut analysis = TextAnalysis::new();
    let mut has_markdown_headers = false;
    let mut has_doc_sections = false;

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                // Check for encoding issues
                analysis.encoding_issues.push(format!(
                    "Line {}: {}",
                    line_num + 1,
                    e
                ));
                continue;
            }
        };

        analysis.line_count += 1;

        // Count words
        let words = line.split_whitespace().count();
        analysis.word_count += words;

        // Check for markdown headers
        if line.trim_start().starts_with('#') {
            has_markdown_headers = true;
        }

        // Check for documentation sections
        let lower = line.to_lowercase();
        if lower.contains("introduction")
            || lower.contains("usage")
            || lower.contains("license")
            || lower.contains("installation")
            || lower.contains("getting started")
            || lower.contains("documentation")
        {
            has_doc_sections = true;
        }

        // Check for null bytes (indicates binary)
        if line.contains('\0') {
            analysis.encoding_issues.push(format!(
                "Line {}: Contains null bytes (may be binary file)",
                line_num + 1
            ));
        }

        // Check for unusual control characters
        for (i, ch) in line.chars().enumerate() {
            if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
                analysis.encoding_issues.push(format!(
                    "Line {}, column {}: Unusual control character (U+{:04X})",
                    line_num + 1,
                    i + 1,
                    ch as u32
                ));
                break; // Only report first per line
            }
        }
    }

    // Determine if this is documentation
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        if ext_lower == "md" || ext_lower == "markdown" || ext_lower == "rst" || ext_lower == "txt" {
            analysis.is_documentation = true;
        }
    }

    // Also consider it documentation if it has markdown headers or doc sections
    if has_markdown_headers || has_doc_sections {
        analysis.is_documentation = true;
    }

    // Check if file name suggests documentation
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        let name_upper = name.to_uppercase();
        if name_upper.starts_with("README")
            || name_upper.starts_with("LICENSE")
            || name_upper.starts_with("CONTRIBUTING")
            || name_upper.starts_with("CHANGELOG")
        {
            analysis.is_documentation = true;
        }
    }

    Ok(analysis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_plain_text() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_text_plain");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.txt");
        std::fs::write(&file_path, "Hello world\nThis is a test\n").unwrap();

        let result = analyze_text(&file_path).unwrap();
        assert_eq!(result.line_count, 2);
        assert_eq!(result.word_count, 6);
        assert!(result.encoding_issues.is_empty());

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_analyze_markdown() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_text_markdown");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("README.md");
        std::fs::write(
            &file_path,
            "# Title\n\n## Introduction\n\nSome content here\n",
        )
        .unwrap();

        let result = analyze_text(&file_path).unwrap();
        assert!(result.is_documentation);
        assert!(result.line_count > 0);

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_detect_documentation_by_name() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_text_readme");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("README");
        std::fs::write(&file_path, "This is a readme file\n").unwrap();

        let result = analyze_text(&file_path).unwrap();
        assert!(result.is_documentation);

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_detect_documentation_by_content() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_text_content");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("guide.txt");
        std::fs::write(&file_path, "Installation\n\nUsage\n\nLicense\n").unwrap();

        let result = analyze_text(&file_path).unwrap();
        assert!(result.is_documentation);

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_word_count() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_text_words");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.txt");
        std::fs::write(&file_path, "one two three\nfour five\n").unwrap();

        let result = analyze_text(&file_path).unwrap();
        assert_eq!(result.word_count, 5);

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_empty_file() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_text_empty");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("empty.txt");
        std::fs::write(&file_path, "").unwrap();

        let result = analyze_text(&file_path).unwrap();
        assert_eq!(result.line_count, 0);
        assert_eq!(result.word_count, 0);

        std::fs::remove_dir_all(temp_dir).ok();
    }
}
