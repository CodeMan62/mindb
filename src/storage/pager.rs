use std::fs::{File, OpenOptions};
pub const PAGE_SIZE: usize = 4096;
pub const CACHE_SIZE: usize = 100;

#[derive(Debug, Clone)]
pub struct PageHeader {
    pgno: u64,
    flags: u64,
    data: [u8; PAGE_SIZE]
}

pub struct Pager {
    file: File,
    page_size: u64,
    page_count: u64,
}

impl PageHeader{
    pub fn new(&self) -> Self {
        PageHeader {
            pgno: self.pgno,
            flags: self.flags,
            data: self.data,
        }
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
            page_count
        })
    }
    pub fn read() {}
}
