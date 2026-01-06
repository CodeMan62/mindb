use crate::storage::checksum::{ChecksumError, ChecksumPage};
use crate::storage::lock::{FileLock, LockType};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

pub const PAGE_SIZE: usize = 4096;
pub const MAX_PAGE: usize = 100;
pub const CACHE_SIZE: usize = 100;
pub const HEADER_SIZE: usize = std::mem::size_of::<PageHeader>();

#[derive(Debug, Clone)]
pub struct PageHeader {
    pgno: u64,
    flags: u64,
    data: [u8; PAGE_SIZE],
    is_dirty: bool,
    checksum_valid: bool,
    next_free_page: u64,
}

pub struct Pager {
    file: File,
    file_path: String,
    page_size: u64,
    page_count: u64,
    lock: FileLock,
    verify_checksums: bool,
    first_free_page: u64
}

impl PageHeader {
    pub fn new(pgno: u64) -> Self {
        PageHeader {
            pgno,
            flags: 0,
            data: [0; PAGE_SIZE],
            is_dirty: false,
            checksum_valid: true,
        }
    }

    /// Create a PageHeader from raw page data, verifying the checksum
    pub fn from_raw(raw: &[u8; PAGE_SIZE], expected_pgno: u64) -> Result<Self, ChecksumError> {
        // Verify checksum
        ChecksumPage::verify_checksum(raw, expected_pgno)?;

        let pgno = ChecksumPage::read_page_number(raw);
        let flags = ChecksumPage::read_flags(raw);

        let mut data = [0u8; PAGE_SIZE];
        data.copy_from_slice(raw);

        Ok(PageHeader {
            pgno,
            flags,
            data,
            is_dirty: false,
            checksum_valid: true,
        })
    }

    /// Create a PageHeader from raw page data without verifying checksum
    pub fn from_raw_unchecked(raw: &[u8; PAGE_SIZE]) -> Self {
        let pgno = ChecksumPage::read_page_number(raw);
        let flags = ChecksumPage::read_flags(raw);

        let mut data = [0u8; PAGE_SIZE];
        data.copy_from_slice(raw);

        PageHeader {
            pgno,
            flags,
            data,
            is_dirty: false,
            checksum_valid: false, // Unknown since we didn't verify
        }
    }

    /// Serialize the page to raw bytes, computing the checksum
    pub fn to_raw(&self) -> [u8; PAGE_SIZE] {
        let mut raw = self.data;

        // Write page number and flags
        ChecksumPage::write_page_number(&mut raw, self.pgno);
        ChecksumPage::write_flags(&mut raw, self.flags);

        // Compute and write checksum
        ChecksumPage::write_checksum(&mut raw);

        raw
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
    }

    pub fn pgno(&self) -> u64 {
        self.pgno
    }

    pub fn flags(&self) -> u64 {
        self.flags
    }

    pub fn set_flags(&mut self, flags: u64) {
        self.flags = flags;
        self.is_dirty = true;
    }

    pub fn data(&self) -> &[u8; PAGE_SIZE] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8; PAGE_SIZE] {
        self.is_dirty = true;
        &mut self.data
    }

    /// Get the user data portion of the page (after the header)
    pub fn user_data(&self) -> &[u8] {
        &self.data[ChecksumPage::data_offset()..]
    }

    /// Get mutable access to the user data portion
    pub fn user_data_mut(&mut self) -> &mut [u8] {
        self.is_dirty = true;
        &mut self.data[ChecksumPage::data_offset()..]
    }

    /// Check if the page's checksum was verified as valid
    pub fn is_checksum_valid(&self) -> bool {
        self.checksum_valid
    }
}

impl Pager {
    /// Open a database file with optional locking
    ///
    /// By default, acquires an exclusive lock for write access.
    /// Use `open_shared` for read-only access with shared locking.
    pub fn open(file_name: String) -> Result<Self, std::io::Error> {
        Self::open_with_options(file_name, true, true)
    }

    /// Open a database file with shared (read) lock
    ///
    /// Multiple processes can open with shared lock simultaneously.
    /// Write operations will fail - use for read-only access.
    pub fn open_shared(file_name: String) -> Result<Self, std::io::Error> {
        let mut lock = FileLock::new(&file_name)?;
        lock.lock_shared()?;

        let file = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(&file_name)?;
        let file_size = file.metadata()?.len();
        let page_count = file_size / PAGE_SIZE as u64;
        Ok(Self {
            file,
            file_path: file_name,
            page_size: file_size,
            page_count,
            lock,
            verify_checksums: true,
        })
    }

    /// Open with configurable options
    fn open_with_options(
        file_name: String,
        verify_checksums: bool,
        truncate: bool,
    ) -> Result<Self, std::io::Error> {
        let mut lock = FileLock::new(&file_name)?;
        lock.lock_exclusive()?;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(truncate)
            .open(&file_name)?;
        let file_size = file.metadata()?.len();
        let page_count = file_size / PAGE_SIZE as u64;
        Ok(Self {
            file,
            file_path: file_name,
            page_size: file_size,
            page_count,
            lock,
            verify_checksums,
        })
    }

