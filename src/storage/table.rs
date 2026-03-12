use super::pager::{Pager, PAGE_SIZE};
use crate::row::{Row, ROW_SIZE};
use crate::schema::{Schema, SCHEMA_SIZE};

pub const META_PAGE: u64 = 0; 
pub const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE; // 15  
#[derive(Debug)]
pub struct Table {
    pub schema: Schema,
    pub row_count: u64,
    pager: Pager,
}

impl Table {
    pub fn open(path: &str, schema: Schema) -> Result<Self, std::io::Error> {
        let mut pager = Pager::open(path)?;

        if pager.page_count == 0 {
            let (meta, row_count) = Self::encode_meta(&schema, 0);
            pager.write_page(META_PAGE, meta)?;
            return Ok(Self {
                schema,
                row_count,
                pager,
            });
        }

        let raw = pager.read_page(META_PAGE)?;
        let (schema, row_count) = Self::decode_meta(&raw);
        Ok(Self {
            schema,
            row_count,
            pager,
        })
    }
    pub fn insert(&mut self, row: &Row) -> Result<(), std::io::Error> {
        let slot = self.row_count as usize;
        let page_no = 1 + (slot / ROWS_PER_PAGE) as u64; 
        let slot_in_page = slot % ROWS_PER_PAGE;
        let mut page = if page_no < self.pager.page_count {
            self.pager.read_page(page_no)?
        } else {
            [0u8; PAGE_SIZE]
        };
        let off = slot_in_page * ROW_SIZE;
        page[off..off + ROW_SIZE].copy_from_slice(&row.to_bytes());
        self.pager.write_page(page_no,page)?;
        self.row_count += 1;
        let (meta, _) = Self::encode_meta(&self.schema, self.row_count);
        self.pager.write_page(META_PAGE, meta)?;
        Ok(())
    }
    pub fn scan(&mut self) -> Result<Vec<Row>, std::io::Error> {
        let mut rows = Vec::with_capacity(self.row_count as usize);
        for slot in 0..self.row_count as usize {
            let page_no = 1 + (slot / ROWS_PER_PAGE) as u64;
            let slot_in_page = slot % ROWS_PER_PAGE;
            let page = self.pager.read_page(page_no)?;
            let off = slot_in_page * ROW_SIZE;
            let buf: &[u8; ROW_SIZE] = page[off..off + ROW_SIZE].try_into().unwrap();
            rows.push(Row::from_bytes(buf));
        }
        Ok(rows)
    }
    // Meta page: [row_count: 8][schema: SCHEMA_SIZE][0s...]
    fn encode_meta(schema: &Schema, row_count: u64) -> ([u8; PAGE_SIZE], u64) {
        let mut page = [0u8; PAGE_SIZE];
        page[..8].copy_from_slice(&row_count.to_le_bytes());
        let sb = schema.to_bytes();
        page[8..8 + SCHEMA_SIZE].copy_from_slice(&sb);
        (page, row_count)
    }

    fn decode_meta(page: &[u8; PAGE_SIZE]) -> (Schema, u64) {
        let row_count = u64::from_le_bytes(page[..8].try_into().unwrap());
        let mut sb = [0u8; SCHEMA_SIZE];
        sb.copy_from_slice(&page[8..8 + SCHEMA_SIZE]);
        let schema = Schema::from_bytes(&sb);
        (schema, row_count)
    }
}

