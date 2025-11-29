# Genesis Preflight Architecture

## System Design

This document describes the architecture and data flow of the Genesis Preflight tool.

## Data Flow Diagram

```
    +------------------+
    |   Command Line   |
    |   Arguments      |
    +--------+---------+
             |
             v
    +------------------+
    |     Scanner      |  Traverses directory, collects file metadata
    +--------+---------+
             |
             v
    +------------------+
    |    Analyzer      |  Parses files, infers structure and types
    +--------+---------+
             |
             v
    +------------------+
    |    Validator     |  Checks FAIR compliance, identifies gaps
    +--------+---------+
             |
             v
    +------------------+
    |    Generator     |  Creates missing documentation files
    +--------+---------+
             |
             v
    +------------------+
    |    Reporter      |  Produces compliance report and score
    +--------+---------+
             |
             v
    +------------------+
    |     Output       |  Terminal display + generated files
    +------------------+
```

## Component Responsibilities

### Scanner Module
- Walks directory tree using std::fs
- Extracts file size, modification time, extension
- Calculates SHA-256 hash of each file using manual implementation
- Builds in-memory representation of dataset structure
- Ignores hidden files (starting with .) by default

### Analyzer Module
- Reads first N bytes/lines of each file to infer type
- For CSV: detects delimiter, header row, column count, row count
- For JSON: validates syntax, extracts top-level keys
- For text: detects encoding issues, line count
- For binary: marks as binary, records size only
- Infers column semantics (timestamp, temperature, ID, etc.) via heuristics

### Validator Module
- Checks for presence of README.md or README.txt
- Checks for presence of LICENSE or LICENSE.txt
- Checks for metadata.json with required fields
- Checks for schema definition files
- Validates naming conventions (no spaces, lowercase, etc.)
- Checks for data/documentation ratio
- Produces list of violations with severity levels

### Generator Module
- Creates metadata.json with dataset description template
- Creates README.md with standard sections
- Creates schema.json describing detected data structure
- Creates MANIFEST.sha256 with cryptographic hashes
- Creates DATACARD.md with provenance template
- All generation is additive (never overwrites existing files)

### Reporter Module
- Calculates compliance score (0-100)
- Categorizes issues by severity (CRITICAL, WARNING, INFO)
- Formats terminal output for readability
- Generates machine-readable JSON report

## Module Structure

```
genesis-preflight/
├── src/
│   ├── main.rs              # Entry point and CLI
│   ├── lib.rs               # Public API
│   ├── scanner/             # Directory traversal
│   │   ├── mod.rs           # Module orchestrator
│   │   ├── directory.rs     # Recursive directory walking
│   │   └── file_info_builder.rs  # FileInfo construction
│   ├── analyzer/            # File content analysis
│   │   ├── mod.rs           # Analysis dispatcher
│   │   ├── csv.rs           # CSV parsing and delimiter detection
│   │   ├── json.rs          # JSON parsing (recursive descent)
│   │   ├── text.rs          # Text file analysis
│   │   ├── binary.rs        # Binary file detection
│   │   └── inference.rs     # Column type and semantic inference
│   ├── validator/           # FAIR compliance checking
│   │   ├── mod.rs           # Validation orchestrator
│   │   ├── structure.rs     # Directory structure validation
│   │   ├── naming.rs        # File naming convention checks
│   │   ├── metadata.rs      # Metadata file validation
│   │   ├── fair.rs          # FAIR principle compliance
│   │   └── data_quality.rs  # Data quality checks
│   ├── generator/           # Documentation generation
│   │   ├── mod.rs           # Generation orchestrator
│   │   ├── readme.rs        # README.md template
│   │   ├── metadata_json.rs # metadata.json generation
│   │   ├── schema.rs        # schema.json from CSV analysis
│   │   ├── manifest.rs      # MANIFEST.txt with SHA-256 hashes
│   │   └── datacard.rs      # DATACARD.md provenance template
│   ├── reporter/            # Report generation
│   │   ├── mod.rs           # Report orchestrator
│   │   ├── score.rs         # Compliance score calculation
│   │   ├── terminal.rs      # Terminal output formatting
│   │   └── json_report.rs   # Machine-readable JSON report
│   ├── crypto/              # Cryptographic functions
│   │   ├── mod.rs           # Crypto module root
│   │   └── sha256.rs        # SHA-256 implementation (FIPS 180-4)
│   └── types/               # Shared type definitions
│       ├── mod.rs           # Type module orchestrator
│       ├── file_type.rs     # FileType enum
│       ├── column_type.rs   # ColumnType enum
│       ├── validation_result.rs  # ValidationResult struct
│       ├── config.rs        # Config struct
│       ├── file_info.rs     # FileInfo struct
│       └── analysis.rs      # Analysis structs
```

## Module Dependencies

