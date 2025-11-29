# FAIR Principles Implementation

## Overview

Genesis Preflight validates scientific datasets against the FAIR principles: Findable, Accessible, Interoperable, and Reusable. This document explains each principle, how it maps to validation checks, and provides examples.

## FAIR Principles Explained

### F - Findable

Scientific data should be easy to find for both humans and computers. Metadata and data should be indexed in a searchable resource.

**Requirements:**
- Dataset has a persistent identifier
- Data is described with rich metadata
- Metadata includes the identifier of the data
- Metadata is registered or indexed in a searchable resource

**Genesis Preflight Checks:**
- Presence of metadata.json with required fields (FAIR-F001)
- Presence of README.md for human discovery (FAIR-F002)
- Presence of descriptive filenames (FAIR-F003)
- Presence of keywords and subjects in metadata (FAIR-F004)

### A - Accessible

Once data is found, the user needs to know how to access it, including authentication and authorization procedures.

**Requirements:**
- Data is retrievable by its identifier using a standard protocol
- The protocol is open, free, and universally implementable
- Metadata remains accessible even when data is no longer available
- Authentication and authorization procedures are specified if needed

**Genesis Preflight Checks:**
- Presence of LICENSE file specifying access terms (FAIR-A001)
- No authentication required for open data (FAIR-A002)
- Metadata.json includes access information (FAIR-A003)
- Data files are in standard, non-proprietary formats (FAIR-A004)

### I - Interoperable

Data needs to work with applications or workflows for analysis, storage, and processing.

**Requirements:**
- Data uses a formal, accessible, shared knowledge representation language
- Data uses vocabularies that follow FAIR principles
- Data includes qualified references to other data
- Data uses community-accepted formats

**Genesis Preflight Checks:**
- Presence of schema.json describing data structure (FAIR-I001)
- Use of standard file formats (CSV, JSON, HDF5, NetCDF) (FAIR-I002)
- Column names follow conventions (lowercase, underscores) (FAIR-I003)
- Metadata includes links to controlled vocabularies (FAIR-I004)

### R - Reusable

Data should be well-described so it can be replicated and/or combined in different settings.

**Requirements:**
- Data has clear usage licenses
- Data is associated with detailed provenance
- Data meets domain-relevant community standards
- Data includes rich descriptive metadata

**Genesis Preflight Checks:**
- Presence of LICENSE file with clear terms (FAIR-R001)
- Presence of DATACARD.md or equivalent provenance (FAIR-R002)
- Metadata includes data collection methods (FAIR-R003)
- Metadata includes versioning information (FAIR-R004)

## Validation Check Mapping

### Check Codes by Principle

#### Findable (FAIR-F)
- **FAIR-F001**: Missing metadata.json
  - Severity: Critical
  - Fix: Run `genesis-preflight generate` to create metadata.json
- **FAIR-F002**: Missing README.md
  - Severity: Critical
  - Fix: Create README.md describing your dataset
- **FAIR-F003**: Poor file naming conventions
  - Severity: Warning
  - Fix: Rename files to lowercase with underscores
- **FAIR-F004**: Missing keywords in metadata
  - Severity: Warning
  - Fix: Add keywords array to metadata.json

#### Accessible (FAIR-A)
- **FAIR-A001**: Missing LICENSE file
  - Severity: Critical
  - Fix: Add a LICENSE file (MIT, CC0, CC-BY, etc.)
- **FAIR-A002**: Unclear access restrictions
  - Severity: Warning
  - Fix: Document access requirements in README.md
- **FAIR-A003**: Missing access information in metadata
  - Severity: Warning
  - Fix: Add "access" field to metadata.json
- **FAIR-A004**: Proprietary file formats detected
  - Severity: Warning
  - Fix: Convert to open formats (CSV, JSON, HDF5)

#### Interoperable (FAIR-I)
- **FAIR-I001**: Missing schema definition
  - Severity: Warning
  - Fix: Run `genesis-preflight generate` to create schema.json
- **FAIR-I002**: Non-standard file formats
  - Severity: Info
  - Fix: Consider converting to CSV or JSON
- **FAIR-I003**: Inconsistent naming conventions
  - Severity: Warning
  - Fix: Standardize column names (lowercase, underscores)
- **FAIR-I004**: Missing vocabulary references
  - Severity: Info
  - Fix: Link to controlled vocabularies in metadata

#### Reusable (FAIR-R)
- **FAIR-R001**: Missing LICENSE file
  - Severity: Critical
  - Fix: Add clear usage license
- **FAIR-R002**: Missing provenance documentation
  - Severity: Warning
  - Fix: Create DATACARD.md with collection details
- **FAIR-R003**: Missing methodology description
  - Severity: Warning
  - Fix: Document data collection methods
- **FAIR-R004**: Missing version information
  - Severity: Info
  - Fix: Add version field to metadata.json

## Compliance Examples

### Example 1: Fully Compliant Dataset

```
climate-data-2024/
├── README.md                    # Human-readable description
├── LICENSE                      # MIT License
├── metadata.json                # Machine-readable metadata
├── DATACARD.md                  # Provenance documentation
├── MANIFEST.txt                 # SHA-256 checksums
├── data/
│   ├── temperature.csv          # Lowercase, descriptive
│   ├── temperature.schema.json  # Schema definition
│   ├── humidity.csv
│   └── humidity.schema.json
└── docs/
    └── methodology.md           # Data collection methods
```

**FAIR Score: 100/100**
- Findable: 25/25 (metadata, README, good naming)
- Accessible: 25/25 (LICENSE, open formats)
- Interoperable: 25/25 (schemas, standard CSV)
- Reusable: 25/25 (LICENSE, provenance, methodology)

