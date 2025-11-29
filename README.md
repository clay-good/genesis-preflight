# Genesis Preflight

Prepare Your Scientific Data for the [DOE Genesis Mission](https://www.whitehouse.gov/presidential-actions/2025/11/launching-the-genesis-mission/)

Genesis Preflight is a zero-dependency command-line tool that validates datasets against FAIR principles (Findable, Accessible, Interoperable, Reusable) and generates the documentation required for AI-ready scientific data.

## Mission

Genesis Preflight is dedicated to the advancement of American scientific research and to the success of the Department of Energy's Genesis Mission. This tool was built to serve the researchers, scientists, and data stewards who work tirelessly to expand humanity's understanding of the natural world and to solve the critical challenges facing our nation.

Scientific data is the foundation of discovery. But too often, valuable datasets remain inaccessible, poorly documented, or incompatible with modern AI-driven research methods. The Genesis Mission aims to change this by creating a comprehensive platform for AI-ready scientific data. Genesis Preflight exists to help researchers prepare their data for this important mission.

This tool is freely available open source software, released under the MIT License. It belongs to the American people and to the global scientific community. There are no paywalls, no subscription fees, no vendor lock-in. Every researcher, from graduate students to principal investigators at national laboratories, can use this tool without restriction.

## What This Tool Does

Genesis Preflight scans scientific datasets and performs the following operations:

1. **Validates dataset structure** - Checks for required files (README, LICENSE, metadata.json)
2. **Analyzes ALL data** - Processes every row of CSV files using streaming (handles large files)
3. **Validates documentation content** - Goes beyond file presence to check that documentation has real content, not just TODO markers
4. **Verifies data integrity** - If a MANIFEST.txt exists, validates that files haven't been modified
5. **Checks FAIR compliance** - Validates actual FAIR principle requirements, including content quality
6. **Generates documentation templates** - Creates starter files with TODO markers for you to complete
7. **Produces compliance reports** - Numerical score (0-100) with actionable recommendations

## Installation

### From crates.io

```bash
cargo install genesis-preflight
```

### From Source

```bash
git clone https://github.com/clay-good/genesis-preflight
cd genesis-preflight
cargo build --release
./target/release/genesis-preflight --version
```

## Quick Start

### 1. Scan Your Dataset

Validate an existing dataset:

```bash
genesis-preflight scan ./my-dataset
```

You'll receive a compliance score (0-100) and a list of issues to fix.

### 2. Generate Missing Documentation

Create required metadata files automatically:

```bash
genesis-preflight generate ./my-dataset
```

This creates:
- `README.md` - Human-readable overview
- `metadata.json` - Machine-readable metadata
- `DATACARD.md` - Provenance documentation
- `MANIFEST.txt` - SHA-256 file hashes
- `*.schema.json` - Data structure definitions (for CSV files)

### 3. Complete TODO Sections

Edit the generated files and replace all `[TODO]` markers with your dataset details.

### 4. Rescan to Verify

```bash
genesis-preflight scan ./my-dataset
```

Aim for a score of 80 or higher.

## Usage Examples

### Basic Scanning

```bash
# Quick scan without hashing (fast)
genesis-preflight scan ./dataset --no-hash

# Verbose output for debugging
genesis-preflight scan ./dataset --verbose

# Quiet mode (errors only)
genesis-preflight scan ./dataset --quiet
```

### Documentation Generation

```bash
# Generate in dataset directory (default)
genesis-preflight generate ./dataset

# Generate in custom location
genesis-preflight generate ./dataset --output-dir ./docs

# Generate with verbose output
genesis-preflight generate ./dataset --verbose
```

### Machine-Readable Reports

```bash
# JSON output for CI/CD
genesis-preflight report ./dataset --json

# Save to file
genesis-preflight report ./dataset --json > compliance-report.json

# Extract specific data
genesis-preflight report ./dataset --json | jq '.score.total'
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Validate Dataset
  run: |
    genesis-preflight scan ./data --json > report.json
    score=$(jq '.score.total' report.json)
    [ "$score" -ge 80 ] || exit 1
```

### Command-Line Options

**Commands:**
- `scan <path>` - Scan and validate a dataset
- `generate <path>` - Scan, validate, and generate documentation
- `report <path>` - Generate detailed compliance report

**Flags:**
- `-o, --output-dir <dir>` - Directory for generated files (default: dataset root)
- `-v, --verbose` - Show detailed progress information
- `-q, --quiet` - Suppress all non-error output
- `--no-hash` - Skip SHA-256 hashing for faster scanning
- `--json` - Output report in JSON format (report command only)
- `-h, --help` - Print help message
- `-V, --version` - Print version information

**Exit Codes:**
- `0` - No issues, score >= 80
- `1` - Warnings present or score < 80
- `2` - Critical issues or score < 50

## Design Principles

This tool is built exclusively with the Rust standard library with zero external dependencies. This deliberate choice ensures the software can be audited, trusted, and used in sensitive research environments, including those handling data subject to export controls or classification review.

**Transparency**: Every algorithm is documented. Every decision is explainable. No black boxes.

**Security**: No supply chain attacks through compromised dependencies. No network access. No data modification. No telemetry.

**Reliability**: No version conflicts or dependency resolution issues. The tool will continue to work regardless of external package availability.

**Accessibility**: Clear, professional output with actionable guidance, not just error messages.

**Respect**: Automates tedious documentation tasks while never transmitting or exfiltrating information.

**Trust**: Researchers can verify every line of code that touches their data within this single repository.

## FAIR Compliance Scoring

Genesis Preflight calculates a compliance score (0-100) based on dataset quality:

### Score Interpretation

- **90-100**: Excellent - Ready for publication
- **80-89**: Good - Meets minimum standards
- **50-79**: Fair - Needs improvement
- **0-49**: Poor - Significant issues

### FAIR Sub-Scores

Each FAIR principle is scored separately (0-25 points each):

- **Findable**: Can others discover your data?
  - Checks: metadata.json exists AND has required fields (title, description, keywords)
  - Checks: README has substantive content (not just TODOs)
  - Checks: Descriptive filenames (not generic like data1.csv)
- **Accessible**: Can others obtain your data?
  - Checks: LICENSE file exists AND contains recognized license text
  - Checks: Data files use open formats (CSV, JSON, not proprietary)
- **Interoperable**: Can others use your data with their tools?
  - Checks: Schema definitions exist AND match actual data structure
  - Checks: Standard formats, consistent delimiters
- **Reusable**: Can others trust and understand your data?
  - Checks: DATACARD has provenance/methodology filled in (not TODO)
  - Checks: Citation information provided
  - Checks: Manifest integrity (files match recorded hashes)

### Issue Severities

**Critical** (-20 points each):
- Missing LICENSE file
- Missing metadata.json or empty required fields
- No README documentation
- File integrity failures (hash mismatch)

**Warning** (-5 points each):
- Missing schema definitions
- TODO markers remaining in documentation
- Files modified since manifest was created
- Unrecognized license format

**Info** (-1 point each):
- Non-descriptive filenames
- Missing citation information
- Documentation sections lacking detail

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[architecture.md](docs/architecture.md)** - System design, data flow, module structure
- **[fair-principles.md](docs/fair-principles.md)** - FAIR validation methodology and check mapping
- **[algorithms.md](docs/algorithms.md)** - SHA-256, CSV parsing, type inference, scoring algorithms
- **[output-formats.md](docs/output-formats.md)** - Complete specifications for all generated files
- **[examples.md](docs/examples.md)** - Detailed usage examples and common workflows

## Features

### Full-File Streaming Analysis

Unlike tools that sample only the first few rows, Genesis Preflight analyzes **every row** of your CSV files using memory-efficient streaming. This ensures accurate type inference even for large datasets:

- **Memory bounded**: O(columns) not O(rows) - handles multi-gigabyte files
- **Type distribution**: Tracks actual type distribution across all values
- **80% threshold**: Types are inferred when 80%+ of values match

### RFC 4180 Compliant CSV Parsing

Properly handles:
- Quoted fields containing delimiters (`"hello, world"`)
- Escaped quotes within fields (`""` for literal `"`)
- Mixed quoted and unquoted fields

### Content Validation (Not Just File Presence)

Goes beyond checking if files exist to validate their actual content:
- **metadata.json**: Required fields have non-empty values
- **README**: Contains substantive content (>200 chars), has sections
- **LICENSE**: Contains recognized license text (MIT, Apache, CC-BY, etc.)
- **DATACARD**: Provenance sections are filled in
- **TODO detection**: Warns about incomplete sections across all docs

### Manifest Integrity Verification

When a MANIFEST.txt exists, validates that:
- All listed files still exist
- File hashes match recorded values
- New files are flagged for manifest update

### Automatic Detection

- **File Types**: CSV, JSON, text, binary (with magic number detection)
- **CSV Delimiters**: Comma, tab, semicolon, pipe (auto-detected)
- **Column Types**: Integer, float, string, boolean, timestamp, date, identifier
- **License Types**: MIT, Apache-2.0, BSD-3-Clause, CC-BY-4.0, CC0, and more
- **Documentation Files**: README, LICENSE, metadata.json, DATACARD.md

### Documentation Generation

Generates templates with TODO markers for completion:
- `README.md` - Dataset overview with auto-populated file statistics
- `metadata.json` - Structured metadata following data catalog standards
- `DATACARD.md` - Provenance documentation template
- `MANIFEST.txt` - SHA-256 checksums for all files
- `*.schema.json` - Inferred structure for CSV files (based on full-file analysis)

### Security Features

- No network access - operates completely offline
- No unsafe code - `#![forbid(unsafe_code)]`
- Read-only operations - never modifies your data files
- No telemetry - no data collection whatsoever
- Cryptographic integrity - SHA-256 hashing (FIPS 180-4 implementation)

## What This Tool Does NOT Do

- Connect to the internet or any network
- Modify your original data files (only creates new documentation)
- Require any runtime dependencies
- Use any unsafe Rust code
- Collect, transmit, or store any information about you or your data

## Target Users

- Researchers at DOE National Laboratories preparing data submissions
- Scientists at universities contributing datasets to the Genesis Mission
- Data stewards ensuring compliance with federal data standards
- Students learning about scientific data management and FAIR principles
- Any researcher who wants their data to be AI-ready

## Enterprise Deployment for DOE

Genesis Preflight can be operationalized at DOE scale through several deployment strategies:

### Centralized Validation Service
- **Deploy as REST API**: Wrap the CLI in a web service for centralized dataset validation
- **Integration with Data Portals**: Embed validation checks into DOE's Genesis Mission submission workflows
- **Batch Processing**: Use as a validation step in data pipeline automation for high-throughput processing
- **Quality Gates**: Implement as a CI/CD check for datasets in institutional repositories

### Internal Network Deployment
- **Air-Gapped Environments**: Zero external dependencies enable deployment in classified or controlled networks
- **Container Deployment**: Package as Docker/Singularity containers for HPC cluster integration
- **Module Systems**: Deploy via Spack/Environment Modules for shared HPC resources at national labs
- **Network Shares**: Install centrally on shared filesystems for lab-wide researcher access

### Database and System Integration
- **Pre-Submission Validation**: Integrate with institutional data management systems to validate datasets before Genesis Mission submission
- **Automated Workflows**: Trigger validation on data repository commits or database exports
- **Metadata Synchronization**: Parse generated metadata.json to populate DOE data catalogs and discovery systems
- **Compliance Dashboards**: Aggregate JSON reports across research groups for institutional compliance tracking

### Extensions for Confidential Data
- **Custom Validators**: Extend the validator module for DOE-specific compliance requirements (export control, classification markings)
- **Internal Catalogs**: Configure to validate against internal controlled vocabularies and ontologies
- **Access Control Integration**: Combine with institutional authentication systems for audit trails
- **Sanitization Workflows**: Use validation results to identify datasets requiring review before public release

### Research Data Management Integration
- **Lab Notebooks**: Integrate validation into electronic lab notebook (ELN) export workflows
- **Instrument Integration**: Validate data at point of collection from experimental facilities
- **Long-term Preservation**: Use as quality check before archival to DOE's long-term data repositories
- **Cross-Lab Collaboration**: Standard validation ensures interoperability across DOE's 17 national laboratories

The tool's zero-dependency architecture, read-only operations, and comprehensive output formats make it suitable for integration into diverse DOE computational and data management environments while maintaining the security requirements of sensitive research infrastructure.
