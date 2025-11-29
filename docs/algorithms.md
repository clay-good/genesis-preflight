# Algorithms and Implementation Details

## Overview

This document provides detailed explanations of the core algorithms implemented in Genesis Preflight. All algorithms use only the Rust standard library with no external dependencies.

## SHA-256 Hash Calculation

### Implementation

**Location**: [src/crypto/sha256.rs](../src/crypto/sha256.rs)

**Standard**: FIPS 180-4 (Federal Information Processing Standards Publication 180-4)

### Algorithm Description

SHA-256 (Secure Hash Algorithm 256-bit) produces a 256-bit (32-byte) hash value from arbitrary input data.

**Key Components**:

1. **Initial Hash Values** (H0-H7):
   ```
   H0 = 0x6a09e667
   H1 = 0xbb67ae85
   H2 = 0x3c6ef372
   H3 = 0xa54ff53a
   H4 = 0x510e527f
   H5 = 0x9b05688c
   H6 = 0x1f83d9ab
   H7 = 0x5be0cd19
   ```
   These are the first 32 bits of the fractional parts of the square roots of the first 8 primes.

2. **Round Constants** (K0-K63):
   64 constants derived from the first 32 bits of the fractional parts of the cube roots of the first 64 primes.

3. **Logical Functions**:
   - `Ch(x, y, z) = (x & y) ^ (!x & z)` - "Choose" function
   - `Maj(x, y, z) = (x & y) ^ (x & z) ^ (y & z)` - "Majority" function
   - `ROTR(x, n)` - Rotate right by n bits
   - `SHR(x, n)` - Shift right by n bits
   - `Σ0(x) = ROTR(x, 2) ^ ROTR(x, 13) ^ ROTR(x, 22)`
   - `Σ1(x) = ROTR(x, 6) ^ ROTR(x, 11) ^ ROTR(x, 25)`
   - `σ0(x) = ROTR(x, 7) ^ ROTR(x, 18) ^ SHR(x, 3)`
   - `σ1(x) = ROTR(x, 17) ^ ROTR(x, 19) ^ SHR(x, 10)`

### Processing Steps

1. **Padding**: Message is padded to be a multiple of 512 bits
   - Append bit '1' to message
   - Append zeros until length ≡ 448 (mod 512)
   - Append original message length as 64-bit big-endian integer

2. **Parsing**: Padded message is parsed into 512-bit blocks

3. **Compression**: For each 512-bit block:
   - Prepare message schedule (W0-W63):
     - W[0..15] = block data (16 x 32-bit words)
     - W[16..63] = σ1(W[i-2]) + W[i-7] + σ0(W[i-15]) + W[i-16]
   - Initialize working variables a-h from current hash
   - For rounds 0-63:
     - T1 = h + Σ1(e) + Ch(e, f, g) + K[i] + W[i]
     - T2 = Σ0(a) + Maj(a, b, c)
     - h = g, g = f, f = e, e = d + T1
     - d = c, c = b, b = a, a = T1 + T2
   - Add working variables to current hash

4. **Output**: Concatenate final hash values H0-H7 as 256-bit digest

### Test Vectors

**Input**: "" (empty string)
**Output**: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`

**Input**: "abc"
**Output**: `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`

**Input**: "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
**Output**: `248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1`

### Performance Characteristics

- **Speed**: Approximately 500 MB/s on modern hardware (limited by disk I/O)
- **Memory**: O(1) - constant memory usage regardless of file size
- **Chunk size**: Files processed in 8192-byte chunks
- **Large file handling**: Streaming implementation supports arbitrarily large files

### Code References

- Hash function: [src/crypto/sha256.rs:sha256()](../src/crypto/sha256.rs)
- Hex encoding: [src/crypto/sha256.rs:sha256_hex()](../src/crypto/sha256.rs)
- File hashing: [src/crypto/sha256.rs:sha256_file()](../src/crypto/sha256.rs)

## CSV Delimiter Detection

### Implementation

**Location**: [src/analyzer/csv.rs:detect_delimiter()](../src/analyzer/csv.rs)

### Algorithm Description

Automatically detects the delimiter character in CSV files by analyzing the first 10 rows.

**Candidates**: `,` (comma), `\t` (tab), `;` (semicolon), `|` (pipe)

**Steps**:

1. Read first 10 rows (or fewer if file is shorter)
2. For each candidate delimiter:
   - Count occurrences in each row
   - Check if count is consistent across rows
   - Calculate consistency score
3. Select delimiter with highest consistency score
4. Fall back to comma if no clear winner

### Consistency Scoring

```
For each delimiter:
  counts = [count in row 0, count in row 1, ..., count in row 9]
  if all counts are equal and > 0:
    score = count (higher is better)
  else:
    score = 0 (inconsistent, reject)
