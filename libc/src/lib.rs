#![cfg_attr(not(test), no_std)]
#![feature(c_variadic)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_long, c_longlong, c_uint, c_ulong, c_ulonglong, c_void, VaListImpl};
use core::ptr::null_mut;

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// ponytail: stub for linker; never called with panic=abort
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

// ============================================================
// Constants
// ============================================================

const PROT_READ: i32 = 1;
const PROT_WRITE: i32 = 2;
const MAP_PRIVATE: i32 = 0x02;
const MAP_ANONYMOUS: i32 = 0x20;

// ============================================================
// Syscall wrappers (raw, no_std)
// ============================================================

#[inline]
unsafe fn sys_write(fd: i64, buf: *const u8, count: usize) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 1i64 => result,
        in("rdi") fd,
        in("rsi") buf,
        in("rdx") count,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_read(fd: i64, buf: *mut u8, count: usize) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 0i64 => result,
        in("rdi") fd,
        in("rsi") buf,
        in("rdx") count,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_open(path: *const u8, flags: i64) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 2i64 => result,
        in("rdi") path,
        in("rsi") flags,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_close(fd: i64) {
    core::arch::asm!(
        "syscall",
        in("rax") 3i64,
        in("rdi") fd,
        lateout("rcx") _,
        lateout("r11") _,
    );
}

#[inline]
unsafe fn sys_lseek(fd: i64, offset: i64, whence: i64) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 8i64 => result,
        in("rdi") fd,
        in("rsi") offset,
        in("rdx") whence,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_mmap(
    addr: *mut u8,
    length: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: i64,
) -> *mut u8 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 9i64 => result,
        in("rdi") addr,
        in("rsi") length,
        in("rdx") prot,
        in("r10") flags,
        in("r8") fd,
        in("r9") offset,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result as *mut u8
}

unsafe fn sys_munmap(addr: *mut u8, length: usize) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 11i64 => result,
        in("rdi") addr,
        in("rsi") length,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

// ============================================================
// String/memory functions
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: c_int, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.add(i) = c as u8;
        i += 1;
    }
    s
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if (dst as usize) < (src as usize) {
        let mut i = 0;
        while i < n {
            *dst.add(i) = *src.add(i);
            i += 1;
        }
    } else {
        let mut i = n;
        while i > 0 {
            i -= 1;
            *dst.add(i) = *src.add(i);
        }
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    let mut i = 0;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return a as c_int - b as c_int;
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    memcmp(s1, s2, n)
}

