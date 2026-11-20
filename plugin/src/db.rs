//! Game data (engine simdata.bin) and lookups on top of it. Same role as the preview's model.Database.
//! Inventory (equipment box, charms, equipped, items) is read from game memory (inventory.rs).
//! The inventory is only changed and read while holding the shared lock (lib STATE).
use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use skillsim_engine::{
    data::{self, GameData},
    types::*,
};

pub struct Db {
    pub gd: &'static GameData,
    armor_index: BTreeMap<u16, u16>,
    jewel_index: BTreeMap<u16, u16>,
    skill_index: BTreeMap<u16, u16>,
    /// Tree -> skills of that tree (indices)
    by_tree: BTreeMap<u16, Vec<u16>>,
    /// Trees with negative skills
    pub negative_trees: Vec<u16>,
    /// Armor not made at the smithy (no recipe): obtained as a reward. Unowned ones are left out of searches unless allowed.
    pub reward_armor: Vec<u16>,
    /// Selectable skill ids per category tab (positive activation points)
    pub categories: Vec<Vec<u16>>,
    inv: core::cell::UnsafeCell<Inv>,
}
/// 'No equipment X' (waist): the database's entry for an empty slot, not real armor. Never searched.
pub const PLACEHOLDER_ARMOR: u16 = 3964;

// The inventory is only handled inside the shared lock
unsafe impl Sync for Db {}

#[derive(Default, Clone, PartialEq)]
pub struct Inv {
    /// Owned armor ids (equipment box)
    pub owned: alloc::collections::BTreeSet<u16>,
    /// Owned charms (equipment box order)
    pub charms: Vec<Charm>,
    /// Currently equipped: armor id per part (0 = none), charm index
    pub equipped_armor: [u16; PARTS],
    pub equipped_charm: Option<usize>,
    /// Equipped weapon kind (7-21 as in the equipment box, 0 = none)
    pub equipped_weapon: u8,
    /// Hunter gender: 1 male, 2 female, 0 unknown (searched as male)
    pub sex: u8,
    /// Decorations in the equipped gear (decoration id, in the torso), weapon and charm included
    pub equipped_jewels: Vec<(u16, u8)>,
    /// Item box + pouch quantities (game item number -> count)
    pub items: BTreeMap<u16, u32>,
    /// Was it read from the game
    pub loaded: bool,
}

impl Db {
    pub fn load() -> Option<Self> {
        let gd: &'static GameData = Box::leak(Box::new(data::parse(crate::SIMDATA)?));
        let armor_index = gd.armor.iter().enumerate().map(|(i, a)| (a.id, i as u16)).collect();
        let jewel_index = gd.jewels.iter().enumerate().map(|(i, j)| (j.id, i as u16)).collect();
        let skill_index = gd.skills.iter().enumerate().map(|(i, s)| (s.id, i as u16)).collect();
        let mut by_tree: BTreeMap<u16, Vec<u16>> = BTreeMap::new();
        for (i, s) in gd.skills.iter().enumerate() {
            by_tree.entry(s.tree).or_default().push(i as u16);
        }
        let mut negative_trees: Vec<u16> = gd.skills.iter().filter(|s| s.threshold < 0).map(|s| s.tree).collect();
        negative_trees.sort();
        negative_trees.dedup();
        let categories = crate::generated::data::CATEGORIES
            .iter()
            .map(|(_, ranges)| {
                gd.skills
                    .iter()
                    .filter(|s| s.threshold > 1 && ranges.iter().any(|&(a, b)| a <= s.tree && s.tree <= b))
                    .map(|s| s.id)
                    .collect()
            })
            .collect();
        // Armor without a smithy recipe: rewards of quests, requests or the arena (47 pieces, none in the game's smithy list)
        let reward_armor = gd
            .armor
            .iter()
            .map(|a| a.id)
            .filter(|&id| id != PLACEHOLDER_ARMOR)
            .filter(|&id| {
                let rows = crate::generated::data::ARMOR_COST;
                rows.binary_search_by_key(&id, |r| r.0).map_or(true, |i| rows[i].3 == 0)
            })
            .collect();
        Some(Self {
            gd,
            reward_armor,
            armor_index,
            jewel_index,
            skill_index,
            by_tree,
            negative_trees,
            categories,
            inv: Default::default(),
        })
    }

    pub fn inv(&self) -> &Inv {
        unsafe { &*self.inv.get() }
    }
    /// Call only while holding the shared lock (the draw and compute threads also read it only inside the lock)
    pub fn set_inv(&self, inv: Inv) {
        unsafe { *self.inv.get() = inv }
    }

    /// Armor id -> original array index used by the engine
    pub fn armor_pos(&self, id: u16) -> Option<u16> {
        self.armor_index.get(&id).copied()
    }

    pub fn armor(&self, id: u16) -> Option<&'static Armor> {
        self.armor_index.get(&id).map(|&i| &self.gd.armor[i as usize])
    }
    pub fn jewel(&self, id: u16) -> Option<&'static Jewel> {
        self.jewel_index.get(&id).map(|&i| &self.gd.jewels[i as usize])
    }
    pub fn skill(&self, id: u16) -> Option<&'static Skill> {
        self.skill_index.get(&id).map(|&i| &self.gd.skills[i as usize])
    }
    pub fn tree_skills(&self, tree: u16) -> impl Iterator<Item = &'static Skill> + '_ {
        let gd = self.gd;
        self.by_tree.get(&tree).into_iter().flatten().map(move |&i| &gd.skills[i as usize])
    }

    /// Tree totals -> active skills (the highest level per tree). For negatives, the lowest level.
    pub fn active_skills(&self, totals: &BTreeMap<u16, i16>) -> Vec<&'static Skill> {
        let mut out = Vec::new();
        for (&tree, &total) in totals {
            let pos = self.tree_skills(tree).filter(|s| s.threshold > 1 && s.threshold as i16 <= total).max_by_key(|s| s.threshold);
            let neg = self.tree_skills(tree).filter(|s| s.threshold < 0 && total <= s.threshold as i16).min_by_key(|s| s.threshold);
            if let Some(s) = pos.or(neg) {
                out.push(s);
            }
        }
        out
    }

    pub fn is_torso_up(a: &Armor) -> bool {
        a.skills.iter().any(|s| s.tree == TREE_TORSO_UP && s.pt > 0)
    }

    /// Armor crafting materials [(item id, game item number, quantity)] and price
    /// Can this armor be crafted now: every material is in the item box + pouch in the needed quantity
    /// (money is not checked; armor without known materials counts as not craftable)
    pub fn craftable(&self, id: u16) -> bool {
        let (mats, _) = self.cost(id);
        let items = &self.inv().items;
        !mats.is_empty() && mats.iter().all(|&(_, game, qty)| game != 0 && items.get(&game).copied().unwrap_or(0) >= qty as u32)
    }

    pub fn cost(&self, id: u16) -> (&'static [(u16, u16, u8)], u32) {
        let rows = crate::generated::data::ARMOR_COST;
        match rows.binary_search_by_key(&id, |r| r.0) {
            Ok(i) => {
                let (_, price, start, n) = rows[i];
                (&crate::generated::data::MATERIALS[start as usize..start as usize + n as usize], price)
            }
            Err(_) => (&[], 0),
        }
    }
}
