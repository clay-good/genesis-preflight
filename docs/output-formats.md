# Output File Formats

## Overview

Genesis Preflight generates several documentation files when run with the `generate` command. This document specifies the format and content of each generated file.

## metadata.json

### Purpose

Machine-readable metadata describing the dataset in a standardized format compatible with data catalogs and search engines.

### Location

`<dataset-root>/metadata.json`

### Format

JSON object with nested structure

### Schema

```json
{
  "name": "string (required)",
  "version": "string (semver format, required)",
  "description": "string (required)",
  "keywords": ["string", "..."],
  "license": "string (SPDX identifier or name)",
  "authors": [
    {
      "name": "string",
      "email": "string (optional)",
      "affiliation": "string (optional)",
      "orcid": "string (optional)"
    }
  ],
  "created": "string (ISO 8601 date)",
  "updated": "string (ISO 8601 date)",
  "contact": {
    "name": "string",
    "email": "string"
  },
  "access": {
    "type": "string (open|restricted|embargoed)",
    "restrictions": "string (optional)",
    "url": "string (optional)"
  },
  "provenance": {
    "collection_method": "string",
    "collection_period": {
      "start": "string (ISO 8601 date)",
      "end": "string (ISO 8601 date, optional)"
    },
    "geographic_coverage": "string (optional)",
    "instruments": ["string", "..."]
  },
  "files": {
    "count": 0,
    "total_size_bytes": 0,
    "formats": ["string", "..."]
  },
  "related_resources": [
    {
      "type": "string (publication|dataset|documentation)",
      "title": "string",
      "url": "string",
      "doi": "string (optional)"
    }
  ]
}
```

### Required Fields

- `name`: Dataset name
- `version`: Version number (e.g., "1.0.0")
- `description`: Brief description of dataset
- `license`: License identifier
- `authors`: At least one author with name
- `created`: Creation date

### Optional Fields

All other fields are optional but recommended for full FAIR compliance.

### Example

```json
{
  "name": "Climate Observations 2024",
  "version": "1.0.0",
  "description": "Hourly temperature and humidity measurements from 15 weather stations in the Pacific Northwest region during January-March 2024.",
  "keywords": [
    "climate",
    "temperature",
    "humidity",
    "weather",
    "Pacific Northwest"
  ],
  "license": "CC-BY-4.0",
  "authors": [
    {
      "name": "Dr. Jane Smith",
      "email": "jsmith@example.gov",
      "affiliation": "Pacific Northwest National Laboratory",
      "orcid": "0000-0002-1234-5678"
    }
  ],
  "created": "2024-04-01",
  "updated": "2024-04-15",
  "contact": {
    "name": "Data Steward",
    "email": "data@example.gov"
  },
  "access": {
    "type": "open",
    "url": "https://data.example.gov/climate-2024"
  },
  "provenance": {
    "collection_method": "Automated weather station sensors",
    "collection_period": {
      "start": "2024-01-01",
      "end": "2024-03-31"
    },
    "geographic_coverage": "Pacific Northwest, USA (45°N-49°N, 117°W-124°W)",
    "instruments": [
      "Davis Vantage Pro2",
      "Campbell Scientific CR1000"
    ]
  },
  "files": {
    "count": 45,
    "total_size_bytes": 12500000,
    "formats": ["CSV", "JSON"]
  },
  "related_resources": [
    {
      "type": "publication",
      "title": "Climate Trends in the Pacific Northwest",
      "url": "https://doi.org/10.1234/example",
      "doi": "10.1234/example"
    }
  ]
}
```

### Generation Behavior

- Created only if `metadata.json` does not exist
- Contains [TODO] markers for user completion
- Auto-populates `files` section with scan results
- Leaves all descriptive fields for manual completion

## schema.json

### Purpose

Machine-readable schema describing the structure of CSV data files, enabling automatic validation and integration.

### Location

`<dataset-root>/<filename>.schema.json` (one per CSV file)

Example: For `data/temperature.csv`, generates `data/temperature.schema.json`

### Format

JSON object following a simplified schema format

### Schema

