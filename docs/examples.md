# Usage Examples

## Overview

This document provides practical examples of using Genesis Preflight to validate and document scientific datasets.

## Example 1: Scanning a Clean Dataset

### Scenario

You have a well-organized climate dataset that already has basic documentation. You want to verify it meets FAIR compliance standards.

### Dataset Structure

```
climate-data/
├── README.md
├── LICENSE
├── metadata.json
├── data/
│   ├── temperature.csv
│   └── humidity.csv
└── docs/
    └── methodology.md
```

### Command

```bash
genesis-preflight scan ./climate-data
```

### Output

```
================================================================
GENESIS PREFLIGHT v0.1.0
================================================================

Scanned 6 files

================================================================
GENESIS PREFLIGHT REPORT
================================================================

Dataset: /Users/researcher/climate-data
Scanned: 2024-04-15 14:30:00 UTC

SUMMARY
-------
Files scanned: 6
Total size: 5.23 MB
File types: 2 CSV, 3 Text, 1 JSON

COMPLIANCE SCORE: 95/100
-----------------------------------------------
Findable:       25/25
Accessible:     25/25
Interoperable:  22/25
Reusable:       23/25

ISSUES FOUND
------------
WARNING (2):
  [FAIR-I001] Missing schema definition for temperature.csv
    -> Run 'genesis-preflight generate' to create schema.json
  [FAIR-R002] Missing DATACARD.md provenance documentation
    -> Create DATACARD.md with collection details

No issues found.

NEXT STEPS
----------
1. Consider addressing 2 warnings to improve score
Dataset meets minimum compliance standards.

================================================================
```

### Interpretation

- **Score: 95/100** - Excellent compliance
- **Exit code: 1** - Minor warnings present
- **Action**: Generate missing schema and datacard, or accept current state

### Follow-up

Generate missing files:
```bash
genesis-preflight generate ./climate-data
```

This will create:
- `data/temperature.schema.json`
- `data/humidity.schema.json`
- `DATACARD.md`
- `MANIFEST.txt`

## Example 2: Scanning a Messy Dataset

### Scenario

You inherited a dataset with poor organization. You want to identify all compliance issues.

### Dataset Structure

```
My Research Data/
├── Data 1 Final.xlsx
├── output (copy).csv
├── readme.txt
└── some notes.docx
```

### Command

```bash
genesis-preflight scan "./My Research Data"
```

### Output

```
================================================================
GENESIS PREFLIGHT v0.1.0
================================================================

Scanned 4 files

================================================================
GENESIS PREFLIGHT REPORT
================================================================

Dataset: /Users/researcher/My Research Data
Scanned: 2024-04-15 15:00:00 UTC

SUMMARY
-------
Files scanned: 4
Total size: 2.1 MB
File types: 1 CSV, 2 Text, 1 Binary

COMPLIANCE SCORE: 25/100
-----------------------------------------------
Findable:       5/25
Accessible:     0/25
Interoperable:  10/25
Reusable:       10/25

ISSUES FOUND
------------
CRITICAL (3):
  [FAIR-A001] Missing LICENSE file
    -> Add a LICENSE file (MIT, CC0, CC-BY, etc.)
  [FAIR-F001] Missing metadata.json
    -> Run 'genesis-preflight generate' to create metadata.json
  [STR-001] Dataset directory name contains spaces
    -> Rename to 'my-research-data' or 'my_research_data'

WARNING (5):
  [FAIR-F003] File name contains spaces: Data 1 Final.xlsx
    -> Rename to 'data_1_final.xlsx'
  [FAIR-F003] File name contains spaces: output (copy).csv
    -> Rename to 'output_copy.csv'
  [FAIR-F003] File name contains spaces: some notes.docx
    -> Rename to 'some_notes.docx'
  [FAIR-A004] Proprietary format detected: Data 1 Final.xlsx
    -> Convert to CSV or JSON for better accessibility
  [FAIR-I001] Missing schema definition
    -> Run 'genesis-preflight generate' to create schemas

INFO (2):
  [NAME-001] Filename not lowercase: Data 1 Final.xlsx
    -> Rename to lowercase for consistency
  [NAME-001] Filename contains 'copy': output (copy).csv
    -> Remove version indicators from filename

NEXT STEPS
----------
1. Address 3 critical issues before submission
2. Consider addressing 5 warnings to improve score

================================================================
```

### Interpretation

- **Score: 25/100** - Poor compliance
- **Exit code: 2** - Critical issues present
- **Action**: Major cleanup required before publication

### Remediation Steps

1. **Rename directory**:
   ```bash
   mv "My Research Data" my-research-data
   cd my-research-data
   ```

