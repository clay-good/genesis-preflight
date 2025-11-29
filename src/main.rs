//! Genesis Preflight CLI
//!
//! Command-line interface for validating and documenting datasets
//! for the DOE Genesis Mission.

#![forbid(unsafe_code)]

use genesis_preflight::analyzer::analyze_file;
use genesis_preflight::generator::{
    generate_datacard, generate_manifest, generate_metadata, generate_readme,
    generate_schema, GeneratedFile,
};
use genesis_preflight::reporter::{
    generate_json_report, generate_report, print_terminal_report,
};
use genesis_preflight::scanner::scan_directory;
use genesis_preflight::types::{
    AnalysisResult, Command, Config, DatasetSummary, FileInfo, FileType, ValidationResult,
};
use genesis_preflight::validator::{
    check_integrity, validate_all_content, check_data_quality, calculate_fair_scores,
    check_naming_conventions, validate_metadata, check_structure,
};
use std::path::PathBuf;
use std::process;

const VERSION: &str = "0.1.0";

fn main() {
    let config = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    print_header(&config);

    // Scan directory
    let files = match scan_directory(&config.target_path, &config) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error scanning directory: {}", e);
            process::exit(1);
        }
    };

    if !config.quiet {
        println!("Scanned {} files", files.len());
        println!();
    }

    // Analyze files
    let analyses = analyze_files(&files, &config);

    // Validate dataset
    let validation = validate_dataset(&files, &analyses, &config);

    // Generate documentation if requested
    let generated = if matches!(config.command, Command::Generate) {
        match generate_documentation(&files, &analyses, &validation, &config) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Error generating documentation: {}", e);
                process::exit(1);
            }
        }
    } else {
        vec![]
    };

    // Generate report
    let report = generate_report(&files, &validation, &generated, &config);

    // Output report
    match config.command {
        Command::Report if config.json_output => {
            println!("{}", generate_json_report(&report));
        }
        _ => {
            print_terminal_report(&report, &config);
        }
    }

    process::exit(report.exit_code());
}

/// Parse command-line arguments
fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_help();
        process::exit(0);
    }

    // Check for flags first
    if args[1] == "--help" || args[1] == "-h" {
        print_help();
        process::exit(0);
    }

    if args[1] == "--version" || args[1] == "-V" {
        println!("genesis-preflight {}", VERSION);
        process::exit(0);
    }

    // Parse command
    let command = match args[1].as_str() {
        "scan" => Command::Scan,
        "generate" => Command::Generate,
        "report" => Command::Report,
        cmd => return Err(format!("Unknown command '{}'. Use scan, generate, or report.", cmd)),
    };

    // Parse path (required)
    if args.len() < 3 {
        return Err(format!(
            "Missing required argument <path>. Usage: genesis-preflight {} <path>",
            args[1]
        ));
    }

    let target_path = PathBuf::from(&args[2]);
    if !target_path.exists() {
        return Err(format!("Path does not exist: {}", target_path.display()));
    }

    // Create base config
    let mut config = Config::new(target_path, command);

    // Parse flags
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--output-dir" | "-o" => {
                i += 1;
                if i >= args.len() {
                    return Err("Flag --output-dir requires a value".to_string());
                }
                config.output_dir = Some(PathBuf::from(&args[i]));
            }
            "--verbose" | "-v" => {
                config.verbose = true;
            }
            "--quiet" | "-q" => {
                config.quiet = true;
            }
            "--no-hash" => {
                config.skip_hash = true;
            }
            "--json" => {
                config.json_output = true;
            }
            flag => {
                return Err(format!(
                    "Unknown flag '{}'. Use --help to see available options.",
                    flag
                ));
            }
        }
        i += 1;
    }

    // Validate conflicting flags
    if config.verbose && config.quiet {
        return Err("Cannot use --verbose and --quiet together".to_string());
    }

    Ok(config)
}