```json
{
  "file": "string (filename)",
  "format": "CSV",
  "delimiter": "string (single character)",
  "has_header": boolean,
  "columns": [
    {
      "name": "string",
      "index": number,
      "type": "string (Integer|Float|String|Boolean|Timestamp|Date)",
      "description": "string (optional)",
      "unit": "string (optional)",
      "constraints": {
        "required": boolean,
        "unique": boolean,
        "min": number (optional),
        "max": number (optional),
        "pattern": "string (regex, optional)"
      }
    }
  ],
  "row_count": number,
  "generated_by": "string (tool name and version)"
}
```

### Field Descriptions

**File-level**:
- `file`: Name of the CSV file
- `format`: Always "CSV"
- `delimiter`: Detected delimiter character
- `has_header`: Whether first row is header
- `row_count`: Number of data rows (excluding header)

**Column-level**:
- `name`: Column name (from header or generated "column_N")
- `index`: Zero-based column index
- `type`: Inferred data type
- `description`: [TODO] marker for user to describe
- `unit`: [TODO] marker for unit of measurement
- `constraints`: Validation rules (optional)

### Example

```json
{
  "file": "temperature.csv",
  "format": "CSV",
  "delimiter": ",",
  "has_header": true,
  "columns": [
    {
      "name": "timestamp",
      "index": 0,
      "type": "Timestamp",
      "description": "Measurement timestamp in UTC",
      "unit": "ISO8601",
      "constraints": {
        "required": true,
        "unique": true
      }
    },
    {
      "name": "station_id",
      "index": 1,
      "type": "String",
      "description": "Weather station identifier",
      "unit": null,
      "constraints": {
        "required": true,
        "pattern": "^[A-Z]{3}[0-9]{3}$"
      }
    },
    {
      "name": "temperature_c",
      "index": 2,
      "type": "Float",
      "description": "Air temperature",
      "unit": "degrees Celsius",
      "constraints": {
        "required": true,
        "min": -50.0,
        "max": 60.0
      }
    },
    {
      "name": "humidity_pct",
      "index": 3,
      "type": "Float",
      "description": "Relative humidity",
      "unit": "percent",
      "constraints": {
        "required": true,
        "min": 0.0,
        "max": 100.0
      }
    }
  ],
  "row_count": 2160,
  "generated_by": "genesis-preflight v0.1.0"
}
```

### Generation Behavior

- Created only if `<filename>.schema.json` does not exist
- Generated for each CSV file in dataset
- Auto-detects delimiter, header, column count, types
- Leaves `description` and `unit` fields with [TODO] markers
- Constraints section left for manual specification

## MANIFEST.txt

### Purpose

Cryptographic manifest listing all files with SHA-256 hashes for integrity verification and provenance.

### Location

`<dataset-root>/MANIFEST.txt`

### Format

Plain text file with one file per line

### Structure

```
SHA256 (filename) = hash
SHA256 (filename) = hash
...
```

Each line contains:
1. Literal text "SHA256 "
2. Filename in parentheses (relative to dataset root)
3. Literal text " = "
4. 64-character hexadecimal SHA-256 hash

### Example

```
SHA256 (README.md) = 5d41402abc4b2a76b9719d911017c592ae6f9d9f7b3f3c6e8d6f5e4d3c2b1a09
SHA256 (LICENSE) = 2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae
SHA256 (metadata.json) = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
SHA256 (data/temperature.csv) = baa5a0964d3320fbc0c6a922140453c8513ea24ab8fd0577034804a967248096
SHA256 (data/humidity.csv) = 3c59dc048e8850243be8079a5c74d079e6f4e9c7d2f7e8c5b4a3d2c1e0f9a8b7
```

### Sorting

Files are listed in alphabetical order by relative path.

### Verification

Users can verify file integrity using standard tools:

```bash
# Linux/macOS
sha256sum -c MANIFEST.txt

# Or manually for single file
sha256sum data/temperature.csv
```

Expected output should match hash in manifest.

### Generation Behavior

- Created only if `MANIFEST.txt` does not exist
- Includes all files in dataset (except MANIFEST.txt itself)
- Hashes calculated using FIPS 180-4 SHA-256 implementation
- Can be skipped with `--no-hash` flag for faster scanning

### Use Cases

