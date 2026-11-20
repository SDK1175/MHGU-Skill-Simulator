//! Game UI sounds. Played only on the game thread (per-frame function): other threads only leave a request.
//! Play function: 0x4eeb78(manager, sound id), manager = [0x1896f5c]. The game menu also uses it via 0x54f260(0, id) ->
//! 0x4eeb78 (2026-11-18 static analysis).
//! The ids per situation are fixed (picked by listening in game, 2026-11-19). The Forge menu section that recorded game
//! sounds and changed the ids was removed (2026-11-20), and with it our hook on the play function.
use core::sync::atomic::{AtomicU32, Ordering};

/// Situations that play a sound
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cue {
    Open,
    Close,
    Move,
    /// The list page changed (left/right on the results screen etc.)
    Page,
    Tab,
    Confirm,
    Cancel,
    Error,
    Found,
    NoResult,
}

impl Cue {
    /// Game sound id
    fn id(self) -> u32 {
        match self {
            Cue::Open => 1,
            Cue::Close => 2,
            Cue::Move => 4,
            Cue::Page => 27,
            Cue::Tab => 7,
            Cue::Confirm => 1,
            Cue::Cancel => 2,
            Cue::Error => 3,
            Cue::Found => 11,
            Cue::NoResult => 13,
        }
    }
}

/// Id to play next frame (u32::MAX = none). Only one per frame (the last request).
static PENDING: AtomicU32 = AtomicU32::new(u32::MAX);
pub fn cue(c: Cue) {
    PENDING.store(c.id(), Ordering::Release);
}

static BASE: AtomicU32 = AtomicU32::new(0);
pub fn install(base: u32) {
    BASE.store(base, Ordering::Relaxed);
}

/// Minimum time between two of our sounds. Sounds asked for faster (the mouse sweeping over many rows) wait for the gap
/// instead of being lost: the game seems to drop a sound started again right after itself.
const MIN_GAP_MS: u64 = 45;
static LAST_PLAY: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Every frame (game thread): play the request, if any
pub fn update() {
    if PENDING.load(Ordering::Acquire) == u32::MAX || crate::clock::ms_since(LAST_PLAY.load(Ordering::Relaxed)) < MIN_GAP_MS {
        return; // nothing to play, or too soon after the last one (kept for a later frame)
    }
    let v = PENDING.swap(u32::MAX, Ordering::AcqRel);
    let base = BASE.load(Ordering::Relaxed);
    if v == u32::MAX || base == 0 {
        return;
    }
    LAST_PLAY.store(crate::clock::ticks(), Ordering::Relaxed);
    unsafe {
        let manager = core::ptr::read_unaligned((base + 0x1896f5c) as *const u32);
        if !(0x10000..0xeffff000).contains(&manager) {
            return;
        }
        // Through the function's entry, so other mods' hooks on it still see our sounds
        let play: unsafe extern "C" fn(u32, u32) -> u32 = core::mem::transmute(base + 0x4eeb78);
        play(manager, v);
    }
}
