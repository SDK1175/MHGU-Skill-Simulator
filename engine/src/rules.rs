//! Rule engine that computes the skill points of one complete set.
//! Written independently of the search engine and used as the reference to verify search results.
//!
//! Assumptions (to be confirmed in game):
//! - A torso-up part adds the torso armor's points (including torso decorations) once more.
//! - Talisman boost doubles the charm's own points and the decorations in its slots (negatives too).
//!   Confirmed: Monster Hunter Encyclopedia wiki (護石強化) and an in-game skill detail screen (pierce jewel 3 in a charm: +8/-4).
//! - Skill points +2 is added after doubling, to every tree with non-zero points.
//! - Activation order: secret skill (total before doubling) -> talisman boost (+2 applied) -> charm doubling -> +2.
use crate::types::*;

pub struct Build<'a> {
    pub armor: [Option<&'a Armor>; PARTS],
    pub charm: Option<&'a Charm>,
    pub weapon_slots: u8,
    pub jewels: [[Option<&'a Jewel>; 3]; CONTAINERS],
}

pub struct Eval {
    pub totals: [i16; NT],
    pub secret: bool,
    pub talisman: bool,
}

impl Eval {
    pub fn has_negative_skill(&self) -> bool {
        self.totals.iter().any(|&t| t <= -10)
    }

    /// Only trees with negative skills (trees) are checked. Empty means all trees.
    pub fn has_negative_skill_in(&self, trees: &[u16]) -> bool {
        if trees.is_empty() {
            return self.has_negative_skill();
        }
        trees.iter().any(|&t| self.totals[t as usize] <= -10)
    }
}

fn add(v: &mut [i16; NT], p: Pt, mult: i16) {
    if p.tree != 0 {
        v[p.tree as usize] += p.pt as i16 * mult;
    }
}

/// None if the decoration placement does not fit the slots.
pub fn evaluate(b: &Build) -> Option<Eval> {
    for c in 0..CONTAINERS {
        let cap = match c {
            WEAPON => b.weapon_slots,
            CHARM => b.charm.map_or(0, |x| x.slots),
            _ => b.armor[c].map_or(0, |a| a.usable_slots()),
        };
        let used: u8 = b.jewels[c].iter().flatten().map(|j| j.size).sum();
        if used > cap {
            return None;
        }
    }
    let copies = (0..PARTS)
        .filter(|&p| p != TORSO && b.armor[p].is_some_and(|a| a.torso_up()))
        .count() as i16;

    let mut totals = [0i16; NT];
    for p in 0..PARTS {
        let Some(a) = b.armor[p] else { continue };
        if a.torso_up() {
            continue;
        }
        let mult = if p == TORSO { 1 + copies } else { 1 };
        for s in a.skills {
            add(&mut totals, s, mult);
        }
        for j in b.jewels[p].iter().flatten() {
            add(&mut totals, j.pos, mult);
            add(&mut totals, j.neg, mult);
        }
    }
    for c in [WEAPON, CHARM] {
        for j in b.jewels[c].iter().flatten() {
            add(&mut totals, j.pos, 1);
            add(&mut totals, j.neg, 1);
        }
    }
    if let Some(c) = b.charm {
        for s in c.skills {
            add(&mut totals, s, 1);
        }
    }

    let secret = totals[TREE_SECRET as usize] >= 10;
    let tal = totals[TREE_TALISMAN as usize];
    let talisman = tal + if secret && tal != 0 { 2 } else { 0 } >= 10;
    if talisman {
        if let Some(c) = b.charm {
            for s in c.skills {
                add(&mut totals, s, 1);
            }
        }
        for j in b.jewels[CHARM].iter().flatten() {
            add(&mut totals, j.pos, 1);
            add(&mut totals, j.neg, 1);
        }
    }
    if secret {
        for t in totals.iter_mut().skip(1) {
            if *t != 0 {
                *t += 2;
            }
        }
    }
    Some(Eval { totals, secret, talisman })
}
