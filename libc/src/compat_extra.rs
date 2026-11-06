// Real implementations for the remaining POSIX/GNU compatibility surface:
// AIO, mqueue attributes, pthread introspection, spawn attributes, stdio
// internals, and assorted legacy calls.

// ============================================================
// AIO (synchronous completion)
// ============================================================

#[repr(C)]
pub struct Aiocb {
    pub aio_fildes: c_int,
    pub aio_lio_opcode: c_int,
    pub aio_reqprio: c_int,
    __pad: c_int,
    pub aio_buf: *mut c_void,
    pub aio_nbytes: usize,
    pub aio_sigevent: [u8; 64],
    pub aio_offset: i64,
}

const _: [(); 104] = [(); core::mem::size_of::<Aiocb>()];

#[derive(Copy, Clone)]
struct AioSlot {
    cb: *mut Aiocb,
    ret: isize,
    err: c_int,
    active: bool,
}

static mut AIO_SLOTS: [AioSlot; 32] = [AioSlot {
    cb: core::ptr::null_mut(),
    ret: 0,
    err: 0,
    active: false,
}; 32];

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SigEvent {
    pub sigev_value: usize,
    pub sigev_signo: c_int,
    pub sigev_notify: c_int,
    pub sigev_un: [u8; 48],
}

const SIGEV_SIGNAL_V: c_int = 0;
const SIGEV_NONE_V: c_int = 1;
const SIGEV_THREAD_V: c_int = 2;

unsafe fn aio_slot(cb: *mut Aiocb) -> *mut AioSlot {
    let base = core::ptr::addr_of_mut!(AIO_SLOTS[0]);
    for i in 0..AIO_SLOTS.len() {
        let s = base.add(i);
        if (*s).cb == cb { return s; }
    }
    for i in 0..AIO_SLOTS.len() {
        let s = base.add(i);
        if !(*s).active {
            (*s).cb = cb;
            return s;
        }
    }
    base
}

unsafe fn aio_record(cb: *mut Aiocb, ret: isize, err: c_int) {
    let s = aio_slot(cb);
    (*s).cb = cb;
    (*s).ret = ret;
    (*s).err = err;
    (*s).active = true;
}

