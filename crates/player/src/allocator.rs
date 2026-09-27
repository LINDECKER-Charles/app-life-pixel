//! The module's allocator: a bump allocator over `memory.grow`, which never frees. The player
//! allocates in `alloc` and `load` only, and keeps everything until the instance goes away.

// `#[global_allocator]` generates `__rust_realloc` beside the static, with the four arguments of
// the allocator ABI; an `allow` on the static does not reach it.
#![allow(
    clippy::too_many_arguments,
    reason = "the allocator ABI's `__rust_realloc` takes four arguments"
)]

use core::alloc::{GlobalAlloc, Layout};
use core::arch::wasm32;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

/// The size of a WebAssembly page, the unit `memory.grow` counts in.
const PAGE_BYTES: usize = 65_536;

/// The module's only memory, exported as `memory`.
const MEMORY: u32 = 0;

/// What `memory.grow` returns when the memory cannot grow.
const GROW_FAILED: usize = usize::MAX;

/// Hands out blocks from the end of the memory the module started with, growing it on demand.
struct BumpAllocator {
    /// Where the next block may start; `0` before the first one.
    next: AtomicUsize,
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator {
    next: AtomicUsize::new(0),
};

impl BumpAllocator {
    /// The address of a new block for `layout`, past every earlier one; `None` when the memory
    /// cannot hold it.
    fn reserve(&self, layout: Layout) -> Option<usize> {
        let memory_end = wasm32::memory_size(MEMORY).saturating_mul(PAGE_BYTES);
        let next = match self.next.load(Ordering::Relaxed) {
            0 => memory_end,
            next => next,
        };
        let start = next.checked_next_multiple_of(layout.align())?;
        let end = start.checked_add(layout.size())?;
        if end > memory_end {
            grow(end - memory_end)?;
        }
        self.next.store(end, Ordering::Relaxed);
        Some(start)
    }
}

/// Grows the memory by at least `bytes`; `None` when it cannot.
fn grow(bytes: usize) -> Option<()> {
    let pages = bytes.div_ceil(PAGE_BYTES);
    (wasm32::memory_grow(MEMORY, pages) != GROW_FAILED).then_some(())
}

// SAFETY: `alloc` returns null or the start of `layout.size()` bytes aligned to
// `layout.align()`, inside the memory — grown first when needed — and past every block handed
// out before, so live blocks never overlap. `dealloc` keeps the block: memory is never reused.
// The module runs on one thread, so the relaxed loads and stores of `next` never race.
#[allow(unsafe_code)]
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.reserve(layout)
            .map_or(ptr::null_mut(), ptr::with_exposed_provenance_mut)
    }

    unsafe fn dealloc(&self, _block: *mut u8, _layout: Layout) {}
}
