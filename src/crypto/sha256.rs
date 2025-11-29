//! SHA-256 implementation following FIPS 180-4 specification
//!
//! This implementation is built from scratch using only the Rust standard library
//! to maintain the zero-dependency design principle of Genesis Preflight.
//!
//! The implementation follows the SHA-256 specification as defined in FIPS PUB 180-4:
//! "Secure Hash Standard (SHS)", published by NIST in August 2015.
//!
//! ## Why implement from scratch?
//!
//! - Zero dependencies: No external crates that could introduce supply chain vulnerabilities
//! - Transparency: Every line of code is auditable in this repository
//! - Trust: Researchers can verify the implementation against the official specification
//! - Longevity: Will work regardless of external package availability
//!
//! ## Performance characteristics
//!
//! - Processes data in 512-bit (64-byte) blocks
//! - Uses streaming for files to handle datasets larger than available memory
//! - Typical performance: ~100-200 MB/s on modern hardware (pure Rust, no SIMD)
//! - Sufficient for validating scientific datasets during preparation
//!
//! ## References
//!
//! - FIPS PUB 180-4: https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Initial hash values (first 32 bits of the fractional parts of the square roots
/// of the first 8 primes: 2, 3, 5, 7, 11, 13, 17, 19)
const H: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// Round constants (first 32 bits of the fractional parts of the cube roots
/// of the first 64 primes: 2..311)
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Rotate right (circular right shift)
#[inline]
fn rotr(x: u32, n: u32) -> u32 {
    (x >> n) | (x << (32 - n))
}

/// Right shift
#[inline]
fn shr(x: u32, n: u32) -> u32 {
    x >> n
}

/// Ch(x, y, z) = (x AND y) XOR (NOT x AND z)
#[inline]
fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

/// Maj(x, y, z) = (x AND y) XOR (x AND z) XOR (y AND z)
#[inline]
fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

/// SIGMA0(x) = ROTR2(x) XOR ROTR13(x) XOR ROTR22(x)
#[inline]
fn sigma0(x: u32) -> u32 {
    rotr(x, 2) ^ rotr(x, 13) ^ rotr(x, 22)
}

/// SIGMA1(x) = ROTR6(x) XOR ROTR11(x) XOR ROTR25(x)
#[inline]
fn sigma1(x: u32) -> u32 {
    rotr(x, 6) ^ rotr(x, 11) ^ rotr(x, 25)
}

/// sigma0(x) = ROTR7(x) XOR ROTR18(x) XOR SHR3(x)
#[inline]
fn sigma_lower_0(x: u32) -> u32 {
    rotr(x, 7) ^ rotr(x, 18) ^ shr(x, 3)
}

/// sigma1(x) = ROTR17(x) XOR ROTR19(x) XOR SHR10(x)
#[inline]
fn sigma_lower_1(x: u32) -> u32 {
    rotr(x, 17) ^ rotr(x, 19) ^ shr(x, 10)
}

