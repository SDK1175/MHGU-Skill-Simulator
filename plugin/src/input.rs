//! Controller input: while the window is open, buttons are ours and are not passed to the game.
//! Follows the approach of Wishlist (our mod):
//!  - Wrap the game's pad input functions (5 HID imports) to read raw buttons, and block them there on hardware/emulators.
//!  - On Recomp, block in the menu input translation hook (+0x3DE4F8) and read directions/decide etc. from its result (with key remapping).
//!  - Other mods (Wishlist, Better Skillinfo) wrap the same functions, so chain them, and attach only once to avoid loops.
use crate::app::Button;
use core::{
    ptr,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
};

const GOT: [u32; 5] = [0x1838494, 0x1838498, 0x183849c, 0x18384a0, 0x18384a4];
// Imports not linked yet (must not be called at plugin start)
const UNRESOLVED: u32 = 0x13efd44;

static ORIGINAL: [AtomicU32; 5] = [const { AtomicU32::new(0) }; 5];
static MENU_INSTALLED: AtomicBool = AtomicBool::new(false);
/// Do we own the input because the window is open (updated by lib every frame)
pub static OWNED: AtomicBool = AtomicBool::new(false);
/// Newly pressed buttons (our button bits). Taken in update.
static EVENTS: AtomicU32 = AtomicU32::new(0);
static HID_HELD: AtomicU32 = AtomicU32::new(0);
static MENU_HELD: AtomicU32 = AtomicU32::new(0);
/// Buttons held when the window closed are not passed to the game until released
static RELEASE: AtomicU32 = AtomicU32::new(0);
static MENU_RELEASE: AtomicU32 = AtomicU32::new(0);
static NAV_LATCH: AtomicU32 = AtomicU32::new(0);
/// Our button bits currently held (for key repeat of the d-pad/stick directions)
static HELD_NOW: AtomicU32 = AtomicU32::new(0);
/// Key repeat: a held direction repeats after REPEAT_DELAY_MS, then every REPEAT_MS (like the game's menus)
const REPEAT_DELAY_MS: u64 = 380;
const REPEAT_MS: u64 = 70;
static REPEAT_START: [core::sync::atomic::AtomicU64; 4] = [const { core::sync::atomic::AtomicU64::new(0) }; 4];
static REPEAT_LAST: [core::sync::atomic::AtomicU64; 4] = [const { core::sync::atomic::AtomicU64::new(0) }; 4];

unsafe fn rd(p: u32) -> u32 {
    unsafe { ptr::read_unaligned(p as *const u32) }
}

const BUTTONS: [Button; 14] = [
    Button::Up,
    Button::Down,
    Button::Left,
    Button::Right,
    Button::A,
    Button::B,
    Button::X,
    Button::Y,
    Button::L,
    Button::R,
    Button::ZL,
    Button::ZR,
    Button::Plus,
    Button::Minus,
];
fn bit(b: Button) -> u32 {
    1 << BUTTONS.iter().position(|&x| x == b).unwrap()
}

/// HID raw buttons (A=1 B=2 X=4 Y=8 L=0x40 R=0x80 ZL=0x100 ZR=0x200 +=0x400 -=0x800,
/// d-pad left/up/right/down 1<<12.., left stick 1<<16..) -> our button bits
fn from_hid(raw: u32) -> u32 {
    let mut v = 0;
    for (mask, b) in [
        (1u32, Button::A),
        (2, Button::B),
        (4, Button::X),
        (8, Button::Y),
        (0x40, Button::L),
        (0x80, Button::R),
        (0x100, Button::ZL),
        (0x200, Button::ZR),
        (0x400, Button::Plus),
        (0x800, Button::Minus),
        ((1 << 12) | (1 << 16), Button::Left),
        ((1 << 13) | (1 << 17), Button::Up),
        ((1 << 14) | (1 << 18), Button::Right),
        ((1 << 15) | (1 << 19), Button::Down),
    ] {
        if raw & mask != 0 {
            v |= bit(b);
        }
    }
    v
}

