//! Parses simdata.bin made by tools/build_data.py.
use crate::types::*;
use alloc::vec::Vec;

pub struct GameData {
    pub skills: Vec<Skill>,
    pub armor: Vec<Armor>,
    pub jewels: Vec<Jewel>,
    pub charm_kinds: Vec<CharmKind>,
}

struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Option<&[u8]> {
        if self.0.len() < n {
            return None;
        }
        let (head, tail) = self.0.split_at(n);
        self.0 = tail;
        Some(head)
    }
    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|b| b[0])
    }
    fn u16(&mut self) -> Option<u16> {
        self.take(2).map(|b| u16::from_le_bytes([b[0], b[1]]))
    }
    fn pt(&mut self) -> Option<Pt> {
        let tree = self.u16()?;
        let pt = self.u8()? as i8;
        self.u8()?;
        Some(Pt { tree, pt })
    }
}

pub fn parse(blob: &[u8]) -> Option<GameData> {
    let mut r = Reader(blob);
    if r.take(8)? != b"MHSS0005" {
        return None;
    }
    let (ns, na, nj) = (r.u16()?, r.u16()?, r.u16()?);
    let mut skills = Vec::with_capacity(ns as usize);
    for _ in 0..ns {
        let (id, tree) = (r.u16()?, r.u16()?);
        let threshold = r.u8()? as i8;
        r.u8()?;
        skills.push(Skill { id, tree, threshold });
    }
    let mut armor = Vec::with_capacity(na as usize);
    for _ in 0..na {
        let id = r.u16()?;
        let (part, kind, sex, rare, slots) = (r.u8()?, r.u8()?, r.u8()?, r.u8()?, r.u8()?);
        let flags = r.u8()?;
        let _base_def = r.u16()?;
        let def = r.u16()?;
        let skills = [r.pt()?, r.pt()?, r.pt()?, r.pt()?];
        let res = [r.u8()? as i8, r.u8()? as i8, r.u8()? as i8, r.u8()? as i8, r.u8()? as i8];
        r.u8()?;
        if part as usize >= PARTS || slots > 3 || skills.iter().any(|s| s.tree as usize >= NT) {
            return None;
        }
        armor.push(Armor { id, part, kind, sex, rare, slots, flags, def, skills, res });
    }
    let mut jewels = Vec::with_capacity(nj as usize);
    for _ in 0..nj {
        let id = r.u16()?;
        let size = r.u8()?;
        let rank = r.u8()?;
        let (pos, neg) = (r.pt()?, r.pt()?);
        if !(1..=3).contains(&size) || pos.tree as usize >= NT || neg.tree as usize >= NT {
            return None;
        }
        jewels.push(Jewel { id, size, rank, pos, neg });
    }
    let mut charm_kinds = Vec::new();
    for _ in 0..r.u8()? {
        let (second_threshold, n1, n2) = (r.u8()?, r.u8()?, r.u8()?);
        r.u8()?;
        let mut list = |n: u8| -> Option<Vec<CharmSkill>> {
            (0..n).map(|_| Some(CharmSkill { tree: r.u16()?, lo: r.u8()?, hi: r.u8()? })).collect()
        };
        let first = list(n1)?;
        let second = list(n2)?;
        let mut slots = [[0u8; 3]; 20];
        for row in slots.iter_mut() {
            *row = [r.u8()?, r.u8()?, r.u8()?];
        }
        if first.iter().chain(&second).any(|s| s.tree as usize >= NT || s.hi == 0) {
            return None;
        }
        charm_kinds.push(CharmKind { second_threshold, first, second, slots });
    }
    Some(GameData { skills, armor, jewels, charm_kinds })
}