### Example 2: Non-Compliant Dataset

```
My Data/
├── data1.xlsx                   # Proprietary format
├── Data 2 Final FINAL.csv       # Poor naming
├── output (copy).txt            # Very poor naming
└── readme.txt                   # Lowercase filename
```

**FAIR Score: 35/100**
- Findable: 10/25 (no metadata.json, poor naming)
- Accessible: 0/25 (no LICENSE, proprietary formats)
- Interoperable: 10/25 (no schemas, inconsistent naming)
- Reusable: 15/25 (has readme but no LICENSE or provenance)

**Critical Issues:**
- Missing LICENSE file (FAIR-A001, FAIR-R001)
- Missing metadata.json (FAIR-F001)
- Proprietary file formats (FAIR-A004)

**Warnings:**
- Poor file naming conventions (FAIR-F003)
- Missing schema definitions (FAIR-I001)
- Missing provenance documentation (FAIR-R002)

### Example 3: Partially Compliant Dataset

```
experiment-results/
├── README.md                    # Present
├── LICENSE                      # MIT License
├── data/
│   ├── measurements.csv         # Good naming
│   └── observations.csv         # Good naming
└── notes.txt                    # Informal documentation
```

**FAIR Score: 65/100**
- Findable: 18/25 (has README, good naming, missing metadata.json)
- Accessible: 20/25 (has LICENSE, open formats)
- Interoperable: 12/25 (no schemas, standard CSV)
- Reusable: 15/25 (has LICENSE, missing provenance)

**Critical Issues:**
- Missing metadata.json (FAIR-F001)

**Warnings:**
- Missing schema definitions (FAIR-I001)
- Missing provenance documentation (FAIR-R002)
- Missing methodology description (FAIR-R003)

**Fix Path:**
1. Run `genesis-preflight generate ./experiment-results`
2. Complete TODO sections in generated metadata.json
3. Complete TODO sections in generated DATACARD.md
4. Re-scan with `genesis-preflight scan ./experiment-results`

## Scoring Algorithm

### Total Score Calculation

Starting from 100 points:
- Critical issue: -20 points each
- Warning issue: -5 points each
- Info issue: -1 point each
- Minimum score: 0

### FAIR Sub-Scores

Each dimension (F, A, I, R) starts at 25 points:
- Critical issue: -10 points
- Warning issue: -3 points
- Info issue: -1 point
- Minimum sub-score: 0

Only issues with codes matching the dimension prefix are counted. For example, FAIR-F001 only affects the Findable sub-score.

### Exit Code Determination

Based on total score and critical issues:
- Exit 0: Score >= 80 and no critical issues
- Exit 1: Score >= 50 or warnings present
- Exit 2: Score < 50 or critical issues present

## Official FAIR Resources

### Primary References

1. **Original FAIR Paper**
   - Wilkinson, M. D. et al. (2016)
   - "The FAIR Guiding Principles for scientific data management and stewardship"
   - Scientific Data 3, 160018
   - https://doi.org/10.1038/sdata.2016.18

2. **GO FAIR Initiative**
   - https://www.go-fair.org/
   - Community-driven FAIR implementation resources

3. **FAIR Metrics**
   - https://fairmetrics.org/
   - Quantitative metrics for FAIR assessment

4. **RDA FAIR Data Maturity Model**
   - Research Data Alliance working group
   - https://www.rd-alliance.org/

### Domain-Specific Guidelines

1. **DOE Data Management**
   - Department of Energy data sharing guidelines
   - https://www.energy.gov/datamanagement

2. **NSF Public Access Plan**
   - National Science Foundation data requirements
   - https://www.nsf.gov/data

3. **NIH Data Sharing**
   - National Institutes of Health policies
   - https://sharing.nih.gov/

## Implementation Notes

### Why These Specific Checks?

Genesis Preflight focuses on checks that can be automatically validated without domain expertise:
- File presence checks (README, LICENSE, metadata.json)
- File format validation (open vs proprietary)
- Naming convention compliance
- Schema availability
- Structural completeness

### What Genesis Preflight Does Not Check

Some FAIR aspects require human judgment:
- Quality of metadata content (only presence)
- Appropriateness of controlled vocabularies
- Compliance with domain-specific standards
- Semantic correctness of data
- Actual persistent identifier registration

These must be verified manually by the researcher or data steward.

### Customization

Future versions may support:
- Custom validation rules via configuration
- Domain-specific vocabulary checks
- Integration with metadata registries
- Persistent identifier validation

## Best Practices

### For Researchers

1. **Start with generated templates**: Run `genesis-preflight generate` to create required files
2. **Complete all TODO sections**: Don't skip metadata fields
3. **Use standard formats**: CSV and JSON are universally accessible
4. **Document everything**: Include methodology, provenance, limitations
5. **Version your data**: Use semantic versioning (1.0.0, 1.1.0, etc.)
6. **Choose appropriate licenses**: MIT, CC0, CC-BY-4.0 for open data

### For Data Stewards

1. **Establish conventions early**: Define naming and format standards
2. **Validate before submission**: Run genesis-preflight in CI/CD
3. **Aim for 80+ score**: This indicates minimum compliance
4. **Address critical issues first**: LICENSE and metadata.json are mandatory
5. **Document domain standards**: Link to discipline-specific requirements
6. **Provide examples**: Share compliant dataset templates

### For Institutions

1. **Integrate into workflows**: Make validation part of submission process
2. **Set minimum requirements**: Score >= 80 for acceptance
3. **Provide training**: Help researchers understand FAIR principles
4. **Offer templates**: Standardized metadata.json and DATACARD.md
5. **Monitor compliance**: Track FAIR scores across repository
6. **Support researchers**: Provide data management consulting
