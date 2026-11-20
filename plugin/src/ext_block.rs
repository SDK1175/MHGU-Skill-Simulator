//! Shared block for a guest mod (Wishlist) that adds its own action to our results / saved-set screens.
//! We only publish facts (what the cursor points at, the raw pad buttons, where the guest may draw its hint) and
//! honour two requests from the guest (a hint width, and "my dialog is open: ignore your input").
//! The guest detects its own shortcut, draws its own hint and dialog, and decides what to register.
//! Readers find the block by scanning our data segment for the magic and version (as with "MHGU-SKSIM-WIN01").
//!
//! Layout (repr(C), little endian, 128 bytes, version 1):
//!   written by us
//!   +0   [u8;16] magic   "MHGU-SKSIM-EXT01"
//!   +16  u32     version 1
//!   +20  u32     open    1 = our window was drawn this frame, 0 = closed
//!   +24  u32     frame   +1 on every frame the window is drawn, written last
//!   +28  u32     screen  same numbers as MHGU-SKSIM-WIN01 (3 results, 4 saved sets; others carry no subject)
//!   +32  u32     overlay 1 = one of our own dialogs / popups / lists is open (the guest should not act)
//!   +36  u32     kind    0 nothing to register, 1 the set under the cursor (set list), 2 one part (set detail)
//!   +40  u32     part    kind 2: the part under the cursor 1-5 (head..legs), else 0
//!   +44  [Piece;5]       the set's armor by part (head..legs), also for kind 2 (the guest uses `part`)
//!                        Piece = { u16 series (game armor number within the part, 0 = none), u8 part 1-5, u8 flags }
//!                        flags: bit0 owned (in the equipment box), bit1 quest reward armor (not made at the smithy)
//!   +64  u32     raw     pad buttons held this frame as the game's HID word, read before we withhold them from the game
//!   +68  u32     serial  +1 whenever screen, overlay, kind, part or the pieces change
//!   +72  f32[4]  hint    left, top, right, bottom in screen pixels: the guest's hint slot at the left end of our hint line
//!                        (all 0 when the guest asked for no width or kind is 0)
//!   +88  f32     scale   screen pixels per unit of our canvas (our text and icon sizes follow it)
//!   written by the guest
//!   +92  u32     guest_frame  +1 every frame the guest wants its fields below honoured (stale for 10 of our frames = ignored)
//!   +96  f32     hint_width   width the guest wants for its hint, in screen pixels (0 = none)
//!   +100 u32     guest_modal  1 = the guest's dialog is open over our window: we ignore our pad and mouse input
//!   +104 [u32;6] reserved (0)

pub const MAGIC: [u8; 16] = *b"MHGU-SKSIM-EXT01";
pub const VERSION: u32 = 1;
/// Our frames without a guest_frame change before the guest's fields are ignored
const GUEST_STALE: u32 = 10;

pub const KIND_NONE: u32 = 0;
pub const KIND_SET: u32 = 1;
pub const KIND_PART: u32 = 2;
pub const FLAG_OWNED: u8 = 1;
pub const FLAG_REWARD: u8 = 2;

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq)]
pub struct Piece {
    pub series: u16,
    pub part: u8,
    pub flags: u8,
}

/// What we publish this frame (built by ui.rs)
#[derive(Clone, Default, PartialEq)]
pub struct Ext {
    pub screen: u32,
    pub overlay: u32,
    pub kind: u32,
    pub part: u32,
    pub pieces: [Piece; 5],
    /// Left end of our button hints and the hint line's top / bottom, in screen pixels
    pub hints_left: f32,
    pub hint_top: f32,
    pub hint_bottom: f32,
    /// Gap kept between the guest slot and our first hint, in screen pixels
    pub gap: f32,
    pub scale: f32,
}

#[repr(C, align(4))]
pub struct ExtBlock {
    magic: [u8; 16],
    version: u32,
    open: u32,
    frame: u32,
    screen: u32,
    overlay: u32,
    kind: u32,
    part: u32,
    pieces: [Piece; 5],
    raw: u32,
    serial: u32,
    hint: [f32; 4],
    scale: f32,
    guest_frame: u32,
    hint_width: f32,
    guest_modal: u32,
    reserved: [u32; 6],
}
const _: () = assert!(core::mem::size_of::<ExtBlock>() == 128);