```

### Examples

**Example 1: Comma-delimited**
```
name,age,city
Alice,30,NYC
Bob,25,LA
```
- Comma: [2, 2, 2] → score = 2
- Tab: [0, 0, 0] → score = 0
- Result: Comma selected

**Example 2: Tab-delimited**
```
name    age    city
Alice   30     NYC
Bob     25     LA
```
- Comma: [0, 0, 0] → score = 0
- Tab: [2, 2, 2] → score = 2
- Result: Tab selected

**Example 3: Ambiguous**
```
data
1,2
3,4|5
```
- Comma: [1, 0] → inconsistent, score = 0
- Pipe: [0, 1] → inconsistent, score = 0
- Result: Comma (default fallback)

### Edge Cases

- **Single column**: Returns comma (no delimiter needed)
- **Inconsistent rows**: Returns comma (default)
- **Empty file**: Returns comma (default)
- **Mixed delimiters**: Returns most consistent one, or comma if tied

### Code Reference

[src/analyzer/csv.rs:detect_delimiter()](../src/analyzer/csv.rs)

## Header Row Detection

### Implementation

**Location**: [src/analyzer/csv.rs:detect_header()](../src/analyzer/csv.rs)

### Algorithm Description

Determines whether the first row is a header or data row.

**Strategy**: Compare first row to subsequent rows using heuristics

**Heuristics**:

1. **All text in first row**: If first row contains no numbers but subsequent rows do, likely header
2. **Different pattern**: If first row has different delimiter count or format than others, likely header
3. **Common header words**: If first row contains words like "id", "name", "date", "value", likely header
4. **Type consistency**: If types in first row differ from dominant types in rest of file, likely header

**Default**: Assume header present (common case)

### Examples

**Example 1: Clear header**
```
name,age,city
Alice,30,NYC
Bob,25,LA
```
- First row: all text, no numbers
- Other rows: contain numbers
- Result: Header detected

**Example 2: No header**
```
Alice,30,NYC
Bob,25,LA
Carol,28,SF
```
- All rows have same pattern
- First row not distinguishable
- Result: No header (or uncertain)

**Example 3: Numeric data**
```
x,y,z
1.5,2.3,3.7
2.1,4.2,5.3
```
- First row: simple text tokens
- Other rows: all floats
- Result: Header detected

### Code Reference

[src/analyzer/csv.rs:detect_header()](../src/analyzer/csv.rs)

## Column Type Inference

### Implementation

**Location**: [src/analyzer/inference.rs](../src/analyzer/inference.rs)

### Algorithm Description

Infers the data type of each column by examining sample values and applying pattern matching.

**Type Categories**:
- Integer
- Float
- Boolean
- Timestamp
- Date
- String (fallback)

### Pattern Matching Rules

**Integer**:
```
Pattern: ^-?\d+$
Examples: "123", "-456", "0"
```

**Float**:
```
Pattern: ^-?\d+\.\d+$
Examples: "3.14", "-0.5", "123.456"
Scientific: "1.5e-10", "2.3E+5"
```

**Boolean**:
```
Values: "true", "false", "True", "False", "TRUE", "FALSE"
        "yes", "no", "Y", "N", "1", "0" (in boolean context)
```

**Timestamp** (ISO 8601 with time):
```
Pattern: YYYY-MM-DD[T ]HH:MM:SS
Examples: "2024-01-15T14:30:00"
          "2024-01-15 14:30:00"
          "2024-01-15T14:30:00Z"
          "2024-01-15T14:30:00-05:00"
