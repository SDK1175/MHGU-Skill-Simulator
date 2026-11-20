//! Reads files from the plugin folder in the game romfs (romfs/nativeNX/plugins/skill-simulator/).
use alloc::{ffi::CString, vec::Vec};
use forge::sys::imgui as sys;

pub const DIR: &str = "app:/nativeNX/plugins/skill-simulator/";

pub fn path(name: &str) -> CString {
    CString::new(alloc::format!("{}{}", DIR, name)).unwrap()
}

/// Reads a whole file. None if missing or on read failure.
pub fn read(name: &str) -> Option<Vec<u8>> {
    let path = path(name);
    unsafe {
        let file = sys::igImFileOpen(path.as_ptr(), c"rb".as_ptr());
        if file.is_null() {
            return None;
        }
        let size = sys::igImFileGetSize(file);
        if size > 8 * 1024 * 1024 {
            sys::igImFileClose(file);
            return None;
        }
        let mut bytes = alloc::vec![0u8; size as usize];
        let n = sys::igImFileRead(bytes.as_mut_ptr().cast(), 1, size, file);
        sys::igImFileClose(file);
        (n == size).then_some(bytes)
    }
}

/// Reads a file of exactly `len` bytes straight into memory the caller owns (no temporary copy on our heap).
/// `dst` is asked for only after the size matched; it returns None to give up. False if missing, another size or a short read.
pub fn read_into(name: &str, len: usize, dst: impl FnOnce() -> Option<*mut u8>) -> bool {
    let path = path(name);
    unsafe {
        let file = sys::igImFileOpen(path.as_ptr(), c"rb".as_ptr());
        if file.is_null() {
            return false;
        }
        let size = sys::igImFileGetSize(file);
        let ok = size as usize == len
            && match dst() {
                Some(ptr) => sys::igImFileRead(ptr.cast(), 1, size, file) == size,
                None => false,
            };
        sys::igImFileClose(file);
        ok
    }
}