/// Recomp's final menu word (with key remapping) -> our button bits. +/- are not in the menu word and are read from HID.
fn from_menu(word: u32) -> u32 {
    let mut v = 0;
    for (mask, b) in [
        (0x2000u32, Button::A),
        (0x4000, Button::B),
        (0x1000, Button::X),
        (0x8000, Button::Y),
        (0x80, Button::Left),
        (0x10, Button::Up),
        (0x20, Button::Right),
        (0x40, Button::Down),
        (0x100, Button::L),
        (0x200, Button::R),
        (0x400, Button::ZL),
        (0x800, Button::ZR),
    ] {
        if word & mask != 0 {
            v |= bit(b);
        }
    }
    v
}

/// Analog stick to direction bits (same threshold as Wishlist bindings::snapshot)
fn snapshot(buttons: u32, lx: i32, ly: i32) -> u32 {
    let mut v = buttons & 0xffff;
    if lx <= -18000 {
        v |= 1 << 16;
    }
    if lx >= 18000 {
        v |= 1 << 18;
    }
    if ly >= 18000 {
        v |= 1 << 17;
    }
    if ly <= -18000 {
        v |= 1 << 19;
    }
    v
}

pub fn is_recomp() -> bool {
    #[cfg(target_arch = "arm")]
    {
        use forge::sys::imgui as sys;
        unsafe extern "C" {
            fn forge_graphics_isMenuVisible() -> bool;
        }
        let a = forge_graphics_isMenuVisible as *const () as usize;
        let b = sys::igImFileOpen as *const () as usize;
        // Recomp links the two imports straight into its own HLE address range.
        [a, b].iter().all(|&p| (0x03000000..0x03100000).contains(&p) && p & 15 == 0)
    }
    #[cfg(not(target_arch = "arm"))]
    false
}

unsafe fn filter(index: usize, state: u32, id: u32) {
    let address = ORIGINAL[index].load(Ordering::Acquire);
    let call: unsafe extern "C" fn(u32, u32) = unsafe { core::mem::transmute(address) };
    unsafe { call(state, id) };
    if state < 0x10000 || unsafe { rd(state + 32) } & 1 == 0 {
        return;
    }
    let buttons = unsafe { rd(state + 8) };
    // For a guest mod's own shortcut and dialog (ext_block): the buttons before we withhold them
    crate::ext_block::set_raw(buttons);
    let raw = snapshot(buttons, unsafe { rd(state + 16) } as i32, unsafe { rd(state + 20) } as i32);
    let overlay = forge::graphics::Graphics::is_menu_visible();
    let owned = OWNED.load(Ordering::Acquire) && !overlay;
    let mine = from_hid(raw);
    let pressed = mine & !HID_HELD.swap(mine, Ordering::AcqRel);
    let recomp = is_recomp();
    if !recomp {
        HELD_NOW.store(mine, Ordering::Release);
    }
    // A while the game menu cursor is on the 'Skill Simulator' entry: open the window instead of passing it to the game
    let menu_pick = !owned && !overlay && crate::game_menu::HOT.load(Ordering::Acquire) && raw & 1 != 0;
    if menu_pick && pressed & bit(Button::A) != 0 {
        crate::game_menu::OPEN_REQUEST.store(true, Ordering::Release);
    }
    if owned {
        // Recomp reads the menu word (with key remapping). Only +/- come from HID.
        let take = if recomp { pressed & (bit(Button::Plus) | bit(Button::Minus)) } else { pressed };
        EVENTS.fetch_or(take, Ordering::AcqRel);
    }
    // Block game input at the HID level (hardware, emulators and Recomp). Recomp also blocks in the menu translation function,
    // but some input (Y hunter search, movement in the field) does not go that way, so it must be blocked here too.
    let release = RELEASE.load(Ordering::Acquire) & raw;
    let blocked = if overlay {
        RELEASE.store(0, Ordering::Release);
        0
    } else if owned {
        RELEASE.store(raw, Ordering::Release);
        u32::MAX
    } else if menu_pick {
        // Do not pass A to the game until it is released
        RELEASE.store(release | 1, Ordering::Release);
        release | 1
    } else {
        RELEASE.store(release, Ordering::Release);
        release
    };
    if blocked != 0 {
        unsafe {
            ptr::write_unaligned((state + 8) as *mut u32, buttons & !blocked);
            if blocked == u32::MAX {
                ptr::write_unaligned((state + 12) as *mut u32, 0);
                for off in [16, 20, 24, 28] {
                    ptr::write_unaligned((state + off) as *mut u32, 0);
                }
            }
        }
    }
}