#[no_mangle]
pub unsafe extern "C" fn strcmp(s1: *const u8, s2: *const u8) -> c_int {
    let mut i = 0;
    loop {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return a as c_int - b as c_int;
        }
        if a == 0 {
            return 0;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strncmp(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    let mut i = 0;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return a as c_int - b as c_int;
        }
        if a == 0 {
            return 0;
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn strcpy(dst: *mut u8, src: *const u8) -> *mut u8 {
    let mut i = 0;
    loop {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn strncpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    while i < n {
        *dst.add(i) = 0;
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn strcat(dst: *mut u8, src: *const u8) -> *mut u8 {
    let mut i = 0;
    while *dst.add(i) != 0 {
        i += 1;
    }
    let mut j = 0;
    loop {
        let c = *src.add(j);
        *dst.add(i + j) = c;
        if c == 0 {
            break;
        }
        j += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn strncat(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while *dst.add(i) != 0 {
        i += 1;
    }
    let mut j = 0;
    while j < n && *src.add(j) != 0 {
        *dst.add(i + j) = *src.add(j);
        j += 1;
    }
    *dst.add(i + j) = 0;
    dst
}

#[no_mangle]
pub unsafe extern "C" fn strchr(s: *const u8, c: c_int) -> *mut u8 {
    let target = c as u8;
    let mut i = 0;
    loop {
        let ch = *s.add(i);
        if ch == target {
            return s.add(i) as *mut u8;
        }
        if ch == 0 {
            return null_mut();
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strrchr(s: *const u8, c: c_int) -> *mut u8 {
    let target = c as u8;
    let mut i = 0;
    let mut last: *mut u8 = null_mut();
    loop {
        let ch = *s.add(i);
        if ch == target {
            last = s.add(i) as *mut u8;
        }
        if ch == 0 {
            return last;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strnlen(s: *const u8, maxlen: usize) -> usize {
    let mut i = 0;
    while i < maxlen && *s.add(i) != 0 {
        i += 1;
    }
    i
}

#[no_mangle]
pub unsafe extern "C" fn strcspn(s: *const u8, reject: *const u8) -> usize {
    let mut i = 0;
    loop {
        let c = *s.add(i);
        if c == 0 {
            return i;
        }
        let mut j = 0;
        loop {
            let r = *reject.add(j);
            if r == 0 {
                break;
            }
            if c == r {
                return i;
            }
            j += 1;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strspn(s: *const u8, accept: *const u8) -> usize {
    let mut i = 0;
    loop {
        let c = *s.add(i);
        if c == 0 {
            return i;
        }
        let mut found = false;
        let mut j = 0;
        loop {
            let a = *accept.add(j);
            if a == 0 {
                break;
            }
            if c == a {
                found = true;
                break;
            }
            j += 1;
        }
        if !found {
            return i;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strpbrk(s: *const u8, accept: *const u8) -> *mut u8 {
    let mut i = 0;
    loop {
        let c = *s.add(i);
        if c == 0 {
            return null_mut();
        }
        let mut j = 0;
        loop {
            let a = *accept.add(j);
            if a == 0 {
                break;
            }
            if c == a {
                return s.add(i) as *mut u8;
            }
            j += 1;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strstr(haystack: *const u8, needle: *const u8) -> *mut u8 {
    if *needle == 0 {
        return haystack as *mut u8;
    }
    let mut i = 0;
    loop {
        let h = *haystack.add(i);
        if h == 0 {
            return null_mut();
        }
        if h == *needle {
            let mut j = 0;
            loop {
                let n = *needle.add(j);
                if n == 0 {
                    return haystack.add(i) as *mut u8;
                }
                if *haystack.add(i + j) != n {
                    break;
                }
                j += 1;
            }
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn memchr(s: *const u8, c: c_int, n: usize) -> *mut u8 {
    let target = c as u8;
    let mut i = 0;
    while i < n {
        if *s.add(i) == target {
            return s.add(i) as *mut u8;
        }
        i += 1;
    }
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn memrchr(s: *const u8, c: c_int, n: usize) -> *mut u8 {
    let target = c as u8;
    let mut i = n;
    while i > 0 {
        i -= 1;
        if *s.add(i) == target {
            return s.add(i) as *mut u8;
        }
    }
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn strtok(s: *mut u8, delim: *const u8) -> *mut u8 {
    static mut STATE: *mut u8 = core::ptr::null_mut();
    let mut p = if s.is_null() { STATE } else { s };
    if p.is_null() {
        return core::ptr::null_mut();
    }
    loop {
        let c = *p;
        if c == 0 {
            STATE = core::ptr::null_mut();
            return core::ptr::null_mut();
        }
        let mut is_delim = false;
        let mut i = 0;
        loop {
            let d = *delim.add(i);
            if d == 0 {
                break;
            }
            if c == d {
                is_delim = true;
                break;
            }
            i += 1;
        }
        if !is_delim {
            break;
        }
        p = p.add(1);
    }
    let start = p;
    loop {
        let c = *p;
        if c == 0 {
            STATE = core::ptr::null_mut();
            return start;
        }
        let mut is_delim = false;
        let mut i = 0;
        loop {
            let d = *delim.add(i);
            if d == 0 {
                break;
            }
            if c == d {
                is_delim = true;
                break;
            }
            i += 1;
        }
        if is_delim {
            *p = 0;
            STATE = p.add(1);
            return start;
        }
        p = p.add(1);
    }
}

// ============================================================
// C type aliases
// ============================================================

type SizeT = usize;
type SSizeT = isize;

// ============================================================
// ctype.h
// ============================================================

const CT_UPPER: u8 = 1 << 0;
const CT_LOWER: u8 = 1 << 1;
const CT_DIGIT: u8 = 1 << 2;
const CT_SPACE: u8 = 1 << 3;
const CT_PUNCT: u8 = 1 << 4;
const CT_CNTRL: u8 = 1 << 5;
const CT_BLANK: u8 = 1 << 6;
const CT_XDIGIT: u8 = 1 << 7;

const CT_TABLE: [u8; 128] = {
    let mut t = [0u8; 128];
    let mut i = 0;
    while i < 128 {
        let c = i as u8;
        let mut flags = 0u8;
        if c >= b'A' && c <= b'Z' {
            flags |= CT_UPPER;
        }
        if c >= b'a' && c <= b'z' {
            flags |= CT_LOWER;
        }
        if c >= b'0' && c <= b'9' {
            flags |= CT_DIGIT;
        }
        if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == b'\x0c' || c == b'\x0b' {
            flags |= CT_SPACE;
        }
        if (c >= b'!' && c <= b'/')
            || (c >= b':' && c <= b'@')
            || (c >= b'[' && c <= b'`')
            || (c >= b'{' && c <= b'~')
        {
            flags |= CT_PUNCT;
        }
        if c < 0x20 || c == 0x7f {
            flags |= CT_CNTRL;
        }
        if c == b' ' || c == b'\t' {
            flags |= CT_BLANK;
        }
        if (c >= b'0' && c <= b'9') || (c >= b'A' && c <= b'F') || (c >= b'a' && c <= b'f') {
            flags |= CT_XDIGIT;
        }
        t[i] = flags;
        i += 1;
    }
    t
};

#[inline]
fn ct_flags(c: c_int) -> u8 {
    if c < 0 || c > 127 {
        0
    } else {
        CT_TABLE[c as usize]
    }
}

#[no_mangle]
pub extern "C" fn isalnum(c: c_int) -> c_int {
    (ct_flags(c) & (CT_UPPER | CT_LOWER | CT_DIGIT) != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isalpha(c: c_int) -> c_int {
    (ct_flags(c) & (CT_UPPER | CT_LOWER) != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isascii(c: c_int) -> c_int {
    ((c as u32) <= 127) as c_int
}

#[no_mangle]
pub extern "C" fn isblank(c: c_int) -> c_int {
    (ct_flags(c) & CT_BLANK != 0) as c_int
}

#[no_mangle]
pub extern "C" fn iscntrl(c: c_int) -> c_int {
    (ct_flags(c) & CT_CNTRL != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isdigit(c: c_int) -> c_int {
    (ct_flags(c) & CT_DIGIT != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isgraph(c: c_int) -> c_int {
    (ct_flags(c) & (CT_UPPER | CT_LOWER | CT_DIGIT | CT_PUNCT) != 0) as c_int
}

#[no_mangle]
pub extern "C" fn islower(c: c_int) -> c_int {
    (ct_flags(c) & CT_LOWER != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isprint(c: c_int) -> c_int {
    let flags = ct_flags(c);
    (flags & (CT_UPPER | CT_LOWER | CT_DIGIT | CT_PUNCT | CT_BLANK) != 0) as c_int
}

#[no_mangle]
pub extern "C" fn ispunct(c: c_int) -> c_int {
    (ct_flags(c) & CT_PUNCT != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isspace(c: c_int) -> c_int {
    (ct_flags(c) & CT_SPACE != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isupper(c: c_int) -> c_int {
    (ct_flags(c) & CT_UPPER != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isxdigit(c: c_int) -> c_int {
    (ct_flags(c) & CT_XDIGIT != 0) as c_int
}

#[no_mangle]
pub extern "C" fn tolower(c: c_int) -> c_int {
    if (ct_flags(c) & CT_UPPER) != 0 {
        c + (b'a' as c_int - b'A' as c_int)
    } else {
        c
    }
}

#[no_mangle]
pub extern "C" fn toupper(c: c_int) -> c_int {
    if (ct_flags(c) & CT_LOWER) != 0 {
        c - (b'a' as c_int - b'A' as c_int)
    } else {
        c
    }
}

// ============================================================
// stdlib.h: integer conversion, abs, rand
// ============================================================

unsafe fn parse_digit(c: u8, base: c_int) -> Option<u8> {
    let d = if c >= b'0' && c <= b'9' {
        c - b'0'
    } else if c >= b'a' && c <= b'z' {
        c - b'a' + 10
    } else if c >= b'A' && c <= b'Z' {
        c - b'A' + 10
    } else {
        return None;
    };
    if (d as c_int) < base {
        Some(d)
    } else {
        None
    }
}

unsafe fn parse_prefix(s: *const u8, base: *mut c_int) -> *const u8 {
    let mut p = s;
    if *base == 0 {
        if *p == b'0' {
            if *p.add(1) == b'x' || *p.add(1) == b'X' {
                *base = 16;
                p = p.add(2);
            } else {
                *base = 8;
            }
        } else {
            *base = 10;
        }
    } else if *base == 16 {
        if *p == b'0' && (*p.add(1) == b'x' || *p.add(1) == b'X') {
            p = p.add(2);
        }
    }
    p
}

#[no_mangle]
pub unsafe extern "C" fn strtol(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_long {
    let mut p = s as *const u8;
    while isspace(*p as c_int) != 0 {
        p = p.add(1);
    }
    let mut neg = false;
    match *p {
        b'-' => {
            neg = true;
            p = p.add(1);
        }
        b'+' => p = p.add(1),
        _ => {}
    }
    let mut base = base;
    p = parse_prefix(p, &mut base);
    let mut val: c_ulong = 0;
    let mut consumed = false;
    while let Some(d) = parse_digit(*p, base) {
        val = val.wrapping_mul(base as c_ulong).wrapping_add(d as c_ulong);
        p = p.add(1);
        consumed = true;
    }
    if !endptr.is_null() {
        *endptr = if consumed { p as *mut c_char } else { s as *mut c_char };
    }
    if neg {
        val.wrapping_neg() as c_long
    } else {
        val as c_long
    }
}

#[no_mangle]
pub unsafe extern "C" fn strtoul(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_ulong {
    let mut p = s as *const u8;
    while isspace(*p as c_int) != 0 {
        p = p.add(1);
    }
    let mut neg = false;
    match *p {
        b'-' => {
            neg = true;
            p = p.add(1);
        }
        b'+' => p = p.add(1),
        _ => {}
    }
    let mut base = base;
    p = parse_prefix(p, &mut base);
    let mut val: c_ulong = 0;
    let mut consumed = false;
    while let Some(d) = parse_digit(*p, base) {
        val = val.wrapping_mul(base as c_ulong).wrapping_add(d as c_ulong);
        p = p.add(1);
        consumed = true;
    }
    if !endptr.is_null() {
        *endptr = if consumed { p as *mut c_char } else { s as *mut c_char };
    }
    if neg {
        val.wrapping_neg()
    } else {
        val
    }
}

#[no_mangle]
pub unsafe extern "C" fn strtoll(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_longlong {
    // ponytail: on x86_64 long == long long; reuse strtol
    strtol(s, endptr, base) as c_longlong
}

#[no_mangle]
pub unsafe extern "C" fn strtoull(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_ulonglong {
    // ponytail: on x86_64 unsigned long == unsigned long long; reuse strtoul
    strtoul(s, endptr, base) as c_ulonglong
}

#[no_mangle]
pub unsafe extern "C" fn atoi(s: *const c_char) -> c_int {
    strtol(s, core::ptr::null_mut(), 10) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn atol(s: *const c_char) -> c_long {
    strtol(s, core::ptr::null_mut(), 10)
}

#[no_mangle]
pub unsafe extern "C" fn atoll(s: *const c_char) -> c_longlong {
    strtoll(s, core::ptr::null_mut(), 10)
}

#[no_mangle]
pub extern "C" fn abs(n: c_int) -> c_int {
    if n < 0 {
        -n
    } else {
        n
    }
}

#[no_mangle]
pub extern "C" fn labs(n: c_long) -> c_long {
    if n < 0 {
        -n
    } else {
        n
    }
}

#[no_mangle]
pub extern "C" fn llabs(n: c_longlong) -> c_longlong {
    if n < 0 {
        -n
    } else {
        n
    }
}

static mut RAND_STATE: c_uint = 1;

#[no_mangle]
pub extern "C" fn srand(seed: c_uint) {
    unsafe {
        RAND_STATE = seed;
    }
}

#[no_mangle]
pub extern "C" fn rand() -> c_int {
    unsafe {
        RAND_STATE = RAND_STATE.wrapping_mul(1103515245).wrapping_add(12345);
        ((RAND_STATE >> 16) & 0x7fff) as c_int
    }
}

// ============================================================
// signal.h
// ============================================================

pub const SA_RESTORER: c_ulong = 0x04000000;

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;
pub const SIG_ERR: usize = !0usize;

pub const SIGINT: c_int = 2;
pub const SIGILL: c_int = 4;
pub const SIGABRT: c_int = 6;
pub const SIGFPE: c_int = 8;
pub const SIGSEGV: c_int = 11;
pub const SIGTERM: c_int = 15;

#[repr(C)]
pub struct sigaction {
    pub sa_handler: usize,
    pub sa_flags: c_ulong,
    pub sa_restorer: usize,
    pub sa_mask: [c_ulong; 1],
}

pub type SigSetT = c_ulong;

core::arch::global_asm!(
    ".global sig_restorer",
    ".type sig_restorer, @function",
    "sig_restorer:",
    "mov eax, 15",
    "syscall",
);

extern "C" {
    fn sig_restorer();
}

#[inline]
unsafe fn sys_rt_sigaction(
    sig: c_int,
    act: *const sigaction,
    oldact: *mut sigaction,
    sigsetsize: usize,
) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 13i64 => result,
        in("rdi") sig as i64,
        in("rsi") act,
        in("rdx") oldact,
        in("r10") sigsetsize,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_kill(pid: c_int, sig: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 62i64 => result,
        in("rdi") pid as i64,
        in("rsi") sig as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_getpid() -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 39i64 => result,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[no_mangle]
pub unsafe extern "C" fn sigaction(
    signum: c_int,
    act: *const sigaction,
    oldact: *mut sigaction,
) -> c_int {
    let r = sys_rt_sigaction(signum, act, oldact, core::mem::size_of::<[c_ulong; 1]>());
    if r < 0 {
        -1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn signal(signum: c_int, handler: usize) -> usize {
    let act = sigaction {
        sa_handler: handler,
        sa_flags: SA_RESTORER,
        sa_restorer: sig_restorer as usize,
        sa_mask: [0],
    };
    let mut old = sigaction {
        sa_handler: 0,
        sa_flags: 0,
        sa_restorer: 0,
        sa_mask: [0],
    };
    if sigaction(signum, &act, &mut old) == -1 {
        SIG_ERR
    } else {
        old.sa_handler
    }
}

#[no_mangle]
pub unsafe extern "C" fn kill(pid: c_int, sig: c_int) -> c_int {
    sys_kill(pid, sig) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn getpid() -> c_int {
    sys_getpid() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn raise(sig: c_int) -> c_int {
    if sys_kill(sys_getpid() as c_int, sig) < 0 {
        -1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn sigemptyset(set: *mut SigSetT) -> c_int {
    *set = 0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigfillset(set: *mut SigSetT) -> c_int {
    *set = !0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigaddset(set: *mut SigSetT, signum: c_int) -> c_int {
    *set |= 1u64 << (signum - 1);
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigdelset(set: *mut SigSetT, signum: c_int) -> c_int {
    *set &= !(1u64 << (signum - 1));
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigismember(set: *const SigSetT, signum: c_int) -> c_int {
    ((*set & (1u64 << (signum - 1))) != 0) as c_int
}

// ============================================================
// setjmp.h
// ponytail: jmp_buf is unsigned long[8]: rbx, rbp, r12-r15, rsp, rip
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn setjmp(env: *mut c_ulong) -> c_int {
    core::arch::asm!(
        "mov rax, [rsp]",
        "lea rcx, [rsp + 8]",
        "mov [r8], rbx",
        "mov [r8 + 8], rbp",
        "mov [r8 + 16], r12",
        "mov [r8 + 24], r13",
        "mov [r8 + 32], r14",
        "mov [r8 + 40], r15",
        "mov [r8 + 48], rcx",
        "mov [r8 + 56], rax",
        in("r8") env,
        lateout("rax") _,
        lateout("rcx") _,
        options(nostack),
    );
    0
}

#[no_mangle]
pub unsafe extern "C" fn longjmp(env: *const c_ulong, val: c_int) -> ! {
    let ret = if val == 0 { 1 } else { val } as u32;
    core::arch::asm!(
        "mov rbx, [r8]",
        "mov rbp, [r8 + 8]",
        "mov r12, [r8 + 16]",
        "mov r13, [r8 + 24]",
        "mov r14, [r8 + 32]",
        "mov r15, [r8 + 40]",
        "mov rsp, [r8 + 48]",
        "mov eax, edx",
        "jmp [r8 + 56]",
        in("r8") env,
        in("edx") ret,
        options(noreturn),
    );
}

// ============================================================
// Syscall wrappers as public C ABI
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: SizeT) -> SSizeT {
    sys_write(fd as i64, buf as *const u8, count) as SSizeT
}

#[no_mangle]
pub unsafe extern "C" fn read(fd: c_int, buf: *mut c_void, count: SizeT) -> SSizeT {
    sys_read(fd as i64, buf as *mut u8, count) as SSizeT
}

#[no_mangle]
pub unsafe extern "C" fn open(path: *const c_char, flags: c_int, _: c_int) -> c_int {
    sys_open(path as *const u8, flags as i64) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn close(fd: c_int) -> c_int {
    sys_close(fd as i64);
    0
}

#[no_mangle]
pub unsafe extern "C" fn lseek(fd: c_int, offset: i64, whence: c_int) -> i64 {
    sys_lseek(fd as i64, offset, whence as i64)
}

// ============================================================
// FILE / stdio
// ============================================================

#[repr(C)]
pub struct FILE {
    fd: c_int,
    has_ungotten: c_int,
    ungotten: c_int,
    // ponytail: minimal FILE, one-byte pushback for ungetc
}

static mut STDIN_FILE: FILE = FILE {
    fd: 0,
    has_ungotten: 0,
    ungotten: 0,
};
static mut STDOUT_FILE: FILE = FILE {
    fd: 1,
    has_ungotten: 0,
    ungotten: 0,
};
static mut STDERR_FILE: FILE = FILE {
    fd: 2,
    has_ungotten: 0,
    ungotten: 0,
};

#[no_mangle]
pub static mut stdin: *mut FILE = &raw mut STDIN_FILE as *mut FILE;
#[no_mangle]
pub static mut stdout: *mut FILE = &raw mut STDOUT_FILE as *mut FILE;
#[no_mangle]
pub static mut stderr: *mut FILE = &raw mut STDERR_FILE as *mut FILE;

// ============================================================
// puts / fputs / fputc
// ============================================================

unsafe fn write_str(fd: c_int, s: *const u8, len: usize) {
    let mut written = 0usize;
    while written < len {
        let n = sys_write(fd as i64, s.add(written), len - written);
        if n <= 0 {
            break;
        }
        written += n as usize;
    }
}

#[no_mangle]
pub unsafe extern "C" fn puts(s: *const c_char) -> c_int {
    let fd = (*stdout).fd;
    let len = strlen(s as *const u8);
    write_str(fd, s as *const u8, len);
    write_str(fd, b"\n".as_ptr(), 1);
    (len + 1) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn fputs(s: *const c_char, stream: *mut FILE) -> c_int {
    let fd = (*stream).fd;
    let len = strlen(s as *const u8);
    write_str(fd, s as *const u8, len);
    len as c_int
}

#[no_mangle]
pub unsafe extern "C" fn fputc(c: c_int, stream: *mut FILE) -> c_int {
    let byte = c as u8;
    let fd = (*stream).fd;
    sys_write(fd as i64, &byte as *const u8, 1);
    c
}

#[no_mangle]
pub unsafe extern "C" fn putchar(c: c_int) -> c_int {
    fputc(c, stdout)
}

// ============================================================
// getc / fgetc / getchar / fgets / fread / ungetc
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn fgetc(stream: *mut FILE) -> c_int {
    let f = &mut *stream;
    if f.has_ungotten != 0 {
        f.has_ungotten = 0;
        return f.ungotten;
    }
    let mut buf = [0u8; 1];
    let n = sys_read(f.fd as i64, buf.as_mut_ptr(), 1);
    if n <= 0 {
        -1
    } else {
        buf[0] as c_int
    }
}

#[no_mangle]
pub unsafe extern "C" fn getc(stream: *mut FILE) -> c_int {
    fgetc(stream)
}

#[no_mangle]
pub unsafe extern "C" fn getchar() -> c_int {
    fgetc(stdin)
}

#[no_mangle]
pub unsafe extern "C" fn ungetc(c: c_int, stream: *mut FILE) -> c_int {
    if c == -1 {
        return -1;
    }
    let f = &mut *stream;
    if f.has_ungotten != 0 {
        return -1;
    }
    f.ungotten = c;
    f.has_ungotten = 1;
    c
}

#[no_mangle]
pub unsafe extern "C" fn fgets(s: *mut c_char, n: c_int, stream: *mut FILE) -> *mut c_char {
    if n <= 0 {
        return null_mut();
    }
    let max = (n - 1) as usize;
    let mut i = 0usize;
    while i < max {
        let c = fgetc(stream);
        if c == -1 {
            if i == 0 {
                return null_mut();
            }
            break;
        }
        *s.add(i) = c as c_char;
        i += 1;
        if c == b'\n' as c_int {
            break;
        }
    }
    *s.add(i) = 0;
    s as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn fread(
    ptr: *mut c_void,
    size: SizeT,
    nmemb: SizeT,
    stream: *mut FILE,
) -> SizeT {
    if size == 0 || nmemb == 0 {
        return 0;
    }
    let total = size * nmemb;
    let mut read = 0usize;
    while read < total {
        let c = fgetc(stream);
        if c == -1 {
            break;
        }
        *(ptr as *mut u8).add(read) = c as u8;
        read += 1;
    }
    read / size
}

// ============================================================
// printf / fprintf / vprintf
// ============================================================

unsafe fn write_u64_hex(fd: c_int, mut val: u64, uppercase: bool) -> usize {
    let digits = if uppercase {
        b"0123456789ABCDEF"
    } else {
        b"0123456789abcdef"
    };
    let mut buf = [0u8; 20];
    let mut pos = buf.len();
    if val == 0 {
        pos -= 1;
        buf[pos] = b'0';
    } else {
        while val > 0 {
            pos -= 1;
            buf[pos] = digits[(val & 0xf) as usize];
            val >>= 4;
        }
    }
    let len = buf.len() - pos;
    write_str(fd, buf.as_ptr().add(pos), len);
    len
}

unsafe fn write_u64_dec(fd: c_int, mut val: u64) -> usize {
    let mut buf = [0u8; 20];
    let mut pos = buf.len();
    if val == 0 {
        pos -= 1;
        buf[pos] = b'0';
    } else {
        while val > 0 {
            pos -= 1;
            buf[pos] = b'0' + (val % 10) as u8;
            val /= 10;
        }
    }
    let len = buf.len() - pos;
    write_str(fd, buf.as_ptr().add(pos), len);
    len
}

unsafe fn write_i64_dec(fd: c_int, val: i64) -> usize {
    if val < 0 {
        write_str(fd, b"-".as_ptr(), 1);
        1 + write_u64_dec(fd, (-(val as i128)) as u64)
    } else {
        write_u64_dec(fd, val as u64)
    }
}

unsafe fn format_and_write(
    fd: c_int,
    fmt: *const c_char,
    args: &mut VaListImpl,
) -> c_int {
    let mut count: usize = 0;
    let mut i = 0usize;
    loop {
        let c = *fmt.add(i) as u8;
        if c == 0 {
            break;
        }
        if c != b'%' {
            write_str(fd, &c as *const u8, 1);
            count += 1;
            i += 1;
            continue;
        }
        i += 1;
        let spec = *fmt.add(i) as u8;
        match spec {
            b's' => {
                let s = args.arg::<*const c_char>();
                if s.is_null() {
                    write_str(fd, b"(null)".as_ptr(), 6);
                    count += 6;
                } else {
                    let len = strlen(s as *const u8);
                    write_str(fd, s as *const u8, len);
                    count += len;
                }
            }
            b'd' | b'i' => {
                let d = args.arg::<c_int>();
                count += write_i64_dec(fd, d as i64);
            }
            b'u' => {
                let u = args.arg::<c_uint>();
                count += write_u64_dec(fd, u as u64);
            }
            b'x' => {
                let x = args.arg::<c_uint>();
                count += write_u64_hex(fd, x as u64, false);
            }
            b'X' => {
                let x = args.arg::<c_uint>();
                count += write_u64_hex(fd, x as u64, true);
            }
            b'c' => {
                let c = args.arg::<c_int>();
                let byte = c as u8;
                write_str(fd, &byte as *const u8, 1);
                count += 1;
            }
            b'p' => {
                let p = args.arg::<*const c_void>();
                write_str(fd, b"0x".as_ptr(), 2);
                count += 2;
                count += write_u64_hex(fd, p as u64, false);
            }
            b'l' => {
                i += 1;
                let sub = *fmt.add(i) as u8;
                match sub {
                    b'd' => {
                        let ld = args.arg::<c_long>();
                        count += write_i64_dec(fd, ld as i64);
                    }
                    b'u' => {
                        let lu = args.arg::<c_ulong>();
                        count += write_u64_dec(fd, lu as u64);
                    }
                    b'x' => {
                        let lx = args.arg::<c_ulong>();
                        count += write_u64_hex(fd, lx as u64, false);
                    }
                    b'X' => {
                        let lx = args.arg::<c_ulong>();
                        count += write_u64_hex(fd, lx as u64, true);
                    }
                    _ => {
                        let pct = b'%';
                        write_str(fd, &pct as *const u8, 1);
                        write_str(fd, &sub as *const u8, 1);
                        count += 2;
                    }
                }
            }
            b'%' => {
                let pct = b'%';
                write_str(fd, &pct as *const u8, 1);
                count += 1;
            }
            b'n' => {
                // ponytail: %n writes count to pointer, no-op for safety
                let _ptr = args.arg::<*mut c_int>();
            }
            _ => {
                let pct = b'%';
                write_str(fd, &pct as *const u8, 1);
                write_str(fd, &spec as *const u8, 1);
                count += 2;
            }
        }
        i += 1;
    }
    count as c_int
}

#[no_mangle]
pub unsafe extern "C" fn vprintf(fmt: *const c_char, mut args: VaListImpl) -> c_int {
    format_and_write((*stdout).fd, fmt, &mut args)
}

#[no_mangle]
pub unsafe extern "C" fn vfprintf(stream: *mut FILE, fmt: *const c_char, mut args: VaListImpl) -> c_int {
    format_and_write((*stream).fd, fmt, &mut args)
}

#[no_mangle]
pub unsafe extern "C" fn printf(fmt: *const c_char, mut args: ...) -> c_int {
    format_and_write((*stdout).fd, fmt, &mut args)
}

#[no_mangle]
pub unsafe extern "C" fn fprintf(stream: *mut FILE, fmt: *const c_char, mut args: ...) -> c_int {
    format_and_write((*stream).fd, fmt, &mut args)
}

#[no_mangle]
pub unsafe extern "C" fn dprintf(fd: c_int, fmt: *const c_char, mut args: ...) -> c_int {
    format_and_write(fd, fmt, &mut args)
}

// ============================================================
// Allocator: mmap-backed bump allocator
// ponytail: per-page mmap, free is a no-op, leaks on realloc
// ============================================================

const MMAP_FAILED: *mut u8 = !0usize as *mut u8;
const PAGE: usize = 4096;

#[no_mangle]
pub unsafe extern "C" fn malloc(size: SizeT) -> *mut c_void {
    if size == 0 {
        return null_mut();
    }
    // Layout: [alloc_size: usize][data]
    let total = size + core::mem::size_of::<usize>();
    let pages = (total + PAGE - 1) & !(PAGE - 1);
    let ptr = sys_mmap(
        null_mut(),
        pages,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );
    if ptr == MMAP_FAILED {
        return null_mut();
    }
    *(ptr as *mut usize) = pages;
    ptr.add(core::mem::size_of::<usize>()) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let header = (ptr as *mut u8).sub(core::mem::size_of::<usize>());
    let pages = *(header as *const usize);
    sys_munmap(header, pages);
}

#[no_mangle]
pub unsafe extern "C" fn calloc(count: SizeT, size: SizeT) -> *mut c_void {
    let total = count.saturating_mul(size);
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
    let header = (ptr as *mut u8).sub(core::mem::size_of::<usize>());
    let old_pages = *(header as *const usize);
    let old_data_size = old_pages - core::mem::size_of::<usize>();
    let new_ptr = malloc(new_size);
    if new_ptr.is_null() {
        return null_mut();
    }
    let copy_size = if old_data_size < new_size {
        old_data_size
    } else {
        new_size
    };
    core::ptr::copy_nonoverlapping(ptr as *const u8, new_ptr as *mut u8, copy_size);
    free(ptr);
    new_ptr
}

// ============================================================
// Process: exit / _exit / _Exit
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn _exit(code: c_int) -> ! {
    core::arch::asm!(
        "syscall",
        in("rax") 60i64,
        in("rdi") code as i64,
        options(noreturn)
    );
}

#[no_mangle]
pub unsafe extern "C" fn _Exit(code: c_int) -> ! {
    _exit(code);
}

#[no_mangle]
pub unsafe extern "C" fn exit(code: c_int) -> ! {
    // ponytail: skip __cxa_atexit handlers and fini arrays, just flush and exit
    _exit(code);
}

// ============================================================
// Startup: __libc_start_main
//
// musl crt1.o _start_c calls:
//   __libc_start_main(main, argc, argv, _init, _fini, rtld_fini, stack_end)
// ============================================================

type MainFn = unsafe extern "C" fn(c_int, *const *const c_char, *const *const c_char) -> c_int;
type InitFn = unsafe extern "C" fn();

#[no_mangle]
pub unsafe extern "C" fn __libc_start_main(
    main: MainFn,
    argc: c_int,
    argv: *const *const c_char,
    _init: *const c_void,
    _fini: *const c_void,
    _rtld_fini: *const c_void,
    _stack_end: *const c_void,
) -> ! {
    let envp = argv.add((argc + 1) as usize);

    if !_init.is_null() {
        let init_fn: InitFn = core::mem::transmute(_init);
        init_fn();
    }

    let result = main(argc, argv, envp);

    if !_fini.is_null() {
        let fini_fn: InitFn = core::mem::transmute(_fini);
        fini_fn();
    }

    exit(result);
}
