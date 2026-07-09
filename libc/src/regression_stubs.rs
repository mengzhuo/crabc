// Stubs and minimal implementations for symbols required by libc-test
// regression cases that currently fail to link.

const SYS_MMAP: c_long = 9;
const SYS_MUNMAP: c_long = 11;
const SYS_PREAD64: c_long = 17;
const SYS_EXECVE: c_long = 59;

const _SC_PAGE_SIZE: c_int = 30;

#[no_mangle]
pub unsafe extern "C" fn mmap(
    addr: *mut c_void,
    len: usize,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    off: i64,
) -> *mut c_void {
    crate::syscall(
        SYS_MMAP,
        addr as c_long,
        len as c_long,
        prot as c_long,
        flags as c_long,
        fd as c_long,
        off as c_long,
    ) as *mut c_void
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

#[no_mangle]
pub unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    match name {
        _SC_PAGE_SIZE => 4096,
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
    crate::syscall(
        SYS_EXECVE,
        path as c_long,
        argv.as_ptr() as c_long,
        envp as c_long,
        0,
        0,
        0,
    ) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn flockfile(f: *mut crate::FILE) {
    if !f.is_null() {
        (*f).lockcount += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn funlockfile(f: *mut crate::FILE) {
    if !f.is_null() {
        if (*f).lockcount > 1 {
            (*f).lockcount -= 1;
        } else {
            (*f).lockcount = 0;
        }
    }
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

#[no_mangle]
pub unsafe extern "C" fn getpwnam_r(
    _name: *const c_char,
    _pwd: *mut c_void,
    _buf: *mut c_char,
    _buflen: usize,
    result: *mut *mut c_void,
) -> c_int {
    if !result.is_null() {
        *result = core::ptr::null_mut();
    }
    0
}