// Recomp: read the final menu action after key remapping, and do not pass it to the game while open.
#[forge::hook(offset = 0x3de4f8)]
fn menu_translate(object: u32, word: u32) -> u32 {
    let overlay = forge::graphics::Graphics::is_menu_visible();
    let owned = OWNED.load(Ordering::Acquire) && !overlay;
    // So inner hooks (other mods) do not use directions: while open, direction bits are not passed inward.
    let mut latch = NAV_LATCH.load(Ordering::Acquire) & word;
    if owned {
        latch |= word & 0x00f0_00f0;
    }
    NAV_LATCH.store(latch, Ordering::Release);
    let mapped = original!(object, word & !latch);
    // Directions from the original word (folding the stick/d-pad high bits into 0xF0 like the game's translation)
    let nav = (word | ((word >> 16) & 0xf0)) & 0xf0;
    let mapped = if owned { (mapped & !0xf0) | nav } else { mapped };
    if overlay {
        MENU_RELEASE.store(0, Ordering::Release);
        return mapped;
    }
    let pressed = mapped & !MENU_HELD.swap(mapped, Ordering::AcqRel);
    HELD_NOW.store(from_menu(mapped), Ordering::Release);
    // Recomp: keyboard decide keys etc. only arrive as menu words. Decide on our entry opens the window and is not passed to the game.
    if !owned && crate::game_menu::HOT.load(Ordering::Acquire) && mapped & 0x2000 != 0 {
        if pressed & 0x2000 != 0 {
            crate::game_menu::OPEN_REQUEST.store(true, Ordering::Release);
        }
        MENU_RELEASE.fetch_or(0x2000, Ordering::AcqRel);
    }
    if owned {
        EVENTS.fetch_or(from_menu(pressed), Ordering::AcqRel);
    }
    let release = MENU_RELEASE.load(Ordering::Relaxed) & mapped;
    MENU_RELEASE.store(if owned { mapped } else { release }, Ordering::Release);
    if owned { 0 } else { mapped & !release }
}

unsafe extern "C" fn full(s: u32, id: u32) {
    unsafe { filter(0, s, id) }
}
unsafe extern "C" fn dual(s: u32, id: u32) {
    unsafe { filter(1, s, id) }
}
unsafe extern "C" fn left(s: u32, id: u32) {
    unsafe { filter(2, s, id) }
}
unsafe extern "C" fn right(s: u32, id: u32) {
    unsafe { filter(3, s, id) }
}
unsafe extern "C" fn handheld(s: u32, id: u32) {
    unsafe { filter(4, s, id) }
}

/// Check whether the hook site is original or another mod's Forge hook (ldr pc,[pc,#-4] + address, original tail).
/// Otherwise only log a warning. Installation is not blocked (a clue for finding the cause when something goes wrong).
pub fn check_site(base: u32, offset: u32, expected: &[u32]) {
    let site = base + offset;
    let words: alloc::vec::Vec<u32> = (0..expected.len() as u32).map(|i| unsafe { rd(site + i * 4) }).collect();
    if words == expected {
        return;
    }
    let target = unsafe { rd(site + 4) };
    let hooked = words[0] == 0xe51ff004 && (0x10000..0xeffff000).contains(&target) && words.get(2..) == expected.get(2..);
    if hooked {
        info!("[skill-sim] +0x{:x}: another mod hooked it first; chaining", offset);
    } else {
        warn!("[skill-sim] warning: unexpected code at +0x{:x} ({:08x?}); hooking anyway", offset, words);
    }
}

fn install_menu(base: u32) {
    if MENU_INSTALLED.load(Ordering::Acquire) {
        return;
    }
    // Even if another mod hooked first, Forge chains them (not blocked by the check).
    check_site(base, 0x3de4f8, &[0xe3a02010, 0xe3a00040, 0xe0022821, 0xe0000821]);
    forge::install_hook!(base, menu_translate);
    MENU_INSTALLED.store(true, Ordering::Release);
}

