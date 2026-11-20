//! Shared block for other mods (Better-Skillinfo): is our window open, where is it, and what is under the cursor.
//! Same approach as Wishlist's "MHGU-WL-SKILLV01" block: a 4-aligned static in our data segment that readers find by
//! scanning for the magic and version. We only write facts; readers decide what to show. Without a reader nothing changes.
//!
//! Layout (repr(C), little endian, 328 bytes, version 3 = the version 2 layout with the v3 subject values below
//! plus page at the end; the first 44 bytes are version 1 unchanged):
//!   +0   [u8;16] magic   "MHGU-SKSIM-WIN01"
//!   +16  u32     version 3
//!   +20  u32     open    1 = our window was drawn this frame, 0 = closed
//!   +24  f32     left    window frame rectangle in screen pixels (the coordinates we draw with)
//!   +28  f32     top
//!   +32  f32     right
//!   +36  f32     bottom
//!   +40  u32     frame   +1 on every frame the window is drawn, written last (a value that stops changing means closed)
//!   +44  u32     screen  SCREEN_* below
//!   +48  u32     area    AREA_* below: the area holding the cursor
//!   +52  u32     subject SUBJECT_* below: what the cursor points at
//!   +56  u32     a       subject value 1
//!   +60  u32     b       subject value 2
//!   +64  u32     count   number of points (0-32)
//!   +68  [(u16 tree, i16 points); 32]
//!   +196 u32     rects   number of rects (0-6)
//!   +200 [Rect; 6] Rect = { u32 area, f32 left, top, right, bottom } (20 bytes), screen pixels
//!   +320 u32     serial  +1 whenever screen, area, subject, a, b, points or page change
//!   +324 u32     page    page of the detail panel ('Equipment details'): 0 = no detail panel on this screen,
//!                        1 = skills and stats (1/2), 2 = crafting materials (2/2)
//! All ids are the game's own: skill tree (game_tree), armor series, decoration item number.
//! Change the version when an area appears, disappears or changes meaning (not when it only moves or resizes).

pub const MAGIC: [u8; 16] = *b"MHGU-SKSIM-WIN01";
pub const VERSION: u32 = 3;
pub const MAX_POINTS: usize = 32;
pub const MAX_RECTS: usize = 6;

#[allow(dead_code)]
pub const SCREEN_NONE: u32 = 0;
pub const SCREEN_MAIN: u32 = 1;
pub const SCREEN_SKILLS: u32 = 2;
pub const SCREEN_RESULTS: u32 = 3;
pub const SCREEN_SAVED: u32 = 4;
pub const SCREEN_MANAGE: u32 = 5;
pub const SCREEN_BOX: u32 = 6;
/// A dialog, menu, dropdown, popup list or search progress over the window
pub const SCREEN_MODAL: u32 = 7;

#[allow(dead_code)]
pub const AREA_NONE: u32 = 0;
pub const AREA_TARGETS: u32 = 1;
pub const AREA_CONDITIONS: u32 = 2;
pub const AREA_FIXED: u32 = 3;
pub const AREA_SET_LIST: u32 = 4;
pub const AREA_SET_DETAIL: u32 = 5;
pub const AREA_LIST: u32 = 6;

#[allow(dead_code)]
pub const SUBJECT_NONE: u32 = 0;
/// a = skill tree, b = that level's points (i32, 0 = every level), no points.
/// Also on the extra-skill list (screen MODAL, area NONE) for the row under its cursor.
pub const SUBJECT_TREE: u32 = 1;
/// a = armor series, b = part 1-5 (head..legs), points = the armor's own skills in the detail panel order,
/// then the decorations placed in that part (same tree merged into one pair)
pub const SUBJECT_ARMOR: u32 = 2;
/// a = slots, points = the charm's skills, then the decorations placed in it (also for a skill-less unowned charm)
pub const SUBJECT_CHARM: u32 = 3;
/// points = the set's final totals: trees of the 'Active skills' list in its order, then the other trees
pub const SUBJECT_SET: u32 = 4;
/// a = decoration item number
#[allow(dead_code)]
pub const SUBJECT_JEWEL: u32 = 5;
/// Weapon slots: points = the decorations placed there (count 0 without any)
pub const SUBJECT_WEAPON: u32 = 6;

