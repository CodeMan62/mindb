use crate::schema::MAX_COL;

pub const COL_VAL_LEN: usize = 16; 
pub const ROW_SIZE: usize = 8 +MAX_COL * COL_VAL_LEN;
#[derive(Debug)]
pub struct Row {
    id: i64,
    values: [String; MAX_COL]
}

impl Row {
    pub fn new(id: i64, values: &[&str]) -> Self {
        let values = std::array::from_fn(|i| values.get(i).copied().unwrap_or("").to_string());
        Self {id, values }
    }
    pub fn to_bytes(&self) -> [u8; ROW_SIZE]{
        let mut buf = [0u8; ROW_SIZE];
        buf[..8].copy_from_slice(&self.id.to_le_bytes());
        for (i, val) in self.values.iter().enumerate() {
            let off = 8 + i * COL_VAL_LEN;
            let b = val.as_bytes();
            let len = b.len().min(COL_VAL_LEN);
            buf[off..off + len].copy_from_slice(&b[..len]);
        }
        buf
    }
    pub fn from_bytes(buf: &[u8; ROW_SIZE]) -> Self {
        let id = i64::from_le_bytes(buf[..8].try_into().unwrap());
        let values = std::array::from_fn(|i| {
            let off = 8 + i * COL_VAL_LEN;
            let slice = &buf[off..off + COL_VAL_LEN];
            let end = slice.iter().position(|&b| b == 0).unwrap_or(COL_VAL_LEN);
            String::from_utf8_lossy(&slice[..end]).into_owned()
        });
        Self { id, values }
    }
}