1. **Integrity verification**: Detect file corruption or modification
2. **Provenance**: Prove dataset hasn't changed since publication
3. **Transfer validation**: Verify complete download
4. **Version control**: Track which files changed between versions

## DATACARD.md

### Purpose

Human-readable provenance documentation describing how the data was collected, processed, and its intended use (data card concept adapted from model cards).

### Location

`<dataset-root>/DATACARD.md`

### Format

Markdown document with standard sections

### Structure

```markdown
# Data Card: [TODO: Dataset Name]

## Overview
[TODO: High-level description]

## Intended Use
[TODO: Primary uses]
[TODO: Out-of-scope uses]

## Data Collection
[TODO: Collection methods]
[TODO: Collection period]
[TODO: Quality control]

## Data Format
[Auto-generated file statistics]

## Limitations
[TODO: Known limitations]

## Provenance
[TODO: Version information]
[TODO: Processing history]

## Maintenance
[TODO: Update frequency]
[TODO: Retention policy]
```

### Sections

**Overview**:
- What the data represents
- Why it was collected
- Time period covered
- Geographic or scientific scope

**Intended Use**:
- Primary intended uses
- Secondary uses
- Out-of-scope uses (what NOT to use it for)

**Data Collection**:
- Collection methods (instruments, techniques, procedures)
- Collection period (start and end dates)
- Quality control procedures
- Calibration information

**Data Format**:
- Auto-populated file counts and sizes
- File type breakdown
- Directory structure description [TODO]

**Limitations**:
- Missing data or gaps in coverage
- Measurement uncertainties
- Known biases or systematic errors
- Constraints on interpretation

**Provenance**:
- Dataset version
- Previous versions (if applicable)
- Processing history
- Transformations applied

**Maintenance**:
- Update frequency
- Retention policy
- Deprecation timeline (if any)

### Example

```markdown
# Data Card: Pacific Northwest Climate Observations 2024

## Overview

Hourly temperature and humidity measurements from 15 automated weather stations across the Pacific Northwest region of the United States. Data collected from January 1, 2024 through March 31, 2024 (Q1 2024).

This dataset supports climate research, weather modeling, and environmental monitoring applications.

## Intended Use

**Primary Uses:**
- Climate trend analysis for Pacific Northwest region
- Weather forecast model validation
- Environmental impact studies
- Educational purposes for atmospheric science courses

**Out-of-Scope Uses:**
- Real-time weather forecasting (data is historical)
- Precision agriculture (station density insufficient)
- Legal proceedings requiring certified measurements

## Data Collection

**Collection Methods:**
- Automated weather stations with calibrated sensors
- Davis Vantage Pro2 and Campbell Scientific CR1000 instruments
- Hourly automated readings stored to on-site data loggers
- Weekly data retrieval via cellular telemetry

**Collection Period:**
- Start: January 1, 2024 00:00 UTC
- End: March 31, 2024 23:59 UTC
- Total: 90 days, 2,160 hourly measurements per station

**Quality Control:**
- Automated range checks (temperature: -50°C to +60°C)
- Automated sensor health monitoring
- Manual review of flagged anomalies
- Cross-station consistency checks
- Instrument calibration: December 2023

## Data Format

This dataset contains 45 files totaling 11.92 MB.

**File Types:**
- CSV: 30 files (temperature and humidity time series)
- JSON: 15 files (station metadata)

**File Structure:**
- data/: Time series data files (CSV)
- metadata/: Station information (JSON)
- schema/: Data schema definitions

## Limitations

- Two stations (PDX007 and SEA012) experienced data logger failures resulting in 48-hour gaps on Feb 15-16
- Temperature sensor at station OLY003 suspected drift of +0.3°C based on comparison with nearby stations
- Humidity readings during freezing conditions (<0°C) may be less accurate due to ice formation on sensors
- No direct solar radiation measurements (future enhancement planned)
- Station coverage concentrated west of Cascade Range; eastern stations underrepresented

## Provenance

**Dataset Version:** 1.0.0

**Previous Versions:** None (initial release)

**Processing History:**
1. Raw logger data downloaded (CSV format)
2. Timestamps converted to UTC
3. Automated QC filters applied
4. Flagged values reviewed manually
5. Station metadata compiled from field logs
6. Data aggregated into single-station files

**No transformations, smoothing, or gap-filling applied.** Data presented as collected.

## Maintenance

**Update Frequency:**
- This is a static historical dataset (Q1 2024 only)
- No updates planned for this version
- Future quarters will be released as separate versioned datasets

**Retention Policy:**
- Minimum 10-year retention guaranteed
- Archived in DOE permanent repository after publication
- Continued access via https://data.example.gov/climate

---

**Generated:** 2024-04-15 10:30:00 UTC
**Tool:** genesis-preflight v0.1.0

Review and complete all [TODO] sections before publication.
```

