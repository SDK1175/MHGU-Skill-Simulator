//! Adds a 'Skill Simulator' entry at the end of the village / gathering hall start menu's top list (the "Menu 1/2"
//! window). That menu has 16 entries on 8-row pages, so ours is the first row of a new page 3. The field (quest) menu is
//! left alone: there the last row is 'Abandon Quest', which players reach by habit (page 2, bottom row), so nothing may
//! follow it (user decision 2026-11-21; open the window from the Forge menu there). Until 2026-11-21 the entry was a 7th
//! row in Hunter's Notes, which another mod, WishFinder, also uses.
//! Research (2026-11-21, static analysis of 1.4.0):
//!  - Menu object (vtable town 0x177e424, field 0x177aeb8): +0x48 state (-1 closed, 0 top list, 1 submenu),
//!    +0xa8/+0xac top name array / count (8 bytes: u8 selectable, u32 UTF-8 text), +0xc0/+0xc4 top description array /
//!    count (12 bytes: vtable, u8 kind, u8, u16 message file, u16 message id, u16 0xFFFF), +0xd8 top list widget
//!    (+0xd cursor, +0xe rows per page, +0x10 page, +0x12 page count, +0x14 total).
//!  - The constructor allocates exactly 16 (field 13) names with the game allocator; the destructor frees them with it.
//!    Init (vtable 18: town 0x78f784, field 0x677d0c) sets the list total (0x67006c) and resets the cursor; the top fill
//!    (vtable 37: town 0x791408, field 0x678e80) writes the names and descriptions on every open.
//!  - The list drawing reads the total, page and names every frame (0x62c62c / 0x62c4d4), so a longer array and total
//!    show on their own; nothing saves or restores the cursor.
//!  - The selected top index is page * rows + cursor (0x66f8f8): it asks 'selectable' (vtable 36: town 0x7918cc, field
//!    0x6785f0), then opens a submenu or runs the entry. Unknown indices do nothing (town > 15, field 13).
//! The selection is not passed to the game: while the cursor is on our entry, A is blocked and our window opens
//! (input.rs). As a backstop the selectable query answers 'no' for our entry, so the game can never act on it.
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

const QUEST_VTABLE: u32 = 0x177aeb8;
const LOBBY_VTABLE: u32 = 0x177e424;
/// Top index of Hunter's Notes: our entry follows its selectable flag and its text tells the shown language
const NOTES: u32 = 5;
/// Message id that returns our description text (an id not in StartMenuMsg). Reserved: 0x7FF0 ours; Wishlist uses
/// 0x7FF1 (7th row) and 0x7FF2-0x7FF3 (side panel rows). Same 0x55BAC0 hook chain, each intercepts only its own ids.
const DESC_ID: u32 = 0x7ff0;
const MSG_FILE: u32 = 9;
/// Game allocator descriptor (GOT entry) used for the menu arrays; 0x7aa47c(descriptor) -> allocator
const ALLOC_GOT: u32 = 0x1838b5c;
/// Entries the game's village / hall constructor allocates (0x78eff4). Only an array of exactly this size is known to be
/// the game's own allocation, so only such an array is freed when we replace it.
const GAME_TOWN_COUNT: u32 = 16;

static BASE: AtomicU32 = AtomicU32::new(0);
static OWNER: AtomicU32 = AtomicU32::new(0);
/// The cursor is on our entry (A then opens the window instead of going to the game)
pub static HOT: AtomicBool = AtomicBool::new(false);
/// Our entry was chosen: lib opens the window
pub static OPEN_REQUEST: AtomicBool = AtomicBool::new(false);
/// Text language (judged from the characters of the game's menu text)
static LANG: AtomicU32 = AtomicU32::new(0);
/// Our top index in the village / hall menu (u32::MAX until the first one is set up): the entry count found then,
/// so we sit after anything another mod appended before us
static TOWN_INDEX: AtomicU32 = AtomicU32::new(u32::MAX);

fn valid(p: u32) -> bool {
    (0x10000..0xeffff000).contains(&p)
}
unsafe fn rd(p: u32) -> u32 {
    unsafe { core::ptr::read_unaligned(p as *const u32) }
}
unsafe fn rb(p: u32) -> u8 {
    unsafe { core::ptr::read(p as *const u8) }
}
unsafe fn rh(p: u32) -> u16 {
    unsafe { core::ptr::read_unaligned(p as *const u16) }
}

