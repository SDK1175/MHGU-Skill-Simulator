//! The character slot being played. Same approach as Wishlist (our mod): the slot is set only when a character is actually
//! loaded with 'Continue', and cleared when returning to the title. Counts as playing 90 frames after loading.
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
static SLOT: AtomicU32 = AtomicU32::new(u32::MAX);
static TITLE: AtomicBool = AtomicBool::new(true);
static QUIET: AtomicU32 = AtomicU32::new(0);
static TITLE_CALLS: AtomicU32 = AtomicU32::new(0);
static LOGGED: AtomicU32 = AtomicU32::new(0); // (the loader's state changes logged: MHXX Recomp's may differ)

fn valid(p: u32) -> bool {
    (0x10000..0xeffff000).contains(&p)
}
unsafe fn rd(p: u32) -> u32 {
    unsafe { core::ptr::read_unaligned(p as *const u32) }
}

// The save data loader of 'Continue' calls +3E0DB8 in state 0x0E and then changes to 0x11.
fn completed_load(before: u32, after: u32, cancelled: bool, slot: u32) -> Option<u32> {
    if before == 0x0e && after == 0x11 && !cancelled && slot < 3 { Some(slot) } else { None }
}

#[forge::hook(offset = 0x3e21b8)]
fn select_character(owner: u32) -> u32 {
    let before = if valid(owner) { unsafe { rd(owner + 0x3c) } } else { u32::MAX };
    let result = original!(owner);
    if valid(owner) {
        unsafe {
            let after = rd(owner + 0x3c);
            let slot = rd(owner + 0x40);
            let cancelled = core::ptr::read((owner + 0x6b) as *const u8) != 0;
            if before != after && LOGGED.fetch_add(1, Ordering::Relaxed) < 24 {
                info!("[skill-sim] save loader: state {before:#x} -> {after:#x}, slot {slot}, cancelled {cancelled}");
            }
            if let Some(slot) = completed_load(before, after, cancelled, slot) {
                SLOT.store(slot, Ordering::Release);
                QUIET.store(0, Ordering::Release);
            }
        }
    }
    result
}

#[forge::hook(offset = 0x3e11a8)]
fn write_character(owner: u32, slot: u32) -> u32 {
    let result = original!(owner, slot);
    if LOGGED.fetch_add(1, Ordering::Relaxed) < 24 {
        info!("[skill-sim] save written: slot {slot}");
    }
    if slot < 3 {
        SLOT.store(slot, Ordering::Release);
    }
    result
}

#[forge::hook(offset = 0x67b544)]
fn title_update(owner: u32) -> u32 {
    TITLE.store(true, Ordering::Release);
    let n = TITLE_CALLS.fetch_add(1, Ordering::Relaxed);
    if n < 4 || n.is_power_of_two() {
        let state = if valid(owner) { unsafe { core::ptr::read((owner + 4) as *const u8) } } else { 0xff };
        info!("[skill-sim] title update #{n}: state {state:#x}, slot {}", SLOT.load(Ordering::Relaxed) as i32);
    }
    // The main menu and logos are states before character selection
    if valid(owner) && unsafe { core::ptr::read((owner + 4) as *const u8) } <= 0x11 {
        SLOT.store(u32::MAX, Ordering::Release);
    }
    original!(owner)
}

/// (character slot, playing)
pub fn update() -> (Option<usize>, bool) {
    let age = if TITLE.swap(false, Ordering::AcqRel) {
        QUIET.store(0, Ordering::Release);
        0
    } else {
        let n = QUIET.load(Ordering::Acquire).min(179) + 1;
        QUIET.store(n, Ordering::Release);
        n
    };
    let slot = SLOT.load(Ordering::Acquire);
    ((slot < 3).then_some(slot as usize), slot < 3 && age >= 90)
}

pub fn install(base: u32) -> bool {
    // There is only one game version, MHGU 1.4.0. Even if another mod hooked first, Forge chains them.
    // If it differs from what we expect, only warn and hook anyway (never block).
    for (offset, word) in [(0x3e21b8u32, 0xe92d41f0u32), (0x3e11a8, 0xe92d4800), (0x67b544, 0xe92d47f0)] {
        crate::input::check_site(base, offset, &[word]);
    }
    forge::install_hook!(base, select_character);
    forge::install_hook!(base, write_character);
    forge::install_hook!(base, title_update);
    true
}

/// (for the log) the slot, the frames since the title screen last ran
pub fn debug() -> (i32, u32) {
    (SLOT.load(Ordering::Relaxed) as i32, QUIET.load(Ordering::Relaxed))
}