```

**Date** (ISO 8601 date only):
```
Pattern: YYYY-MM-DD
Examples: "2024-01-15"
          "2024/01/15"
```

**String**:
```
Default: Any value that doesn't match above patterns
```

### Semantic Inference

Column names provide hints for type inference:

**Temperature indicators**:
- Names: "temp", "temperature", "degrees", "celsius", "fahrenheit", "kelvin"
- Inferred type: Float (even if integers present)
- Suggested unit: Check name for unit hint

**ID indicators**:
- Names: "id", "identifier", "uuid", "key"
- Inferred type: String (preserve leading zeros)

**Timestamp indicators**:
- Names: "time", "timestamp", "datetime", "created_at", "updated_at"
- Inferred type: Timestamp (parse accordingly)

**Coordinate indicators**:
- Names: "lat", "latitude", "lon", "longitude", "x", "y", "z"
- Inferred type: Float (high precision required)

### Inference Process

1. **Sample collection**: Examine first 100 rows (configurable)
2. **Pattern testing**: Try each pattern in order of specificity
3. **Majority voting**: Type that matches most samples wins
4. **Semantic override**: Column name can override pattern-based inference
5. **Confidence scoring**: Track percentage of matching samples

### Examples

**Example 1: Mixed integer/float column**
```
Column: "temperature"
Values: ["20", "21.5", "19", "22.3", "20"]
Pattern analysis: 60% float, 40% integer
Semantic hint: "temperature" suggests float
Result: Float with high confidence
```

**Example 2: ID column**
```
Column: "user_id"
Values: ["001", "002", "003", "004"]
Pattern analysis: 100% integer
Semantic hint: "id" suggests preserve as string
Result: String (to preserve leading zeros)
```

**Example 3: Timestamp column**
```
Column: "created_at"
Values: ["2024-01-15 14:30:00", "2024-01-15 15:45:00"]
Pattern analysis: 100% timestamp
Semantic hint: "created_at" confirms timestamp
Result: Timestamp with high confidence
```

### Code References

- Type inference: [src/analyzer/inference.rs:infer_column_type()](../src/analyzer/inference.rs)
- Pattern matching: [src/analyzer/inference.rs:matches_pattern()](../src/analyzer/inference.rs)
- Semantic hints: [src/analyzer/inference.rs:semantic_type_hint()](../src/analyzer/inference.rs)

## Compliance Score Calculation

### Implementation

**Location**: [src/reporter/score.rs:calculate_score()](../src/reporter/score.rs)

### Algorithm Description

Calculates a compliance score (0-100) based on validation issues found.

### Scoring Formula

**Total Score**:
```
Start: 100 points
Deductions:
  - Critical issue: -20 points each
  - Warning issue: -5 points each
  - Info issue: -1 point each
Floor: 0 (cannot go negative)
```

**Formula**:
```
score = max(0, 100 - (critical_count × 20) - (warning_count × 5) - (info_count × 1))
```

### FAIR Sub-Scores

Each FAIR dimension (Findable, Accessible, Interoperable, Reusable) has a sub-score (0-25).

**Sub-Score Formula**:
```
Start: 25 points per dimension
Deductions (for issues matching dimension prefix):
  - Critical issue: -10 points
  - Warning issue: -3 points
  - Info issue: -1 point
Floor: 0 per dimension
```

**Example**:
```
Issues:
  - FAIR-F001 (Critical): Missing metadata.json
  - FAIR-F003 (Warning): Poor file naming
  - FAIR-A001 (Critical): Missing LICENSE

Findable sub-score:
  25 - (1 × 10) - (1 × 3) = 12

Accessible sub-score:
  25 - (1 × 10) = 15

Interoperable sub-score:
  25 (no issues)

Reusable sub-score:
  25 (no issues)

Total score:
  100 - (2 × 20) - (1 × 5) = 55
```

### Interpretation Thresholds

**Excellent (90-100)**:
- No critical issues
- 0-2 warnings
- Ready for publication

**Good (80-89)**:
- No critical issues
- 3-4 warnings
- Meets minimum standards

**Fair (50-79)**:
- Possible critical issues
- Multiple warnings
- Requires improvement

**Poor (0-49)**:
- Multiple critical issues
- Many warnings
- Not ready for publication

### Exit Code Mapping

Based on score and issue severity:

```
if critical_count > 0 || total < 50:
    exit_code = 2  # Failure
