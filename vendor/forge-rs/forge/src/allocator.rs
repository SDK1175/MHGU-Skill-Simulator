use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

use forge_sys::mem::{free, malloc, realloc};

struct ForgeAllocator;

// Local change (Skill Simulator): live heap bytes requested through this allocator, the peak, and live blocks.
// Sizes are the requested sizes (malloc's own overhead and the alignment header are not counted).
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static BLOCKS: AtomicUsize = AtomicUsize::new(0);

fn added(size: usize) {
    let now = LIVE.fetch_add(size, Ordering::Relaxed) + size;
    PEAK.fetch_max(now, Ordering::Relaxed);
}
fn removed(size: usize) {
    LIVE.fetch_sub(size, Ordering::Relaxed);
}

/// (live bytes, peak bytes since start or the last reset, live blocks) of this plugin's Rust heap
pub fn heap_stats() -> (usize, usize, usize) {
    (LIVE.load(Ordering::Relaxed), PEAK.load(Ordering::Relaxed), BLOCKS.load(Ordering::Relaxed))
}
/// Start a new peak from the current live size
pub fn heap_reset_peak() {
    PEAK.store(LIVE.load(Ordering::Relaxed), Ordering::Relaxed);
}

#[global_allocator]
static GLOBAL_ALLOCATOR: ForgeAllocator = ForgeAllocator;

#[repr(C)]
struct Header {
    base: *mut u8,
}

impl ForgeAllocator {
    const HEADER_SIZE: usize = core::mem::size_of::<Header>();

    fn is_header_compatible(layout: Layout) -> bool {
        layout.align() <= core::mem::align_of::<Header>()
    }

    unsafe fn alloc_with_header(layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let total = match size.checked_add(align).and_then(|n| n.checked_add(Self::HEADER_SIZE)) {
            Some(n) => n,
            None => return core::ptr::null_mut(),
        };

        let base = unsafe { malloc(total) as *mut u8 };
        if base.is_null() {
            return core::ptr::null_mut();
        }

        let start = unsafe { base.add(Self::HEADER_SIZE) } as usize;
        let aligned = (start + (align - 1)) & !(align - 1);
        let user = aligned as *mut u8;

        let header = unsafe { (user as *mut Header).sub(1) };
        unsafe {
            *header = Header { base };
        }

        user
    }
}

unsafe impl GlobalAlloc for ForgeAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 {
            return layout.align() as *mut u8;
        }

        let p = if Self::is_header_compatible(layout) {
            unsafe { malloc(layout.size()) as *mut u8 }
        } else {
            unsafe { Self::alloc_with_header(layout) }
        };
        if !p.is_null() {
            added(layout.size());
            BLOCKS.fetch_add(1, Ordering::Relaxed);
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() == 0 {
            return;
        }
        removed(layout.size());
        BLOCKS.fetch_sub(1, Ordering::Relaxed);

        if Self::is_header_compatible(layout) {
            unsafe {
                free(ptr.cast());
            }
            return;
        }

        let header = unsafe { (ptr as *mut Header).sub(1) };
        unsafe {
            free((*header).base.cast());
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if layout.size() == 0 {
            return unsafe { self.alloc(Layout::from_size_align_unchecked(new_size, layout.align())) };
        }

        if new_size == 0 {
            unsafe {
                self.dealloc(ptr, layout);
            }
            return layout.align() as *mut u8;
        }

        if Self::is_header_compatible(layout) {
            let p = unsafe { realloc(ptr.cast(), new_size) as *mut u8 };
            if !p.is_null() {
                added(new_size);
                removed(layout.size());
            }
            p
        } else {
            let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
            let new_ptr = unsafe { self.alloc(new_layout) };
            if new_ptr.is_null() {
                return core::ptr::null_mut();
            }

            let to_copy = core::cmp::min(layout.size(), new_size);
            unsafe {
                core::ptr::copy_nonoverlapping(ptr, new_ptr, to_copy);
                self.dealloc(ptr, layout);
            }
            new_ptr
        }
    }
}
