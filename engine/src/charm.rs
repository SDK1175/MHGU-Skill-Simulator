//! Charm generation rules. Estimates "the chance to get this charm or better from an appraisal".
//!
//! Generation steps (quest reward appraisal, from the apmnnn/mhxx-rng analysis):
//! 1. Pick the first skill from the candidates and its points from min..max.
//! 2. If the roll is at or above the threshold a second skill is added. Positive/negative 50/50; if positive, 1..max.
//! 3. The slot count is decided by fill = floor(10*p1/max1 + 10*p2/max2).
//!
//! The chance is an estimate for ranking. The effect of the unrequested skill on the slot roll is ignored.
use crate::types::*;
use alloc::vec::Vec;

/// One charm kind, reduced to the trees of interest for fast lookups.
#[derive(Clone)]
pub struct KindView {
    second_rate: f64,
    n1: f64,
    n2: f64,
    /// First skill per tree (min, max)
    first: Vec<Option<(u8, u8)>>,
    /// Positive max of the second skill per tree
    second: Vec<Option<u8>>,
    slots: [[u8; 3]; 20],
    /// Average chance of s or more slots when the first skill is anything
    any_first: [f64; 4],
}

impl KindView {
    /// The order of `trees` becomes the index used by later lookups.
    pub fn new(kind: &CharmKind, trees: &[u16]) -> Self {
        let find = |list: &[CharmSkill], t: u16| list.iter().find(|s| s.tree == t).copied();
        let mut v = KindView {
            second_rate: (100.0 - kind.second_threshold as f64) / 100.0,
            n1: kind.first.len() as f64,
            n2: kind.second.len() as f64,
            first: trees.iter().map(|&t| find(&kind.first, t).map(|s| (s.lo, s.hi))).collect(),
            second: trees.iter().map(|&t| find(&kind.second, t).map(|s| s.hi)).collect(),
            slots: kind.slots,
            any_first: [0.0; 4],
        };
        for s in 0..4 {
            let mut sum = 0.0;
            for f in &kind.first {
                let range = (f.hi - f.lo + 1) as f64;
                for p in f.lo..=f.hi {
                    sum += v.slot_rate(10 * p as u32 / f.hi.max(1) as u32, s) / range;
                }
            }
            v.any_first[s as usize] = if kind.first.is_empty() { 0.0 } else { sum / v.n1 };
        }
        v
    }

    /// Max points tree i can have in the first slot (0 if none).
    pub fn max_first(&self, i: usize) -> u8 {
        self.first[i].map_or(0, |x| x.1)
    }

    /// Max points tree i can have in the second slot (0 if none).
    pub fn max_second(&self, i: usize) -> u8 {
        self.second[i].unwrap_or(0)
    }

    /// Chance of s or more slots at fill value c.
    fn slot_rate(&self, c: u32, s: u8) -> f64 {
        if s == 0 {
            return 1.0;
        }
        let t = self.slots[(c.clamp(1, 20) - 1) as usize];
        (100.0 - t[(s.min(3) - 1) as usize] as f64) / 100.0
    }

    /// Chance of tree i with pi or more in the first slot, tree j with pj or more in the second, and s or more slots.
    /// i, j are indices into the trees given to `new`. None means anything goes in that slot.
    pub fn chance(&self, i: Option<(usize, u8)>, j: Option<(usize, u8)>, s: u8) -> f64 {
        if s > 3 {
            return 0.0;
        }
        match (i, j) {
            (None, None) => self.any_first[s as usize],
            (Some((i, pi)), None) => {
                let Some((lo, hi)) = self.first[i] else { return 0.0 };
                let range = (hi - lo + 1) as f64;
                let sum: f64 = (pi.max(lo)..=hi).map(|p| self.slot_rate(10 * p as u32 / hi as u32, s)).sum();
                sum / range / self.n1
            }
            (None, Some((j, pj))) => {
                let Some(max2) = self.second[j] else { return 0.0 };
                if pj > max2 {
                    return 0.0;
                }
                let hit = (max2 - pj.max(1) + 1) as f64 / max2 as f64;
                self.second_rate / self.n2 / 2.0 * hit * self.any_first[s as usize]
            }
            (Some((i, pi)), Some((j, pj))) => {
                let (Some((lo, hi)), Some(max2)) = (self.first[i], self.second[j]) else { return 0.0 };
                let range = (hi - lo + 1) as f64;
                let mut sum = 0.0;
                for p1 in pi.max(lo)..=hi {
                    for p2 in pj.max(1)..=max2 {
                        let c = (10 * p1 as u32 * max2 as u32 + 10 * p2 as u32 * hi as u32) / (hi as u32 * max2 as u32);
                        sum += self.slot_rate(c, s);
                    }
                }
                sum / range / max2 as f64 / self.n1 / self.n2 / 2.0 * self.second_rate
            }
        }
    }
}

/// The easiest charm kind (index) and its chance. None if no kind can give it.
pub fn best_kind(kinds: &[CharmKind], need: &Charm) -> Option<(usize, f64)> {
    let trees = [need.skills[0].tree, need.skills[1].tree];
    let pick = |n: usize| (need.skills[n].tree != 0 && need.skills[n].pt > 0).then(|| (n, need.skills[n].pt as u8));
    kinds
        .iter()
        .enumerate()
        .map(|(i, k)| (i, KindView::new(k, &trees).chance(pick(0), pick(1), need.slots)))
        .filter(|x| x.1 > 0.0)
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

/// Charm rarity. Decided by the charm kind (0 timeworn, 1 old, 2 shining, 3 mystery) and slots x2 + fill
/// (the rare function of apmnnn/mhxx-rng). The game colors the charm icon by this rarity.
pub fn rarity(kind_index: usize, kind: &CharmKind, charm: &Charm) -> u8 {
    let first = kind.first.iter().find(|s| s.tree == charm.skills[0].tree).map_or(1, |s| s.hi.max(1)) as u32;
    let second = kind.second.iter().find(|s| s.tree == charm.skills[1].tree).map_or(1, |s| s.hi.max(1)) as u32;
    let p1 = charm.skills[0].pt.max(0) as u32;
    // A missing or negative second skill counts as 0
    let p2 = if charm.skills[1].tree == 0 { 0 } else { charm.skills[1].pt.max(0) as u32 };
    let fill = (10 * p1 * second + 10 * p2 * first) / (first * second);
    let n = charm.slots as u32 * 2 + fill;
    match kind_index {
        0 => if n >= 13 { 10 } else if n >= 8 { 9 } else { 8 },
        1 => if n >= 13 { 7 } else if n >= 8 { 6 } else { 5 },
        2 => if n >= 8 { 4 } else { 3 },
        _ => if n >= 8 { 2 } else { 1 },
    }
}

/// Chance to get the required charm (skills[0] = first slot, skills[1] = second slot, tree 0 = "anything") or better.
/// The highest value over charm kinds. 0 if no kind can give it.
pub fn chance(kinds: &[CharmKind], need: &Charm) -> f64 {
    let trees = [need.skills[0].tree, need.skills[1].tree];
    let pick = |n: usize| (need.skills[n].tree != 0 && need.skills[n].pt > 0).then(|| (n, need.skills[n].pt as u8));
    kinds
        .iter()
        .map(|k| KindView::new(k, &trees).chance(pick(0), pick(1), need.slots))
        .fold(0.0, f64::max)
}
