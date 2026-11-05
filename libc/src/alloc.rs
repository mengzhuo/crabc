// Allocator: segregated free lists for small blocks, mmap for large blocks.
//
// This replaces the old mmap-per-allocation allocator. Small allocations are
// carved from page-aligned chunks and recycled through per-size free lists;
// large allocations still use mmap directly. A single short-lived spin lock
// protects the free lists. fork() resets it in the child.

const ALLOC_STEP: usize = 16;
const ALLOC_MIN_CLASS: usize = 32;
const ALLOC_MAX_CLASS: usize = 32768;
const ALLOC_CLASSES: usize = ALLOC_MAX_CLASS / ALLOC_STEP + 1;
const ALLOC_HDR: usize = core::mem::size_of::<AllocBlock>();
const ALLOC_LARGE: usize = 1usize << 63;
const ALLOC_ALIGNED: usize = 1usize << 62;
const ALLOC_PAGES_MASK: usize = ALLOC_LARGE - 1;

#[repr(C)]
struct AllocBlock {
    next: *mut AllocBlock,
    tag: usize,
}

static ALLOC_LOCK: AtomicUsize = AtomicUsize::new(0);
static mut ALLOC_FREE: [*mut AllocBlock; ALLOC_CLASSES] =
    [null_mut(); ALLOC_CLASSES];

#[inline]
unsafe fn alloc_lock() {
    while ALLOC_LOCK.swap(1, Ordering::Acquire) != 0 {
        core::hint::spin_loop();
    }
}

#[inline]
unsafe fn alloc_unlock() {
    ALLOC_LOCK.store(0, Ordering::Release);
}

pub(crate) unsafe fn allocator_after_fork() {
    ALLOC_LOCK.store(0, Ordering::Release);
}

#[inline]
fn alloc_class_for(size: usize) -> Option<usize> {
    let total = size.checked_add(ALLOC_HDR)?;
    let class = if total <= ALLOC_MIN_CLASS {
        ALLOC_MIN_CLASS
    } else {
        match total.checked_add(ALLOC_STEP - 1) {
            Some(v) => v & !(ALLOC_STEP - 1),
            None => return None,
        }
    };
    if class <= ALLOC_MAX_CLASS { Some(class) } else { None }
}

unsafe fn alloc_mmap(size: usize) -> *mut u8 {
    let pages = (size + PAGE - 1) & !(PAGE - 1);
    let p = sys_mmap(
        null_mut(),
        pages,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );
    if p == MMAP_FAILED { null_mut() } else { p }
}

unsafe fn alloc_large(size: usize) -> *mut u8 {
    let total = match size.checked_add(ALLOC_HDR) {
        Some(v) => v,
        None => {
            ERRNO = ENOMEM;
            return null_mut();
        }
    };
    if total > usize::MAX - (PAGE - 1) {
        ERRNO = ENOMEM;
        return null_mut();
    }
    let pages = (total + PAGE - 1) / PAGE;
    let map_size = pages * PAGE;
    let base = alloc_mmap(map_size);
    if base.is_null() {
        return null_mut();
    }
    let block = base as *mut AllocBlock;
    (*block).next = block;
    (*block).tag = ALLOC_LARGE | pages;
    base.add(ALLOC_HDR)
}

unsafe fn alloc_aligned_impl(alignment: usize, size: usize) -> *mut u8 {
    if alignment == 0 || !alignment.is_power_of_two() {
        ERRNO = EINVAL;
        return null_mut();
    }
    if alignment <= ALLOC_STEP {
        return malloc(size) as *mut u8;
    }
    let total = match size.checked_add(alignment).and_then(|v| v.checked_add(ALLOC_HDR)) {
        Some(v) => v,
        None => {
            ERRNO = ENOMEM;
            return null_mut();
        }
    };
    if total > usize::MAX - (PAGE - 1) {
        ERRNO = ENOMEM;
        return null_mut();
    }
    let pages = (total + PAGE - 1) / PAGE;
    let map_size = pages * PAGE;
    let base = alloc_mmap(map_size);
    if base.is_null() {
        return null_mut();
    }
    let data_addr = (base as usize + ALLOC_HDR + alignment - 1) & !(alignment - 1);
    let block = (data_addr - ALLOC_HDR) as *mut AllocBlock;
    (*block).next = base as *mut AllocBlock;
    (*block).tag = ALLOC_ALIGNED | pages;
    data_addr as *mut u8
}

