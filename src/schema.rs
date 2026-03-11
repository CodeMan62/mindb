pub const COL_NAME_LEN: usize = 8;
pub const MAX_COL: usize = 4;
pub const SCHEMA_SIZE: usize = 1 + MAX_COL * (COL_NAME_LEN + 1);

#[derive(Debug, Clone)]
pub struct Cols {
    name: [u8; COL_NAME_LEN],
    active: bool,
}

impl Cols {
    pub fn new(name: &str) -> Self {
        let mut buf=[0u8; COL_NAME_LEN]; // [0;32]
        let b = name.as_bytes();                               
        buf[..b.len().min(COL_NAME_LEN)].copy_from_slice(&b[..b.len().min(COL_NAME_LEN)]);
        Self {
            name: buf,
            active: true,
        }
    }
    pub fn name_str(&self) -> &str {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(COL_NAME_LEN);
        std::str::from_utf8(&self.name[..end]).unwrap_or("")
    }
}

#[derive(Debug, Clone)]
pub struct  Schema {
    num_cols: usize,
    cols: [Cols; MAX_COL],
}

impl Schema {
    pub fn new(names: &[&str]) -> Self {
        let mut cols = std::array::from_fn(|_| Cols {
            name: [0u8; COL_NAME_LEN],
            active: false
        });
        for (i, name) in names.iter().enumerate() {
            cols[i] = Cols::new(name);
        }
        Self {
            num_cols: names.len(),
            cols
        }
    }
    pub fn to_bytes(&self) -> [u8; SCHEMA_SIZE] {
        let mut buf = [0u8; SCHEMA_SIZE];
        buf[0] = self.num_cols as u8;
        for i in 0..MAX_COL {
            let off= 1 + i * (COL_NAME_LEN + 1);
            buf[off..off + COL_NAME_LEN].copy_from_slice(&self.cols[i].name);
            buf[off + COL_NAME_LEN] = self.cols[i].active as u8;
        }
        buf
    }
    pub fn from_bytes(&self,buf: &[u8; SCHEMA_SIZE]) -> Self {
        let cols = std::array::from_fn(|i| {
            let off= 1 + i * (COL_NAME_LEN + 1);
            let mut name = [0u8; COL_NAME_LEN];
            name.copy_from_slice(&buf[off..off + COL_NAME_LEN]);
            let active = buf[off + COL_NAME_LEN] != 0;
            Cols { name, active }
        });
        Self {
            num_cols: buf[0] as usize,
            cols
        }
    }
}