```
main.rs
  ├─> scanner (scan_directory)
  ├─> analyzer (analyze_csv, analyze_json, analyze_text, detect_binary)
  ├─> validator (validate_structure, validate_naming, validate_metadata,
  │               validate_fair_compliance, validate_data_quality)
  ├─> generator (generate_readme, generate_metadata_json, generate_schema,
  │               generate_manifest, generate_datacard)
  └─> reporter (generate_report, print_terminal_report, generate_json_report)

scanner
  ├─> crypto (sha256_file)
  └─> types (FileInfo, FileType, Config)

analyzer
  └─> types (Analysis, CsvAnalysis, JsonAnalysis, ColumnInfo, ColumnType)

validator
  └─> types (ValidationResult, ValidationSeverity, FileInfo, Analysis)

generator
  └─> types (DatasetSummary, FileInfo, Config, GeneratedFile)

reporter
  └─> types (Report, ComplianceScore, ValidationResult, FileInfo, Config)

crypto
  └─> std only (no external dependencies)

types
  └─> std only (no external dependencies)
```

## Data Flow Details

### Phase 1: Scanning
Input: Target directory path from CLI
Process:
1. Recursively traverse directory using std::fs::read_dir
2. Skip hidden files and common ignore patterns (.git, node_modules, etc.)
3. Extract file metadata (size, modification time, extension)
4. Calculate SHA-256 hash for each file (unless --no-hash flag set)
5. Classify file type by extension
Output: Vec<FileInfo>

### Phase 2: Analysis
Input: Vec<FileInfo>
Process:
1. For each file, dispatch to appropriate analyzer based on FileType
2. CSV analyzer: detect delimiter, headers, column count, infer types
3. JSON analyzer: parse structure, validate syntax, extract keys
4. Text analyzer: detect encoding, count lines, check for documentation markers
5. Binary analyzer: confirm binary format via magic number detection
Output: Vec<Analysis>

### Phase 3: Validation
Input: Vec<FileInfo>, Vec<Analysis>
Process:
1. Structure validation: check for required files (README, LICENSE, metadata.json)
2. Naming validation: verify lowercase, no spaces, descriptive names
3. Metadata validation: parse metadata.json and validate required fields
4. FAIR validation: check compliance with each FAIR principle
5. Data quality validation: check for empty files, encoding issues, type consistency
Output: Vec<ValidationResult>

### Phase 4: Generation (conditional)
Input: Vec<FileInfo>, Vec<Analysis>, Vec<ValidationResult>, Config
Process:
1. Check if Command::Generate was specified
2. Build DatasetSummary from file information
3. Generate README.md (only if missing)
4. Generate metadata.json (only if missing)
5. Generate DATACARD.md (only if missing)
6. Generate MANIFEST.txt with SHA-256 hashes (only if missing)
7. For each CSV file, generate schema.json (only if missing)
Output: Vec<GeneratedFile>

### Phase 5: Reporting
Input: Vec<FileInfo>, Vec<ValidationResult>, Vec<GeneratedFile>, Config
Process:
1. Calculate compliance score using scoring algorithm
2. Count issues by severity (Critical, Warning, Info)
3. Calculate FAIR sub-scores (Findable, Accessible, Interoperable, Reusable)
4. Build Report struct
5. Format and display terminal report (unless --quiet)
6. Optionally output JSON report (if --json flag)
7. Determine exit code based on score and critical issues
Output: Formatted report to stdout, exit code

## Design Principles

1. **Zero Dependencies**: Only Rust standard library is used
2. **Single Responsibility**: Each module has one clear purpose
3. **Immutability**: Data structures are passed immutably between stages
4. **Error Handling**: All errors are handled gracefully without panics
5. **Streaming**: Large files are processed in chunks to avoid memory issues
6. **Deterministic**: Same input always produces same output
7. **Security**: No unsafe code, no network access, no data exfiltration
8. **Transparency**: All algorithms fully documented and explainable

## Key Algorithms

### SHA-256 Hash Calculation
Implementation: src/crypto/sha256.rs
Standard: FIPS 180-4
Details: See docs/algorithms.md

### CSV Delimiter Detection
Implementation: src/analyzer/csv.rs:detect_delimiter
Algorithm: Frequency analysis of candidate delimiters across first 10 rows
Details: See docs/algorithms.md

### Column Type Inference
Implementation: src/analyzer/inference.rs
Algorithm: Pattern matching with semantic hints from column names
Details: See docs/algorithms.md

### Compliance Score Calculation
Implementation: src/reporter/score.rs:calculate_score
Algorithm: Deduction-based scoring from base of 100 points
Details: See docs/algorithms.md

## Exit Codes

- 0: Success, no issues, score >= 80
- 1: Warnings present or score between 50-79
- 2: Critical issues present or score < 50

## Performance Characteristics

- Scan rate: 10,000+ files per second on modern hardware
- Memory usage: O(n) where n is number of files (each FileInfo ~200 bytes)
- Hash rate: Limited by disk I/O, approximately 500 MB/s on SSD
- Analysis: Samples first 100 rows of CSV, first 1 MB of other files