/// Process a single 512-bit block
fn process_block(block: &[u8; 64], state: &mut [u32; 8]) {
    // Prepare message schedule array W
    let mut w = [0u32; 64];

    // First 16 words are the message block
    for i in 0..16 {
        w[i] = u32::from_be_bytes([
            block[i * 4],
            block[i * 4 + 1],
            block[i * 4 + 2],
            block[i * 4 + 3],
        ]);
    }

    // Extend the first 16 words into the remaining 48 words
    for i in 16..64 {
        w[i] = sigma_lower_1(w[i - 2])
            .wrapping_add(w[i - 7])
            .wrapping_add(sigma_lower_0(w[i - 15]))
            .wrapping_add(w[i - 16]);
    }

    // Initialize working variables
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    // Main compression loop (64 rounds)
    for i in 0..64 {
        let t1 = h
            .wrapping_add(sigma1(e))
            .wrapping_add(ch(e, f, g))
            .wrapping_add(K[i])
            .wrapping_add(w[i]);
        let t2 = sigma0(a).wrapping_add(maj(a, b, c));

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    // Add the compressed chunk to the current hash value
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

/// Compute SHA-256 hash of data
///
/// Returns the 32-byte hash as a fixed-size array.
///
/// # Examples
///
/// ```
/// use genesis_preflight::crypto::sha256;
///
/// let hash = sha256(b"hello");
/// assert_eq!(hash.len(), 32);
/// ```
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut state = H;
    let data_len = data.len();
    let data_len_bits = (data_len as u64) * 8;

    // Process complete 64-byte blocks
    let mut pos = 0;
    while pos + 64 <= data_len {
        let mut block = [0u8; 64];
        block.copy_from_slice(&data[pos..pos + 64]);
        process_block(&block, &mut state);
        pos += 64;
    }

    // Process final block with padding
    let remaining = data_len - pos;
    let mut final_block = [0u8; 64];
    final_block[..remaining].copy_from_slice(&data[pos..]);

    // Append the '1' bit (0x80 = 10000000 in binary)
    final_block[remaining] = 0x80;

    // If there's not enough room for the length (need 8 bytes), process this block
    // and create another padding block
    if remaining >= 56 {
        process_block(&final_block, &mut state);
        final_block = [0u8; 64];
    }

    // Append length in bits as 64-bit big-endian integer
    let len_bytes = data_len_bits.to_be_bytes();
    final_block[56..64].copy_from_slice(&len_bytes);

    process_block(&final_block, &mut state);

    // Produce final hash value (big-endian)
    let mut hash = [0u8; 32];
    for i in 0..8 {
        let bytes = state[i].to_be_bytes();
        hash[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }

    hash
}

/// Compute SHA-256 hash and return as hexadecimal string
///
/// Returns a 64-character lowercase hexadecimal string representation of the hash.
///
/// # Examples
///
/// ```
/// use genesis_preflight::crypto::sha256_hex;
///
/// let hash = sha256_hex(b"hello");
/// assert_eq!(hash.len(), 64);
/// ```
pub fn sha256_hex(data: &[u8]) -> String {
    let hash = sha256(data);
    hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

/// Compute SHA-256 hash of a file
///
/// Reads the file in chunks to handle files larger than available memory.
/// Returns the hash as a hexadecimal string.
///
/// # Examples
///
/// ```no_run
/// use genesis_preflight::crypto::sha256_file;
/// use std::path::Path;
///
/// let hash = sha256_file(Path::new("data.csv")).unwrap();
/// println!("SHA-256: {}", hash);
/// ```
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read.
pub fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut state = H;
    let mut buffer = [0u8; 8192]; // 8KB buffer for reading
    let mut total_bytes = 0u64;
    let mut pending = Vec::new();

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        total_bytes += bytes_read as u64;

        // Combine pending data with new data
        pending.extend_from_slice(&buffer[..bytes_read]);

        // Process complete 64-byte blocks
        let mut pos = 0;
        while pos + 64 <= pending.len() {
            let mut block = [0u8; 64];
            block.copy_from_slice(&pending[pos..pos + 64]);
            process_block(&block, &mut state);
            pos += 64;
        }

        // Keep remaining bytes for next iteration
        pending = pending[pos..].to_vec();
    }

    // Process final block with padding
    let data_len_bits = total_bytes * 8;
    let remaining = pending.len();
    let mut final_block = [0u8; 64];
    final_block[..remaining].copy_from_slice(&pending);

    // Append the '1' bit
    final_block[remaining] = 0x80;

    // If there's not enough room for the length, process this block
    // and create another padding block
    if remaining >= 56 {
        process_block(&final_block, &mut state);
        final_block = [0u8; 64];
    }

    // Append length in bits as 64-bit big-endian integer
    let len_bytes = data_len_bits.to_be_bytes();
    final_block[56..64].copy_from_slice(&len_bytes);

    process_block(&final_block, &mut state);

    // Produce final hash value
    let mut hash = [0u8; 32];
    for i in 0..8 {
        let bytes = state[i].to_be_bytes();
        hash[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }

    Ok(hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_sha256_empty() {
        // Test vector from FIPS 180-4
        let hash = sha256_hex(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_abc() {
        // Test vector from FIPS 180-4
        let hash = sha256_hex(b"abc");
        assert_eq!(
            hash,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_sha256_longer() {
        // Test vector from FIPS 180-4
        let hash = sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        assert_eq!(
            hash,
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn test_sha256_file() {
        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("genesis_preflight_test_sha256.txt");

        {
            let mut file = File::create(&temp_file).unwrap();
            file.write_all(b"abc").unwrap();
        }

        let hash = sha256_file(&temp_file).unwrap();
        assert_eq!(
            hash,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );

        // Clean up
        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_sha256_file_large() {
        // Test with a file larger than the buffer size
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("genesis_preflight_test_sha256_large.txt");

        {
            let mut file = File::create(&temp_file).unwrap();
            // Write more than 8KB to test buffering
            for _ in 0..1000 {
                file.write_all(b"0123456789").unwrap();
            }
        }

        // Read file and compute hash with sha256_hex for comparison
        let file_contents = std::fs::read(&temp_file).unwrap();
        let expected = sha256_hex(&file_contents);

        let hash = sha256_file(&temp_file).unwrap();
        assert_eq!(hash, expected);

        // Clean up
        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_logical_functions() {
        // Test the logical functions with known values
        // ch(x, y, z) = (x & y) ^ (!x & z)
        assert_eq!(ch(0x12345678, 0xabcdef00, 0x11111111), 0x03054701);
        // maj(x, y, z) = (x & y) ^ (x & z) ^ (y & z)
        assert_eq!(maj(0x12345678, 0xabcdef00, 0x11111111), 0x13155710);
    }

    #[test]
    fn test_rotate_and_shift() {
        assert_eq!(rotr(0x12345678, 4), 0x81234567);
        assert_eq!(shr(0x12345678, 4), 0x01234567);
    }
}