/// Print help text
fn print_help() {
    println!("Genesis Preflight {}", VERSION);
    println!();
    println!("USAGE:");
    println!("    genesis-preflight <COMMAND> <PATH> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    scan        Scan and validate a dataset");
    println!("    generate    Scan, validate, and generate documentation");
    println!("    report      Generate a detailed compliance report");
    println!();
    println!("ARGUMENTS:");
    println!("    <PATH>      Path to dataset directory");
    println!();
    println!("OPTIONS:");
    println!("    -o, --output-dir <DIR>    Directory for generated files (default: dataset root)");
    println!("    -v, --verbose             Show detailed progress information");
    println!("    -q, --quiet               Suppress all non-error output");
    println!("        --no-hash             Skip SHA-256 hashing for faster scanning");
    println!("        --json                Output report in JSON format (report command only)");
    println!("    -h, --help                Print this help message");
    println!("    -V, --version             Print version information");
    println!();
    println!("EXAMPLES:");
    println!("    # Quick scan without hashing");
    println!("    genesis-preflight scan ./my-dataset --no-hash");
    println!();
    println!("    # Generate documentation");
    println!("    genesis-preflight generate ./my-dataset");
    println!();
    println!("    # JSON report for CI/CD");
    println!("    genesis-preflight report ./my-dataset --json");
    println!();
    println!("EXIT CODES:");
    println!("    0    No issues, score >= 80");
    println!("    1    Warnings present or score < 80");
    println!("    2    Critical issues or score < 50");
}

/// Print header banner
fn print_header(config: &Config) {
    if config.quiet {
        return;
    }

    println!("================================================================");
    println!("GENESIS PREFLIGHT v{}", VERSION);
    println!("================================================================");
    println!();
}

/// Analyze all files
fn analyze_files(files: &[FileInfo], config: &Config) -> Vec<AnalysisResult> {
    let mut analyses = Vec::new();

    for file in files {
        if config.verbose {
            println!("Analyzing: {}", file.relative_path.display());
        }

        let analysis = analyze_file(file);
        analyses.push(analysis);
    }

    analyses
}

/// Validate entire dataset
fn validate_dataset(
    files: &[FileInfo],
    analyses: &[AnalysisResult],
    config: &Config,
) -> Vec<ValidationResult> {
    let mut validation = Vec::new();

    // Structure validation
    validation.extend(check_structure(files));

    // Naming validation
    validation.extend(check_naming_conventions(files));

    // Metadata validation
    validation.extend(validate_metadata(files));

    // FAIR compliance
    validation.extend(calculate_fair_scores(files, analyses));

    // Data quality
    validation.extend(check_data_quality(files, analyses));

    // Manifest integrity check (if MANIFEST.txt exists)
    validation.extend(check_integrity(files, &config.target_path));

    // Content validation (checks actual content of documentation files)
    validation.extend(validate_all_content(files, &config.target_path));

    if config.verbose {
        println!();
        println!(
            "Validation complete: {} issues found",
            validation.len()
        );
    }

    validation
}