elif total < 80 || warning_count > 0:
    exit_code = 1  # Warning
else:
    exit_code = 0  # Success
```

### Examples

**Example 1: Perfect dataset**
```
Issues: None
Score: 100/100
  - Findable: 25/25
  - Accessible: 25/25
  - Interoperable: 25/25
  - Reusable: 25/25
Exit code: 0
```

**Example 2: Missing LICENSE**
```
Issues: FAIR-A001 (Critical), FAIR-R001 (Critical)
Score: 60/100
  - Findable: 25/25
  - Accessible: 15/25
  - Interoperable: 25/25
  - Reusable: 15/25
Exit code: 2
```

**Example 3: Minor warnings**
```
Issues: FAIR-F003 (Warning), FAIR-I001 (Warning)
Score: 90/100
  - Findable: 22/25
  - Accessible: 25/25
  - Interoperable: 22/25
  - Reusable: 25/25
Exit code: 1
```

### Code References

- Score calculation: [src/reporter/score.rs:calculate_score()](../src/reporter/score.rs)
- Sub-score calculation: [src/reporter/score.rs:calculate_fair_subscore()](../src/reporter/score.rs)
- Exit code logic: [src/reporter/mod.rs:Report::exit_code()](../src/reporter/mod.rs)

## JSON Parsing

### Implementation

**Location**: [src/analyzer/json.rs](../src/analyzer/json.rs)

### Algorithm Description

Recursive descent parser for JSON syntax validation.

**Grammar**:
```
value   → object | array | string | number | true | false | null
object  → '{' (pair (',' pair)*)? '}'
pair    → string ':' value
array   → '[' (value (',' value)*)? ']'
string  → '"' char* '"'
number  → '-'? digit+ ('.' digit+)? (('e'|'E') ('+'|'-')? digit+)?
```

**Features**:
- Validates JSON syntax
- Extracts top-level keys (for objects)
- Counts nesting depth
- Detects arrays vs objects
- Handles escaped characters in strings

**Limitations** (intentional for simplicity):
- Does not build full AST
- Does not preserve value types beyond validation
- Limited to structural analysis only

### Code Reference

[src/analyzer/json.rs](../src/analyzer/json.rs)

## Performance Optimizations

### File Reading Strategy

**Chunked Reading**:
- Read files in 8192-byte chunks
- Avoids loading entire file into memory
- Enables streaming hash calculation

**Sampling**:
- CSV analysis: First 100 rows only
- Text analysis: First 1 MB only
- Binary detection: First 512 bytes only

### Memory Management

**Stack Usage**:
- Avoid recursive algorithms with unbounded depth
- JSON parser limits nesting to prevent stack overflow

**Heap Usage**:
- Reuse buffers where possible
- Clear vectors after processing
- Avoid unnecessary string allocations

### Concurrency

Current implementation is single-threaded for simplicity. Future versions may parallelize:
- File hashing (process multiple files concurrently)
- Analysis phase (independent file analyses)
- Validation checks (independent validators)

## Error Handling

### Strategy

**Graceful Degradation**:
- File read errors: Skip file, report error
- Parse errors: Mark as unparseable, continue
- Invalid UTF-8: Treat as binary

**Never Panic**:
- All file operations wrapped in Result
- All parsing wrapped in Result
- User input validated before processing

### Code Pattern

```rust
match std::fs::read_to_string(path) {
    Ok(content) => analyze(content),
    Err(e) => {
        eprintln!("Error reading {}: {}", path, e);
        Analysis::error(path, e.to_string())
    }
}
```

## Testing

All algorithms include comprehensive unit tests:

- **SHA-256**: FIPS 180-4 test vectors
- **CSV parsing**: Various delimiters and formats
- **Type inference**: Boundary cases and edge cases
- **Scoring**: All score combinations
- **JSON parsing**: Valid and invalid inputs

Test coverage target: >90% for all algorithm modules.

### Running Tests

```bash
cargo test                    # Run all tests
cargo test --lib crypto       # Test crypto module only
cargo test inference          # Test type inference
```