unsafe fn aio_find(cb: *const Aiocb) -> *mut AioSlot {
    let base = core::ptr::addr_of_mut!(AIO_SLOTS[0]);
    for i in 0..AIO_SLOTS.len() {
        let s = base.add(i);
        if (*s).cb == cb as *mut Aiocb && (*s).active { return s; }
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn aio_read(aiocbp: *mut Aiocb) -> c_int {
    if aiocbp.is_null() { ERRNO = EINVAL; return -1; }
    let r = crate::pread(
        (*aiocbp).aio_fildes,
        (*aiocbp).aio_buf,
        (*aiocbp).aio_nbytes,
        (*aiocbp).aio_offset,
    );
    if r < 0 {
        aio_record(aiocbp, -1, ERRNO);
    } else {
        aio_record(aiocbp, r, 0);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn aio_write(aiocbp: *mut Aiocb) -> c_int {
    if aiocbp.is_null() { ERRNO = EINVAL; return -1; }
    let r = crate::pwrite(
        (*aiocbp).aio_fildes,
        (*aiocbp).aio_buf as *const u8,
        (*aiocbp).aio_nbytes,
        (*aiocbp).aio_offset,
    );
    if r < 0 {
        aio_record(aiocbp, -1, ERRNO);
    } else {
        aio_record(aiocbp, r, 0);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn aio_fsync(op: c_int, aiocbp: *mut Aiocb) -> c_int {
    if aiocbp.is_null() { ERRNO = EINVAL; return -1; }
    let r = if op == 1 { crate::fdatasync((*aiocbp).aio_fildes) } else { crate::fsync((*aiocbp).aio_fildes) };
    if r < 0 {
        aio_record(aiocbp, -1, ERRNO);
    } else {
        aio_record(aiocbp, 0, 0);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn aio_error(aiocbp: *const Aiocb) -> c_int {
    if aiocbp.is_null() { return EINVAL; }
    let s = aio_find(aiocbp);
    if s.is_null() { return EINVAL; }
    if !(*s).active { return EINVAL; }
    (*s).err
}

#[no_mangle]
pub unsafe extern "C" fn aio_return(aiocbp: *mut Aiocb) -> isize {
    if aiocbp.is_null() { ERRNO = EINVAL; return -1; }
    let s = aio_find(aiocbp);
    if s.is_null() { ERRNO = EINVAL; return -1; }
    let r = (*s).ret;
    (*s).active = false;
    (*s).cb = core::ptr::null_mut();
    r
}

#[no_mangle]
pub unsafe extern "C" fn aio_cancel(fd: c_int, aiocbp: *mut Aiocb) -> c_int {
    if !aiocbp.is_null() {
        let s = aio_find(aiocbp);
        if !s.is_null() && (*s).active && (*aiocbp).aio_fildes == fd {
            aio_record(aiocbp, -1, ECANCELED);
            return 1; // AIO_CANCELED
        }
        return 2; // AIO_ALLDONE
    }
    let base = core::ptr::addr_of_mut!(AIO_SLOTS[0]);
    let mut canceled = false;
    for i in 0..AIO_SLOTS.len() {
        let s = base.add(i);
        if (*s).active && !(*s).cb.is_null() && (*(*s).cb).aio_fildes == fd {
            (*s).ret = -1;
            (*s).err = ECANCELED;
            canceled = true;
        }
    }
    if canceled { 1 } else { 2 }
}

#[no_mangle]
pub unsafe extern "C" fn aio_suspend(
    aiocb_list: *const *const Aiocb,
    nent: c_int,
    _timeout: *const timespec,
) -> c_int {
    if nent <= 0 || aiocb_list.is_null() { ERRNO = EINVAL; return -1; }
    // Every request completes synchronously, so all of them are already done.
    for i in 0..nent as usize {
        let cb = *aiocb_list.add(i);
        if cb.is_null() { continue; }
        if aio_find(cb).is_null() {
            ERRNO = EINVAL;
            return -1;
        }
    }
    0
}

unsafe fn aio_notify(sev: *const SigEvent) {
    if sev.is_null() { return; }
    match (*sev).sigev_notify {
        SIGEV_SIGNAL_V => {
            crate::sigqueue(crate::getpid(), (*sev).sigev_signo, (*sev).sigev_value as *mut c_void);
        }
        SIGEV_THREAD_V => {
            let fnptr = core::ptr::read_unaligned((*sev).sigev_un.as_ptr().add(0) as *const usize);
            if fnptr != 0 {
                let f: unsafe extern "C" fn(usize) = core::mem::transmute(fnptr);
                f((*sev).sigev_value);
            }
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn lio_listio(
    mode: c_int,
    aiocb_list: *const *mut Aiocb,
    nent: c_int,
    sevp: *mut SigEvent,
) -> c_int {
    if nent < 0 || aiocb_list.is_null() { ERRNO = EINVAL; return -1; }
    let mut ok = 0;
    for i in 0..nent as usize {
        let cb = *aiocb_list.add(i);
        if cb.is_null() { continue; }
        let r = match (*cb).aio_lio_opcode {
            0 => aio_read(cb),
            1 => aio_write(cb),
            _ => 0,
        };
        if r == 0 { ok += 1; }
    }
    if mode == 1 && !sevp.is_null() {
        aio_notify(sevp);
    }
    if ok == 0 && nent > 0 { -1 } else { 0 }
}

// ============================================================
// POSIX message queues: attributes and notification
// ============================================================

#[repr(C)]
pub struct MqAttr {
    pub mq_flags: c_long,
    pub mq_maxmsg: c_long,
    pub mq_msgsize: c_long,
    pub mq_curmsgs: c_long,
}

#[derive(Copy, Clone)]
struct MqNotify {
    mqd: i64,
    sev: SigEvent,
    used: bool,
}

static mut MQ_NOTIFIES: [MqNotify; 8] = [MqNotify {
    mqd: -1,
    sev: SigEvent { sigev_value: 0, sigev_signo: 0, sigev_notify: 0, sigev_un: [0; 48] },
    used: false,
}; 8];

unsafe fn mq_attr_fill(mqd: i64, attr: *mut MqAttr) -> c_int {
    if attr.is_null() { ERRNO = EINVAL; return -1; }
    let mut st: crate::Stat = core::mem::zeroed();
    if crate::fstat(mqd as c_int, &mut st) != 0 { return -1; }
    (*attr).mq_flags = if (crate::fcntl(mqd as c_int, 3) & 0o4000) != 0 { 0o4000 } else { 0 };
    (*attr).mq_msgsize = 4096;
    (*attr).mq_maxmsg = if st.st_size > 0 { (st.st_size as c_long / 4096) + 1 } else { 1 };
    (*attr).mq_curmsgs = 0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn mq_getattr(mqd: i64, attr: *mut MqAttr) -> c_int {
    mq_attr_fill(mqd, attr)
}

#[no_mangle]
pub unsafe extern "C" fn mq_setattr(mqd: i64, attr: *const MqAttr, oldattr: *mut MqAttr) -> c_int {
    if attr.is_null() { ERRNO = EINVAL; return -1; }
    if !oldattr.is_null() && mq_attr_fill(mqd, oldattr) != 0 { return -1; }
    let fd = mqd as c_int;
    let mut fl = crate::fcntl(fd, 3);
    if fl < 0 { return -1; }
    if ((*attr).mq_flags & 0o4000) != 0 { fl |= 0o4000; } else { fl &= !0o4000; }
    crate::fcntl(fd, 4, fl)
}

#[no_mangle]
pub unsafe extern "C" fn mq_notify(mqd: i64, notification: *const SigEvent) -> c_int {
    let base = core::ptr::addr_of_mut!(MQ_NOTIFIES[0]);
    // NULL deregisters.
    if notification.is_null() {
        for i in 0..MQ_NOTIFIES.len() {
            let s = base.add(i);
            if (*s).used && (*s).mqd == mqd { (*s).used = false; return 0; }
        }
        ERRNO = EINVAL;
        return -1;
    }
    for i in 0..MQ_NOTIFIES.len() {
        let s = base.add(i);
        if (*s).used && (*s).mqd == mqd {
            (*s).sev = *notification;
            return 0;
        }
    }
    for i in 0..MQ_NOTIFIES.len() {
        let s = base.add(i);
        if !(*s).used {
            (*s).used = true;
            (*s).mqd = mqd;
            (*s).sev = *notification;
            return 0;
        }
    }
    ERRNO = EBUSY;
    -1
}

unsafe fn mq_notify_fire(mqd: i64) {
    let base = core::ptr::addr_of_mut!(MQ_NOTIFIES[0]);
    for i in 0..MQ_NOTIFIES.len() {
        let s = base.add(i);
        if (*s).used && (*s).mqd == mqd {
            let sev = (*s).sev;
            (*s).used = false;
            aio_notify(&sev);
            return;
        }
    }
}

// ============================================================
// pthread introspection
// ============================================================

static mut PTHREAD_DEFAULT_STACK: usize = STACK_SIZE;
static mut PTHREAD_DEFAULT_GUARD: usize = 4096;
static mut PTHREAD_CONCURRENCY: c_int = 0;

/// Find the mapping that contains `addr` by reading the tail of
/// /proc/self/maps. Used for the main thread, whose stack was set up by the
/// loader and is not tracked in a `Thread` slot.
unsafe fn mapping_containing(addr: usize) -> Option<(usize, usize)> {
    let fd = crate::open(b"/proc/self/maps\0".as_ptr() as *const core::ffi::c_char, 0, 0);
    if fd < 0 {
        return None;
    }
    let size: i64 = crate::lseek(fd, 0, 2);
    let want: i64 = if size > 16384 { 16384 } else { size };
    let start = if size > want { size - want } else { 0 };
    if crate::lseek(fd, start, 0) < 0 {
        crate::close(fd);
        return None;
    }
    let mut buf = [0u8; 16384];
    let mut len = 0usize;
    while len < buf.len() {
        let n = crate::read(
            fd,
            buf.as_mut_ptr().add(len) as *mut core::ffi::c_void,
            buf.len() - len,
        );
        if n <= 0 {
            break;
        }
        len += n as usize;
    }
    crate::close(fd);

    let mut result = None;
    let mut line_start = 0usize;
    while line_start < len {
        let mut line_end = line_start;
        while line_end < len && buf[line_end] != b'\n' {
            line_end += 1;
        }
        let line = &buf[line_start..line_end];
        if let Some(dash) = line.iter().position(|&c| c == b'-') {
            let mut lo = 0usize;
            let mut ok = true;
            for &c in &line[..dash] {
                if c.is_ascii_digit() {
                    lo = lo * 16 + (c - b'0') as usize;
                } else if (b'a'..=b'f').contains(&c) {
                    lo = lo * 16 + (c - b'a' + 10) as usize;
                } else {
                    ok = false;
                    break;
                }
            }
            if ok && !line[..dash].is_empty() {
                let mut hi = 0usize;
                let mut i = dash + 1;
                while i < line.len() {
                    let c = line[i];
                    if c.is_ascii_digit() {
                        hi = hi * 16 + (c - b'0') as usize;
                    } else if (b'a'..=b'f').contains(&c) {
                        hi = hi * 16 + (c - b'a' + 10) as usize;
                    } else {
                        break;
                    }
                    i += 1;
                }
                if i > dash + 1 && lo <= addr && addr < hi {
                    result = Some((lo, hi - lo));
                }
            }
        }
        line_start = line_end + 1;
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn pthread_getattr_np(thread: PthreadT, attr: *mut pthread_attr_t) -> c_int {
    if attr.is_null() || thread == 0 { return EINVAL; }
    let slot = thread as *mut Thread;
    core::ptr::write_bytes(attr, 0, 1);
    let mut stack_base = (*slot).stack as usize;
    let mut stack_size = if (*slot).stack_size > 0 { (*slot).stack_size } else { PTHREAD_DEFAULT_STACK };
    if stack_base == 0 {
        // Main thread: the loader allocated its stack, so ask the kernel which
        // mapping holds the current stack pointer. Returning 0 here makes
        // runtimes such as V8 conclude they are running on a foreign stack.
        let sp: usize;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, rsp", out(reg) sp, options(nomem, nostack));
        #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
        core::arch::asm!("mv {}, sp", out(reg) sp, options(nomem, nostack));
        if let Some((base, size)) = mapping_containing(sp) {
            stack_base = base;
            stack_size = size;
        } else {
            stack_base = sp & !0xfff;
            stack_size = 4096;
        }
    }
    *((*attr).__i.as_mut_ptr() as *mut usize) = stack_size;
    *((*attr).__i.as_mut_ptr().add(2) as *mut usize) = PTHREAD_DEFAULT_GUARD;
    *((*attr).__i.as_mut_ptr().add(4) as *mut *mut c_void) = stack_base as *mut c_void;
    (*attr).__i[6] = if (*slot).detach_state == DT_DETACHED { 1 } else { 0 };
    (*attr).__i[7] = 0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_getattr_default_np(attr: *mut pthread_attr_t) -> c_int {
    if attr.is_null() { return EINVAL; }
    core::ptr::write_bytes(attr, 0, 1);
    *((*attr).__i.as_mut_ptr() as *mut usize) = PTHREAD_DEFAULT_STACK;
    *((*attr).__i.as_mut_ptr().add(2) as *mut usize) = PTHREAD_DEFAULT_GUARD;
    (*attr).__i[6] = 0;
    (*attr).__i[7] = 1;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_setattr_default_np(attr: *const pthread_attr_t) -> c_int {
    if attr.is_null() { return EINVAL; }
    let stack = *((*attr).__i.as_ptr() as *const usize);
    let guard = *((*attr).__i.as_ptr().add(2) as *const usize);
    // Reject attributes that cannot be made defaults.
    if (*attr).__i[6] != 0 { return EINVAL; }
    if stack > 0 {
        if stack < 16384 { return EINVAL; }
        PTHREAD_DEFAULT_STACK = stack;
    }
    if guard > 0 { PTHREAD_DEFAULT_GUARD = guard; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_getconcurrency() -> c_int {
    PTHREAD_CONCURRENCY
}

#[no_mangle]
pub unsafe extern "C" fn pthread_setconcurrency(level: c_int) -> c_int {
    if level < 0 { return EINVAL; }
    PTHREAD_CONCURRENCY = level;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_getcpuclockid(thread: PthreadT, clockid: *mut i32) -> c_int {
    if thread == 0 || clockid.is_null() { return EINVAL; }
    let slot = thread as *mut Thread;
    let tid = (*slot).tid as i64;
    *clockid = (((-tid - 1) * 8 + 6) & 0xffffffff) as i32;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_setname_np(thread: PthreadT, name: *const c_char) -> c_int {
    if thread == 0 || name.is_null() { return EINVAL; }
    if crate::strnlen(name as *const u8, 16) > 15 { return 34; }
    let slot = thread as *mut Thread;
    if slot == pthread_self() as *mut Thread {
        return if crate::prctl(15, name as c_long) < 0 { ERRNO } else { 0 };
    }
    let mut path = [0 as c_char; 48];
    let mut o = 0usize;
    for &c in b"/proc/self/task/" { path[o] = c as c_char; o += 1; }
    let mut tid = (*slot).tid as u32;
    let mut digits = [0u8; 10];
    let mut d = 0usize;
    if tid == 0 { digits[0] = b'0'; d = 1; }
    while tid > 0 { digits[d] = b'0' + (tid % 10) as u8; tid /= 10; d += 1; }
    while d > 0 { d -= 1; path[o] = digits[d] as c_char; o += 1; }
    for &c in b"/comm\0" { path[o] = c as c_char; o += 1; }
    let fd = crate::open(path.as_ptr(), 1, 0);
    if fd < 0 { return ERRNO; }
    let n = crate::strlen(name);
    let w = crate::write(fd, name as *const c_void, n);
    crate::close(fd);
    if w < 0 { ERRNO } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_getname_np(thread: PthreadT, name: *mut c_char, len: usize) -> c_int {
    if thread == 0 || name.is_null() { return EINVAL; }
    if len < 16 { return 34; }
    let slot = thread as *mut Thread;
    if slot == pthread_self() as *mut Thread {
        return if crate::prctl(16, name as c_long) < 0 { ERRNO } else { 0 };
    }
    let mut path = [0 as c_char; 48];
    let mut o = 0usize;
    for &c in b"/proc/self/task/" { path[o] = c as c_char; o += 1; }
    let mut tid = (*slot).tid as u32;
    let mut digits = [0u8; 10];
    let mut d = 0usize;
    if tid == 0 { digits[0] = b'0'; d = 1; }
    while tid > 0 { digits[d] = b'0' + (tid % 10) as u8; tid /= 10; d += 1; }
    while d > 0 { d -= 1; path[o] = digits[d] as c_char; o += 1; }
    for &c in b"/comm\0" { path[o] = c as c_char; o += 1; }
    let fd = crate::open(path.as_ptr(), 0, 0);
    if fd < 0 { return ERRNO; }
    let n = crate::read(fd, name as *mut c_void, len);
    crate::close(fd);
    if n < 0 { return ERRNO; }
    let mut l = n as usize;
    if l > 0 && *name.add(l - 1) == b'\n' as c_char { l -= 1; }
    *name.add(l) = 0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_getschedparam(thread: PthreadT, policy: *mut c_int, param: *mut sched_param) -> c_int {
    if thread == 0 || policy.is_null() || param.is_null() { return EINVAL; }
    let tid = (*(thread as *mut Thread)).tid;
    *policy = crate::sched_getscheduler(tid);
    crate::sched_getparam(tid, param as *mut c_void)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_setschedparam(thread: PthreadT, policy: c_int, param: *const sched_param) -> c_int {
    if thread == 0 || param.is_null() { return EINVAL; }
    let tid = (*(thread as *mut Thread)).tid;
    crate::sched_setscheduler(tid, policy, param as *const c_void)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_setschedprio(thread: PthreadT, prio: c_int) -> c_int {
    if thread == 0 { return EINVAL; }
    let tid = (*(thread as *mut Thread)).tid;
    let p = sched_param { sched_priority: prio };
    crate::sched_setparam(tid, &p as *const sched_param as *const c_void)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_getprioceiling(mutex: *const pthread_mutex_t, ceiling: *mut c_int) -> c_int {
    if mutex.is_null() || ceiling.is_null() { return EINVAL; }
    *ceiling = 0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_setprioceiling(mutex: *mut pthread_mutex_t, ceiling: c_int, old: *mut c_int) -> c_int {
    if mutex.is_null() { return EINVAL; }
    if !old.is_null() { *old = 0; }
    let _ = ceiling;
    0
}

// ============================================================
// posix_spawn attributes and file actions
// ============================================================


#[repr(C)]
pub struct posix_spawn_file_actions_t {
    __pad0: [c_int; 2],
    __actions: *mut c_void,
    __pad: [c_int; 16],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct posix_spawnattr_t {
    __flags: c_int,
    __pgrp: c_int,
    __def: [c_ulong; 16],
    __mask: [c_ulong; 16],
    __prio: c_int,
    __pol: c_int,
    __fn: *mut c_void,
    __pad: [u8; 64 - core::mem::size_of::<*mut c_void>()],
}

#[repr(C)]
struct SpawnAction {
    next: *mut SpawnAction,
    kind: c_int,
    fd: c_int,
    newfd: c_int,
    oflag: c_int,
    mode: u32,
    path: *mut c_char,
    path_owned: bool,
}

static mut SPAWN_DEFAULT_ATTR: posix_spawnattr_t = posix_spawnattr_t {
    __flags: 0,
    __pgrp: 0,
    __def: [0; 16],
    __mask: [0; 16],
    __prio: 0,
    __pol: 0,
    __fn: core::ptr::null_mut(),
    __pad: [0; 64 - core::mem::size_of::<*mut c_void>()],
};

unsafe fn spawn_actions(fa: *const posix_spawn_file_actions_t) -> *mut SpawnAction {
    (*fa).__actions as *mut SpawnAction
}

unsafe fn spawn_action_add(
    fa: *mut posix_spawn_file_actions_t,
    kind: c_int,
    fd: c_int,
    newfd: c_int,
    oflag: c_int,
    mode: u32,
    path: *const c_char,
) -> c_int {
    let node = crate::calloc(1, core::mem::size_of::<SpawnAction>()) as *mut SpawnAction;
    if node.is_null() { return ENOMEM; }
    (*node).kind = kind;
    (*node).fd = fd;
    (*node).newfd = newfd;
    (*node).oflag = oflag;
    (*node).mode = mode;
    (*node).next = core::ptr::null_mut();
    if !path.is_null() {
        let n = crate::strlen(path);
        let p = crate::malloc(n + 1) as *mut c_char;
        if p.is_null() { crate::free(node as *mut c_void); return ENOMEM; }
        core::ptr::copy_nonoverlapping(path, p, n + 1);
        (*node).path = p;
        (*node).path_owned = true;
    }
    if (*fa).__actions.is_null() {
        (*fa).__actions = node as *mut c_void;
    } else {
        let mut tail = (*fa).__actions as *mut SpawnAction;
        while !(*tail).next.is_null() { tail = (*tail).next; }
        (*tail).next = node;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_init(fa: *mut posix_spawn_file_actions_t) -> c_int {
    if fa.is_null() { return EINVAL; }
    core::ptr::write_bytes(fa, 0, 1);
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_destroy(fa: *mut posix_spawn_file_actions_t) -> c_int {
    if fa.is_null() { return EINVAL; }
    let mut p = spawn_actions(fa);
    while !p.is_null() {
        let next = (*p).next;
        if (*p).path_owned && !(*p).path.is_null() { crate::free((*p).path as *mut c_void); }
        crate::free(p as *mut c_void);
        p = next;
    }
    (*fa).__actions = core::ptr::null_mut();
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_addclose(fa: *mut posix_spawn_file_actions_t, fd: c_int) -> c_int {
    if fa.is_null() || fd < 0 { return EINVAL; }
    spawn_action_add(fa, 0, fd, 0, 0, 0, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_adddup2(fa: *mut posix_spawn_file_actions_t, fd: c_int, newfd: c_int) -> c_int {
    if fa.is_null() || fd < 0 || newfd < 0 { return EINVAL; }
    spawn_action_add(fa, 1, fd, newfd, 0, 0, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_addopen(
    fa: *mut posix_spawn_file_actions_t,
    fd: c_int,
    path: *const c_char,
    oflag: c_int,
    mode: u32,
) -> c_int {
    if fa.is_null() || fd < 0 || path.is_null() { return EINVAL; }
    spawn_action_add(fa, 2, fd, 0, oflag, mode, path)
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_addchdir_np(fa: *mut posix_spawn_file_actions_t, path: *const c_char) -> c_int {
    if fa.is_null() || path.is_null() { return EINVAL; }
    spawn_action_add(fa, 3, 0, 0, 0, 0, path)
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_addfchdir_np(fa: *mut posix_spawn_file_actions_t, fd: c_int) -> c_int {
    if fa.is_null() || fd < 0 { return EINVAL; }
    spawn_action_add(fa, 4, fd, 0, 0, 0, core::ptr::null())
}

unsafe fn spawn_apply_actions(fa: *const posix_spawn_file_actions_t) {
    if fa.is_null() { return; }
    let mut p = spawn_actions(fa);
    while !p.is_null() {
        match (*p).kind {
            0 => { crate::close((*p).fd); }
            1 => { crate::dup2((*p).fd, (*p).newfd); }
            2 => {
                let fd = crate::open((*p).path, (*p).oflag, (*p).mode as c_int);
                if fd >= 0 && fd != (*p).fd {
                    crate::dup2(fd, (*p).fd);
                    crate::close(fd);
                }
            }
            3 => { crate::chdir((*p).path); }
            4 => { crate::fchdir((*p).fd); }
            _ => {}
        }
        p = (*p).next;
    }
}

unsafe fn spawn_apply_attr(attr: *const posix_spawnattr_t) {
    if attr.is_null() { return; }
    let flags = (*attr).__flags;
    if flags & 1 != 0 {
        if crate::geteuid() == 0 {
            crate::setgid(crate::getgid());
            crate::setuid(crate::getuid());
        }
    }
    if flags & 2 != 0 { crate::setpgid(0, (*attr).__pgrp); }
    if flags & 4 != 0 {
        for sig in 1..65 {
            let idx = (sig - 1) / 64;
            let bit = (sig - 1) % 64;
            if idx < 16 && ((*attr).__def[idx] >> bit) & 1 != 0 {
                crate::signal(sig as c_int, 0);
            }
        }
    }
    if flags & 8 != 0 {
        crate::sigprocmask(2, (*attr).__mask.as_ptr() as *const SigSetT, core::ptr::null_mut());
    }
    if flags & 16 != 0 {
        let p = sched_param { sched_priority: (*attr).__prio };
        crate::sched_setparam(0, &p as *const sched_param as *const c_void);
    }
    if flags & 32 != 0 {
        let p = sched_param { sched_priority: (*attr).__prio };
        crate::sched_setscheduler(0, (*attr).__pol, &p as *const sched_param as *const c_void);
    }
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_init(attr: *mut posix_spawnattr_t) -> c_int {
    if attr.is_null() { return EINVAL; }
    *attr = SPAWN_DEFAULT_ATTR;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_destroy(attr: *mut posix_spawnattr_t) -> c_int {
    if attr.is_null() { return EINVAL; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_getflags(attr: *const posix_spawnattr_t, flags: *mut i16) -> c_int {
    if attr.is_null() || flags.is_null() { return EINVAL; }
    *flags = (*attr).__flags as i16;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_setflags(attr: *mut posix_spawnattr_t, flags: i16) -> c_int {
    if attr.is_null() { return EINVAL; }
    if (flags as c_int) & !0x7f != 0 { return EINVAL; }
    (*attr).__flags = flags as c_int;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_getpgroup(attr: *const posix_spawnattr_t, pgroup: *mut c_int) -> c_int {
    if attr.is_null() || pgroup.is_null() { return EINVAL; }
    *pgroup = (*attr).__pgrp;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_setpgroup(attr: *mut posix_spawnattr_t, pgroup: c_int) -> c_int {
    if attr.is_null() { return EINVAL; }
    (*attr).__pgrp = pgroup;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_getschedparam(attr: *const posix_spawnattr_t, param: *mut sched_param) -> c_int {
    if attr.is_null() || param.is_null() { return EINVAL; }
    (*param).sched_priority = (*attr).__prio;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_setschedparam(attr: *mut posix_spawnattr_t, param: *const sched_param) -> c_int {
    if attr.is_null() || param.is_null() { return EINVAL; }
    (*attr).__prio = (*param).sched_priority;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_getschedpolicy(attr: *const posix_spawnattr_t, policy: *mut c_int) -> c_int {
    if attr.is_null() || policy.is_null() { return EINVAL; }
    *policy = (*attr).__pol;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_setschedpolicy(attr: *mut posix_spawnattr_t, policy: c_int) -> c_int {
    if attr.is_null() { return EINVAL; }
    (*attr).__pol = policy;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_getsigdefault(attr: *const posix_spawnattr_t, set: *mut SigSetT) -> c_int {
    if attr.is_null() || set.is_null() { return EINVAL; }
    *set = (*attr).__def[0];
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_setsigdefault(attr: *mut posix_spawnattr_t, set: *const SigSetT) -> c_int {
    if attr.is_null() || set.is_null() { return EINVAL; }
    (*attr).__def = [0; 16];
    (*attr).__def[0] = *set;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_getsigmask(attr: *const posix_spawnattr_t, set: *mut SigSetT) -> c_int {
    if attr.is_null() || set.is_null() { return EINVAL; }
    *set = (*attr).__mask[0];
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_setsigmask(attr: *mut posix_spawnattr_t, set: *const SigSetT) -> c_int {
    if attr.is_null() || set.is_null() { return EINVAL; }
    (*attr).__mask = [0; 16];
    (*attr).__mask[0] = *set;
    0
}

// ============================================================
// stdio internals
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn __fbufsize(f: *mut FILE) -> usize {
    if f.is_null() { return 0; }
    (*f).buf_size
}

#[no_mangle]
pub unsafe extern "C" fn __flbf(f: *mut FILE) -> c_int {
    if f.is_null() { return 0; }
    if (*f).lbf >= 0 { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn __fpending(f: *mut FILE) -> usize {
    if f.is_null() || (*f).wend.is_null() { return 0; }
    (*f).wpos as usize - (*f).wbase as usize
}

#[no_mangle]
pub unsafe extern "C" fn __fpurge(f: *mut FILE) {
    if f.is_null() { return; }
    (*f).wpos = core::ptr::null_mut();
    (*f).wbase = core::ptr::null_mut();
    (*f).wend = core::ptr::null_mut();
    (*f).rpos = core::ptr::null_mut();
    (*f).rend = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn __freadable(f: *mut FILE) -> c_int {
    if f.is_null() { return 0; }
    if (*f).flags & F_NORD == 0 { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn __freadahead(f: *mut FILE) -> usize {
    if f.is_null() || (*f).rend.is_null() { return 0; }
    (*f).rend as usize - (*f).rpos as usize
}

#[no_mangle]
pub unsafe extern "C" fn __freading(f: *mut FILE) -> c_int {
    if f.is_null() { return 0; }
    if (*f).flags & F_NOWR != 0 || !(*f).rend.is_null() { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn __freadptr(f: *mut FILE, sizep: *mut usize) -> *const c_char {
    if f.is_null() || sizep.is_null() { return core::ptr::null(); }
    if (*f).rpos == (*f).rend { *sizep = 0; return core::ptr::null(); }
    *sizep = (*f).rend as usize - (*f).rpos as usize;
    (*f).rpos as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn __freadptrinc(f: *mut FILE, inc: usize) {
    if f.is_null() || (*f).rpos.is_null() { return; }
    let left = (*f).rend as usize - (*f).rpos as usize;
    let step = if inc > left { left } else { inc };
    (*f).rpos = (*f).rpos.add(step);
}

#[no_mangle]
pub unsafe extern "C" fn __fsetlocking(_f: *mut FILE, _type: c_int) -> c_int {
    1
}

#[no_mangle]
pub unsafe extern "C" fn __fwritable(f: *mut FILE) -> c_int {
    if f.is_null() { return 0; }
    if (*f).flags & F_NOWR == 0 { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn __fwriting(f: *mut FILE) -> c_int {
    if f.is_null() { return 0; }
    if (*f).flags & F_NORD != 0 || !(*f).wend.is_null() { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn __fseterr(f: *mut FILE) {
    if !f.is_null() { (*f).flags |= F_ERR; }
}

#[no_mangle]
pub unsafe extern "C" fn __fputwc_unlocked(c: wchar_t, f: *mut FILE) -> wint_t {
    crate::fputwc(c, f)
}

// ============================================================
// Miscellaneous legacy calls
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn sigqueue(pid: c_int, sig: c_int, value: *mut c_void) -> c_int {
    let mut si = [0u8; 128];
    *(si.as_mut_ptr() as *mut c_int) = sig;
    *(si.as_mut_ptr().add(8) as *mut c_int) = -1; // SI_QUEUE
    *(si.as_mut_ptr().add(16) as *mut c_int) = crate::getpid();
    *(si.as_mut_ptr().add(20) as *mut u32) = crate::getuid();
    *(si.as_mut_ptr().add(24) as *mut *mut c_void) = value;
    let r = <Arch as Syscalls>::syscall3(sysno(129, 138), pid as i64, sig as i64, si.as_ptr() as i64);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn timespec_get(ts: *mut timespec, base: c_int) -> c_int {
    if base != 1 || ts.is_null() { return 0; }
    if crate::clock_gettime(0, ts) != 0 { return 0; }
    base
}

#[no_mangle]
pub unsafe extern "C" fn ualarm(usecs: u32, interval: u32) -> u32 {
    #[repr(C)]
    struct ITV {
        sec: i64,
        usec: i64,
    }
    let mut new: [ITV; 2] = [
        ITV { sec: (interval / 1_000_000) as i64, usec: (interval % 1_000_000) as i64 },
        ITV { sec: (usecs / 1_000_000) as i64, usec: (usecs % 1_000_000) as i64 },
    ];
    let mut old: [ITV; 2] = [ITV { sec: 0, usec: 0 }, ITV { sec: 0, usec: 0 }];
    if crate::setitimer(0, new.as_mut_ptr() as *const c_void, old.as_mut_ptr() as *mut c_void) != 0 {
        return u32::MAX;
    }
    (old[0].sec as u32) * 1_000_000 + old[0].usec as u32
}

static mut LCKPWDF_FD: c_int = -1;

#[repr(C)]
#[derive(Copy, Clone)]
struct FlockT {
    l_type: i16,
    l_whence: i16,
    l_start: i64,
    l_len: i64,
    l_pid: i32,
    __pad: i32,
}

#[no_mangle]
pub unsafe extern "C" fn lckpwdf() -> c_int {
    if LCKPWDF_FD >= 0 { return 0; }
    let fd = crate::open(
        b"/etc/.pwd.lock\0".as_ptr() as *const c_char,
        1 | 0x40 | 0x80000,
        0o600,
    );
    if fd < 0 { return -1; }
    let fl = FlockT { l_type: 1, l_whence: 0, l_start: 0, l_len: 0, l_pid: 0, __pad: 0 };
    if crate::fcntl(fd, 7, &fl as *const FlockT as *mut c_void) != 0 {
        crate::close(fd);
        return -1;
    }
    LCKPWDF_FD = fd;
    0
}

#[no_mangle]
pub unsafe extern "C" fn ulckpwdf() -> c_int {
    if LCKPWDF_FD < 0 { return -1; }
    let fd = LCKPWDF_FD;
    LCKPWDF_FD = -1;
    crate::close(fd);
    0
}

#[no_mangle]
pub unsafe extern "C" fn malloc_usable_size(ptr: *mut u8) -> usize {
    crate::alloc_usable_size(ptr)
}

#[no_mangle]
pub unsafe extern "C" fn __wcsftime_l(
    s: *mut wchar_t,
    max: usize,
    fmt: *const wchar_t,
    tm: *const c_void,
    _loc: locale_t,
) -> usize {
    crate::wcsftime(s as *mut u32, max, fmt as *const u32, tm)
}

#[no_mangle]
pub unsafe extern "C" fn __wctype_l(name: *const c_char, _loc: locale_t) -> wctype_t {
    crate::wctype(name)
}

#[no_mangle]
pub unsafe extern "C" fn __wctrans_l(name: *const c_char, _loc: locale_t) -> wctrans_t {
    crate::wctrans(name)
}

#[no_mangle]
pub unsafe extern "C" fn __fgetwc_unlocked(f: *mut FILE) -> wint_t {
    crate::fgetwc(f)
}

#[no_mangle]
pub unsafe extern "C" fn _flushlbf() {
    crate::fflush(core::ptr::null_mut());
}
