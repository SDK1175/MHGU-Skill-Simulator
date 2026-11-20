//! Adds a 'Skill Simulator' entry below 'Manual' in the start menu > Hunter's Notes submenu.
//! Research (2026-11-18, diagnostic logs + static analysis):
//!  - Menu object: this+0x48 state (1 = choosing in a submenu), +0x4C submenu index (5 = Hunter's Notes),
//!    +0xB0 name array (8 bytes: u8 selectable, u32 UTF-8 text address), +0xB4 slot count (7),
//!    +0xC8 description array (12 bytes: u32 object, u8, u8, u16 message file (9), u16 message id, u16 0xFFFF), +0xCC slot count,
//!    +0x109 cursor, +0x10A item count (list widget this+0xFC).
//!  - Opening a submenu: field 0x6786D0, village/gathering hall 0x792490 (for 5, item count 6 via 0x670098(this, n, 1)).
//!  - Filling the text: field 0x6793D0, village/gathering hall 0x7918D8.
//! The selection is not passed to the game: while the cursor is on our entry, A is blocked and our window opens (input.rs).
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

const QUEST_VTABLE: u32 = 0x177aeb8;
const LOBBY_VTABLE: u32 = 0x177e424;
const NOTES: u32 = 5;
const OUR_INDEX: u32 = 6;
/// Message id that returns our description text (an id not in StartMenuMsg). Reserved: 0x7FF0 ours; Wishlist uses
/// 0x7FF1 (7th row) and 0x7FF2-0x7FF3 (side panel rows). Same 0x55BAC0 hook chain, each intercepts only its own ids.
const DESC_ID: u32 = 0x7ff0;
const MSG_FILE: u32 = 9;

static BASE: AtomicU32 = AtomicU32::new(0);
static OWNER: AtomicU32 = AtomicU32::new(0);
/// The cursor is on our entry (A then opens the window instead of going to the game)
pub static HOT: AtomicBool = AtomicBool::new(false);
/// Our entry was chosen: lib opens the window
pub static OPEN_REQUEST: AtomicBool = AtomicBool::new(false);
/// Text language (judged from the characters of the game's menu text)
static LANG: AtomicU32 = AtomicU32::new(0);

fn valid(p: u32) -> bool {
    (0x10000..0xeffff000).contains(&p)
}
unsafe fn rd(p: u32) -> u32 {
    unsafe { core::ptr::read_unaligned(p as *const u32) }
}
unsafe fn rb(p: u32) -> u8 {
    unsafe { core::ptr::read(p as *const u8) }
}