### Generation Behavior

- Created only if `DATACARD.md` does not exist
- Includes [TODO] markers for all descriptive sections
- Auto-populates file statistics from scan results
- Includes footer with generation timestamp

## README.md

### Purpose

Primary human-readable documentation providing overview, usage instructions, and context for the dataset.

### Location

`<dataset-root>/README.md`

### Format

Markdown document with standard sections

### Structure

```markdown
# [TODO: Dataset Name]

## Description
[TODO: Brief description]

## Contents
[Auto-generated file listing]

## Format
[TODO: Format details]

## Usage
[TODO: How to use this data]

## Citation
[TODO: How to cite]

## License
[TODO: License terms]

## Contact
[TODO: Contact information]
```

### Sections

**Description**: Brief overview of what this dataset contains

**Contents**: Auto-generated list of files and directories

**Format**: Description of file formats and structure

**Usage**: Instructions for accessing and using the data

**Citation**: How to cite this dataset in publications

**License**: Summary of usage terms and license

**Contact**: Who to contact with questions

### Example

```markdown
# Pacific Northwest Climate Observations 2024

## Description

This dataset contains hourly temperature and humidity measurements from 15 automated weather stations in the Pacific Northwest region. Data covers the first quarter of 2024 (January-March).

Measurements are provided in standard CSV format with accompanying metadata and schema definitions.

## Contents

This dataset contains 45 files totaling 11.92 MB.

**File Types:**
- CSV: 30 files
- JSON: 15 files

**Directory Structure:**
```
climate-data-2024/
├── README.md (this file)
├── LICENSE
├── metadata.json
├── DATACARD.md
├── MANIFEST.txt
├── data/
│   ├── station_*/
│   │   ├── temperature.csv
│   │   ├── temperature.schema.json
│   │   ├── humidity.csv
│   │   └── humidity.schema.json
└── metadata/
    └── stations.json
```

## Format

**Temperature Data** (temperature.csv):
- Timestamp (UTC, ISO 8601)
- Station ID
- Temperature (degrees Celsius)
- Measurement quality flag

**Humidity Data** (humidity.csv):
- Timestamp (UTC, ISO 8601)
- Station ID
- Relative humidity (percent)
- Measurement quality flag

See individual `.schema.json` files for complete column specifications.

## Usage

**Quick Start:**
```python
import pandas as pd

# Load temperature data
df = pd.read_csv('data/station_PDX001/temperature.csv')
df['timestamp'] = pd.to_datetime(df['timestamp'])

# Basic analysis
print(df['temperature_c'].describe())
```

**Data Integrity:**
Verify file integrity using the provided manifest:
```bash
sha256sum -c MANIFEST.txt
```

## Citation

If you use this dataset in your research, please cite:

```
Smith, J. et al. (2024). Pacific Northwest Climate Observations Q1 2024.
Pacific Northwest National Laboratory. https://doi.org/10.xxxxx/example
```

BibTeX:
```bibtex
@dataset{smith2024climate,
  author = {Smith, Jane and others},
  title = {Pacific Northwest Climate Observations Q1 2024},
  year = {2024},
  publisher = {Pacific Northwest National Laboratory},
  doi = {10.xxxxx/example}
}
```

## License

This dataset is released under the Creative Commons Attribution 4.0 International License (CC-BY-4.0).

You are free to:
- Share: Copy and redistribute the material
- Adapt: Remix, transform, and build upon the material

Under the following terms:
- Attribution: You must give appropriate credit

See LICENSE file for full terms.

## Contact

**Data Steward:** Dr. Jane Smith
**Email:** jsmith@example.gov
**Institution:** Pacific Northwest National Laboratory

For questions, issues, or data requests, please contact the data steward or open an issue at https://github.com/example/climate-data-2024

---

**Generated with:** genesis-preflight v0.1.0
**Scan Date:** 2024-04-15
```