/// Some(true) village / gathering hall menu, Some(false) field menu, None anything else
fn menu_kind(owner: u32) -> Option<bool> {
    let base = BASE.load(Ordering::Relaxed);
    if !valid(owner) {
        return None;
    }
    match unsafe { rd(owner) }.wrapping_sub(base) {
        LOBBY_VTABLE => Some(true),
        QUEST_VTABLE => Some(false),
        _ => None,
    }
}
/// Our top index in this menu, if it is a village / hall menu that has been set up (none in the field menu)
fn our_index(owner: u32) -> Option<u32> {
    if menu_kind(owner) != Some(true) {
        return None;
    }
    let idx = TOWN_INDEX.load(Ordering::Relaxed);
    (idx != u32::MAX && unsafe { rd(owner + 0xac) } > idx).then_some(idx)
}

// Name (drawn with the game font) and description. 0 English, 1 Simplified Chinese, 2 Traditional Chinese, 3 Japanese, 4 Korean, 5 German, 6 French, 7 Italian, 8 Spanish
pub static LABELS: [&core::ffi::CStr; 9] = [
    c"Skill Simulator",
    c"技能模拟器",
    c"技能模擬器",
    c"スキルシミュレーター",
    c"스킬 시뮬레이터",
    c"Skill-Simulator",
    c"Simulateur de talents",
    c"Simulatore di abilità",
    c"Simulador de habilidades",
];
static DESCS: [&core::ffi::CStr; 9] = [
    c"Find armor sets that activate\nthe skills you want.",
    c"寻找能发动想要的技能的防具组合。",
    c"尋找能發動想要的技能的防具組合。",
    c"欲しいスキルが発動する\n装備の組み合わせを探します。",
    c"원하는 스킬이 발동하는\n장비 조합을 찾습니다.",
    c"Sucht Rüstungssets, die die\ngewünschten Fähigkeiten aktivieren.",
    c"Trouve les ensembles d'armure qui\nactivent les talents voulus.",
    c"Trova i set di armature che\nattivano le abilità desiderate.",
    c"Busca conjuntos de armadura que\nactiven las habilidades deseadas.",
];

/// Pick the language from the characters of the menu text the game shows now. The Korean patch replaces the English slot with Korean,
/// so look at the real text rather than the game language setting (to avoid characters missing from the game font).
/// For Latin text, follow the plugin language (when it is a Latin-script one).
fn detect(text: u32, plugin_lang: usize) -> usize {
    if !valid(text) {
        return 0;
    }
    let mut v = alloc::vec::Vec::new();
    for i in 0..64 {
        let b = unsafe { rb(text + i) };
        if b == 0 {
            break;
        }
        v.push(b);
    }
    let s = alloc::string::String::from_utf8_lossy(&v);
    if s.chars().any(|c| ('\u{ac00}'..='\u{d7a3}').contains(&c)) {
        return 4;
    }
    if s.chars().any(|c| ('\u{3040}'..='\u{30ff}').contains(&c)) {
        return 3;
    }
    if s.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)) {
        return if plugin_lang == 2 { 2 } else { 1 };
    }
    if matches!(plugin_lang, 0 | 5 | 6 | 7 | 8) { plugin_lang } else { 0 }
}

static PLUGIN_LANG: AtomicU32 = AtomicU32::new(0);
pub fn set_plugin_language(lang: usize) {
    PLUGIN_LANG.store(lang as u32, Ordering::Relaxed);
}

/// The game allocator the menu arrays come from: (allocator, alloc(allocator, size, align), free(allocator, ptr))
unsafe fn allocator() -> Option<(u32, extern "C" fn(u32, u32, u32) -> u32, extern "C" fn(u32, u32))> {
    let base = BASE.load(Ordering::Relaxed);
    unsafe {
        let desc = rd(base + ALLOC_GOT);
        if !valid(desc) {
            return None;
        }
        let get: extern "C" fn(u32) -> u32 = core::mem::transmute(base + 0x7aa47c);
        let a = get(desc);
        if !valid(a) || !valid(rd(a)) {
            return None;
        }
        let vt = rd(a);
        Some((a, core::mem::transmute(rd(vt + 0x1c)), core::mem::transmute(rd(vt + 0x34))))
    }
}