2. **Rename files**:
   ```bash
   mv "Data 1 Final.xlsx" data_1_final.xlsx
   mv "output (copy).csv" output.csv
   mv "some notes.docx" notes.docx
   ```

3. **Convert proprietary formats**:
   - Open `data_1_final.xlsx` in Excel/LibreOffice
   - Export as CSV: `data_1.csv`

4. **Add LICENSE**:
   ```bash
   echo "MIT License

Copyright (c) 2024 Your Name

Permission is hereby granted..." > LICENSE
   ```

5. **Generate documentation**:
   ```bash
   genesis-preflight generate ./my-research-data
   ```

6. **Re-scan**:
   ```bash
   genesis-preflight scan ./my-research-data
   ```

Expected new score: 70-80/100

## Example 3: Generating Documentation

### Scenario

You have data files but no documentation. You want Genesis Preflight to create templates.

### Dataset Structure (Before)

```
experiment-2024/
└── data/
    ├── measurements.csv
    └── observations.csv
```

### Command

```bash
genesis-preflight generate ./experiment-2024 --verbose
```

### Output

```
================================================================
GENESIS PREFLIGHT v0.1.0
================================================================

Scanned 2 files
Analyzing: data/measurements.csv
Analyzing: data/observations.csv

Validation complete: 3 issues found

Generating documentation in: /Users/researcher/experiment-2024
Created: README.md
Created: metadata.json
Created: DATACARD.md
Created: MANIFEST.txt
Created: data/measurements.schema.json
Created: data/observations.schema.json

================================================================
GENESIS PREFLIGHT REPORT
================================================================

Dataset: /Users/researcher/experiment-2024
Scanned: 2024-04-15 16:00:00 UTC

SUMMARY
-------
Files scanned: 2
Total size: 1.5 MB
File types: 2 CSV

COMPLIANCE SCORE: 60/100
-----------------------------------------------
Findable:       15/25
Accessible:     0/25
Interoperable:  20/25
Reusable:       15/25

ISSUES FOUND
------------
CRITICAL (1):
  [FAIR-A001] Missing LICENSE file
    -> Add a LICENSE file (MIT, CC0, CC-BY, etc.)

WARNING (2):
  [META-001] metadata.json contains [TODO] markers
    -> Complete all TODO sections in metadata.json
  [DATACARD-001] DATACARD.md contains [TODO] markers
    -> Complete all TODO sections in DATACARD.md

GENERATED FILES
---------------
Created: README.md
Created: metadata.json
Created: DATACARD.md
Created: MANIFEST.txt
Created: measurements.schema.json
Created: observations.schema.json

NEXT STEPS
----------
1. Review and complete all [TODO] sections in generated files
2. Address 1 critical issue before submission
3. Consider addressing 2 warnings to improve score

================================================================
```

### Dataset Structure (After)

```
experiment-2024/
├── README.md                    # Generated
├── metadata.json                # Generated (with [TODO] markers)
├── DATACARD.md                  # Generated (with [TODO] markers)
├── MANIFEST.txt                 # Generated
└── data/
    ├── measurements.csv
    ├── measurements.schema.json # Generated
    ├── observations.csv
    └── observations.schema.json # Generated
```

### Next Steps

1. **Add LICENSE file**:
   ```bash
   # Choose appropriate license
   curl -o LICENSE https://opensource.org/licenses/MIT
   ```

2. **Complete metadata.json**:
   Open `metadata.json` and replace all [TODO] markers:
   ```json
   {
     "name": "Experiment 2024 Results",
     "version": "1.0.0",
     "description": "Experimental measurements from January 2024 study",
     ...
   }
   ```

3. **Complete DATACARD.md**:
   Fill in provenance details, methodology, limitations

4. **Re-scan**:
   ```bash
   genesis-preflight scan ./experiment-2024
   ```

## Example 4: Interpreting the Report

### Understanding Scores

**Total Score (0-100)**:
- 90-100: Excellent - Ready for publication
- 80-89: Good - Meets minimum standards
- 50-79: Fair - Needs improvement
- 0-49: Poor - Significant issues

**FAIR Sub-Scores (0-25 each)**:
- Findable: Can others discover this data?
- Accessible: Can others obtain this data?
- Interoperable: Can others use this data with their tools?
- Reusable: Can others understand and trust this data?

### Issue Severities

**Critical** (-20 points each):
- Missing LICENSE file
- Missing metadata.json
- No README
- Must fix before publication

**Warning** (-5 points each):
- Missing schema definitions
- Poor naming conventions
- Missing provenance
- Should fix for better compliance

**Info** (-1 point each):
- Style issues
- Optimization suggestions
- Nice-to-have improvements

### Exit Codes

**Exit 0**: Success
- Score >= 80
- No critical issues
- CI/CD passes