/// Attach only once. If a mod attached later already calls us, leave it (avoid loops).
fn can_attach(saved: u32, candidate: u32, unresolved: u32, own: &[u32]) -> bool {
    saved == 0 && candidate >= 0x10000 && candidate & 3 == 0 && candidate != unresolved && !own.contains(&candidate)
}

fn refresh_imports(base: u32) {
    if is_recomp() && !MENU_INSTALLED.load(Ordering::Acquire) {
        return;
    }
    let detours = [full as *const () as u32, dual as *const () as u32, left as *const () as u32, right as *const () as u32, handheld as *const () as u32];
    for (i, offset) in GOT.iter().enumerate() {
        let slot = base + offset;
        let address = unsafe { rd(slot) };
        if !can_attach(ORIGINAL[i].load(Ordering::Acquire), address, base + UNRESOLVED, &detours) {
            continue;
        }
        ORIGINAL[i].store(address, Ordering::Release);
        unsafe { ptr::write_volatile(slot as *mut u32, detours[i]) };
    }
}

/// At start and every frame: attach input methods that get linked later. True when input is ready.
pub fn retry(base: u32) -> bool {
    if is_recomp() {
        install_menu(base);
    }
    refresh_imports(base);
    (!is_recomp() || MENU_INSTALLED.load(Ordering::Acquire)) && ORIGINAL.iter().any(|p| p.load(Ordering::Acquire) != 0)
}

/// Mouse this frame (Recomp's mod pointer, console screen pixels 1280x720): position, new left / right clicks, wheel notches
pub struct Mouse {
    pub x: f32,
    pub y: f32,
    pub left: bool,
    pub right: bool,
    /// +1 wheel up, -1 wheel down, 0 none
    pub wheel: i8,
}

#[cfg(target_arch = "arm")]
mod recomp_mouse {
    use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    // Recomp's own Forge extensions (runtime/forge.c). Only called on Recomp: Forge on hardware/emulators binds
    // imports it lacks to stubs (nn::ro), and an older Recomp makes them return 0.
    unsafe extern "C" {
        fn forge_input_getMouse(x: *mut f32, y: *mut f32, buttons: *mut u32) -> bool;
        fn forge_input_setMouseFree(free: bool);
        fn forge_input_getKeyPressed(out: *mut u8, size: u32) -> bool;
    }
    static FREE: AtomicBool = AtomicBool::new(false);
    static PREV: AtomicU32 = AtomicU32::new(0);
    /// Clicks count only after every button was seen released: the click that opened the window (a mod menu row, a
    /// game menu entry) is still held, or reported, on our first frames and must not land on our window
    static ARMED: AtomicBool = AtomicBool::new(false);

    /// Free pointer while our window is open: Recomp shows the pointer, does not capture the mouse, and its clicks go
    /// only to us (not the game, not its key mapping). Called every frame on the game thread.
    pub fn set_free(free: bool) {
        if !super::is_recomp() || FREE.swap(free, Ordering::Relaxed) == free {
            return;
        }
        ARMED.store(false, Ordering::Relaxed);
        unsafe { forge_input_setMouseFree(free) };
    }

    static LAST_X: AtomicU32 = AtomicU32::new(0);
    static LAST_Y: AtomicU32 = AtomicU32::new(0);

    /// This frame: a key / mouse button / wheel notch was pressed, or the free pointer moved (keyboard-mouse input)
    pub fn kbm_input() -> bool {
        if !super::is_recomp() {
            return false;
        }
        let mut name = [0u8; 32];
        if unsafe { forge_input_getKeyPressed(name.as_mut_ptr(), name.len() as u32) } {
            return true;
        }
        let (mut x, mut y, mut buttons) = (0f32, 0f32, 0u32);
        if !FREE.load(Ordering::Relaxed) || !unsafe { forge_input_getMouse(&mut x, &mut y, &mut buttons) } {
            return false;
        }
        let (px, py) = (f32::from_bits(LAST_X.swap(x.to_bits(), Ordering::Relaxed)), f32::from_bits(LAST_Y.swap(y.to_bits(), Ordering::Relaxed)));
        (px - x).abs() + (py - y).abs() > 3.
    }

