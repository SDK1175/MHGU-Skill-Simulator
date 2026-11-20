//! Hunter's Notes 'Mods' entry protocol (MHGU-MODMENU-V01, spec: MHGU-Wishlist/docs/mod-menu-protocol.md).
//! The submenu has room for 7 rows only (an 8th row read past the layout's row slots and crashed, 2026-11-20), so mods
//! share the 7th row. A host mod (Wishlist, order 10) makes the row and the side panel; we only take part:
//!  - publish our block (order 20, not a host): ready, our names and one-line descriptions
//!  - open our window when the host bumps `request`
//!  - leave the Hunter's Notes row to the host when one with a lower order exists (game_menu.rs checks `yield_row`)
//! Without a host (Skill Simulator alone) game_menu.rs adds its own 7th row as before.
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub const MAGIC: [u8; 16] = *b"MHGU-MODMENU-V01";
pub const VERSION: u32 = 1;
const ORDER: u32 = 20;
const FLAG_HOST: u32 = 1;

#[repr(C, align(4))]
pub struct Block {
    magic: [u8; 16],
    version: u32,
    order: u32,
    flags: u32,
    ready: u32,
    request: u32,
    names: [u32; 9],
    descs: [u32; 9],
    reserved: [u32; 5],
}
const _: () = assert!(core::mem::size_of::<Block>() == 128);

/// Non-zero initial contents keep it in the data segment (readers scan there, not .bss)
#[used]
static mut BLOCK: Block = Block {
    magic: MAGIC,
    version: VERSION,
    order: ORDER,
    flags: 0,
    ready: 0,
    request: 0,
    names: [0; 9],
    descs: [0; 9],
    reserved: [0; 5],
};

/// One-line descriptions for the host's description line (DESCS in game_menu.rs without the line break)
static DESCS: [&core::ffi::CStr; 9] = [
    c"Find armor sets that activate the skills you want.",
    c"寻找能发动想要的技能的防具组合。",
    c"尋找能發動想要的技能的防具組合。",
    c"欲しいスキルが発動する装備の組み合わせを探します。",
    c"원하는 스킬이 발동하는 장비 조합을 찾습니다.",
    c"Sucht Rüstungssets, die die gewünschten Fähigkeiten aktivieren.",
    c"Trouve les ensembles d'armure qui activent les talents voulus.",
    c"Trova i set di armature che attivano le abilità desiderate.",
    c"Busca conjuntos de armadura que activen las habilidades deseadas.",
];

/// A host with a lower order exists: the Hunter's Notes row is the host's
static YIELD: AtomicBool = AtomicBool::new(false);
static SCANNED: AtomicBool = AtomicBool::new(false);
/// Last `request` handled (u32::MAX until the first frame: the value at start is not a request)
static LAST_REQUEST: AtomicU32 = AtomicU32::new(u32::MAX);

/// At plugin start: fill the string addresses
pub fn init() {
    use core::ptr::{addr_of_mut, write_volatile};
    unsafe {
        let b = addr_of_mut!(BLOCK);
        for i in 0..9 {
            write_volatile(addr_of_mut!((*b).names[i]), crate::game_menu::LABELS[i].as_ptr() as u32);
            write_volatile(addr_of_mut!((*b).descs[i]), DESCS[i].as_ptr() as u32);
        }
    }
}

/// Leave the Hunter's Notes row to the host
pub fn yield_row() -> bool {
    YIELD.load(Ordering::Relaxed)
}

/// The window can be opened now (same conditions as opening it from the menu). Set every frame by lib.
pub fn set_ready(ready: bool) {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(BLOCK.ready), ready as u32) };
}

/// Every frame on the game thread, outside the shared lock. True when the host asked us to open the window.
pub fn update() -> bool {
    if !SCANNED.swap(true, Ordering::AcqRel) {
        // First update: every plugin has been loaded by now
        scan();
    }
    let now = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(BLOCK.request)) };
    let last = LAST_REQUEST.swap(now, Ordering::Relaxed);
    last != u32::MAX && last != now
}