    /// Try to open with exclusive lock without blocking
    ///
    /// Returns None if the file is already locked by another process.
    pub fn try_open(file_name: String) -> Result<Option<Self>, std::io::Error> {
        let mut lock = FileLock::new(&file_name)?;
        if !lock.try_lock_exclusive()? {
            return Ok(None);
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_name)?;
        let file_size = file.metadata()?.len();
        let page_count = file_size / PAGE_SIZE as u64;
        Ok(Some(Self {
            file,
            file_path: file_name,
            page_size: file_size,
            page_count,
            lock,
            verify_checksums: true,
        }))
    }

    /// Get the current lock type
    pub fn lock_type(&self) -> LockType {
        self.lock.lock_type()
    }

    /// Enable or disable checksum verification on reads
    pub fn set_verify_checksums(&mut self, verify: bool) {
        self.verify_checksums = verify;
    }

    /// Upgrade from shared to exclusive lock
    ///
    /// Call this before performing write operations on a shared-locked pager.
    pub fn upgrade_lock(&mut self) -> Result<(), std::io::Error> {
        self.lock.upgrade_to_exclusive()
    }

    /// Downgrade from exclusive to shared lock
    ///
    /// Allows other readers to access the file.
    pub fn downgrade_lock(&mut self) -> Result<(), std::io::Error> {
        self.lock.downgrade_to_shared()
    }

    /// Read a page from the database file (raw bytes)
    pub fn read_raw(&mut self, pgno: u64, buf: &mut [u8; PAGE_SIZE]) -> Result<(), std::io::Error> {
        if pgno == 0 || pgno > self.page_count {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid page number: {}", pgno),
            ));
        }
        let offset = (pgno - 1) * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(buf)?;
        Ok(())
    }

    /// Read a page and return a PageHeader, optionally verifying checksum
    pub fn read_page(&mut self, pgno: u64) -> Result<PageHeader, std::io::Error> {
        let mut buf = [0u8; PAGE_SIZE];
        self.read_raw(pgno, &mut buf)?;

        if self.verify_checksums {
            PageHeader::from_raw(&buf, pgno)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
        } else {
            Ok(PageHeader::from_raw_unchecked(&buf))
        }
    }

    #[deprecated(note = "Use read_raw or read_page instead")]
    pub fn read(&mut self, pgno: u64, mut buf: [u8; PAGE_SIZE]) -> Result<(), std::io::Error> {
        self.read_raw(pgno, &mut buf)
    }

    /// Write raw bytes to a page (caller must handle checksum)
    pub fn write_raw(
        &mut self,
        page_number: u64,
        data: &[u8; PAGE_SIZE],
    ) -> Result<(), std::io::Error> {
        if self.lock.lock_type() != LockType::Exclusive {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Cannot write without exclusive lock",
            ));
        }
        let offset = (page_number - 1) * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(data)?;
        self.file.sync_all()?;

        // Update page count if we extended the file
        if page_number > self.page_count {
            self.page_count = page_number;
        }

        Ok(())
    }

    /// Write a PageHeader to disk, computing the checksum automatically
    pub fn write_page(&mut self, page: &PageHeader) -> Result<(), std::io::Error> {
        let raw = page.to_raw();
        self.write_raw(page.pgno(), &raw)
    }

    /// Get the current page count
    pub fn page_count(&self) -> u64 {
        self.page_count
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.file.flush()?;
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), std::io::Error> {
        self.file.sync_all()?;
        self.lock.unlock()?;
        Ok(())
    }
    pub fn page_alloc(&mut self) {
        if self.first_free_page != 0 {
            let page_id = self.first_free_page;
            let mut page: PageHeader = self.read_page(self, page_id);
            self.first_free_page = page.next_free_page;
            page = unsafe { std::mem::zeroed() };
            page.pgno = page_id;
        } else {
            let page_id = self.page_count;
            self.page_size += 1;
            let mut page: PageHeader = PageHeader {
                pgno: 0,
                data: [0u8, PAGE_SIZE - HEADER_SIZE ],
            };
            page.pgno = page_id;

            // Write the page
            self.write_page(self, page_id, &page);
        }
    }
    pub fn page_dealloc(&mut self, page_id: u64) {
        let mut page: PageHeader = self.read_page(self, page_id);
        page.next_free_page = self.first_free_page;
        page.data = [0u8, PAGE_SIZE];
        self.write_page(self, page_id, &page);
        self.first_free_page = page_id;
    }
}

impl Drop for Pager {
    fn drop(&mut self) {
        let _ = self.flush();
        let _ = self.lock.unlock();
    }
}