pub(crate) unsafe fn alloc_usable_size(ptr: *mut u8) -> usize {
    if ptr.is_null() {
        return 0;
    }
    let block = ptr.sub(ALLOC_HDR) as *mut AllocBlock;
    let tag = (*block).tag;
    if tag & ALLOC_ALIGNED != 0 {
        let base = (*block).next as usize;
        let pages = tag & ALLOC_PAGES_MASK;
        pages * PAGE - (ptr as usize - base)
    } else if tag & ALLOC_LARGE != 0 {
        let pages = tag & ALLOC_PAGES_MASK;
        pages * PAGE - ALLOC_HDR
    } else if tag >= ALLOC_MIN_CLASS && tag <= ALLOC_MAX_CLASS {
        tag - ALLOC_HDR
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn malloc(size: SizeT) -> *mut c_void {
    let class = match alloc_class_for(size) {
        Some(c) => c,
        None => return alloc_large(size) as *mut c_void,
    };
    let idx = class / ALLOC_STEP;

    alloc_lock();
    let head = ALLOC_FREE[idx];
    if !head.is_null() {
        ALLOC_FREE[idx] = (*head).next;
        alloc_unlock();
        return (head as *mut u8).add(ALLOC_HDR) as *mut c_void;
    }

    // Refill this size class with a chunk of blocks.
    let chunk_size = if class < 4096 { 65536 } else { class * 2 };
    let map_size = (chunk_size + PAGE - 1) & !(PAGE - 1);
    let base = alloc_mmap(map_size);
    if base.is_null() {
        alloc_unlock();
        return null_mut();
    }
    let count = map_size / class;
    let mut i = count;
    while i > 1 {
        i -= 1;
        let block = base.add(i * class) as *mut AllocBlock;
        (*block).tag = class;
        (*block).next = ALLOC_FREE[idx];
        ALLOC_FREE[idx] = block;
    }
    let first = base as *mut AllocBlock;
    (*first).tag = class;
    alloc_unlock();
    base.add(ALLOC_HDR) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let block = (ptr as *mut u8).sub(ALLOC_HDR) as *mut AllocBlock;
    let tag = (*block).tag;
    if tag & ALLOC_ALIGNED != 0 {
        let base = (*block).next as *mut u8;
        let pages = tag & ALLOC_PAGES_MASK;
        sys_munmap(base, pages * PAGE);
        return;
    }
    if tag & ALLOC_LARGE != 0 {
        let pages = tag & ALLOC_PAGES_MASK;
        sys_munmap(block as *mut u8, pages * PAGE);
        return;
    }
    if tag < ALLOC_MIN_CLASS || tag > ALLOC_MAX_CLASS {
        return;
    }
    let idx = tag / ALLOC_STEP;
    alloc_lock();
    (*block).next = ALLOC_FREE[idx];
    ALLOC_FREE[idx] = block;
    alloc_unlock();
}

#[no_mangle]
pub unsafe extern "C" fn calloc(count: SizeT, size: SizeT) -> *mut c_void {
    let total = match count.checked_mul(size) {
        Some(v) => v,
        None => {
            ERRNO = ENOMEM;
            return null_mut();
        }
    };
    let ptr = malloc(total);
    if !ptr.is_null() {
        core::ptr::write_bytes(ptr as *mut u8, 0, total);
    }
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, new_size: SizeT) -> *mut c_void {
    if ptr.is_null() {
        return malloc(new_size);
    }
    if new_size == 0 {
        free(ptr);
        return null_mut();
    }
    let old_size = alloc_usable_size(ptr as *mut u8);
    if new_size <= old_size {
        return ptr;
    }
    let new_ptr = malloc(new_size);
    if new_ptr.is_null() {
        return null_mut();
    }
    core::ptr::copy_nonoverlapping(ptr as *const u8, new_ptr as *mut u8, old_size);
    free(ptr);
    new_ptr
}
