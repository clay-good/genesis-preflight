//! Terminal report output

use super::{ComplianceScore, Report};
use crate::types::{Config, FileType, ValidationSeverity};
use std::collections::HashMap;

/// Print report to terminal
///
/// Outputs a formatted compliance report to stdout.
/// No ANSI colors or emojis for maximum portability.
pub fn print_terminal_report(report: &Report, config: &Config) {
    if config.quiet {
        return;
    }

    println!("================================================================");
    println!("GENESIS PREFLIGHT REPORT");
    println!("================================================================");
    println!();
    println!("Dataset: {}", report.dataset_path);
    println!("Scanned: {}", report.scan_timestamp);
    println!();

    print_summary(report);
    print_compliance_score(&report.score);
    print_issues(report);
    print_generated_files(report);
    print_next_steps(report);

    println!("================================================================");
}

/// Print summary section
fn print_summary(report: &Report) {
    println!("SUMMARY");
    println!("-------");
    println!("Files scanned: {}", report.files.len());

    let total_size: u64 = report.files.iter().map(|f| f.size_bytes).sum();
    println!("Total size: {}", format_size(total_size));

    // Count file types
    let mut type_counts: HashMap<FileType, usize> = HashMap::new();
    for file in &report.files {
        *type_counts.entry(file.file_type).or_insert(0) += 1;
    }

    let mut types: Vec<_> = type_counts.into_iter().collect();
    types.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    print!("File types: ");
    for (idx, (file_type, count)) in types.iter().enumerate() {
        if idx > 0 {
            print!(", ");
        }
        print!("{} {}", count, file_type);
    }
    println!();
    println!();
}

/// Print compliance score section
fn print_compliance_score(score: &ComplianceScore) {
    println!("COMPLIANCE SCORE: {}/100", score.total);
    println!("-----------------------------------------------");
    println!("Findable:       {}/25", score.findable);
    println!("Accessible:     {}/25", score.accessible);
    println!("Interoperable:  {}/25", score.interoperable);
    println!("Reusable:       {}/25", score.reusable);
    println!();
}

/// Print issues section
fn print_issues(report: &Report) {
    println!("ISSUES FOUND");
    println!("------------");

    // Group by severity
    let critical: Vec<_> = report
        .validation_results
        .iter()
        .filter(|v| matches!(v.severity, ValidationSeverity::Critical))
        .collect();

    let warnings: Vec<_> = report
        .validation_results
        .iter()
        .filter(|v| matches!(v.severity, ValidationSeverity::Warning))
        .collect();

    let info: Vec<_> = report
        .validation_results
        .iter()
        .filter(|v| matches!(v.severity, ValidationSeverity::Info))
        .collect();

    // Print critical issues
    if !critical.is_empty() {
        println!("CRITICAL ({}):", critical.len());
        for issue in critical.iter().take(10) {
            println!("  [{}] {}", issue.code, issue.message);
            println!("    -> {}", issue.suggestion);
        }
        if critical.len() > 10 {
            println!("  ... and {} more", critical.len() - 10);
        }
        println!();
    }

    // Print warnings
    if !warnings.is_empty() {
        println!("WARNING ({}):", warnings.len());
        for issue in warnings.iter().take(10) {
            println!("  [{}] {}", issue.code, issue.message);
            println!("    -> {}", issue.suggestion);
        }
        if warnings.len() > 10 {
            println!("  ... and {} more", warnings.len() - 10);
        }
        println!();
    }

    // Print info
    if !info.is_empty() {
        println!("INFO ({}):", info.len());
        for issue in info.iter().take(5) {
            println!("  [{}] {}", issue.code, issue.message);
            println!("    -> {}", issue.suggestion);
        }
        if info.len() > 5 {
            println!("  ... and {} more", info.len() - 5);
        }
        println!();
    }

    if critical.is_empty() && warnings.is_empty() && info.is_empty() {
        println!("No issues found.");
        println!();
    }
}

/// Print generated files section
fn print_generated_files(report: &Report) {
    if report.generated_files.is_empty() {
        return;
    }

    println!("GENERATED FILES");
    println!("---------------");

    for gen_file in &report.generated_files {
        let filename = gen_file
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if gen_file.was_created {
            println!("Created: {}", filename);
        } else {
            println!("Skipped: {} (already exists)", filename);
        }
    }
    println!();
}

/// Print next steps section
fn print_next_steps(report: &Report) {
    println!("NEXT STEPS");
    println!("----------");

    let has_generated = report.generated_files.iter().any(|f| f.was_created);

    if has_generated {
        println!("1. Review and complete all [TODO] sections in generated files");
    }

    if report.score.critical_count > 0 {
        println!(
            "{}. Address {} critical issue{} before submission",
            if has_generated { 2 } else { 1 },
            report.score.critical_count,
            if report.score.critical_count == 1 {
                ""
            } else {
                "s"
            }
        );
    }

    if report.score.warning_count > 0 {
        let step = if has_generated { 2 } else { 1 } + if report.score.critical_count > 0 { 1 } else { 0 };
        println!(
            "{}. Consider addressing {} warning{} to improve score",
            step,
            report.score.warning_count,
            if report.score.warning_count == 1 {
                ""
            } else {
                "s"
            }
        );
    }

    if report.score.total >= 80 && report.score.critical_count == 0 {
        println!("Dataset meets minimum compliance standards.");
    }

    println!();
}

/// Format byte size as human-readable string
fn format_size(bytes: u64) -> String {
    let size = bytes as f64;
    if size < 1024.0 {
        format!("{} B", bytes)
    } else if size < 1024.0 * 1024.0 {
        format!("{:.2} KB", size / 1024.0)
    } else if size < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} MB", size / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(2048), "2.00 KB");
        assert_eq!(format_size(2 * 1024 * 1024), "2.00 MB");
        assert_eq!(format_size(3 * 1024 * 1024 * 1024), "3.00 GB");
    }
}