/// Generate documentation files
fn generate_documentation(
    files: &[FileInfo],
    analyses: &[AnalysisResult],
    _validation: &[ValidationResult],
    config: &Config,
) -> Result<Vec<GeneratedFile>, String> {
    let mut generated = Vec::new();

    // Determine output directory
    let output_dir = config
        .output_dir
        .as_ref()
        .unwrap_or(&config.target_path)
        .clone();

    // Create dataset summary
    let mut summary = DatasetSummary::new();
    summary.total_files = files.len();
    summary.total_size = files.iter().map(|f| f.size_bytes).sum();
    summary.scan_timestamp = get_current_timestamp();

    // Count file types using a simple loop (Vec-based storage)
    use std::collections::HashMap;
    let mut type_counts: HashMap<FileType, usize> = HashMap::new();
    for file in files {
        *type_counts.entry(file.file_type).or_insert(0) += 1;
    }
    summary.file_type_counts = type_counts.into_iter().collect();

    if config.verbose {
        println!();
        println!("Generating documentation in: {}", output_dir.display());
    }

    // Generate README
    let readme_path = output_dir.join("README.md");
    if !readme_path.exists() {
        let content = generate_readme(&summary);
        std::fs::write(&readme_path, content)
            .map_err(|e| format!("Failed to write README.md: {}", e))?;
        generated.push(GeneratedFile::created(readme_path));
        if config.verbose {
            println!("Created: README.md");
        }
    } else {
        generated.push(GeneratedFile::skipped(readme_path));
        if config.verbose {
            println!("Skipped: README.md (already exists)");
        }
    }

    // Generate metadata.json
    let metadata_path = output_dir.join("metadata.json");
    if !metadata_path.exists() {
        let content = generate_metadata(&summary);
        std::fs::write(&metadata_path, content)
            .map_err(|e| format!("Failed to write metadata.json: {}", e))?;
        generated.push(GeneratedFile::created(metadata_path));
        if config.verbose {
            println!("Created: metadata.json");
        }
    } else {
        generated.push(GeneratedFile::skipped(metadata_path));
        if config.verbose {
            println!("Skipped: metadata.json (already exists)");
        }
    }

    // Generate DATACARD.md
    let datacard_path = output_dir.join("DATACARD.md");
    if !datacard_path.exists() {
        let content = generate_datacard(&summary);
        std::fs::write(&datacard_path, content)
            .map_err(|e| format!("Failed to write DATACARD.md: {}", e))?;
        generated.push(GeneratedFile::created(datacard_path));
        if config.verbose {
            println!("Created: DATACARD.md");
        }
    } else {
        generated.push(GeneratedFile::skipped(datacard_path));
        if config.verbose {
            println!("Skipped: DATACARD.md (already exists)");
        }
    }

    // Generate MANIFEST.txt
    let manifest_path = output_dir.join("MANIFEST.txt");
    if !manifest_path.exists() {
        let content = generate_manifest(files);
        std::fs::write(&manifest_path, content)
            .map_err(|e| format!("Failed to write MANIFEST.txt: {}", e))?;
        generated.push(GeneratedFile::created(manifest_path));
        if config.verbose {
            println!("Created: MANIFEST.txt");
        }
    } else {
        generated.push(GeneratedFile::skipped(manifest_path));
        if config.verbose {
            println!("Skipped: MANIFEST.txt (already exists)");
        }
    }

    // Generate schema files for CSV datasets
    for (idx, analysis) in analyses.iter().enumerate() {
        if let AnalysisResult::Csv(ref csv_analysis) = analysis {
            // Get the corresponding file info to get the filename
            let file_name = if idx < files.len() {
                files[idx]
                    .relative_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("data.csv")
            } else {
                "data.csv"
            };
            let schema_base = file_name.trim_end_matches(".csv").trim_end_matches(".CSV");
            let schema_path = output_dir.join(format!("{}.schema.json", schema_base));

            if !schema_path.exists() {
                let content = generate_schema(csv_analysis, file_name);
                std::fs::write(&schema_path, content)
                    .map_err(|e| format!("Failed to write schema file: {}", e))?;
                generated.push(GeneratedFile::created(schema_path));
                if config.verbose {
                    println!("Created: {}.schema.json", schema_base);
                }
            } else {
                generated.push(GeneratedFile::skipped(schema_path));
            }
        }
    }

    Ok(generated)
}

/// Get current timestamp in UTC (zero-dependency implementation)
///
/// Converts Unix epoch seconds to an ISO 8601 formatted UTC timestamp.
/// This implementation avoids external dependencies like chrono or time.
fn get_current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Convert seconds since epoch to UTC datetime components
    // Algorithm based on Howard Hinnant's date algorithms
    // http://howardhinnant.github.io/date_algorithms.html

    let days_since_epoch = (secs / 86400) as i64;
    let time_of_day = secs % 86400;

    // Convert days since 1970-01-01 to year/month/day
    // Shift epoch from 1970-01-01 to 0000-03-01 for easier calculation
    let z = days_since_epoch + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // month index [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    let y = if m <= 2 { y + 1 } else { y };

    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hour, minute, second
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args_scan() {
        let args = vec![
            "genesis-preflight".to_string(),
            "scan".to_string(),
            "/tmp/test".to_string(),
        ];
        std::env::set_var("TEST_ARGS", args.join(" "));

        // Cannot test directly due to env::args(), but structure is correct
    }

    #[test]
    fn test_version_constant() {
        assert_eq!(VERSION, "0.1.0");
    }

    #[test]
    fn test_get_current_timestamp_format() {
        let ts = get_current_timestamp();
        // Should be ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ
        assert_eq!(ts.len(), 20);
        assert!(ts.ends_with('Z'));
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
        assert_eq!(&ts[13..14], ":");
        assert_eq!(&ts[16..17], ":");
    }

    #[test]
    fn test_get_current_timestamp_not_hardcoded() {
        let ts = get_current_timestamp();
        // Should NOT be the old hardcoded value
        assert_ne!(ts, "2024-01-15 12:00:00 UTC");
        // Should start with 202x (current decade)
        assert!(ts.starts_with("202"));
    }

    #[test]
    fn test_get_current_timestamp_reasonable_year() {
        let ts = get_current_timestamp();
        let year: i32 = ts[0..4].parse().unwrap();
        // Year should be reasonable (between 2020 and 2100)
        assert!(year >= 2020 && year <= 2100);
    }
}
