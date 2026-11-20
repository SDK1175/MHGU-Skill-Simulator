//! System tick (19.2 MHz). Used for search time limits and measurements.

#[cfg(target_arch = "arm")]
pub fn ticks() -> u64 {
    let (lo, hi): (u32, u32);
    // Besides its results (r0, r1), svc may change registers the calling convention does not preserve (r2, r3, r12, ...).
    // Without telling the compiler, values it kept there (pointers etc.) get corrupted: on Ryujinx the time came out wrong
    // and then a crash reading address 0 followed (2026-11-18). Recomp does not touch these registers, so it never showed there.
    unsafe { core::arch::asm!("svc 0x1e", lateout("r0") lo, lateout("r1") hi, clobber_abi("C"), options(nostack)) };
    lo as u64 | ((hi as u64) << 32)
}
#[cfg(not(target_arch = "arm"))]
pub fn ticks() -> u64 {
    0
}

pub const PER_MS: u64 = 19_200;

pub fn ms_since(start: u64) -> u64 {
    ticks().saturating_sub(start) / PER_MS
}