// ---- Finding the other blocks (once): every loaded NRO's data segment ----
#[repr(C)]
#[derive(Default)]
struct Map {
    base: u64,
    size: u64,
    kind: u32,
    attr: u32,
    perm: u32,
    ipc: u32,
    device: u32,
    pad: u32,
}
/// svcQueryMemory (Horizon AArch32 ABI; Recomp implements the same call)
#[cfg(target_arch = "arm")]
#[unsafe(naked)]
unsafe extern "C" fn query(_info: *mut Map, _address: u32) -> u32 {
    core::arch::naked_asm!("mov r2, r1", "svc #6", "bx lr");
}
#[cfg(not(target_arch = "arm"))]
unsafe extern "C" fn query(_info: *mut Map, _address: u32) -> u32 {
    1
}
fn readable(address: u32, size: u32) -> bool {
    let Some(end) = address.checked_add(size) else { return false };
    let mut at = address;
    while at < end {
        let mut m = Map::default();
        if unsafe { query(&mut m, at) } != 0 || m.perm & 1 == 0 {
            return false;
        }
        let next = m.base.saturating_add(m.size);
        if next <= at as u64 {
            return false;
        }
        at = next.min(end as u64) as u32;
    }
    true
}
unsafe fn word(a: u32) -> u32 {
    unsafe { core::ptr::read_unaligned(a as *const u32) }
}

fn scan() {
    let own = scan as *const () as u32;
    let mut at = 0u32;
    let mut host: Option<(u32, u32)> = None;
    // QueryMemory skips unmapped gaps; never dereference blindly
    for _ in 0..16384 {
        let mut m = Map::default();
        if unsafe { query(&mut m, at) } != 0 {
            break;
        }
        let next = m.base.saturating_add(m.size);
        if next <= at as u64 {
            break;
        }
        // An NRO image: readable + executable mapping with the "NRO0" header
        if m.perm & 5 == 5 && m.base >= 0x10000 && m.base <= u32::MAX as u64 && m.size >= 0x80 {
            let base = m.base as u32;
            if readable(base, 0x80) && unsafe { word(base + 0x10) } == 0x304f_524e {
                let (image, bss) = unsafe { (word(base + 0x18), word(base + 0x38)) };
                let (data, data_len) = unsafe { (word(base + 0x30), word(base + 0x34)) };
                if let Some(total) = image.checked_add(bss) {
                    let ours = (base..base.saturating_add(total)).contains(&own);
                    if !ours && data_len <= 16 * 1024 * 1024 && data.checked_add(data_len).is_some_and(|n| n <= total) {
                        if let Some(start) = base.checked_add(data) {
                            if readable(start, data_len) {
                                find_hosts(start, data_len, &mut host);
                            }
                        }
                    }
                }
            }
        }
        if next >= 0x1_0000_0000 {
            break;
        }
        at = next as u32;
    }
    match host {
        Some((order, addr)) => {
            YIELD.store(true, Ordering::Relaxed);
            info!("[skill-sim] mod menu host found (order {}, block {:08x}): Hunter's Notes row left to it", order, addr);
        }
        None => info!("[skill-sim] no mod menu host: own Hunter's Notes row"),
    }
}

/// Blocks in one data segment: keep the host-capable one with the lowest order below ours
fn find_hosts(start: u32, len: u32, host: &mut Option<(u32, u32)>) {
    let mut off = 0;
    while off + 128 <= len {
        let a = start + off;
        let head = unsafe { core::slice::from_raw_parts(a as *const u8, 16) };
        if head == MAGIC && unsafe { word(a + 16) } == VERSION {
            let (order, flags) = unsafe { (word(a + 20), word(a + 24)) };
            if flags & FLAG_HOST != 0 && order < ORDER && host.is_none_or(|h| order < h.0) {
                *host = Some((order, a));
            }
        }
        off += 4;
    }
}
