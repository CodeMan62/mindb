use std::{collections::HashMap, fs::{File, OpenOptions}, hash::Hash, io::{Read, Seek, Write}};

use fs2::FileExt;

pub const PAGE_SIZE: usize = 4096;
pub const CACHE_SIZE: usize = 16;

pub struct PageHdr {
    pgno: u64,
    flags: u64,
    data: [u8; PAGE_SIZE],
    is_dirty: bool
}
pub struct Pager{
    file: File,
    file_size: u64,
    page_count: u64,
    cache: PageCache,
}
pub struct PageCache{
    slots: HashMap<u64, PageHdr>,
    capacity: usize
}

impl PageHdr {
    pub fn new(pgno: u64) -> Self{
        Self { pgno, flags: 0, data: [0u8; PAGE_SIZE], is_dirty: false }
    }
    pub fn is_dirty(&self) -> bool{
        self.is_dirty
    }
    pub fn data_mut(&mut self) -> &mut [u8; PAGE_SIZE]{
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
    pub fn get(&self, pgno: u64)-> Option<&PageHdr> {
        self.slots.get(&pgno)
    }
    pub fn contains(&self, pgno: u64) -> bool{
        self.slots.contains_key(&pgno)
    }

}
impl Pager {
    pub fn open(file_name: &str) -> Result<Self, std::io::Error>{
        let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_name)?;
        file.try_lock_exclusive()?;
        let file_size = file.metadata()?.len();
        let page_count = file_size / PAGE_SIZE as u64;
        Ok(Self{
            file,
            file_size,
            page_count,
            cache: PageCache::new(CACHE_SIZE),
        })
    }
    // write data to pgno
    pub fn write_page(&mut self, pgno: u64, data: [u8; PAGE_SIZE]) -> Result<(), std::io::Error> {
        let offset = (pgno - 1) * PAGE_SIZE as u64;
        self.file.seek(std::io::SeekFrom::Start(offset))?;
        self.file.write_all(&data)?;
        self.file.sync_all()?;
        let end_offset = offset + PAGE_SIZE as u64;
        if end_offset > self.file_size {
            self.file_size = end_offset;
            self.page_count = self.file_size / PAGE_SIZE as u64; 
        }
        let mut page = PageHdr::new(pgno);
        *page.data_mut() = data;
        page.is_dirty == false;
        Ok(())
    }
    pub fn read(&mut self, pgno: u64, data: &mut [u8; PAGE_SIZE]) -> Result<(), std::io::Error> {
        let offset = (pgno - 1) * PAGE_SIZE as u64;
        self.file.seek(std::io::SeekFrom::Start(offset))?;
        self.file.read_exact(data)?;
        Ok(())
    }
    pub fn fetch(&mut self, pgno: u64) -> Result<&PageHdr, std::io::Error>{
        if !self.cache.contains(pgno) {
            let mut data = [0u8; PAGE_SIZE];
            let offset = (pgno - 1) * PAGE_SIZE as u64;
            self.file.seek(std::io::SeekFrom::Start(offset))?;
            self.file.read_exact(&mut data)?;
            let mut page = PageHdr::new(pgno);
            page.data_mut().copy_from_slice(&data);
        }
        Ok(self.cache.get(pgno).unwrap())
    }
    pub fn flush(&mut self) -> Result<(), std::io::Error>{
        self.file.flush()
    }
    pub fn close(&mut self) ->  Result<(), std::io::Error>{
        self.file.sync_all()
    }
}
impl Drop for Pager {
    fn drop(&mut self) {
        self.file.sync_all();
        self.file.unlock();
    }
}