### Generation Behavior

- Created only if `README.md` does not exist
- Auto-populates file statistics and directory tree
- Includes [TODO] markers for user completion
- Never overwrites existing README

## JSON Report Format

### Purpose

Machine-readable compliance report for CI/CD integration and automated processing.

### Output Method

Printed to stdout when using `--json` flag:
```bash
genesis-preflight report ./dataset --json > report.json
```

### Schema

```json
{
  "dataset_path": "string",
  "scan_timestamp": "string (ISO 8601)",
  "score": {
    "total": number (0-100),
    "findable": number (0-25),
    "accessible": number (0-25),
    "interoperable": number (0-25),
    "reusable": number (0-25),
    "critical_count": number,
    "warning_count": number,
    "info_count": number
  },
  "files": {
    "count": number,
    "total_size_bytes": number
  },
  "validation_results": [
    {
      "severity": "string (Critical|Warning|Info)",
      "code": "string",
      "message": "string",
      "suggestion": "string",
      "file_path": "string (optional)",
      "line_number": number (optional)
    }
  ],
  "generated_files": [
    {
      "path": "string",
      "was_created": boolean
    }
  ],
  "exit_code": number (0|1|2)
}
```

### Example

```json
{
  "dataset_path": "/Users/alice/my-dataset",
  "scan_timestamp": "2024-04-15 14:30:00 UTC",
  "score": {
    "total": 65,
    "findable": 18,
    "accessible": 15,
    "interoperable": 20,
    "reusable": 15,
    "critical_count": 2,
    "warning_count": 3,
    "info_count": 1
  },
  "files": {
    "count": 15,
    "total_size_bytes": 5242880
  },
  "validation_results": [
    {
      "severity": "Critical",
      "code": "FAIR-A001",
      "message": "Missing LICENSE file",
      "suggestion": "Add a LICENSE file to specify usage terms",
      "file_path": null,
      "line_number": null
    },
    {
      "severity": "Critical",
      "code": "FAIR-F001",
      "message": "Missing metadata.json",
      "suggestion": "Run 'genesis-preflight generate' to create metadata.json",
      "file_path": null,
      "line_number": null
    },
    {
      "severity": "Warning",
      "code": "FAIR-F003",
      "message": "File contains spaces in name",
      "suggestion": "Rename to use underscores instead of spaces",
      "file_path": "data/my file.csv",
      "line_number": null
    }
  ],
  "generated_files": [],
  "exit_code": 2
}
```

### Use Cases

**CI/CD Integration:**
```bash
#!/bin/bash
genesis-preflight report ./dataset --json > report.json
score=$(jq '.score.total' report.json)
if [ "$score" -lt 80 ]; then
  echo "Dataset does not meet minimum compliance score of 80"
  exit 1
fi
```

**Automated Monitoring:**
```python
import json
import subprocess

result = subprocess.run(
    ['genesis-preflight', 'report', './dataset', '--json'],
    capture_output=True,
    text=True
)

report = json.loads(result.stdout)

if report['score']['critical_count'] > 0:
    send_alert(f"Critical issues found: {report['score']['critical_count']}")
```

## File Generation Rules

### Never Overwrite

Genesis Preflight follows a strict no-overwrite policy:
- All generated files are skipped if they already exist
- Users can safely re-run `generate` command
- Manual edits are never lost

### Conditional Generation

Files are only generated when running `generate` command:
```bash
genesis-preflight generate ./dataset
```

Scanning alone does not create files:
```bash
genesis-preflight scan ./dataset  # No files created
```

### TODO Markers

All generated files include [TODO] markers where user input is required:
- Descriptive text fields
- Domain-specific metadata
- Contact information
- Citation details

Users must complete these before publication.

### Output Directory

By default, files are created in the dataset root directory. Use `--output-dir` to specify alternative location:
```bash
genesis-preflight generate ./dataset --output-dir ./generated-docs
```