    pub fn read() -> Option<super::Mouse> {
        if !super::is_recomp() || !FREE.load(Ordering::Relaxed) {
            return None;
        }
        let (mut x, mut y, mut buttons) = (0f32, 0f32, 0u32);
        // Wheel notches come as key names ("Wheel Up" / "Wheel Down"), the first key pressed this frame
        let mut name = [0u8; 32];
        let wheel = if unsafe { forge_input_getKeyPressed(name.as_mut_ptr(), name.len() as u32) } {
            let n = name.iter().position(|&c| c == 0).unwrap_or(name.len());
            match &name[..n] {
                b"Wheel Up" => 1,
                b"Wheel Down" => -1,
                _ => 0,
            }
        } else {
            0
        };
        if !unsafe { forge_input_getMouse(&mut x, &mut y, &mut buttons) } {
            PREV.store(0, Ordering::Relaxed);
            ARMED.store(false, Ordering::Relaxed); // coming back with a button held is not a click either
            return None; // captured, Forge's menu open, or off the picture
        }
        // Buttons held or pressed since the last frame (1 left, 2 middle, 4 right): new presses only
        let prev = PREV.swap(buttons, Ordering::Relaxed);
        let new = if ARMED.load(Ordering::Relaxed) {
            buttons & !prev
        } else {
            if buttons == 0 {
                ARMED.store(true, Ordering::Relaxed);
            }
            0
        };
        Some(super::Mouse { x, y, left: new & 1 != 0, right: new & 4 != 0, wheel })
    }
}

/// While our window is open, ask Recomp for a free pointer of our own (see recomp_mouse). No-op elsewhere.
pub fn set_mouse_free(free: bool) {
    #[cfg(target_arch = "arm")]
    recomp_mouse::set_free(free);
    #[cfg(not(target_arch = "arm"))]
    let _ = free;
}

/// The mouse this frame, or None (not Recomp, window closed, mouse captured or off the picture)
pub fn mouse() -> Option<Mouse> {
    #[cfg(target_arch = "arm")]
    return recomp_mouse::read();
    #[cfg(not(target_arch = "arm"))]
    None
}

/// The last input came from the keyboard / mouse (Recomp only): button prompts show keys (prompts.rs)
static KBM: AtomicBool = AtomicBool::new(false);
static KEY_AT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

pub fn kbm_active() -> bool {
    KBM.load(Ordering::Relaxed)
}

/// Newly pressed buttons
pub fn take() -> alloc::vec::Vec<Button> {
    let mut e = EVENTS.swap(0, Ordering::AcqRel);
    // Last input device. Keys reach us as pad buttons too, so a fresh button press counts as the controller only
    // when no key was pressed just before it.
    #[cfg(target_arch = "arm")]
    if recomp_mouse::kbm_input() {
        KEY_AT.store(crate::clock::ticks(), Ordering::Relaxed);
        KBM.store(true, Ordering::Relaxed);
    } else if e != 0 && crate::clock::ms_since(KEY_AT.load(Ordering::Relaxed)) > 250 {
        KBM.store(false, Ordering::Relaxed);
    }
    // Key repeat for held directions (called every frame on the game thread)
    let held = HELD_NOW.load(Ordering::Acquire);
    let now = crate::clock::ticks();
    for (i, b) in [Button::Up, Button::Down, Button::Left, Button::Right].into_iter().enumerate() {
        if held & bit(b) == 0 {
            continue;
        }
        if e & bit(b) != 0 {
            REPEAT_START[i].store(now, Ordering::Relaxed);
            REPEAT_LAST[i].store(now, Ordering::Relaxed);
        } else if crate::clock::ms_since(REPEAT_START[i].load(Ordering::Relaxed)) >= REPEAT_DELAY_MS
            && crate::clock::ms_since(REPEAT_LAST[i].load(Ordering::Relaxed)) >= REPEAT_MS
        {
            REPEAT_LAST[i].store(now, Ordering::Relaxed);
            e |= bit(b);
        }
    }
    BUTTONS.iter().copied().filter(|&b| e & bit(b) != 0).collect()
}