**Exit 1**: Warning
- Score 50-79 or warnings present
- Usable but should improve
- CI/CD warning state

**Exit 2**: Failure
- Score < 50 or critical issues
- Not ready for publication
- CI/CD fails

## Example 5: Fixing Common Issues

### Issue: Missing LICENSE

**Error**:
```
[FAIR-A001] Missing LICENSE file
```

**Fix**:
Create a LICENSE file with appropriate terms:

```bash
# For open data (public domain)
cat > LICENSE << 'EOF'
Creative Commons Zero v1.0 Universal (CC0)

This dataset is dedicated to the public domain.
EOF
```

```bash
# For open data with attribution
cat > LICENSE << 'EOF'
Creative Commons Attribution 4.0 International (CC-BY-4.0)

You are free to share and adapt this data with attribution.
EOF
```

```bash
# For code/software components
cat > LICENSE << 'EOF'
MIT License

Copyright (c) 2024 Your Name

Permission is hereby granted, free of charge...
EOF
```

### Issue: Files with Spaces

**Error**:
```
[FAIR-F003] File name contains spaces: my data.csv
```

**Fix**:
Rename files to use underscores or hyphens:

```bash
# Using underscores
mv "my data.csv" my_data.csv
mv "Final Report.pdf" final_report.pdf

# Using hyphens
mv "my data.csv" my-data.csv
mv "Final Report.pdf" final-report.pdf
```

Batch rename:
```bash
# Replace all spaces with underscores
for file in *\ *; do
  mv "$file" "${file// /_}"
done
```

### Issue: Proprietary Formats

**Error**:
```
[FAIR-A004] Proprietary format detected: data.xlsx
```

**Fix**:
Convert to open formats:

```python
# Excel to CSV
import pandas as pd
df = pd.read_excel('data.xlsx')
df.to_csv('data.csv', index=False)
```

```bash
# Using LibreOffice (command line)
libreoffice --headless --convert-to csv data.xlsx
```

### Issue: Missing Metadata

**Error**:
```
[FAIR-F001] Missing metadata.json
```

**Fix**:
Generate template:

```bash
genesis-preflight generate ./dataset
```

Then edit `metadata.json`:
```json
{
  "name": "My Dataset Name",
  "version": "1.0.0",
  "description": "Brief description of what this data contains",
  "keywords": ["climate", "temperature", "measurements"],
  "license": "CC-BY-4.0",
  "authors": [
    {
      "name": "Dr. Jane Smith",
      "email": "jsmith@institution.edu",
      "affiliation": "University Name",
      "orcid": "0000-0002-1234-5678"
    }
  ],
  "created": "2024-01-15",
  "contact": {
    "name": "Jane Smith",
    "email": "jsmith@institution.edu"
  }
}
```

### Issue: TODO Markers

**Warning**:
```
[META-001] metadata.json contains [TODO] markers
```

**Fix**:
Search and replace all [TODO] sections:

```bash
# Find all TODOs
grep -r "\[TODO" .

# Edit each file and complete the sections
```

In metadata.json:
```json
// BEFORE:
"description": "[TODO: Describe this dataset]"

// AFTER:
"description": "Hourly temperature measurements from 15 weather stations in Pacific Northwest during Q1 2024"
```

In DATACARD.md:
```markdown
// BEFORE:
## Overview
[TODO: Provide a high-level overview...]

// AFTER:
## Overview
This dataset contains hourly temperature and humidity measurements collected from 15 automated weather stations across the Pacific Northwest region...
```

## Example 6: CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Dataset Validation

on:
  push:
    paths:
      - 'data/**'
  pull_request:

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Genesis Preflight
        run: cargo install genesis-preflight

      - name: Scan Dataset
        run: genesis-preflight scan ./data --json > report.json

      - name: Check Compliance Score
        run: |
          score=$(jq '.score.total' report.json)
          echo "Compliance score: $score"
          if [ "$score" -lt 80 ]; then
            echo "Error: Score below minimum threshold of 80"
            exit 1
          fi

      - name: Check Critical Issues
        run: |
          critical=$(jq '.score.critical_count' report.json)
          if [ "$critical" -gt 0 ]; then
            echo "Error: $critical critical issues found"
            jq '.validation_results[] | select(.severity == "Critical")' report.json
            exit 1
          fi

      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: compliance-report
          path: report.json
```

### GitLab CI

```yaml
dataset_validation:
  stage: test
  script:
    - cargo install genesis-preflight
    - genesis-preflight scan ./data --json > report.json
    - score=$(jq '.score.total' report.json)
    - echo "Compliance score:" $score
    - '[ "$score" -ge 80 ] || exit 1'
  artifacts:
    reports:
      junit: report.json
    paths:
      - report.json
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Validating dataset compliance..."
genesis-preflight scan ./data --quiet --json > /tmp/preflight-report.json

