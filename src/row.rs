pub const MAX_COLS: usize = 4;
pub const COL_NAME_LEN: usize = 32;
pub const COL_VAL_LEN: usize = 64;
pub const ROW_SIZE: usize = 8 + MAX_COLS * COL_VAL_LEN; // 264

#[derive(Debug, Clone)]
pub struct ColDef {
    pub name: [u8; COL_NAME_LEN],
    pub active: bool,
}

impl ColDef {
    pub fn new(name: &str) -> Self {
        let mut buf = [0u8; COL_NAME_LEN];
        let b = name.as_bytes();
        buf[..b.len().min(COL_NAME_LEN)].copy_from_slice(&b[..b.len().min(COL_NAME_LEN)]);
        Self {
            name: buf,
            active: true,
        }
    }

    pub fn name_str(&self) -> &str {
        let end = self
            .name
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(COL_NAME_LEN);
        std::str::from_utf8(&self.name[..end]).unwrap_or("")
    }
}

pub const SCHEMA_SIZE: usize = 1 + MAX_COLS * (COL_NAME_LEN + 1);

#[derive(Debug, Clone)]
pub struct Schema {
    pub cols: [ColDef; MAX_COLS],
    pub col_count: usize,
}

impl Schema {
    pub fn new(names: &[&str]) -> Self {
        assert!(names.len() <= MAX_COLS, "too many columns");
        let mut cols = std::array::from_fn(|_| ColDef {
            name: [0u8; COL_NAME_LEN],
            active: false,
        });
        for (i, name) in names.iter().enumerate() {
            cols[i] = ColDef::new(name);
        }
        Self {
            cols,
            col_count: names.len(),
        }
    }

    pub fn to_bytes(&self) -> [u8; SCHEMA_SIZE] {
        let mut buf = [0u8; SCHEMA_SIZE];
        buf[0] = self.col_count as u8;
        for i in 0..MAX_COLS {
            let off = 1 + i * (COL_NAME_LEN + 1);
            buf[off..off + COL_NAME_LEN].copy_from_slice(&self.cols[i].name);
            buf[off + COL_NAME_LEN] = self.cols[i].active as u8;
        }
        buf
    }

    pub fn from_bytes(buf: &[u8; SCHEMA_SIZE]) -> Self {
        let col_count = buf[0] as usize;
        let cols = std::array::from_fn(|i| {
            let off = 1 + i * (COL_NAME_LEN + 1);
            let mut name = [0u8; COL_NAME_LEN];
            name.copy_from_slice(&buf[off..off + COL_NAME_LEN]);
            let active = buf[off + COL_NAME_LEN] != 0;
            ColDef { name, active }
        });
        Self { cols, col_count }
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    pub id: i64,
    pub values: [String; MAX_COLS],
}

impl Row {
    pub fn new(id: i64, values: &[&str]) -> Self {
        let values = std::array::from_fn(|i| values.get(i).copied().unwrap_or("").to_string());
        Self { id, values }
    }

    pub fn to_bytes(&self) -> [u8; ROW_SIZE] {
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
