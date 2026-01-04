use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

pub const PAGE_SIZE: usize = 4096;
pub const MAX_PAGE: usize = 100;
pub const CACHE_SIZE: usize = 100;

#[derive(Debug, Clone)]
pub struct PageHeader {
    pgno: u64,
    flags: u64,
    data: [u8; PAGE_SIZE],
    is_dirty: bool,
}

pub struct Pager {
    file: File,
    page_size: u64,
    page_count: u64,
}

impl PageHeader {
    pub fn new(pgno: u64) -> Self {
        PageHeader {
            pgno,
            flags: 0,
            data: [0; PAGE_SIZE],
            is_dirty: false,
        }
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

    pub fn data(&self) -> &[u8; PAGE_SIZE] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8; PAGE_SIZE] {
        self.is_dirty = true;
        &mut self.data
    }
}

impl Pager {
    // this creates and opens
    pub fn open(file_name: String) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_name)?;
        let file_size = file.metadata()?.len();
        let page_count = file_size / PAGE_SIZE as u64;
        Ok(Self {
            file,
            page_size: file_size,
            page_count,
        })
    }
    pub fn read(&mut self, pgno: u64, mut buf: [u8; PAGE_SIZE]) -> Result<(), std::io::Error> {
        if pgno == 0 || pgno > self.page_count {
            println!("database file not found");
        }
        // set offset -> pagno - 1 * PAGE_SIZE
        let offset = (pgno - 1) * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read(&mut buf)?;
        Ok(())
    }
    // write a page to database
    pub fn write(&mut self, page_number: u64, data: [u8; PAGE_SIZE]) -> Result<(), std::io::Error> {
        let offset = (page_number - 1) * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&data)?;
        self.file.sync_all()?;
        Ok(())
    }
    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.file.flush()?;
        Ok(())
    }
    pub fn close(&mut self) -> Result<(), std::io::Error> {
        self.file.sync_all()?;
        Ok(())
    }
}

impl Drop for Pager {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