score=$(jq '.score.total' /tmp/preflight-report.json)
critical=$(jq '.score.critical_count' /tmp/preflight-report.json)

echo "Compliance score: $score/100"

if [ "$critical" -gt 0 ]; then
  echo "Error: Critical issues found. Fix before committing."
  jq '.validation_results[] | select(.severity == "Critical")' /tmp/preflight-report.json
  exit 1
fi

if [ "$score" -lt 80 ]; then
  echo "Warning: Score below 80. Consider improving compliance."
  echo "Proceed anyway? (y/n)"
  read -r response
  if [ "$response" != "y" ]; then
    exit 1
  fi
fi

echo "Validation passed!"
```

## Example 7: Quick Scan (No Hashing)

### Scenario

You have a large dataset (100GB+) and want a quick validation without calculating SHA-256 hashes.

### Command

```bash
genesis-preflight scan ./large-dataset --no-hash --quiet
```

### Output

Only compliance report is shown, file hashing is skipped. Scan completes in seconds instead of minutes.

```
COMPLIANCE SCORE: 85/100
-----------------------------------------------
Findable:       20/25
Accessible:     22/25
Interoperable:  22/25
Reusable:       21/25
```

Use `--no-hash` for:
- Initial quick checks
- Development/testing
- Large datasets where hashing is slow

Note: MANIFEST.txt cannot be generated without hashes.

## Example 8: JSON Output for Automation

### Command

```bash
genesis-preflight report ./dataset --json | jq '.score'
```

### Output

```json
{
  "total": 85,
  "findable": 20,
  "accessible": 22,
  "interoperable": 22,
  "reusable": 21,
  "critical_count": 0,
  "warning_count": 4,
  "info_count": 2
}
```

### Use Cases

**Extract specific data**:
```bash
# Get total score
genesis-preflight report ./dataset --json | jq '.score.total'

# List critical issues only
genesis-preflight report ./dataset --json | \
  jq '.validation_results[] | select(.severity == "Critical")'

# Count file types
genesis-preflight report ./dataset --json | \
  jq '.files'
```

**Monitoring dashboard**:
```python
import json
import subprocess

result = subprocess.run(
    ['genesis-preflight', 'report', './dataset', '--json'],
    capture_output=True,
    text=True
)

report = json.loads(result.stdout)

# Send to monitoring system
metrics = {
    'dataset.compliance.score': report['score']['total'],
    'dataset.compliance.critical': report['score']['critical_count'],
    'dataset.compliance.warnings': report['score']['warning_count'],
    'dataset.files.count': report['files']['count'],
    'dataset.files.size_mb': report['files']['total_size_bytes'] / 1024 / 1024
}

send_to_datadog(metrics)
```

## Best Practices

### 1. Run Early and Often

Don't wait until publication to validate:
```bash
# During data collection
genesis-preflight scan ./ongoing-experiment --no-hash

# Before sharing with collaborators
genesis-preflight scan ./dataset

# Before final publication
genesis-preflight report ./dataset --json
```

### 2. Use Version Control

Track compliance over time:
```bash
git add .
git commit -m "Initial data collection"

genesis-preflight scan . --json > compliance-history/$(date +%Y%m%d).json
git add compliance-history/
git commit -m "Compliance snapshot"
```

### 3. Establish Baselines

Set minimum thresholds for your organization:
```bash
# Institutional standard: Score >= 80, no critical issues
REQUIRED_SCORE=80

score=$(genesis-preflight report ./dataset --json | jq '.score.total')

if [ "$score" -lt "$REQUIRED_SCORE" ]; then
  echo "Does not meet institutional standard"
  exit 1
fi
```

### 4. Document Decisions

If you choose not to fix certain warnings, document why:
```markdown
# Compliance Notes

## FAIR-I001: Missing schema for binary files
Decision: Binary files are instrument-specific format with vendor documentation.
Schema generation not applicable.

## NAME-001: Uppercase in filenames
Decision: Filenames match instrument output exactly for traceability.
Renaming would break automated pipelines.
```

### 5. Automate Generation

Include documentation generation in your workflow:
```bash
#!/bin/bash
# scripts/finalize-dataset.sh

# Generate documentation
genesis-preflight generate ./dataset

# Validate
genesis-preflight scan ./dataset --json > validation-report.json

# Check score
score=$(jq '.score.total' validation-report.json)

if [ "$score" -lt 80 ]; then
  echo "Dataset not ready. Score: $score"
  exit 1
fi

echo "Dataset ready for publication. Score: $score"
```
