//! The game's picture inside the host window. On Recomp the window and the picture can differ in shape (e.g. 16:10
//! forced on a 16:9 monitor): the game then draws only in the middle and leaves black bars. Our canvas goes into the
//! picture, not the whole window. The renderer data is the one Wishlist and Better-Skillinfo read for their HUD anchors
//! (MHGU global 1.4.0). Without it (other versions, Ryujinx, hardware): the whole window, as before.

/// The picture in display pixels, and the share of its width the console's 1280-wide screen maps to (Recomp's mouse:
/// 1 unless the HUD is narrower than the picture, e.g. 16:9 HUD on a 21:9 picture)
#[derive(Clone, Copy)]
pub struct Area {
    pub left: f32,
    pub top: f32,
    pub w: f32,
    pub h: f32,
    pub canvas: f32,
}

/// Renderer sizes: (render width, render height, HUD reference width), or None
#[cfg(target_arch = "arm")]
fn renderer() -> Option<(u32, u32, u32)> {
    fn valid(p: u32) -> bool {
        (0x10000..0xeffff000).contains(&p)
    }
    unsafe fn rd(p: u32) -> u32 {
        unsafe { core::ptr::read_unaligned(p as *const u32) }
    }
    let base = crate::base();
    if !valid(base) {
        return None;
    }
    let render = unsafe { rd(base + 0x2126998) };
    if !valid(render) || render & 3 != 0 || render.checked_add(0x44BACC).is_none_or(|e| !valid(e)) {
        return None;
    }
    if unsafe { rd(render) } != base + 0x1754A58 {
        return None; // not the renderer we know
    }
    Some(unsafe { (rd(render + 0x44BAC0), rd(render + 0x44BAC4), rd(render + 0x44BAC8)) })
}
#[cfg(not(target_arch = "arm"))]
fn renderer() -> Option<(u32, u32, u32)> {
    None
}

/// The picture in a (w, h) window
pub fn area(w: f32, h: f32) -> Area {
    let whole = Area { left: 0., top: 0., w, h, canvas: 1. };
    let found = renderer();
    // Log what we use once, and again whenever it changes (window resize, settings)
    {
        use core::sync::atomic::{AtomicU32, Ordering};
        static LAST: AtomicU32 = AtomicU32::new(u32::MAX);
        let (rw, rh, uw) = found.unwrap_or((0, 0, 0));
        let key = rw.wrapping_mul(31).wrapping_add(rh).wrapping_mul(31).wrapping_add(uw).wrapping_mul(31).wrapping_add(w as u32).wrapping_mul(31).wrapping_add(h as u32);
        if LAST.swap(key, Ordering::Relaxed) != key {
            match found {
                Some(_) => info!("[skill-sim] picture: window {}x{}, render {}x{}, HUD width {}", w as u32, h as u32, rw, rh, uw),
                None => info!("[skill-sim] picture: window {}x{}, no renderer data (whole window)", w as u32, h as u32),
            }
        }
    }
    let Some((rw, rh, uw)) = found else { return whole };
    let ok = |v: u32| (64..=16384).contains(&v);
    if !ok(rw) || !ok(rh) {
        return whole;
    }
    let aspect = rw as f32 / rh as f32;
    let ph = h.min(w / aspect);
    let pw = ph * aspect;
    // Recomp maps the mouse's x over the HUD's width only (its window_to_screen): HUD width / 720 is that aspect
    let hud = uw as f32 / 720.;
    let canvas = if (1280..=16384).contains(&uw) && hud < aspect { hud / aspect } else { 1. };
    Area { left: (w - pw) * 0.5, top: (h - ph) * 0.5, w: pw, h: ph, canvas }
}

impl Area {
    /// A Recomp mouse position (console screen pixels, 1280x720 over the picture's HUD part) in display pixels
    pub fn mouse(&self, x: f32, y: f32) -> (f32, f32) {
        let u = (x / 1280.) * self.canvas + (1. - self.canvas) * 0.5;
        (self.left + u * self.w, self.top + y / 720. * self.h)
    }
}
