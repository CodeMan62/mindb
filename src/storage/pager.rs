use fs2::FileExt;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
};

pub const PAGE_SIZE: usize = 4096;
pub const CACHE_SIZE: usize = 16;

pub struct PageHdr {
    pub pgno: u64,
    pub flags: u64,
    pub data: [u8; PAGE_SIZE],
    pub is_dirty: bool,
}

pub struct PageCache {
    slots: HashMap<u64, PageHdr>,
    capacity: usize,
}

pub struct Pager {
    file: File,
    pub file_size: u64,
    pub page_count: u64,
    cache: PageCache,
}

impl PageHdr {
    pub fn new(pgno: u64) -> Self {
        Self {
            pgno,
            flags: 0,
            data: [0u8; PAGE_SIZE],
            is_dirty: false,
        }
    }
    pub fn data_mut(&mut self) -> &mut [u8; PAGE_SIZE] {
        self.is_dirty = true;
        &mut self.data
    }
}

impl PageCache {
    pub fn new(capacity: usize) -> Self {
        PageCache {
            slots: HashMap::with_capacity(capacity),
            capacity,
        }
    }
    pub fn get(&self, pgno: u64) -> Option<&PageHdr> {
        self.slots.get(&pgno)
    }
    pub fn get_mut(&mut self, pgno: u64) -> Option<&mut PageHdr> {
        self.slots.get_mut(&pgno)
    }
    pub fn insert(&mut self, page: PageHdr) {
        self.slots.insert(page.pgno, page);
    }
    pub fn contains(&self, pgno: u64) -> bool {
        self.slots.contains_key(&pgno)
    }
}

impl Pager {
    pub fn open(file_name: &str) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_name)?;
        file.try_lock_exclusive()?;
        let file_size = file.metadata()?.len();
        let page_count = file_size / PAGE_SIZE as u64;
        Ok(Self {
            file,
            file_size,
            page_count,
            cache: PageCache::new(CACHE_SIZE),
        })
    }

    pub fn write_page(&mut self, pgno: u64, data: [u8; PAGE_SIZE]) -> Result<(), std::io::Error> {
        let offset = pgno * PAGE_SIZE as u64;
        self.file.seek(std::io::SeekFrom::Start(offset))?;
        self.file.write_all(&data)?;
        self.file.sync_all()?;
        let end = offset + PAGE_SIZE as u64;
        if end > self.file_size {
            self.file_size = end;
            self.page_count = self.file_size / PAGE_SIZE as u64;
        }
        let mut page = PageHdr::new(pgno);
        page.data_mut().copy_from_slice(&data);
        page.is_dirty = false;
        self.cache.insert(page);
        Ok(())
    }

    pub fn read_page(&mut self, pgno: u64) -> Result<[u8; PAGE_SIZE], std::io::Error> {
        if let Some(page) = self.cache.get(pgno) {
            return Ok(page.data);
        }
        let offset = pgno * PAGE_SIZE as u64;
        self.file.seek(std::io::SeekFrom::Start(offset))?;
        let mut data = [0u8; PAGE_SIZE];
        self.file.read_exact(&mut data)?;
        let mut page = PageHdr::new(pgno);
        page.data_mut().copy_from_slice(&data);
        page.is_dirty = false;
        self.cache.insert(page);
        Ok(data)
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.file.flush()
    }
}

impl Drop for Pager {
    fn drop(&mut self) {
        let _ = self.file.sync_all();
        let _ = self.file.unlock();
    }
}