/// Our entry in this menu: grow the name and description arrays to hold our index and keep the list total.
/// Order on page 3: Wishlist first, then us. Wishlist takes its index (the count it sees) in the init hook, so we take
/// ours only at the first top fill (`claim`), which the top-list update (0x66f868) runs every frame before input,
/// always after every init hook in the chain. Hook order then does not matter. Once taken, the index is kept for every
/// menu object; a fresh one (village <-> hall) is grown up to it with placeholders that the owner of each slot fills.
/// Init runs again on later openings and resets the total, so the init hook only restores what is set up already.
fn set_up(this: u32, claim: bool) {
    if menu_kind(this) != Some(true) {
        return;
    }
    let base = BASE.load(Ordering::Relaxed);
    unsafe {
        let count = rd(this + 0xac);
        let names = rd(this + 0xa8);
        if !valid(names) || count == 0 || count > 64 {
            return;
        }
        let mut idx = TOWN_INDEX.load(Ordering::Relaxed);
        let claimed = idx == u32::MAX;
        if claimed {
            if !claim {
                return;
            }
            idx = count;
            TOWN_INDEX.store(idx, Ordering::Relaxed);
        }
        let n = idx + 1;
        let list = this + 0xd8;
        if count >= n && rd(this + 0xc4) >= n && rh(list + 0x14) as u32 >= n {
            return; // set up already (the fill hook runs every frame)
        }
        // Names: a new array from the same allocator, the old entries copied, ours pointing at a valid text right away
        if count < n {
            let Some((a, alloc, free)) = allocator() else { return };
            let fresh = alloc(a, n * 8, 0x10);
            if !valid(fresh) {
                return;
            }
            core::ptr::copy_nonoverlapping(names as *const u8, fresh as *mut u8, (count * 8) as usize);
            // Slots before ours belong to mods that set up earlier (their fill writes them): an empty text until then
            for i in count..n {
                let text = if i == idx { LABELS[0].as_ptr() } else { c"".as_ptr() };
                core::ptr::write((fresh + i * 8) as *mut u8, 0);
                core::ptr::write_unaligned((fresh + i * 8 + 4) as *mut u32, text as u32);
            }
            core::ptr::write_unaligned((this + 0xa8) as *mut u32, fresh);
            core::ptr::write_unaligned((this + 0xac) as *mut u32, n);
            // Free the old array only if it is the game's own (16 entries from the game allocator). A different size
            // means another mod replaced it already, possibly from its own heap: freeing that with the game allocator
            // could corrupt memory, so it is left alone (about 130 bytes per menu object).
            if count == GAME_TOWN_COUNT {
                free(a, names);
            } else {
                info!("[skill-sim] start menu: names array already replaced by another mod ({} entries); old one not freed", count);
            }
        }
        // Descriptions: the game's own resize (0x625138, default element from 0x54f9c0)
        if rd(this + 0xc4) < n {
            let mut tmpl = [0u32; 3];
            let make: extern "C" fn(u32) = core::mem::transmute(base + 0x54f9c0);
            make(tmpl.as_mut_ptr() as u32);
            let resize: extern "C" fn(u32, u32, u32) = core::mem::transmute(base + 0x625138);
            resize(this + 0xc0, n, tmpl.as_ptr() as u32);
        }
        if rd(this + 0xc4) < n {
            return; // the resize failed: keep the total as the game set it
        }
        // List total (pages are recounted); init set it back to the game's count
        if (rh(list + 0x14) as u32) < n {
            let set_total: extern "C" fn(u32, u32, u32) = core::mem::transmute(base + 0x54ee34);
            set_total(list, n, rd(list + 0x1c));
        }
        if claimed || count < n {
            info!(
                "[skill-sim] start menu (village/hub): our entry at index {} (found {} entries), {} entries, {} rows per page, {} pages",
                idx,
                count,
                rh(list + 0x14),
                rb(list + 0xe),
                rh(list + 0x12)
            );
        }
    }
}