/// What the cursor points at this frame (built by ui.rs)
#[derive(Clone, Default, PartialEq)]
pub struct View {
    pub screen: u32,
    pub area: u32,
    pub subject: u32,
    pub a: u32,
    pub b: u32,
    pub points: alloc::vec::Vec<(u16, i16)>,
    /// (area, left, top, right, bottom) in screen pixels
    pub rects: alloc::vec::Vec<(u32, [f32; 4])>,
    /// Detail panel page: 0 none, 1 skills and stats, 2 crafting materials
    pub page: u32,
}

/// Our skill tree id (the database's) -> the game's. Checked against Better-Skillinfo's tree table (205 trees):
/// the game numbers are one lower, except Torso Up (1 -> 203) and the two that keep their number (204, 205).
pub fn game_tree(tree: u16) -> u16 {
    match tree {
        1 => 203,
        204 | 205 => tree,
        0 => 0,
        t => t - 1,
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Rect {
    area: u32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

#[repr(C, align(4))]
pub struct WindowBlock {
    magic: [u8; 16],
    version: u32,
    open: u32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    frame: u32,
    screen: u32,
    area: u32,
    subject: u32,
    a: u32,
    b: u32,
    count: u32,
    points: [(u16, i16); MAX_POINTS],
    rects: u32,
    rect: [Rect; MAX_RECTS],
    serial: u32,
    page: u32,
}
const _: () = assert!(core::mem::size_of::<WindowBlock>() == 328);

const EMPTY_RECT: Rect = Rect { area: 0, left: 0., top: 0., right: 0., bottom: 0. };
/// Non-zero initial contents keep it in the data segment (readers scan there, not .bss)
#[used]
static mut BLOCK: WindowBlock = WindowBlock {
    magic: MAGIC,
    version: VERSION,
    open: 0,
    left: 0.,
    top: 0.,
    right: 0.,
    bottom: 0.,
    frame: 0,
    screen: 0,
    area: 0,
    subject: 0,
    a: 0,
    b: 0,
    count: 0,
    points: [(0, 0); MAX_POINTS],
    rects: 0,
    rect: [EMPTY_RECT; MAX_RECTS],
    serial: 0,
    page: 0,
};

/// Last published content (for the serial)
static mut LAST: Option<View> = None;

/// Every frame on the game thread: the window rectangle and cursor view when drawn this frame, else None
pub fn publish(drawn: Option<([f32; 4], View)>) {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    unsafe {
        let b = addr_of_mut!(BLOCK);
        let Some(([l, t, r, btm], view)) = drawn else {
            write_volatile(addr_of_mut!((*b).open), 0);
            return;
        };
        // Everything else first, the frame counter last (a reader that sees a new frame sees complete fields)
        write_volatile(addr_of_mut!((*b).left), l);
        write_volatile(addr_of_mut!((*b).top), t);
        write_volatile(addr_of_mut!((*b).right), r);
        write_volatile(addr_of_mut!((*b).bottom), btm);
        let last = &mut *addr_of_mut!(LAST);
        let same = last.as_ref().is_some_and(|v| {
            (v.screen, v.area, v.subject, v.a, v.b, v.page) == (view.screen, view.area, view.subject, view.a, view.b, view.page) && v.points == view.points
        });
        if !same {
            write_volatile(addr_of_mut!((*b).screen), view.screen);
            write_volatile(addr_of_mut!((*b).area), view.area);
            write_volatile(addr_of_mut!((*b).subject), view.subject);
            write_volatile(addr_of_mut!((*b).a), view.a);
            write_volatile(addr_of_mut!((*b).b), view.b);
            write_volatile(addr_of_mut!((*b).page), view.page);
            let n = view.points.len().min(MAX_POINTS);
            for i in 0..MAX_POINTS {
                write_volatile(addr_of_mut!((*b).points[i]), view.points.get(i).copied().unwrap_or((0, 0)));
            }
            write_volatile(addr_of_mut!((*b).count), n as u32);
            let s = read_volatile(addr_of!((*b).serial));
            write_volatile(addr_of_mut!((*b).serial), s.wrapping_add(1));
        }
        let n = view.rects.len().min(MAX_RECTS);
        for i in 0..MAX_RECTS {
            let rect = view.rects.get(i).map_or(EMPTY_RECT, |&(area, [l, t, r, b])| Rect { area, left: l, top: t, right: r, bottom: b });
            write_volatile(addr_of_mut!((*b).rect[i]), rect);
        }
        write_volatile(addr_of_mut!((*b).rects), n as u32);
        *last = Some(view);
        write_volatile(addr_of_mut!((*b).open), 1);
        let f = read_volatile(addr_of!((*b).frame));
        write_volatile(addr_of_mut!((*b).frame), f.wrapping_add(1));
    }
}
