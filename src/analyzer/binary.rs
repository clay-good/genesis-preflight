//! Binary file detection and type identification

use crate::types::BinaryType;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Size of sample to read for binary detection
const SAMPLE_SIZE: usize = 8192;

/// Threshold for non-printable character ratio to classify as binary
const BINARY_THRESHOLD: f32 = 0.3;

/// Check if a file is binary
///
/// Reads the first 8KB of the file and checks for null bytes and
/// high concentration of non-printable characters.
///
/// # Arguments
///
/// * `path` - Path to the file
///
/// # Returns
///
/// True if the file appears to be binary, false otherwise.
pub fn is_binary(path: &Path) -> io::Result<bool> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; SAMPLE_SIZE];
    let bytes_read = file.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(false); // Empty file is not binary
    }

    let sample = &buffer[..bytes_read];

    // Check for null bytes (strong indicator of binary)
    if sample.contains(&0) {
        return Ok(true);
    }

    // Count non-printable characters
    let non_printable_count = sample
        .iter()
        .filter(|&&b| !is_printable(b))
        .count();

    let ratio = non_printable_count as f32 / bytes_read as f32;

    Ok(ratio > BINARY_THRESHOLD)
}

/// Detect the type of binary file using magic numbers
///
/// Examines the file header to identify common scientific data formats
/// and other binary file types.
///
/// # Arguments
///
/// * `path` - Path to the file
///
/// # Returns
///
/// The detected binary type, or BinaryType::Unknown if not recognized.
pub fn detect_binary_type(path: &Path) -> Option<BinaryType> {
    let mut file = File::open(path).ok()?;
    let mut header = [0u8; 16]; // Read first 16 bytes for magic number detection
    let bytes_read = file.read(&mut header).ok()?;

    if bytes_read < 4 {
        return Some(BinaryType::Unknown);
    }

    // HDF5: 89 48 44 46 0D 0A 1A 0A (137, 72, 68, 70, 13, 10, 26, 10)
    if bytes_read >= 8
        && header[0] == 137
        && header[1] == 72
        && header[2] == 68
        && header[3] == 70
        && header[4] == 13
        && header[5] == 10
        && header[6] == 26
        && header[7] == 10
    {
        return Some(BinaryType::Hdf5);
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A (137, 80, 78, 71, 13, 10, 26, 10)
    if bytes_read >= 8
        && header[0] == 137
        && header[1] == 80
        && header[2] == 78
        && header[3] == 71
        && header[4] == 13
        && header[5] == 10
        && header[6] == 26
        && header[7] == 10
    {
        return Some(BinaryType::Png);
    }

    // JPEG: FF D8 FF
    if bytes_read >= 3 && header[0] == 0xFF && header[1] == 0xD8 && header[2] == 0xFF {
        return Some(BinaryType::Jpeg);
    }

    // PDF: %PDF-
    if bytes_read >= 5
        && header[0] == b'%'
        && header[1] == b'P'
        && header[2] == b'D'
        && header[3] == b'F'
        && header[4] == b'-'
    {
        return Some(BinaryType::Pdf);
    }

    // NetCDF: CDF\x01 or CDF\x02
    if bytes_read >= 4
        && header[0] == b'C'
        && header[1] == b'D'
        && header[2] == b'F'
        && (header[3] == 0x01 || header[3] == 0x02)
    {
        return Some(BinaryType::Netcdf);
    }

    // If it's binary but we don't recognize the format
    if is_binary(path).unwrap_or(false) {
        Some(BinaryType::Unknown)
    } else {
        None
    }
}

/// Check if a byte represents a printable character
fn is_printable(b: u8) -> bool {
    // Printable ASCII (32-126) plus common whitespace (9, 10, 13)
    matches!(b, 9 | 10 | 13 | 32..=126)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_is_printable() {
        assert!(is_printable(b'A'));
        assert!(is_printable(b'z'));
        assert!(is_printable(b'0'));
        assert!(is_printable(b' '));
        assert!(is_printable(b'\n'));
        assert!(is_printable(b'\t'));
        assert!(!is_printable(0));
        assert!(!is_printable(255));
    }

    #[test]
    fn test_is_binary_text_file() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_binary_text");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("text.txt");
        std::fs::write(&file_path, "This is plain text\n").unwrap();

        let result = is_binary(&file_path).unwrap();
        assert!(!result);

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_is_binary_with_null_bytes() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_binary_null");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("binary.dat");
        {
            let mut file = File::create(&file_path).unwrap();
            file.write_all(&[0, 1, 2, 3, 4, 5]).unwrap();
        }

        let result = is_binary(&file_path).unwrap();
        assert!(result);

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_detect_png() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_binary_png");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.png");
        {
            let mut file = File::create(&file_path).unwrap();
            // PNG magic number
            file.write_all(&[137, 80, 78, 71, 13, 10, 26, 10]).unwrap();
        }

        let result = detect_binary_type(&file_path);
        assert_eq!(result, Some(BinaryType::Png));

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_detect_pdf() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_binary_pdf");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.pdf");
        {
            let mut file = File::create(&file_path).unwrap();
            // PDF magic number
            file.write_all(b"%PDF-1.4").unwrap();
        }

        let result = detect_binary_type(&file_path);
        assert_eq!(result, Some(BinaryType::Pdf));

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_detect_hdf5() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_binary_hdf5");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.h5");
        {
            let mut file = File::create(&file_path).unwrap();
            // HDF5 magic number
            file.write_all(&[137, 72, 68, 70, 13, 10, 26, 10]).unwrap();
        }

        let result = detect_binary_type(&file_path);
        assert_eq!(result, Some(BinaryType::Hdf5));

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_detect_jpeg() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_binary_jpeg");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.jpg");
        {
            let mut file = File::create(&file_path).unwrap();
            // JPEG magic number - need more bytes to pass binary check
            file.write_all(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00]).unwrap();
        }

        let result = detect_binary_type(&file_path);
        assert_eq!(result, Some(BinaryType::Jpeg));

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_detect_netcdf() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_binary_netcdf");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.nc");
        {
            let mut file = File::create(&file_path).unwrap();
            // NetCDF magic number
            file.write_all(&[b'C', b'D', b'F', 0x01]).unwrap();
        }

        let result = detect_binary_type(&file_path);
        assert_eq!(result, Some(BinaryType::Netcdf));

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_detect_unknown_binary() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_binary_unknown");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.dat");
        {
            let mut file = File::create(&file_path).unwrap();
            // Random binary data
            file.write_all(&[0xFF, 0xFE, 0xFD, 0xFC]).unwrap();
        }

        let result = detect_binary_type(&file_path);
        assert_eq!(result, Some(BinaryType::Unknown));

        std::fs::remove_dir_all(temp_dir).ok();
    }
}