/// After the top list was filled (on every open): our name and description
fn after_fill(this: u32) {
    let Some(idx) = our_index(this) else { return };
    unsafe {
        let names = rd(this + 0xa8);
        let descs = rd(this + 0xc0);
        if !valid(names) || !valid(descs) || rd(this + 0xc4) <= idx || rd(this + 0xac) <= NOTES {
            return;
        }
        let lang = detect(rd(names + NOTES * 8 + 4), PLUGIN_LANG.load(Ordering::Relaxed) as usize);
        LANG.store(lang as u32, Ordering::Relaxed);
        let e = names + idx * 8;
        // Selectable like Hunter's Notes (dimmed where the game dims it)
        core::ptr::write(e as *mut u8, rb(names + NOTES * 8));
        core::ptr::write_unaligned((e + 4) as *mut u32, LABELS[lang].as_ptr() as u32);
        // Description: message file 9, our id (the message read hook returns our text)
        let d = descs + idx * 12;
        core::ptr::write((d + 4) as *mut u8, 0);
        core::ptr::write_unaligned((d + 6) as *mut u16, MSG_FILE as u16);
        core::ptr::write_unaligned((d + 8) as *mut u16, DESC_ID as u16);
        core::ptr::write_unaligned((d + 10) as *mut u16, 0xffff);
    }
}

/// The game's 'is this top entry selectable' query: never for ours (our window opens from the input hook instead)
fn can_pick(this: u32, idx: u32) -> Option<u32> {
    (our_index(this) == Some(idx)).then_some(0)
}

#[forge::hook(offset = 0x66ed24)]
fn update_start_menu(owner: u32) -> u32 {
    // Open menus only (a closed menu has state +0x48 = -1): the village, gathering hall and field have separate menu objects
    if menu_kind(owner).is_some() && unsafe { rd(owner + 0x48) } != u32::MAX {
        OWNER.store(owner, Ordering::Relaxed);
    }
    original!(owner)
}

#[forge::hook(offset = 0x78f784)]
fn init_town(this: u32) -> u32 {
    let r = original!(this);
    set_up(this, false);
    r
}
#[forge::hook(offset = 0x791408)]
fn fill_town(this: u32) -> u32 {
    let r = original!(this);
    set_up(this, true);
    after_fill(this);
    r
}
#[forge::hook(offset = 0x7918cc)]
fn pick_town(this: u32, idx: u32) -> u32 {
    can_pick(this, idx).unwrap_or_else(|| original!(this, idx))
}

// The description box is drawn by the help bar below: cursor handling (0x66f744 for the top list) -> 0x65e5bc -> 0x6429a4
// calls this function (manager, kind, file, id) -> text address with the description slot's (u8 +4 kind, u16 +6 file,
// u16 +8 id). Only our description id is intercepted.
#[forge::hook(offset = 0x55bac0)]
fn message(mgr: u32, kind: u32, file: u32, id: u32) -> u32 {
    if kind == 0 && file == MSG_FILE && id == DESC_ID {
        return DESCS[LANG.load(Ordering::Relaxed) as usize % 9].as_ptr() as u32;
    }
    original!(mgr, kind, file, id)
}

pub fn install(base: u32) {
    BASE.store(base, Ordering::Relaxed);
    forge::install_hook!(base, update_start_menu);
    forge::install_hook!(base, init_town);
    forge::install_hook!(base, fill_town);
    forge::install_hook!(base, pick_town);
    forge::install_hook!(base, message);
}

/// Every frame (game thread): is the top-list cursor on our entry
pub fn update() {
    let owner = OWNER.load(Ordering::Relaxed);
    let hot = our_index(owner).is_some_and(|idx| unsafe {
        let list = owner + 0xd8;
        let at = rh(list + 0x10) as u32 * rb(list + 0xe) as u32 + rb(list + 0xd) as u32;
        rd(owner + 0x48) == 0
            && at == idx
            && valid(rd(owner + 0xa8))
            && rb(rd(owner + 0xa8) + idx * 8) == 1 // dimmed (not selectable): the game handles A as 'cannot select'
    });
    HOT.store(hot, Ordering::Release);
}
