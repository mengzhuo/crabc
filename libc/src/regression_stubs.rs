// Stubs and minimal implementations for symbols required by libc-test
// regression cases that currently fail to link.

const _SC_ARG_MAX: c_int = 0;
const _SC_CHILD_MAX: c_int = 1;
const _SC_CLK_TCK: c_int = 2;
const _SC_NGROUPS_MAX: c_int = 3;
const _SC_OPEN_MAX: c_int = 4;
const _SC_STREAM_MAX: c_int = 5;
const _SC_TZNAME_MAX: c_int = 6;
const _SC_PAGE_SIZE: c_int = 30;
const _SC_NPROCESSORS_CONF: c_int = 83;
const _SC_NPROCESSORS_ONLN: c_int = 84;
const _SC_PHYS_PAGES: c_int = 85;
const _SC_AVPHYS_PAGES: c_int = 86;
const _SC_IOV_MAX: c_int = 60;
const _SC_SEM_NSEMS_MAX: c_int = 32;
const _SC_SEM_VALUE_MAX: c_int = 33;
const _SC_MQ_OPEN_MAX: c_int = 27;
const _SC_MQ_PRIO_MAX: c_int = 28;
const _SC_DELAYTIMER_MAX: c_int = 26;
const _SC_THREAD_DESTRUCTOR_ITERATIONS: c_int = 73;
const _SC_THREAD_KEYS_MAX: c_int = 74;
const _SC_THREAD_STACK_MIN: c_int = 75;
const _SC_LOGIN_NAME_MAX: c_int = 71;
const _SC_TTY_NAME_MAX: c_int = 72;
const _SC_SYMLOOP_MAX: c_int = 173;
const _SC_HOST_NAME_MAX: c_int = 180;
const _SC_NZERO: c_int = 109;
const _SC_VERSION: c_int = 29;
const _SC_2_VERSION: c_int = 46;
const _SC_XOPEN_VERSION: c_int = 89;