fn is_menu(owner: u32) -> bool {
    let base = BASE.load(Ordering::Relaxed);
    valid(owner) && matches!(unsafe { rd(owner) }.wrapping_sub(base), QUEST_VTABLE | LOBBY_VTABLE)
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

/// After opening a submenu: if it is Hunter's Notes (6 items), grow it to 7. Only when the arrays have 7 or more slots.
/// The submenu shows 7 rows at most (an 8th row crashed, 2026-11-20). With a mod menu host (Wishlist) the 7th row is its.
fn after_open(this: u32, id: u32, opened: u32) {
    if opened == 0 || id != NOTES || crate::mod_menu::yield_row() || !is_menu(this) {
        return;
    }
    unsafe {
        if rb(this + 0x10a) as u32 != 6 || rd(this + 0xb4) < 7 || rd(this + 0xcc) < 7 {
            return;
        }
        let set_count: extern "C" fn(u32, u32, u32) = core::mem::transmute(BASE.load(Ordering::Relaxed) + 0x670098);
        set_count(this, 7, 1);
    }
}

/// After filling the text: our name and description in the 7th slot (not when a mod menu host owns the row)
fn after_fill(this: u32) {
    if crate::mod_menu::yield_row() || !is_menu(this) {
        return;
    }
    unsafe {
        if rd(this + 0x4c) != NOTES || rb(this + 0x10a) as u32 != 7 || rd(this + 0xb4) < 7 || rd(this + 0xcc) < 7 {
            return;
        }
        let names = rd(this + 0xb0);
        let descs = rd(this + 0xc8);
        if !valid(names) || !valid(descs) {
            return;
        }
        let lang = detect(rd(names + 5 * 8 + 4), PLUGIN_LANG.load(Ordering::Relaxed) as usize);
        LANG.store(lang as u32, Ordering::Relaxed);
        let e = names + OUR_INDEX * 8;
        // The selectable flag follows 'Manual': in the field the game sets it to 0, so it is dimmed and cannot be chosen
        core::ptr::write(e as *mut u8, rb(names + 5 * 8));
        core::ptr::write_unaligned((e + 4) as *mut u32, LABELS[lang].as_ptr() as u32);
        // Description: reuse the previous slot's object and change only the message id (the message read hook returns our text)
        let src = descs + 5 * 12;
        let dst = descs + OUR_INDEX * 12;
        core::ptr::write_unaligned(dst as *mut u32, rd(src));
        core::ptr::write((dst + 4) as *mut u8, 0);
        core::ptr::write((dst + 5) as *mut u8, rb(src + 5));
        core::ptr::write_unaligned((dst + 6) as *mut u16, MSG_FILE as u16);
        core::ptr::write_unaligned((dst + 8) as *mut u16, DESC_ID as u16);
        core::ptr::write_unaligned((dst + 10) as *mut u16, 0xffff);
    }
}

#[forge::hook(offset = 0x66ed24)]
fn update_start_menu(owner: u32) -> u32 {
    // Open menus only (a closed menu has state +0x48 = -1): the village, gathering hall and field have separate menu objects
    if is_menu(owner) && unsafe { rd(owner + 0x48) } != u32::MAX {
        OWNER.store(owner, Ordering::Relaxed);
    }
    original!(owner)
}

#[forge::hook(offset = 0x6786d0)]
fn open_field(this: u32, id: u32) -> u32 {
    let r = original!(this, id);
    after_open(this, id, r);
    r
}
#[forge::hook(offset = 0x792490)]
fn open_town(this: u32, id: u32) -> u32 {
    let r = original!(this, id);
    after_open(this, id, r);
    r
}
#[forge::hook(offset = 0x6793d0)]
fn fill_field(this: u32) -> u32 {
    let r = original!(this);
    after_fill(this);
    r
}
#[forge::hook(offset = 0x7918d8)]
fn fill_town(this: u32) -> u32 {
    let r = original!(this);
    after_fill(this);
    r
}

// The description box is drawn by the help bar below: cursor handling (0x66fd7c etc.) -> 0x65e5bc -> 0x6429a4 calls this function
// (manager, kind, file, id) -> text address with the description slot's (u8 +4 kind, u16 +6 file, u16 +8 id).
// (It does not go through 0x55bc4c used by the name slot: 2026-11-18 static analysis.) Only our description id is intercepted.
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
    forge::install_hook!(base, open_field);
    forge::install_hook!(base, open_town);
    forge::install_hook!(base, fill_field);
    forge::install_hook!(base, fill_town);
    forge::install_hook!(base, message);
}

/// Every frame (game thread): is the cursor on our entry
pub fn update() {
    let owner = OWNER.load(Ordering::Relaxed);
    let hot = !crate::mod_menu::yield_row()
        && is_menu(owner)
        && unsafe {
            rd(owner + 0x48) == 1
                && rd(owner + 0x4c) == NOTES
                && rb(owner + 0x10a) == 7
                && rb(owner + 0x109) as u32 == OUR_INDEX
                && valid(rd(owner + 0xb0))
                && rb(rd(owner + 0xb0) + OUR_INDEX * 8) == 1 // If not selectable (field), the game handles A as 'cannot select'
        };
    HOT.store(hot, Ordering::Release);
}
