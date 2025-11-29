# Development Guide

## Overview

This document provides detailed guidance for developers working on genesis-preflight.

## Prerequisites

- Rust 1.70 or later (stable channel)
- Git
- Text editor or IDE with Rust support

## Quick Start

```bash
# Clone repository
git clone https://github.com/clay-good/genesis-preflight.git
cd genesis-preflight

# Build and test
cargo build
cargo test

# Run locally
cargo run -- --help
cargo run -- scan ./test-data
```

## Project Structure

See [architecture.md](architecture.md) for complete module documentation.

```
genesis-preflight/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library root
│   ├── types/               # Shared type definitions
│   ├── crypto/              # SHA-256 implementation
│   ├── scanner/             # Directory traversal
│   ├── analyzer/            # File content analysis
│   ├── validator/           # FAIR compliance checks
│   ├── generator/           # Documentation generation
│   └── reporter/            # Report generation
├── tests/
│   ├── fixtures/            # Test datasets
│   └── integration/         # Integration tests
├── docs/                    # Documentation
└── scripts/                 # Build and test scripts
```

## Development Workflow

### 1. Create Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Changes

Follow code standards in [CONTRIBUTING.md](../CONTRIBUTING.md).

### 3. Run Tests

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run all tests
cargo test

# Or use comprehensive test script
./scripts/test-all.sh
```

### 4. Commit Changes

```bash
git add .
git commit -m "Brief description of change

More detailed explanation if needed."
```

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
# Then create pull request on GitHub
```

## Testing

### Unit Tests

Unit tests live in the same file as the code they test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        let result = function_under_test();
        assert_eq!(result, expected_value);
    }
}
```

Run unit tests:
```bash
cargo test --lib
```

### Integration Tests

Integration tests live in `tests/integration/`:

```rust
// tests/integration/my_test.rs
use genesis_preflight::scanner::scan_directory;

#[test]
fn test_integration_scenario() {
    // Test end-to-end functionality
}
```

Run integration tests:
```bash
cargo test --test '*'
```

### Test Fixtures

Test fixtures are in `tests/fixtures/`:
- `valid_dataset/` - Fully compliant dataset
- `invalid_dataset/` - Multiple compliance issues
- `partial_dataset/` - Partially compliant

Add new fixtures as needed for edge cases.

## Code Standards

### Zero Dependencies

This project uses only the Rust standard library. Do not add external dependencies.

### No Unsafe Code

Unsafe code is forbidden via `#![forbid(unsafe_code)]`. This is enforced by the compiler.

### Error Handling

```rust
// Use Result for fallible operations
pub fn parse_file(path: &Path) -> Result<Data, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Process content
    Ok(data)
}

// Never panic on user input
// Use ? operator for error propagation
```

### Documentation

All public items must have doc comments:

```rust
/// Calculates SHA-256 hash of file contents
///
/// # Arguments
///
/// * `path` - Path to file to hash
///
/// # Returns
///
/// SHA-256 hash as 64-character hex string
///
/// # Errors
///
/// Returns error if file cannot be read
pub fn sha256_file(path: &Path) -> Result<String, std::io::Error> {
    // Implementation
}
```

## Common Tasks

### Add New Validation Check

1. Add check function to appropriate validator module
2. Add test case in module's tests
3. Document error code in `docs/fair-principles.md`
4. Update scoring if critical/warning severity

Example:
```rust
// In src/validator/structure.rs
pub fn check_new_requirement(files: &[FileInfo]) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Check logic
    if !requirement_met {
        results.push(ValidationResult::critical(
            "REQ-001",
            "Requirement not met",
            "Do this to fix"
        ));
    }

    results
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_check_new_requirement() {
        // Test the check
    }
}
```

### Add New File Format Support

1. Add file type to `types/file_type.rs`
2. Create analyzer in `analyzer/`
3. Add analysis struct to `types/analysis.rs`
4. Update file type detection
5. Add tests
6. Update documentation

### Debug Issues

```rust
// Use eprintln! for debug output (goes to stderr)
eprintln!("Debug: value = {:?}", value);

// Run with verbose flag
cargo run -- scan ./dataset --verbose

// Use rust-gdb/lldb for debugging
cargo build
rust-gdb target/debug/genesis-preflight
```

## Performance Considerations

### File I/O

- Read files in chunks (8KB default)
- Avoid loading entire files into memory
- Use streaming for large files

```rust
use std::io::{BufReader, Read};

let file = File::open(path)?;
let mut reader = BufReader::new(file);
let mut buffer = [0; 8192];

loop {
    let n = reader.read(&mut buffer)?;
    if n == 0 { break; }
    // Process chunk
}
```

### Memory Usage

- Prefer iterators over collecting into vectors
- Clear vectors after use if reusing
- Use references where possible

```rust
// Good - streaming
files.iter()
    .filter(|f| f.size > 1000)
    .map(|f| process(f))
    .collect()

// Avoid - unnecessary collection
let filtered: Vec<_> = files.iter().filter(...).collect();
let mapped: Vec<_> = filtered.iter().map(...).collect();
```

### Profiling

```bash
# Build with profiling symbols
cargo build --release

# Use system profilers
# Linux: perf
perf record ./target/release/genesis-preflight scan large-dataset
perf report

# macOS: Instruments
# Run Instruments.app and attach to process
```

## Release Process

See [RELEASE_CHECKLIST.md](../RELEASE_CHECKLIST.md) for complete checklist.

### Version Numbers

Follow semantic versioning:
- MAJOR.MINOR.PATCH
- 0.1.0 → 0.1.1 (bug fix)
- 0.1.1 → 0.2.0 (new feature)
- 0.9.0 → 1.0.0 (stable API)

### Steps

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Update VERSION constant in `src/main.rs`
4. Run full test suite
5. Build release binary
6. Create git tag
7. Publish to crates.io

```bash
# Full test suite
./scripts/test-all.sh

# Build release
./scripts/build-release.sh

# Tag release
git tag -a v0.1.0 -m "Release 0.1.0"
git push origin v0.1.0

# Publish
cargo publish --dry-run
cargo publish
```

## Troubleshooting

### Tests Failing

```bash
# Run specific test
cargo test test_name -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test

# Run single-threaded
cargo test -- --test-threads=1
```

### Build Issues

```bash
# Clean build
cargo clean
cargo build

# Update toolchain
rustup update stable
```

### Clippy Warnings

```bash
# See all lints
cargo clippy -- -W clippy::all

# Fix automatically (when possible)
cargo clippy --fix
```

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)

## Getting Help

- Open an issue on GitHub
- Check existing issues for similar problems
- Review documentation in `docs/`
- Ask in pull request discussions