/// Non-zero initial contents keep it in the data segment (readers scan there, not .bss)
#[used]
static mut BLOCK: ExtBlock = ExtBlock {
    magic: MAGIC,
    version: VERSION,
    open: 0,
    frame: 0,
    screen: 0,
    overlay: 0,
    kind: 0,
    part: 0,
    pieces: [Piece { series: 0, part: 0, flags: 0 }; 5],
    raw: 0,
    serial: 0,
    hint: [0.; 4],
    scale: 0.,
    guest_frame: 0,
    hint_width: 0.,
    guest_modal: 0,
    reserved: [0; 6],
};

static mut LAST: Option<Ext> = None;
/// (last guest_frame seen, our frames since it changed)
static mut GUEST: (u32, u32) = (0, u32::MAX);
/// Raw HID buttons from the input hook (written on the input thread)
static RAW: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

/// Input hook: the game's HID button word before we withhold anything
pub fn set_raw(buttons: u32) {
    RAW.store(buttons, core::sync::atomic::Ordering::Relaxed);
}

/// Is a guest alive (its frame counter moved recently)
fn guest_alive() -> bool {
    unsafe { (*core::ptr::addr_of!(GUEST)).1 <= GUEST_STALE }
}

/// The guest's dialog is open over our window: our pad and mouse input are ignored
pub fn guest_modal() -> bool {
    guest_alive() && unsafe { core::ptr::read_volatile(core::ptr::addr_of!(BLOCK.guest_modal)) } == 1
}

/// Every frame on the game thread: the view when the window was drawn this frame, else None
pub fn publish(drawn: Option<Ext>) {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    unsafe {
        let b = addr_of_mut!(BLOCK);
        // Guest heartbeat
        let g = &mut *addr_of_mut!(GUEST);
        let gf = read_volatile(addr_of!((*b).guest_frame));
        *g = if gf != g.0 { (gf, 0) } else { (g.0, g.1.saturating_add(1)) };
        let Some(ext) = drawn else {
            write_volatile(addr_of_mut!((*b).open), 0);
            write_volatile(addr_of_mut!((*b).kind), KIND_NONE);
            write_volatile(addr_of_mut!((*b).hint), [0.; 4]);
            return;
        };
        let last = &mut *addr_of_mut!(LAST);
        if last.as_ref().is_none_or(|v| (v.screen, v.overlay, v.kind, v.part, v.pieces) != (ext.screen, ext.overlay, ext.kind, ext.part, ext.pieces)) {
            write_volatile(addr_of_mut!((*b).screen), ext.screen);
            write_volatile(addr_of_mut!((*b).overlay), ext.overlay);
            write_volatile(addr_of_mut!((*b).kind), ext.kind);
            write_volatile(addr_of_mut!((*b).part), ext.part);
            write_volatile(addr_of_mut!((*b).pieces), ext.pieces);
            let s = read_volatile(addr_of!((*b).serial));
            write_volatile(addr_of_mut!((*b).serial), s.wrapping_add(1));
        }
        write_volatile(addr_of_mut!((*b).raw), RAW.load(core::sync::atomic::Ordering::Relaxed));
        // The guest slot: left of our first hint, as wide as the guest asked
        let w = if guest_alive() { read_volatile(addr_of!((*b).hint_width)) } else { 0. };
        let hint = if ext.kind != KIND_NONE && ext.overlay == 0 && w > 0. && w.is_finite() {
            let right = ext.hints_left - ext.gap;
            [right - w, ext.hint_top, right, ext.hint_bottom]
        } else {
            [0.; 4]
        };
        write_volatile(addr_of_mut!((*b).hint), hint);
        write_volatile(addr_of_mut!((*b).scale), ext.scale);
        *last = Some(ext);
        write_volatile(addr_of_mut!((*b).open), 1);
        let f = read_volatile(addr_of!((*b).frame));
        write_volatile(addr_of_mut!((*b).frame), f.wrapping_add(1));
    }
}
