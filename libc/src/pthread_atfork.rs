// pthread_atfork() handler registry and fork hook, musl-compatible.
// Include in libc/src/lib.rs; fork() must call __fork_handler(-1/0/1).

const MAX_ATFORK: usize = 64;

#[derive(Copy, Clone)]
struct atfork_funcs {
    prepare: Option<unsafe extern "C" fn()>,
    parent: Option<unsafe extern "C" fn()>,
    child: Option<unsafe extern "C" fn()>,
}

const EMPTY: atfork_funcs = atfork_funcs { prepare: None, parent: None, child: None };

// ponytail: static array, no malloc; upgrade to linked list + malloc if >64 handlers needed
static mut ATFORK_FUNCS: [atfork_funcs; MAX_ATFORK] = [EMPTY; MAX_ATFORK];
static ATFORK_NFUNCS: AtomicUsize = AtomicUsize::new(0);
static ATFORK_LOCK: AtomicI32 = AtomicI32::new(0);

#[inline]
unsafe fn atfork_lock() {
    while ATFORK_LOCK
        .compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}

#[inline]
unsafe fn atfork_unlock() {
    ATFORK_LOCK.store(0, Ordering::Release);
}

// who < 0: prepare (reverse order, acquires lock)
// who == 0: parent (forward order, releases lock)
// who != 0: child (forward order, releases lock)
#[no_mangle]
pub unsafe extern "C" fn __fork_handler(who: c_int) {
    let n = ATFORK_NFUNCS.load(Ordering::Relaxed);
    if n == 0 {
        return;
    }

    let base = core::ptr::addr_of!(ATFORK_FUNCS).cast::<atfork_funcs>();
    if who < 0 {
        atfork_lock();
        let n = ATFORK_NFUNCS.load(Ordering::Relaxed);
        let mut i = n;
        while i > 0 {
            i -= 1;
            let entry = &*base.add(i);
            if let Some(f) = entry.prepare {
                f();
            }
        }
    } else {
        let n = ATFORK_NFUNCS.load(Ordering::Relaxed);
        for i in 0..n {
            let entry = &*base.add(i);
            if who == 0 {
                if let Some(f) = entry.parent {
                    f();
                }
            } else if let Some(f) = entry.child {
                f();
            }
        }
        atfork_unlock();
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_atfork(
    prepare: Option<unsafe extern "C" fn()>,
    parent: Option<unsafe extern "C" fn()>,
    child: Option<unsafe extern "C" fn()>,
) -> c_int {
    atfork_lock();
    let n = ATFORK_NFUNCS.load(Ordering::Relaxed);
    if n >= MAX_ATFORK {
        atfork_unlock();
        return ENOMEM;
    }
    let dst = core::ptr::addr_of_mut!(ATFORK_FUNCS).cast::<atfork_funcs>().add(n);
    *dst = atfork_funcs { prepare, parent, child };
    ATFORK_NFUNCS.store(n + 1, Ordering::Relaxed);
    atfork_unlock();
    0
}
