//! Reads and writes only our own files in the MHGU save folder (same approach as Wishlist).
//!   skill_simulator.ini              language, button layout
//!   skill_simulator_character_N.ini  per character: last conditions, saved sets, excluded armor, recent skills
use alloc::{string::String, vec::Vec};
use core::ffi::{CStr, c_char, c_void};

pub const SETTINGS: &CStr = c"savedata:/skill_simulator.ini";
const CHARACTERS: [&CStr; 3] = [c"savedata:/skill_simulator_character_1.ini", c"savedata:/skill_simulator_character_2.ini", c"savedata:/skill_simulator_character_3.ini"];
/// The search memory file is large because of the result sets (last 4 searches)
pub const MAX_BYTES: usize = 512 * 1024;

pub fn character(slot: usize) -> &'static CStr {
    CHARACTERS[slot.min(2)]
}


#[repr(C)]
#[derive(Clone, Copy)]
struct FileHandle(u32);
#[repr(C)]
struct WriteOption(i32);
unsafe extern "C" {
    fn forge_fs_mountSaveData() -> bool;
    fn forge_fs_unmountSaveData();
    #[link_name = "_ZN2nn2fs14CommitSaveDataEPKc"]
    fn commit_save(name: *const c_char) -> u32;
    #[link_name = "_ZN2nn2fs8OpenFileEPNS0_10FileHandleEPKci"]
    fn open_file(handle: *mut FileHandle, path: *const c_char, mode: i32) -> u32;
    #[link_name = "_ZN2nn2fs11GetFileSizeEPxNS0_10FileHandleE"]
    fn get_file_size(size: *mut i64, handle: FileHandle) -> u32;
    #[link_name = "_ZN2nn2fs8ReadFileENS0_10FileHandleExPvj"]
    fn read_file(handle: FileHandle, offset: i64, bytes: *mut c_void, size: usize) -> u32;
    #[link_name = "_ZN2nn2fs10CreateFileEPKcx"]
    fn create_file(path: *const c_char, size: i64) -> u32;
    #[link_name = "_ZN2nn2fs11SetFileSizeENS0_10FileHandleEx"]
    fn set_file_size(handle: FileHandle, size: i64) -> u32;
    #[link_name = "_ZN2nn2fs9WriteFileENS0_10FileHandleExPKvjRKNS0_11WriteOptionE"]
    fn write_file(handle: FileHandle, offset: i64, bytes: *const c_void, size: usize, option: *const WriteOption) -> u32;
    #[link_name = "_ZN2nn2fs9CloseFileENS0_10FileHandleE"]
    fn close_file(handle: FileHandle);
}

struct Mount;
impl Mount {
    fn open() -> Option<Self> {
        unsafe { forge_fs_mountSaveData() }.then_some(Mount)
    }
}
impl Drop for Mount {
    fn drop(&mut self) {
        unsafe { forge_fs_unmountSaveData() };
    }
}

fn missing(code: u32) -> bool {
    // nn::fs::ResultPathNotFound
    code & 0x3F_FFFF == 0x202
}

/// Reads a file. Ok(empty string) if it does not exist.
pub fn read(path: &CStr) -> Result<String, ()> {
    let _m = Mount::open().ok_or(())?;
    let mut file = FileHandle(0);
    let code = unsafe { open_file(&mut file, path.as_ptr(), 1) };
    if missing(code) {
        return Ok(String::new());
    }
    if code != 0 {
        return Err(());
    }
    let mut size = 0i64;
    let mut code = unsafe { get_file_size(&mut size, file) };
    let mut bytes: Vec<u8> = Vec::new();
    if code == 0 && (0..=MAX_BYTES as i64).contains(&size) {
        bytes.resize(size as usize, 0);
        code = unsafe { read_file(file, 0, bytes.as_mut_ptr().cast(), size as usize) };
    } else {
        code = 1;
    }
    unsafe { close_file(file) };
    if code != 0 {
        return Err(());
    }
    String::from_utf8(bytes).map_err(|_| ())
}

pub fn write(path: &CStr, text: &str) -> Result<(), ()> {
    if text.len() > MAX_BYTES {
        return Err(());
    }
    let _m = Mount::open().ok_or(())?;
    let mut file = FileHandle(0);
    let mut code = unsafe { open_file(&mut file, path.as_ptr(), 6) };
    if missing(code) {
        if unsafe { create_file(path.as_ptr(), 0) } != 0 {
            return Err(());
        }
        code = unsafe { open_file(&mut file, path.as_ptr(), 6) };
    }
    if code != 0 {
        return Err(());
    }
    let mut code = unsafe { set_file_size(file, text.len() as i64) };
    if code == 0 {
        code = unsafe { write_file(file, 0, text.as_ptr().cast(), text.len(), &WriteOption(1)) };
    }
    unsafe { close_file(file) };
    if code != 0 || unsafe { commit_save(c"savedata".as_ptr()) } != 0 {
        return Err(());
    }
    Ok(())
}
