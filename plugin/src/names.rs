//! Names of armor, decorations, skill trees, skills and items. Only names.<language>.bin of the current language is loaded.
//! See tools/build_plugin_data.py for the format.
use alloc::vec::Vec;

pub const ARMOR: usize = 0;
pub const JEWEL: usize = 1;
pub const TREE: usize = 2;
pub const SKILL: usize = 3;
pub const ITEM: usize = 4;

pub const CODES: [&str; 9] = ["en", "zh-Hans", "zh-Hant", "ja", "ko", "de", "fr", "it", "es"];

struct Table {
    ids: usize,   // Start of the id list
    count: usize,
    offs: usize,  // Start of the offset table
    pool: usize,  // Start of the string pool
}

pub struct Names {
    pub language: usize,
    bytes: Vec<u8>,
    tables: Vec<Table>,
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}
fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

impl Names {
    pub fn load(language: usize) -> Option<Self> {
        let bytes = crate::res::read(&alloc::format!("names.{}.bin", CODES.get(language)?))?;
        Self::parse(language, bytes)
    }

    fn parse(language: usize, bytes: Vec<u8>) -> Option<Self> {
        if bytes.get(..4)? != b"SKN2" {
            return None;
        }
        let n = *bytes.get(4)? as usize;
        let mut at = 5;
        let mut tables = Vec::new();
        for _ in 0..n {
            let count = u16_at(&bytes, at)? as usize;
            let ids = at + 2;
            let pool_size = u32_at(&bytes, ids + count * 2)? as usize;
            let offs = ids + count * 2 + 4;
            let pool = offs + (count + 1) * 4;
            if pool + pool_size > bytes.len() || core::str::from_utf8(&bytes[pool..pool + pool_size]).is_err() {
                return None;
            }
            tables.push(Table { ids, count, offs, pool });
            at = pool + pool_size;
        }
        Some(Self { language, bytes, tables })
    }

    pub fn get(&self, table: usize, id: u16) -> &str {
        let Some(t) = self.tables.get(table) else { return "?" };
        let (mut lo, mut hi) = (0, t.count);
        while lo < hi {
            let mid = (lo + hi) / 2;
            let v = u16_at(&self.bytes, t.ids + mid * 2).unwrap_or(0);
            if v < id {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if lo >= t.count || u16_at(&self.bytes, t.ids + lo * 2) != Some(id) {
            return "?";
        }
        let a = u32_at(&self.bytes, t.offs + lo * 4).unwrap_or(0) as usize;
        let b = u32_at(&self.bytes, t.offs + lo * 4 + 4).unwrap_or(0) as usize;
        core::str::from_utf8(&self.bytes[t.pool + a..t.pool + b]).unwrap_or("?")
    }
}
