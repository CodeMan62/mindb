use crate::storage::{PageHeader, CACHE_SIZE};
use std::collections::{HashMap, VecDeque};

pub struct PageCache {
    hash: HashMap<u64, PageHeader>,
    num_pages: usize,
    max_pages: usize,
    lru_queue: VecDeque<u64>,
}

impl PageCache {
    pub fn new() -> Self {
        let max_pages = CACHE_SIZE;
        Self {
            hash: HashMap::with_capacity(max_pages),
            lru_queue: VecDeque::with_capacity(max_pages),
            num_pages: 0,
            max_pages,
        }
    }

    pub fn lru_evict(&mut self) -> Option<(u64, PageHeader)> {
        if let Some(page_id) = self.lru_queue.pop_front() {
            if let Some(page) = self.hash.remove(&page_id) {
                self.num_pages -= 1;
                return Some((page_id, page));
            }
        }
        None
    }

    pub fn get(&mut self, page_id: u64) -> Option<&PageHeader> {
        if self.hash.contains_key(&page_id) {
            self.touch(page_id);
            return self.hash.get(&page_id);
        }
        None
    }

    pub fn put(&mut self, page_id: u64, page: PageHeader) -> Option<(u64, PageHeader)> {
        let mut evicted = None;

        if self.hash.contains_key(&page_id) {
            self.remove_from_lru(page_id);
        } else {
            if self.num_pages >= self.max_pages {
                evicted = self.lru_evict();
            }
            self.num_pages += 1;
        }

        self.hash.insert(page_id, page);
        self.lru_queue.push_back(page_id);

        evicted
    }

    pub fn remove(&mut self, page_id: u64) -> Option<PageHeader> {
        if let Some(page) = self.hash.remove(&page_id) {
            self.remove_from_lru(page_id);
            self.num_pages -= 1;
            return Some(page);
        }
        None
    }

    pub fn contains(&self, page_id: u64) -> bool {
        self.hash.contains_key(&page_id)
    }

    pub fn clear(&mut self) {
        self.hash.clear();
        self.lru_queue.clear();
        self.num_pages = 0;
    }

    pub fn len(&self) -> usize {
        self.num_pages
    }

    pub fn is_empty(&self) -> bool {
        self.num_pages == 0
    }

    pub fn is_full(&self) -> bool {
        self.num_pages >= self.max_pages
    }

    pub fn capacity(&self) -> usize {
        self.max_pages
    }

    fn touch(&mut self, page_id: u64) {
        self.remove_from_lru(page_id);
        self.lru_queue.push_back(page_id);
    }

    fn remove_from_lru(&mut self, page_id: u64) {
        if let Some(pos) = self.lru_queue.iter().position(|&id| id == page_id) {
            self.lru_queue.remove(pos);
        }
    }

    pub fn dirty_pages(&self) -> impl Iterator<Item = (&u64, &PageHeader)> {
        self.hash.iter().filter(|(_, page)| page.is_dirty())
    }

    pub fn flush_dirty<F>(&mut self, mut flush_fn: F) -> Result<usize, std::io::Error>
    where
        F: FnMut(u64, &mut PageHeader) -> Result<(), std::io::Error>,
    {
        let mut flushed = 0;
        for (page_id, page) in self.hash.iter_mut() {
            if page.is_dirty() {
                flush_fn(*page_id, page)?;
                flushed += 1;
            }
        }
        Ok(flushed)
    }
}

impl Default for PageCache {
    fn default() -> Self {
        Self::new()
    }
}
