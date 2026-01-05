//! Page checksum functionality for data integrity verification.
//!
//! Implements CRC32 checksums to detect data corruption in database pages.

/// CRC32 polynomial (IEEE 802.3 standard)
const CRC32_POLYNOMIAL: u32 = 0xEDB88320;

/// Precomputed CRC32 lookup table for faster computation
const CRC32_TABLE: [u32; 256] = generate_crc32_table();

/// Generate CRC32 lookup table at compile time
const fn generate_crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ CRC32_POLYNOMIAL;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

/// Compute CRC32 checksum for a byte slice
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        let index = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLE[index];
    }
    !crc
}

/// Page checksum error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecksumError {
    pub page_number: u64,
    pub expected: u32,
    pub actual: u32,
}

impl std::fmt::Display for ChecksumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Checksum mismatch on page {}: expected 0x{:08X}, got 0x{:08X}",
            self.page_number, self.expected, self.actual
        )
    }
}

impl std::error::Error for ChecksumError {}

/// Page layout with checksum support
///
/// The page is organized as follows:
/// - Bytes 0-3: CRC32 checksum of the rest of the page
/// - Bytes 4-11: Page number (u64, little endian)
/// - Bytes 12-19: Flags (u64, little endian)
/// - Bytes 20+: User data
pub const CHECKSUM_SIZE: usize = 4;
pub const PAGE_HEADER_SIZE: usize = CHECKSUM_SIZE + 8 + 8; // checksum + pgno + flags

/// Checksum-aware page operations
pub struct ChecksumPage;

impl ChecksumPage {
    /// Calculate checksum for page data (excluding the checksum field itself)
    ///
    /// The checksum covers bytes 4 onwards (page number, flags, and data).
    pub fn calculate_checksum(page_data: &[u8]) -> u32 {
        crc32(&page_data[CHECKSUM_SIZE..])
    }

    /// Write checksum to the first 4 bytes of page data
    pub fn write_checksum(page_data: &mut [u8]) {
        let checksum = Self::calculate_checksum(page_data);
        page_data[0..4].copy_from_slice(&checksum.to_le_bytes());
    }

    /// Read checksum from the first 4 bytes of page data
    pub fn read_checksum(page_data: &[u8]) -> u32 {
        u32::from_le_bytes([page_data[0], page_data[1], page_data[2], page_data[3]])
    }

    /// Verify page checksum
    ///
    /// Returns Ok(()) if checksum matches, Err with details otherwise.
    pub fn verify_checksum(page_data: &[u8], page_number: u64) -> Result<(), ChecksumError> {
        let stored = Self::read_checksum(page_data);
        let calculated = Self::calculate_checksum(page_data);

        if stored == calculated {
            Ok(())
        } else {
            Err(ChecksumError {
                page_number,
                expected: stored,
                actual: calculated,
            })
        }
    }

    /// Write page number to page data (bytes 4-11)
    pub fn write_page_number(page_data: &mut [u8], pgno: u64) {
        page_data[4..12].copy_from_slice(&pgno.to_le_bytes());
    }

    /// Read page number from page data (bytes 4-11)
    pub fn read_page_number(page_data: &[u8]) -> u64 {
        u64::from_le_bytes([
            page_data[4],
            page_data[5],
            page_data[6],
            page_data[7],
            page_data[8],
            page_data[9],
            page_data[10],
            page_data[11],
        ])
    }

    /// Write flags to page data (bytes 12-19)
    pub fn write_flags(page_data: &mut [u8], flags: u64) {
        page_data[12..20].copy_from_slice(&flags.to_le_bytes());
    }

    /// Read flags from page data (bytes 12-19)
    pub fn read_flags(page_data: &[u8]) -> u64 {
        u64::from_le_bytes([
            page_data[12],
            page_data[13],
            page_data[14],
            page_data[15],
            page_data[16],
            page_data[17],
            page_data[18],
            page_data[19],
        ])
    }

    /// Get the offset where user data starts
    pub fn data_offset() -> usize {
        PAGE_HEADER_SIZE
    }

    /// Get the maximum size of user data in a page
    pub fn max_data_size(page_size: usize) -> usize {
        page_size - PAGE_HEADER_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_empty() {
        assert_eq!(crc32(&[]), 0x00000000);
    }

    #[test]
    fn test_crc32_known_values() {
        // "123456789" should produce 0xCBF43926
        let data = b"123456789";
        assert_eq!(crc32(data), 0xCBF43926);
    }

    #[test]
    fn test_crc32_consistency() {
        let data = b"Hello, World!";
        let checksum1 = crc32(data);
        let checksum2 = crc32(data);
        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_crc32_different_data() {
        let data1 = b"Hello";
        let data2 = b"World";
        assert_ne!(crc32(data1), crc32(data2));
    }

    #[test]
    fn test_checksum_write_read() {
        let mut page = [0u8; 4096];

        // Write some data
        page[20..25].copy_from_slice(b"Hello");

        // Write checksum
        ChecksumPage::write_checksum(&mut page);

        // Read and verify
        let stored = ChecksumPage::read_checksum(&page);
        let calculated = ChecksumPage::calculate_checksum(&page);
        assert_eq!(stored, calculated);
    }

    #[test]
    fn test_checksum_verification_success() {
        let mut page = [0u8; 4096];

        ChecksumPage::write_page_number(&mut page, 42);
        ChecksumPage::write_flags(&mut page, 0x01);
        page[20..25].copy_from_slice(b"Data!");
        ChecksumPage::write_checksum(&mut page);

        assert!(ChecksumPage::verify_checksum(&page, 42).is_ok());
    }

    #[test]
    fn test_checksum_verification_failure() {
        let mut page = [0u8; 4096];

        ChecksumPage::write_page_number(&mut page, 42);
        ChecksumPage::write_checksum(&mut page);

        // Corrupt the data
        page[100] = 0xFF;

        let result = ChecksumPage::verify_checksum(&page, 42);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.page_number, 42);
        assert_ne!(err.expected, err.actual);
    }

    #[test]
    fn test_page_number_roundtrip() {
        let mut page = [0u8; 4096];

        ChecksumPage::write_page_number(&mut page, 12345);
        assert_eq!(ChecksumPage::read_page_number(&page), 12345);

        ChecksumPage::write_page_number(&mut page, u64::MAX);
        assert_eq!(ChecksumPage::read_page_number(&page), u64::MAX);
    }

    #[test]
    fn test_flags_roundtrip() {
        let mut page = [0u8; 4096];

        ChecksumPage::write_flags(&mut page, 0xDEADBEEF);
        assert_eq!(ChecksumPage::read_flags(&page), 0xDEADBEEF);
    }

    #[test]
    fn test_data_offset() {
        assert_eq!(ChecksumPage::data_offset(), PAGE_HEADER_SIZE);
        assert_eq!(ChecksumPage::data_offset(), 20); // 4 + 8 + 8
    }

    #[test]
    fn test_max_data_size() {
        assert_eq!(ChecksumPage::max_data_size(4096), 4096 - PAGE_HEADER_SIZE);
        assert_eq!(ChecksumPage::max_data_size(4096), 4076);
    }

    #[test]
    fn test_single_bit_detection() {
        let mut page = [0u8; 4096];
        page[500] = 0b10101010;
        ChecksumPage::write_checksum(&mut page);

        // Verify original is valid
        assert!(ChecksumPage::verify_checksum(&page, 1).is_ok());

        // Flip a single bit
        page[500] = 0b10101011;

        // Should detect the corruption
        assert!(ChecksumPage::verify_checksum(&page, 1).is_err());
    }
}
