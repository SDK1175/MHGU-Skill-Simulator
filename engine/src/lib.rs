//! Pure skill simulator engine, independent of the game. The plugin and the PC tests share this code.
#![no_std]
extern crate alloc;

pub mod charm;
pub mod data;
pub mod rules;
pub mod search;
pub mod types;

/// Diagnostic progress: when the plugin sets HOOK, the engine reports step numbers (to locate crashes on hardware/emulators).
pub mod trace {
    use core::sync::atomic::{AtomicUsize, Ordering};
    static HOOK: AtomicUsize = AtomicUsize::new(0);
    pub fn set(f: fn(u32)) {
        HOOK.store(f as usize, Ordering::Release);
    }
    #[inline(never)]
    pub fn mark(n: u32) {
        let h = HOOK.load(Ordering::Acquire);
        if h != 0 {
            let f: fn(u32) = unsafe { core::mem::transmute(h) };
            f(n);
        }
    }
}