#[no_mangle]
pub unsafe extern "C" fn mmap(
    addr: *mut c_void,
    len: usize,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    off: i64,
) -> *mut c_void {
    let r = crate::syscall(
        SYS_MMAP,
        addr as c_long,
        len as c_long,
        prot as c_long,
        flags as c_long,
        fd as c_long,
        off as c_long,
    );
    if r == -1 {
        return -1isize as *mut c_void;
    }
    r as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn munmap(addr: *mut c_void, len: usize) -> c_int {
    crate::syscall(SYS_MUNMAP, addr as c_long, len as c_long, 0, 0, 0, 0) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn pread(fd: c_int, buf: *mut c_void, count: usize, offset: i64) -> isize {
    crate::syscall(
        SYS_PREAD64,
        fd as c_long,
        buf as c_long,
        count as c_long,
        offset as c_long,
        0,
        0,
    ) as isize
}

unsafe fn rlimit_sc(resource: c_int) -> c_long {
    let mut lim: crate::Rlimit = core::mem::zeroed();
    if crate::getrlimit(resource, &mut lim) != 0 {
        return -1;
    }
    if lim.rlim_cur == !0u64 {
        return -1;
    }
    if lim.rlim_cur > c_long::MAX as u64 {
        return c_long::MAX;
    }
    lim.rlim_cur as c_long
}

unsafe fn online_cpus() -> c_long {
    let mut set = [0u8; 128];
    let r = crate::sched_getaffinity(0, set.len(), set.as_mut_ptr());
    if r != 0 {
        return 1;
    }
    let mut count = 0u32;
    for b in set.iter() {
        count += b.count_ones();
    }
    if count == 0 { 1 } else { count as c_long }
}

unsafe fn memory_pages(free: bool) -> c_long {
    // struct sysinfo: totalram/freeram/bufferram/mem_unit at fixed offsets.
    let mut info = [0u64; 16];
    if crate::sysinfo(info.as_mut_ptr() as *mut core::ffi::c_void) != 0 {
        return -1;
    }
    let total = if free {
        info[5].saturating_add(info[7])
    } else {
        info[4]
    };
    let unit = (info[13] & 0xffff_ffff) as u64;
    let bytes = total.saturating_mul(if unit == 0 { 1 } else { unit });
    (bytes / 4096) as c_long
}

#[no_mangle]
pub unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    const POSIX_VERSION: c_long = 200809;
    match name {
        _SC_ARG_MAX => 131072,
        _SC_CHILD_MAX => rlimit_sc(crate::RLIMIT_NPROC),
        _SC_CLK_TCK => 100,
        _SC_NGROUPS_MAX => 32,
        _SC_OPEN_MAX => rlimit_sc(crate::RLIMIT_NOFILE),
        _SC_STREAM_MAX => -1,
        _SC_TZNAME_MAX => 6,
        7 | 8 => 1,                    // JOB_CONTROL, SAVED_IDS
        9 | 11 | 12 => POSIX_VERSION,  // REALTIME_SIGNALS, TIMERS, ASYNC_IO
        10 | 13 | 14 => -1,            // PRIORITY_SCHEDULING, PRIORITIZED_IO, SYNC_IO
        15..=22 => POSIX_VERSION,      // FSYNC .. SHARED_MEMORY_OBJECTS
        23 | 24 => -1,                 // AIO_LISTIO_MAX, AIO_MAX
        25 => 0,                       // AIO_PRIO_DELTA_MAX
        _SC_DELAYTIMER_MAX => 0x7fffffff,
        _SC_MQ_OPEN_MAX => -1,
        _SC_MQ_PRIO_MAX => 32768,
        _SC_VERSION => POSIX_VERSION,
        _SC_PAGE_SIZE => 4096,
        31 => 30,                      // _SC_RTSIG_MAX
        _SC_SEM_NSEMS_MAX => 256,
        _SC_SEM_VALUE_MAX => 0x7fffffff,
        34 | 35 => -1,                 // SIGQUEUE_MAX, TIMER_MAX
        36 => 99, 37 => 2048, 38 => 99, 39 => 1000, // POSIX2 bc limits
        40 => 2,                       // COLL_WEIGHTS_MAX
        42 | 43 => -1,                 // EXPR_NEST_MAX, LINE_MAX
        44 => 255,                     // RE_DUP_MAX
        _SC_2_VERSION => POSIX_VERSION,
        47 => POSIX_VERSION,           // _SC_2_C_BIND
        48..=52 => -1,                 // 2_C_DEV .. 2_LOCALEDEF
        _SC_IOV_MAX => 1024,
        67 | 68 => POSIX_VERSION,      // THREADS, THREAD_SAFE_FUNCTIONS
        69 | 70 => -1,                 // GETGR/PW_R_SIZE_MAX
        _SC_LOGIN_NAME_MAX => 256,
        _SC_TTY_NAME_MAX => 32,
        _SC_THREAD_DESTRUCTOR_ITERATIONS => 4,
        _SC_THREAD_KEYS_MAX => 128,
        _SC_THREAD_STACK_MIN => 2048,
        76 => -1,                      // THREAD_THREADS_MAX
        77 | 78 | 79 => POSIX_VERSION, // ATTR_STACKADDR/SIZE, PRIO_SCHED
        80 | 81 => -1,                 // PRIO_INHERIT/PROTECT
        82 => POSIX_VERSION,           // PROCESS_SHARED
        _SC_NPROCESSORS_CONF | _SC_NPROCESSORS_ONLN => online_cpus(),
        _SC_PHYS_PAGES => memory_pages(false),
        _SC_AVPHYS_PAGES => memory_pages(true),
        87 | 88 => -1,                 // ATEXIT_MAX, PASS_MAX
        _SC_XOPEN_VERSION => 700,
        90 => 700,                     // XOPEN_XCU_VERSION
        91 => 1,                       // XOPEN_UNIX
        92 => -1,                      // XOPEN_CRYPT
        93 | 94 => 1,                  // XOPEN_ENH_I18N, XOPEN_SHM
        95 => -1,                      // 2_CHAR_TERM
        97 => -1,                      // 2_UPE
        98..=100 => -1,                // XOPEN_XPG2..4
        _SC_NZERO => 20,
        125 | 126 | 128 => -1,         // XBS5 ILP32/LPBIG
        127 => 1,                      // XBS5 LP64_OFF64
        129..=131 => -1,               // XOPEN legacy/realtime
        132 | 133 => POSIX_VERSION,    // ADVISORY_INFO, BARRIERS
        137 | 138 | 139 => POSIX_VERSION,
        149 => POSIX_VERSION,          // MONOTONIC_CLOCK
        153 | 154 | 155 | 157 | 159 => POSIX_VERSION,
        160 | 161 => -1,
        164 => POSIX_VERSION,          // TIMEOUTS
        165 => -1,                     // TYPED_MEMORY_OBJECTS
        168..=172 => -1,               // 2_PBS
        _SC_SYMLOOP_MAX => 40,
        174 => 0,                      // STREAMS
        175 => -1,
        176 | 177 | 179 => -1,         // V6 ILP32/LPBIG
        178 => 1,                      // V6_LP64_OFF64
        _SC_HOST_NAME_MAX => 255,
        181..=184 => -1,               // TRACE
        235 | 236 => POSIX_VERSION,    // IPV6, RAW_SOCKETS
        237 | 238 | 240 => -1,         // V7 variants
        239 => 1,                      // V7_LP64_OFF64
        241..=245 => -1,               // SS_REPL_MAX, TRACE_*
        _ => {
            crate::__errno_location().write(EINVAL);
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn execle(path: *const c_char, arg: *const c_char, mut args: ...) -> c_int {
    let mut argv: [*const c_char; 128] = [core::ptr::null(); 128];
    argv[0] = arg;
    let mut n = 1usize;
    loop {
        let a: *const c_char = args.next_arg();
        if a.is_null() {
            break;
        }
        if n >= argv.len() - 1 {
            crate::__errno_location().write(E2BIG);
            return -1;
        }
        argv[n] = a;
        n += 1;
    }
    argv[n] = core::ptr::null();
    let envp: *const *const c_char = args.next_arg();
    crate::execve(path, argv.as_ptr(), envp)
}

#[no_mangle]
pub unsafe extern "C" fn flockfile(f: *mut crate::FILE) {
    if (*f).lock < 0 { return; }
    let tid = crate::sys_gettid() as i32;
    loop {
        let v = crate::a_load(&raw const (*f).lock);
        if v == tid {
            (*f).lockcount += 1;
            return;
        }
        if v == 0 && crate::a_cas(&raw mut (*f).lock, 0, tid) == 0 {
            (*f).lockcount = 1;
            return;
        }
        if v != 0 {
            crate::futex_wait(&raw mut (*f).lock, v);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn funlockfile(f: *mut crate::FILE) {
    if (*f).lock < 0 { return; }
    if (*f).lockcount > 1 {
        (*f).lockcount -= 1;
        return;
    }
    (*f).lockcount = 0;
    crate::a_store(&raw mut (*f).lock, 0);
    crate::futex_wake(&raw mut (*f).lock, 1);
}

#[no_mangle]
pub unsafe extern "C" fn __libc_current_sigrtmin() -> c_int {
    34
}

#[no_mangle]
pub unsafe extern "C" fn strdup(s: *const c_char) -> *mut c_char {
    if s.is_null() {
        return core::ptr::null_mut();
    }
    let len = crate::strlen(s);
    let p = crate::malloc(len + 1) as *mut c_char;
    if p.is_null() {
        return core::ptr::null_mut();
    }
    crate::memcpy(p as *mut c_void, s as *const c_void, len + 1);
    p
}

static mut MKTEMP_COUNTER: c_uint = 0;

unsafe fn mktemp_internal(
    template: *mut c_char,
    mkdir_fn: Option<unsafe extern "C" fn(*const c_char, c_uint) -> c_int>,
) -> *mut c_char {
    if template.is_null() {
        crate::__errno_location().write(EINVAL);
        return core::ptr::null_mut();
    }
    let len = crate::strlen(template);
    if len < 6 {
        crate::__errno_location().write(EINVAL);
        return core::ptr::null_mut();
    }
    let mut xcount = 0usize;
    while xcount < 6 {
        let ch = *template.add(len - 1 - xcount) as u8;
        if ch != b'X' {
            break;
        }
        xcount += 1;
    }
    if xcount != 6 {
        crate::__errno_location().write(EINVAL);
        return core::ptr::null_mut();
    }
    let _pid = crate::getpid();
    let mut c = MKTEMP_COUNTER;
    for _ in 0..1000 {
        let mut n = c;
        for i in 0..6 {
            let digit = (n % 36) as u8;
            let ch = if digit < 10 { b'0' + digit } else { b'a' + digit - 10 };
            *template.add(len - 6 + i) = ch as c_char;
            n /= 36;
        }
        c = c.wrapping_add(1);
        if mkdir_fn.is_some() {
            if crate::mkdir(template, 0o700) == 0 {
                MKTEMP_COUNTER = c;
                return template;
            }
            let e = crate::__errno_location().read();
            if e != EEXIST && e != EINTR {
                break;
            }
        } else {
            MKTEMP_COUNTER = c;
            return template;
        }
    }
    MKTEMP_COUNTER = c;
    template
}

#[no_mangle]
pub unsafe extern "C" fn mkdtemp(template: *mut c_char) -> *mut c_char {
    mktemp_internal(template, Some(crate::mkdir))
}

#[no_mangle]
pub unsafe extern "C" fn mktemp(template: *mut c_char) -> *mut c_char {
    mktemp_internal(template, None)
}
