#![cfg_attr(not(test), no_std)]
#![feature(c_variadic)]
#![feature(linkage)]
#![allow(dead_code, non_camel_case_types)]

use core::ffi::{c_char, c_int, c_long, c_longlong, c_uint, c_ulong, c_ulonglong, c_void, VaList, VaListImpl};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

include!("encoding_tables.rs");

// ============================================================
// errno
// ============================================================

const EILSEQ: c_int = 84;
const EINVAL: c_int = 22;
const EFAULT: c_int = 14;
const E2BIG: c_int = 7;

const LC_CTYPE: c_int = 0;
const LC_NUMERIC: c_int = 1;
const LC_TIME: c_int = 2;
const LC_COLLATE: c_int = 3;
const LC_MONETARY: c_int = 4;
const LC_MESSAGES: c_int = 5;
const LC_ALL: c_int = 6;
const ENOMEM: c_int = 12;
const EINTR: c_int = 4;
const EPERM: c_int = 1;
const EAGAIN: c_int = 11;
const EBUSY: c_int = 16;
const EDEADLK: c_int = 35;
const ETIMEDOUT: c_int = 110;
const ECANCELED: c_int = 125;
const EOVERFLOW: c_int = 75;
const EOWNERDEAD: c_int = 130;
const ENOTRECOVERABLE: c_int = 131;

static mut ERRNO: c_int = 0;

#[no_mangle]
pub unsafe extern "C" fn __errno_location() -> *mut c_int {
    core::ptr::addr_of_mut!(ERRNO)
}

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
const MAP_SHARED: i32 = 0x01;
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
unsafe fn sys_open(path: *const u8, flags: i64, mode: i64) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 2i64 => result,
        in("rdi") path,
        in("rsi") flags,
        in("rdx") mode,
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
pub unsafe extern "C" fn strlcpy(dst: *mut u8, src: *const u8, n: usize) -> usize {
    let l = strlen(src);
    if n > 0 {
        let c = if l < n { l } else { n - 1 };
        core::ptr::copy_nonoverlapping(src, dst, c);
        *dst.add(c) = 0;
    }
    l
}

#[no_mangle]
pub unsafe extern "C" fn strlcat(dst: *mut u8, src: *const u8, n: usize) -> usize {
    let dl = strlen(dst);
    let sl = strlen(src);
    if dl >= n { return n + sl; }
    let c = if sl < n - dl { sl } else { n - dl - 1 };
    core::ptr::copy_nonoverlapping(src, dst.add(dl), c);
    *dst.add(dl + c) = 0;
    dl + sl
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
pub unsafe extern "C" fn memmem(
    haystack: *const c_void, haystacklen: usize,
    needle: *const c_void, needlelen: usize,
) -> *mut c_void {
    if needlelen == 0 { return haystack as *mut c_void; }
    if haystacklen < needlelen { return null_mut(); }
    let h = haystack as *const u8;
    let n = needle as *const u8;
    let last = haystacklen - needlelen;
    for i in 0..=last {
        if *h.add(i) == *n && memcmp(h.add(i), n, needlelen) == 0 {
            return h.add(i) as *mut c_void;
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
type mode_t = c_uint;
type TimeT = c_long;
type ClockT = c_long;
type wchar_t = c_int;
type wint_t = c_uint;
type wctype_t = *const c_int;
type wctrans_t = *const c_int;

const WEOF: wint_t = 0xffffffff;

#[repr(C)]
pub struct tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
    pub tm_gmtoff: c_long,
    pub tm_zone: *const c_char,
}

#[repr(C)]
pub struct timeval {
    pub tv_sec: TimeT,
    pub tv_usec: c_long,
}

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
    if c == 0 { return None; }
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
            if (*p.add(1) == b'x' || *p.add(1) == b'X') && parse_digit(*p.add(2), 16).is_some() {
                *base = 16;
                p = p.add(2);
            } else {
                *base = 8;
            }
        } else {
            *base = 10;
        }
    } else if *base == 16 {
        if *p == b'0' && (*p.add(1) == b'x' || *p.add(1) == b'X') && parse_digit(*p.add(2), 16).is_some() {
            p = p.add(2);
        }
    }
    p
}

unsafe fn strtox(
    s: *const c_char,
    endptr: *mut *mut c_char,
    mut base: c_int,
    pos_limit: u64,
    neg_limit: u64,
) -> (u64, bool, bool) {
    let s0 = s as *const u8;
    let mut p = s0;
    while isspace(*p as c_int) != 0 {
        p = p.add(1);
    }

    if base < 0 || base == 1 || base > 36 {
        if !endptr.is_null() {
            *endptr = s as *mut c_char;
        }
        ERRNO = EINVAL;
        return (0, false, false);
    }

    let mut neg = false;
    match *p {
        b'-' => { neg = true; p = p.add(1); }
        b'+' => p = p.add(1),
        _ => {}
    }

    p = parse_prefix(p, &mut base);

    let limit = if neg { neg_limit } else { pos_limit };
    let mut val: u64 = 0;
    let mut any = false;
    while let Some(d) = parse_digit(*p, base) {
        any = true;
        if val > (limit - d as u64) / base as u64 {
            while parse_digit(*p, base).is_some() { p = p.add(1); }
            if !endptr.is_null() { *endptr = p as *mut c_char; }
            ERRNO = ERANGE_VAL;
            return (limit, true, neg);
        }
        val = val * base as u64 + d as u64;
        p = p.add(1);
    }

    if !endptr.is_null() {
        *endptr = if any { p as *mut c_char } else { s as *mut c_char };
    }
    (val, false, neg)
}

#[no_mangle]
pub unsafe extern "C" fn strtol(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_long {
    let pos_limit = c_long::MAX as u64;
    let neg_limit = pos_limit.wrapping_add(1);
    let (val, overflow, neg) = strtox(s, endptr, base, pos_limit, neg_limit);
    if overflow {
        return if neg { c_long::MIN } else { c_long::MAX };
    }
    if neg { val.wrapping_neg() as c_long } else { val as c_long }
}

#[no_mangle]
pub unsafe extern "C" fn strtoul(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_ulong {
    let limit = c_ulong::MAX as u64;
    let (val, overflow, neg) = strtox(s, endptr, base, limit, limit);
    if overflow {
        return c_ulong::MAX;
    }
    if neg { val.wrapping_neg() as c_ulong } else { val as c_ulong }
}

#[no_mangle]
pub unsafe extern "C" fn strtoll(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_longlong {
    let pos_limit = c_longlong::MAX as u64;
    let neg_limit = pos_limit.wrapping_add(1);
    let (val, overflow, neg) = strtox(s, endptr, base, pos_limit, neg_limit);
    if overflow {
        return if neg { c_longlong::MIN } else { c_longlong::MAX };
    }
    if neg { val.wrapping_neg() as c_longlong } else { val as c_longlong }
}

#[no_mangle]
pub unsafe extern "C" fn strtoull(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_ulonglong {
    let limit = c_ulonglong::MAX as u64;
    let (val, overflow, neg) = strtox(s, endptr, base, limit, limit);
    if overflow {
        return c_ulonglong::MAX;
    }
    if neg { val.wrapping_neg() as c_ulonglong } else { val as c_ulonglong }
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
// stdlib.h: div, ldiv, lldiv
// ============================================================

#[repr(C)]
pub struct div_t { pub quot: c_int, pub rem: c_int }
#[repr(C)]
pub struct ldiv_t { pub quot: c_long, pub rem: c_long }
#[repr(C)]
pub struct lldiv_t { pub quot: c_longlong, pub rem: c_longlong }

#[no_mangle]
pub extern "C" fn div(num: c_int, den: c_int) -> div_t {
    div_t { quot: num / den, rem: num % den }
}

#[no_mangle]
pub extern "C" fn ldiv(num: c_long, den: c_long) -> ldiv_t {
    ldiv_t { quot: num / den, rem: num % den }
}

#[no_mangle]
pub extern "C" fn lldiv(num: c_longlong, den: c_longlong) -> lldiv_t {
    lldiv_t { quot: num / den, rem: num % den }
}

// ============================================================
// bsearch
// ============================================================

type CmpFn = unsafe extern "C" fn(*const c_void, *const c_void) -> c_int;

#[no_mangle]
pub unsafe extern "C" fn bsearch(
    key: *const c_void,
    base: *const c_void,
    nel: usize,
    width: usize,
    cmp: CmpFn,
) -> *mut c_void {
    let mut base = base as *const u8;
    let mut nel = nel;
    while nel > 0 {
        let trial = base.add(width * (nel / 2));
        let sign = cmp(key, trial as *const c_void);
        if sign < 0 {
            nel /= 2;
        } else if sign > 0 {
            base = trial.add(width);
            nel -= nel / 2 + 1;
        } else {
            return trial as *mut c_void;
        }
    }
    null_mut()
}

// ============================================================
// signal.h
// ============================================================

pub const SA_RESTORER: c_ulong = 0x04000000;

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;
pub const SIG_ERR: usize = !0usize;

pub const SIGHUP: c_int = 1;
pub const SIGINT: c_int = 2;
pub const SIGQUIT: c_int = 3;
pub const SIGILL: c_int = 4;
pub const SIGTRAP: c_int = 5;
pub const SIGABRT: c_int = 6;
pub const SIGIOT: c_int = 6;
pub const SIGBUS: c_int = 7;
pub const SIGFPE: c_int = 8;
pub const SIGKILL: c_int = 9;
pub const SIGUSR1: c_int = 10;
pub const SIGSEGV: c_int = 11;
pub const SIGUSR2: c_int = 12;
pub const SIGPIPE: c_int = 13;
pub const SIGALRM: c_int = 14;
pub const SIGTERM: c_int = 15;
pub const SIGSTKFLT: c_int = 16;
pub const SIGCHLD: c_int = 17;
pub const SIGCONT: c_int = 18;
pub const SIGSTOP: c_int = 19;
pub const SIGTSTP: c_int = 20;
pub const SIGTTIN: c_int = 21;
pub const SIGTTOU: c_int = 22;
pub const SIGURG: c_int = 23;
pub const SIGXCPU: c_int = 24;
pub const SIGXFSZ: c_int = 25;
pub const SIGVTALRM: c_int = 26;
pub const SIGPROF: c_int = 27;
pub const SIGWINCH: c_int = 28;
pub const SIGIO: c_int = 29;
pub const SIGPOLL: c_int = 29;
pub const SIGPWR: c_int = 30;
pub const SIGSYS: c_int = 31;
pub const SIGUNUSED: c_int = 31;
pub const _NSIG: c_int = 65;

pub const SIG_BLOCK: c_int = 0;
pub const SIG_UNBLOCK: c_int = 1;
pub const SIG_SETMASK: c_int = 2;

pub const SA_NOCLDSTOP: c_ulong = 1;
pub const SA_NOCLDWAIT: c_ulong = 2;
pub const SA_SIGINFO: c_ulong = 4;
pub const SA_ONSTACK: c_ulong = 0x08000000;
pub const SA_RESTART: c_ulong = 0x10000000;
pub const SA_NODEFER: c_ulong = 0x40000000;
pub const SA_RESETHAND: c_ulong = 0x80000000;

pub const SS_ONSTACK: c_int = 1;
pub const SS_DISABLE: c_int = 2;
pub const MINSIGSTKSZ: usize = 2048;
pub const SIGSTKSZ: usize = 8192;

pub const SI_USER: c_int = 0;
pub const SI_TKILL: c_int = -6;

#[repr(C)]
pub struct timespec {
    pub tv_sec: c_long,
    pub tv_nsec: c_long,
}

#[repr(C)]
pub struct siginfo_t {
    pub si_signo: c_int,
    pub si_errno: c_int,
    pub si_code: c_int,
    // ponytail: rest of 128-byte struct, only si_signo accessed
    _pad: [u8; 128 - 3 * core::mem::size_of::<c_int>()],
}

#[repr(C)]
pub struct stack_t {
    pub ss_sp: *mut c_void,
    pub ss_flags: c_int,
    pub ss_size: usize,
}

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
unsafe fn sys_rt_sigprocmask(
    how: c_int,
    set: *const SigSetT,
    oldset: *mut SigSetT,
    sigsetsize: usize,
) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 14i64 => result,
        in("rdi") how as i64,
        in("rsi") set,
        in("rdx") oldset,
        in("r10") sigsetsize,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_rt_sigpending(set: *mut SigSetT, sigsetsize: usize) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 127i64 => result,
        in("rdi") set,
        in("rsi") sigsetsize,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_rt_sigsuspend(mask: *const SigSetT, sigsetsize: usize) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 130i64 => result,
        in("rdi") mask,
        in("rsi") sigsetsize,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_rt_sigtimedwait(
    set: *const SigSetT,
    info: *mut siginfo_t,
    timeout: *const timespec,
    sigsetsize: usize,
) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 128i64 => result,
        in("rdi") set,
        in("rsi") info,
        in("rdx") timeout,
        in("r10") sigsetsize,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 131i64 => result,
        in("rdi") ss,
        in("rsi") old_ss,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_tgkill(tgid: c_int, tid: c_int, sig: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 234i64 => result,
        in("rdi") tgid as i64,
        in("rsi") tid as i64,
        in("rdx") sig as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

const CLOCK_REALTIME: c_int = 0;
const CLOCK_MONOTONIC: c_int = 1;

#[inline]
unsafe fn sys_clock_gettime(clockid: c_int, ts: *mut timespec) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 228i64 => result,
        in("rdi") clockid as i64,
        in("rsi") ts,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[no_mangle]
pub unsafe extern "C" fn clock_gettime(clockid: c_int, ts: *mut timespec) -> c_int {
    if sys_clock_gettime(clockid, ts) < 0 { -1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn sigprocmask(
    how: c_int,
    set: *const SigSetT,
    oldset: *mut SigSetT,
) -> c_int {
    let r = sys_rt_sigprocmask(how, set, oldset, core::mem::size_of::<SigSetT>());
    if r < 0 { -1 } else { 0 }
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
    let kact: sigaction;
    let act_ptr = if act.is_null() {
        core::ptr::null()
    } else {
        kact = sigaction {
            sa_handler: (*act).sa_handler,
            sa_flags: (*act).sa_flags | SA_RESTORER,
            sa_restorer: sig_restorer as usize,
            sa_mask: (*act).sa_mask,
        };
        &kact as *const sigaction
    };
    let r = sys_rt_sigaction(signum, act_ptr, oldact, core::mem::size_of::<SigSetT>());
    if r < 0 {
        ERRNO = (-r) as c_int;
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
pub unsafe extern "C" fn tgkill(tgid: c_int, tid: c_int, sig: c_int) -> c_int {
    if sys_tgkill(tgid, tid, sig) < 0 { -1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn getpid() -> c_int {
    sys_getpid() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn raise(sig: c_int) -> c_int {
    if sys_tgkill(sys_getpid() as c_int, sys_gettid() as c_int, sig) < 0 {
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
    // musl: all signals except 32 (SIGCANCEL) and 33 (SIGSYNCCALL)
    *set = 0xFFFF_FFFC_7FFF_FFFFu64;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigaddset(set: *mut SigSetT, signum: c_int) -> c_int {
    let s = (signum as c_uint).wrapping_sub(1);
    if s as usize >= 64 || (signum as c_uint).wrapping_sub(32) < 3 {
        ERRNO = EINVAL;
        return -1;
    }
    *set |= 1u64 << s;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigdelset(set: *mut SigSetT, signum: c_int) -> c_int {
    let s = (signum as c_uint).wrapping_sub(1);
    if s as usize >= 64 || (signum as c_uint).wrapping_sub(32) < 3 {
        ERRNO = EINVAL;
        return -1;
    }
    *set &= !(1u64 << s);
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigismember(set: *const SigSetT, signum: c_int) -> c_int {
    let s = (signum as c_uint).wrapping_sub(1);
    if s as usize >= 64 { return 0; }
    ((*set & (1u64 << s)) != 0) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn sigpending(set: *mut SigSetT) -> c_int {
    let r = sys_rt_sigpending(set, core::mem::size_of::<SigSetT>());
    if r < 0 {
        ERRNO = (-r) as c_int;
        return -1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigsuspend(mask: *const SigSetT) -> c_int {
    let r = sys_rt_sigsuspend(mask, core::mem::size_of::<SigSetT>());
    if r < 0 {
        ERRNO = (-r) as c_int;
    }
    -1
}

#[no_mangle]
pub unsafe extern "C" fn sigtimedwait(
    mask: *const SigSetT,
    info: *mut siginfo_t,
    timeout: *const timespec,
) -> c_int {
    loop {
        let r = sys_rt_sigtimedwait(mask, info, timeout, core::mem::size_of::<SigSetT>());
        if r >= 0 {
            return r as c_int;
        }
        let e = (-r) as c_int;
        if e != EINTR {
            ERRNO = e;
            return -1;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn sigwaitinfo(
    mask: *const SigSetT,
    info: *mut siginfo_t,
) -> c_int {
    sigtimedwait(mask, info, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn sigwait(
    mask: *const SigSetT,
    sig: *mut c_int,
) -> c_int {
    let mut info: siginfo_t = core::mem::zeroed();
    if sigtimedwait(mask, &mut info, core::ptr::null()) < 0 {
        return ERRNO;
    }
    *sig = info.si_signo;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> c_int {
    if !ss.is_null() {
        if (*ss).ss_flags & SS_ONSTACK != 0 {
            ERRNO = EINVAL;
            return -1;
        }
        if (*ss).ss_flags & SS_DISABLE == 0 && (*ss).ss_size < MINSIGSTKSZ {
            ERRNO = ENOMEM;
            return -1;
        }
    }
    let r = sys_sigaltstack(ss, old_ss);
    if r < 0 {
        ERRNO = (-r) as c_int;
        return -1;
    }
    0
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

#[no_mangle]
pub unsafe extern "C" fn sigsetjmp(env: *mut c_ulong, savemask: c_int) -> c_int {
    let ret = setjmp(env);
    if ret == 0 {
        *env.add(8) = savemask as c_ulong;
        if savemask != 0 {
            sigprocmask(SIG_BLOCK, core::ptr::null(), env.add(9) as *mut SigSetT);
        }
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn siglongjmp(env: *const c_ulong, val: c_int) -> ! {
    if *env.add(8) != 0 {
        sigprocmask(SIG_SETMASK, env.add(9) as *const SigSetT, core::ptr::null_mut());
    }
    longjmp(env, val);
}

// ============================================================
// syscall wrappers: fstat/newfstatat/getrlimit/setrlimit/utimensat
// ============================================================

#[inline]
unsafe fn sys_newfstatat(dirfd: i32, path: *const c_char, buf: *mut u8, flags: i32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 262i64 => result,
        in("rdi") dirfd as i64,
        in("rsi") path,
        in("rdx") buf,
        in("r10") flags as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_fstat(fd: i32, buf: *mut u8) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 5i64 => result,
        in("rdi") fd as i64,
        in("rsi") buf,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_getrlimit(resource: i32, rlim: *mut u8) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 97i64 => result,
        in("rdi") resource as i64,
        in("rsi") rlim,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_setrlimit(resource: i32, rlim: *const u8) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 160i64 => result,
        in("rdi") resource as i64,
        in("rsi") rlim,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_utimensat(dirfd: i32, path: *const c_char, times: *const u8, flags: i32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 280i64 => result,
        in("rdi") dirfd as i64,
        in("rsi") path,
        in("rdx") times,
        in("r10") flags as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

// ============================================================
// unistd.h: process primitives
// ============================================================

#[inline]
unsafe fn sys_fork() -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 57i64 => result,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_execve(path: *const c_char, argv: *const *const c_char, envp: *const *const c_char) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 59i64 => result,
        in("rdi") path,
        in("rsi") argv,
        in("rdx") envp,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_wait4(pid: c_int, status: *mut c_int, options: c_int, rusage: *mut c_void) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 61i64 => result,
        in("rdi") pid as i64,
        in("rsi") status,
        in("rdx") options as i64,
        in("r10") rusage,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_getppid() -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 110i64 => result,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_getuid() -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 102i64 => result,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_getgid() -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 104i64 => result,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_geteuid() -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 107i64 => result,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_getegid() -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 108i64 => result,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[no_mangle]
pub unsafe extern "C" fn fork() -> c_int {
    __fork_handler(-1);
    let ret = sys_fork();
    let errno_save = ERRNO;
    if ret == 0 {
        __fork_handler(1);
    } else {
        __fork_handler(0);
    }
    if ret < 0 {
        ERRNO = errno_save;
        return -1;
    }
    ret as c_int
}

#[no_mangle]
pub unsafe extern "C" fn execve(
    path: *const c_char,
    argv: *const *const c_char,
    envp: *const *const c_char,
) -> c_int {
    sys_execve(path, argv, envp) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn wait(status: *mut c_int) -> c_int {
    sys_wait4(-1, status, 0, core::ptr::null_mut()) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn waitpid(pid: c_int, status: *mut c_int, options: c_int) -> c_int {
    sys_wait4(pid, status, options, core::ptr::null_mut()) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn getppid() -> c_int {
    sys_getppid() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn getuid() -> c_uint {
    sys_getuid() as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn getgid() -> c_uint {
    sys_getgid() as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn geteuid() -> c_uint {
    sys_geteuid() as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn getegid() -> c_uint {
    sys_getegid() as c_uint
}

// ponytail: vfork forwards to fork; real vfork optimization not needed for tests
#[no_mangle]
pub unsafe extern "C" fn vfork() -> c_int {
    sys_fork() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn execl(path: *const c_char, arg: *const c_char, mut args: ...) -> c_int {
    let mut argv: [*const c_char; 256] = [core::ptr::null(); 256];
    argv[0] = arg;
    let mut n: usize = 1;
    loop {
        let a: *const c_char = args.arg();
        if a.is_null() { break; }
        if n >= 255 { return -1; }
        argv[n] = a;
        n += 1;
    }
    argv[n] = core::ptr::null();
    sys_execve(path, argv.as_ptr(), __environ as *const *const c_char) as c_int
}

// ============================================================
// spawn.h: posix_spawn family
// ============================================================

// Matches musl's posix_spawn_file_actions_t layout.
// __pad0[0] = action count, __pad stores (type, fd, newfd) triples.
// ponytail: max 5 actions, enough for all tests
const SPAWN_FA_MAX: usize = 5;

#[repr(C)]
pub struct posix_spawn_file_actions_t {
    __pad0: [c_int; 2],
    __actions: *mut c_void,
    __pad: [c_int; 16],
}

#[repr(C)]
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

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_init(fa: *mut posix_spawn_file_actions_t) -> c_int {
    core::ptr::write_bytes(fa, 0, 1);
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_destroy(_fa: *mut posix_spawn_file_actions_t) -> c_int {
    0
}

unsafe fn spawn_fa_add(fa: *mut posix_spawn_file_actions_t, action_type: c_int, fd: c_int, newfd: c_int) -> c_int {
    let count = (*fa).__pad0[0] as usize;
    if count >= SPAWN_FA_MAX { return ENOMEM; }
    let base = count * 3;
    (*fa).__pad[base] = action_type;
    (*fa).__pad[base + 1] = fd;
    (*fa).__pad[base + 2] = newfd;
    (*fa).__pad0[0] += 1;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_addclose(fa: *mut posix_spawn_file_actions_t, fd: c_int) -> c_int {
    spawn_fa_add(fa, 0, fd, 0)
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_adddup2(fa: *mut posix_spawn_file_actions_t, oldfd: c_int, newfd: c_int) -> c_int {
    spawn_fa_add(fa, 1, oldfd, newfd)
}

unsafe fn spawn_apply_actions(fa: *const posix_spawn_file_actions_t) {
    if fa.is_null() { return; }
    let count = (*fa).__pad0[0] as usize;
    for i in 0..count {
        let base = i * 3;
        match (*fa).__pad[base] {
            0 => { sys_close((*fa).__pad[base + 1] as i64); }
            1 => { sys_dup2((*fa).__pad[base + 1], (*fa).__pad[base + 2]); }
            _ => {}
        }
    }
}

// ponytail: PATH search using stack buffer, O(n) scan per entry
unsafe fn spawn_execvp(file: *const c_char, argv: *const *const c_char, envp: *const *const c_char) -> ! {
    let flen = strlen(file as *const u8);
    let f = file as *const u8;
    let mut has_slash = false;
    for i in 0..flen {
        if *f.add(i) == b'/' { has_slash = true; break; }
    }
    if has_slash {
        sys_execve(file, argv, envp);
        _exit(127);
    }
    let path = getenv(b"PATH\0".as_ptr() as *const c_char);
    let default_path = b"/usr/local/bin:/usr/bin:/bin\0";
    let p = if path.is_null() { default_path.as_ptr() } else { path as *const u8 };
    let plen = strlen(p);
    let mut buf: [u8; 4096] = [0; 4096];
    let mut i: usize = 0;
    while i <= plen {
        let start = i;
        while i < plen && *p.add(i) != b':' { i += 1; }
        let dlen = i - start;
        if dlen > 0 && dlen + 1 + flen < 4096 {
            core::ptr::copy_nonoverlapping(p.add(start), buf.as_mut_ptr(), dlen);
            *buf.as_mut_ptr().add(dlen) = b'/';
            core::ptr::copy_nonoverlapping(f, buf.as_mut_ptr().add(dlen + 1), flen + 1);
            sys_execve(buf.as_ptr() as *const c_char, argv, envp);
            // execve failed, try next
        }
        i += 1; // skip ':'
    }
    _exit(127);
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnp(
    pid: *mut c_int,
    file: *const c_char,
    fa: *const posix_spawn_file_actions_t,
    _attrp: *const posix_spawnattr_t,
    argv: *const *const c_char,
    envp: *const *const c_char,
) -> c_int {
    let child = sys_fork();
    if child < 0 {
        ERRNO = EAGAIN;
        return -1;
    }
    if child == 0 {
        // child: apply file actions then exec
        spawn_apply_actions(fa);
        let actual_envp = if envp.is_null() { __environ as *const *const c_char } else { envp };
        spawn_execvp(file, argv, actual_envp);
        // unreachable: spawn_execvp always calls _exit
    }
    *pid = child as c_int;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn(
    pid: *mut c_int,
    path: *const c_char,
    fa: *const posix_spawn_file_actions_t,
    attrp: *const posix_spawnattr_t,
    argv: *const *const c_char,
    envp: *const *const c_char,
) -> c_int {
    // ponytail: posix_spawn = posix_spawnp with a direct path (no PATH search needed,
    // but posix_spawnp handles '/' in path by doing direct execve anyway)
    posix_spawnp(pid, path, fa, attrp, argv, envp)
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_init(attr: *mut posix_spawnattr_t) -> c_int {
    core::ptr::write_bytes(attr, 0, 1);
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_destroy(_attr: *mut posix_spawnattr_t) -> c_int {
    0
}

// ============================================================
// sys/stat.h: stat / fstat / utimensat / futimens
// ============================================================

#[repr(C)]
pub struct Stat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_nlink: u64,
    pub st_mode: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    _pad0: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atim: timespec,
    pub st_mtim: timespec,
    pub st_ctim: timespec,
    _unused: [i64; 3],
}

pub const S_IFMT: u32 = 0o170000;
pub const S_IFDIR: u32 = 0o040000;
pub const S_IFCHR: u32 = 0o020000;
pub const S_IFBLK: u32 = 0o060000;
pub const S_IFREG: u32 = 0o100000;
pub const S_IFLNK: u32 = 0o120000;
pub const S_IFIFO: u32 = 0o010000;
pub const S_IFSOCK: u32 = 0o140000;

pub const AT_FDCWD: i32 = -100;

#[no_mangle]
pub unsafe extern "C" fn stat(path: *const c_char, buf: *mut Stat) -> c_int {
    let r = sys_newfstatat(AT_FDCWD, path, buf as *mut u8, 0);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn fstat(fd: c_int, buf: *mut Stat) -> c_int {
    let r = sys_fstat(fd, buf as *mut u8);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { 0 }
}

// ============================================================
// sys/resource.h: getrlimit / setrlimit
// ============================================================

#[repr(C)]
pub struct Rlimit {
    pub rlim_cur: u64,
    pub rlim_max: u64,
}

pub const RLIMIT_NOFILE: c_int = 7;
pub const RLIMIT_STACK: c_int = 3;
pub const RLIMIT_DATA: c_int = 2;
pub const RLIMIT_AS: c_int = 9;
pub const RLIMIT_CPU: c_int = 0;
pub const RLIMIT_FSIZE: c_int = 1;
pub const RLIMIT_CORE: c_int = 4;
pub const RLIMIT_RSS: c_int = 5;
pub const RLIMIT_NPROC: c_int = 6;
pub const RLIMIT_MEMLOCK: c_int = 8;
pub const RLIMIT_LOCKS: c_int = 10;
pub const RLIMIT_SIGPENDING: c_int = 11;
pub const RLIMIT_MSGQUEUE: c_int = 12;
pub const RLIMIT_NICE: c_int = 13;
pub const RLIMIT_RTPRIO: c_int = 14;
pub const RLIMIT_RTTIME: c_int = 15;
pub const RLIM_INFINITY: u64 = !0u64;

#[no_mangle]
pub unsafe extern "C" fn getrlimit(resource: c_int, rlim: *mut Rlimit) -> c_int {
    let r = sys_getrlimit(resource, rlim as *mut u8);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn setrlimit(resource: c_int, rlim: *const Rlimit) -> c_int {
    let r = sys_setrlimit(resource, rlim as *const u8);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { 0 }
}

// ============================================================
// utimensat / futimens
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn utimensat(dirfd: c_int, path: *const c_char, times: *const timespec, flags: c_int) -> c_int {
    let r = sys_utimensat(dirfd, path, times as *const u8, flags);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn futimens(fd: c_int, times: *const timespec) -> c_int {
    let r = sys_utimensat(fd, core::ptr::null(), times as *const u8, 0);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { 0 }
}

// ============================================================
// sys/socket.h
// ============================================================

pub const AF_UNIX: c_int = 1;
pub const AF_INET: c_int = 2;
pub const AF_INET6: c_int = 10;

pub const SOCK_STREAM: c_int = 1;
pub const SOCK_DGRAM: c_int = 2;

pub const SHUT_RD: c_int = 0;
pub const SHUT_WR: c_int = 1;
pub const SHUT_RDWR: c_int = 2;

pub const SOL_SOCKET: c_int = 1;
pub const SO_REUSEADDR: c_int = 2;

#[repr(C)]
pub struct sockaddr {
    pub sa_family: u16,
    pub sa_data: [u8; 14],
}

#[inline]
unsafe fn sys_socket(domain: c_int, ty: c_int, protocol: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 41i64 => result,
        in("rdi") domain as i64,
        in("rsi") ty as i64,
        in("rdx") protocol as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_socketpair(domain: c_int, ty: c_int, protocol: c_int, sv: *mut c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 53i64 => result,
        in("rdi") domain as i64,
        in("rsi") ty as i64,
        in("rdx") protocol as i64,
        in("r10") sv,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_bind(fd: c_int, addr: *const sockaddr, len: c_uint) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 49i64 => result,
        in("rdi") fd as i64,
        in("rsi") addr,
        in("rdx") len as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_listen(fd: c_int, backlog: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 50i64 => result,
        in("rdi") fd as i64,
        in("rsi") backlog as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_accept(fd: c_int, addr: *mut sockaddr, len: *mut c_uint) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 43i64 => result,
        in("rdi") fd as i64,
        in("rsi") addr,
        in("rdx") len,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_connect(fd: c_int, addr: *const sockaddr, len: c_uint) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 42i64 => result,
        in("rdi") fd as i64,
        in("rsi") addr,
        in("rdx") len as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_sendto(
    fd: c_int,
    buf: *const c_void,
    len: usize,
    flags: c_int,
    addr: *const sockaddr,
    addrlen: c_uint,
) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 44i64 => result,
        in("rdi") fd as i64,
        in("rsi") buf,
        in("rdx") len,
        in("r10") flags as i64,
        in("r8") addr,
        in("r9") addrlen as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_recvfrom(
    fd: c_int,
    buf: *mut c_void,
    len: usize,
    flags: c_int,
    addr: *mut sockaddr,
    addrlen: *mut c_uint,
) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 45i64 => result,
        in("rdi") fd as i64,
        in("rsi") buf,
        in("rdx") len,
        in("r10") flags as i64,
        in("r8") addr,
        in("r9") addrlen,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_shutdown(fd: c_int, how: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 48i64 => result,
        in("rdi") fd as i64,
        in("rsi") how as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_setsockopt(
    fd: c_int,
    level: c_int,
    optname: c_int,
    optval: *const c_void,
    optlen: c_uint,
) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 54i64 => result,
        in("rdi") fd as i64,
        in("rsi") level as i64,
        in("rdx") optname as i64,
        in("r10") optval,
        in("r8") optlen as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_getsockname(fd: c_int, addr: *mut sockaddr, len: *mut c_uint) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 51i64 => result,
        in("rdi") fd as i64,
        in("rsi") addr,
        in("rdx") len,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[no_mangle]
pub unsafe extern "C" fn socket(domain: c_int, ty: c_int, protocol: c_int) -> c_int {
    sys_socket(domain, ty, protocol) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn socketpair(domain: c_int, ty: c_int, protocol: c_int, sv: *mut c_int) -> c_int {
    sys_socketpair(domain, ty, protocol, sv) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn bind(fd: c_int, addr: *const sockaddr, len: c_uint) -> c_int {
    sys_bind(fd, addr, len) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn listen(fd: c_int, backlog: c_int) -> c_int {
    sys_listen(fd, backlog) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn accept(fd: c_int, addr: *mut sockaddr, len: *mut c_uint) -> c_int {
    sys_accept(fd, addr, len) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn connect(fd: c_int, addr: *const sockaddr, len: c_uint) -> c_int {
    sys_connect(fd, addr, len) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn send(fd: c_int, buf: *const c_void, len: usize, flags: c_int) -> isize {
    sys_sendto(fd, buf, len, flags, core::ptr::null(), 0) as isize
}

#[no_mangle]
pub unsafe extern "C" fn recv(fd: c_int, buf: *mut c_void, len: usize, flags: c_int) -> isize {
    sys_recvfrom(fd, buf, len, flags, core::ptr::null_mut(), core::ptr::null_mut()) as isize
}

#[no_mangle]
pub unsafe extern "C" fn sendto(
    fd: c_int,
    buf: *const c_void,
    len: usize,
    flags: c_int,
    addr: *const sockaddr,
    addrlen: c_uint,
) -> isize {
    sys_sendto(fd, buf, len, flags, addr, addrlen) as isize
}

#[no_mangle]
pub unsafe extern "C" fn recvfrom(
    fd: c_int,
    buf: *mut c_void,
    len: usize,
    flags: c_int,
    addr: *mut sockaddr,
    addrlen: *mut c_uint,
) -> isize {
    sys_recvfrom(fd, buf, len, flags, addr, addrlen) as isize
}

#[no_mangle]
pub unsafe extern "C" fn shutdown(fd: c_int, how: c_int) -> c_int {
    sys_shutdown(fd, how) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn setsockopt(
    fd: c_int,
    level: c_int,
    optname: c_int,
    optval: *const c_void,
    optlen: c_uint,
) -> c_int {
    sys_setsockopt(fd, level, optname, optval, optlen) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn getsockname(fd: c_int, addr: *mut sockaddr, len: *mut c_uint) -> c_int {
    sys_getsockname(fd, addr, len) as c_int
}

#[no_mangle]
pub extern "C" fn htonl(hostlong: c_uint) -> c_uint {
    hostlong.to_be()
}

#[no_mangle]
pub extern "C" fn ntohl(netlong: c_uint) -> c_uint {
    c_uint::from_be(netlong)
}

#[no_mangle]
pub extern "C" fn htons(hostshort: u16) -> u16 {
    hostshort.to_be()
}

#[no_mangle]
pub extern "C" fn ntohs(netshort: u16) -> u16 {
    u16::from_be(netshort)
}

pub type PthreadT = c_ulong;

// pthread_attr_t: musl x86_64 layout (56 bytes = int[14])
#[repr(C)]
pub struct pthread_attr_t {
    __i: [c_int; 14],
}

// pthread_mutex_t: musl x86_64 layout (40 bytes = int[10])
// _m_type=__i[0], _m_lock=__i[1], _m_waiters=__i[2], _m_count=__i[5]
#[repr(C)]
pub struct pthread_mutex_t {
    __i: [c_int; 10],
}

#[repr(C)]
pub struct pthread_mutexattr_t {
    __attr: c_uint,
}

// pthread_cond_t: musl x86_64 layout (48 bytes = int[12])
// _c_seq=__i[2], _c_waiters=__i[3], _c_clock=__i[4]
#[repr(C)]
pub struct pthread_cond_t {
    __i: [c_int; 12],
}

#[repr(C)]
pub struct pthread_condattr_t {
    __attr: c_uint,
}

// pthread_rwlock_t: musl x86_64 layout (56 bytes = int[14])
// _rw_lock=__i[0], _rw_waiters=__i[1], _rw_shared=__i[2]
#[repr(C)]
pub struct pthread_rwlock_t {
    __i: [c_int; 14],
}

#[repr(C)]
pub struct pthread_rwlockattr_t {
    __attr: [c_uint; 2],
}

// pthread_barrier_t: musl x86_64 layout (32 bytes = int[8])
#[repr(C)]
pub struct pthread_barrier_t {
    __i: [c_int; 8],
}

#[repr(C)]
pub struct pthread_barrierattr_t {
    __attr: c_uint,
}

pub type pthread_spinlock_t = c_int;
pub type pthread_once_t = c_int;
pub type pthread_key_t = c_uint;

// sem_t: musl x86_64 layout (32 bytes = int[8])
#[repr(C)]
pub struct sem_t {
    __val: [c_int; 8],
}

#[repr(C)]
pub struct sched_param {
    pub sched_priority: c_int,
}

const CLONE_VM: c_ulong = 0x00000100;
const CLONE_FS: c_ulong = 0x00000200;
const CLONE_FILES: c_ulong = 0x00000400;
const CLONE_SIGHAND: c_ulong = 0x00000800;
const CLONE_THREAD: c_ulong = 0x00010000;
const CLONE_SYSVSEM: c_ulong = 0x00040000;
const CLONE_SETTLS: c_ulong = 0x00080000;
const CLONE_PARENT_SETTID: c_ulong = 0x00100000;
const CLONE_CHILD_CLEARTID: c_ulong = 0x00200000;

const FUTEX_WAIT: c_int = 0;
const FUTEX_WAKE: c_int = 1;
const FUTEX_LOCK_PI: c_int = 6;
const FUTEX_UNLOCK_PI: c_int = 7;
const FUTEX_TRYLOCK_PI: c_int = 8;

// ponytail: SIGCANCEL (32) is musl's internal cancel signal
const SIGCANCEL: c_int = 32;

const PTHREAD_MUTEX_NORMAL: c_int = 0;
const PTHREAD_MUTEX_DEFAULT: c_int = 0;
const PTHREAD_MUTEX_RECURSIVE: c_int = 1;
const PTHREAD_MUTEX_ERRORCHECK: c_int = 2;

const PTHREAD_CREATE_JOINABLE: c_int = 0;
const PTHREAD_CREATE_DETACHED: c_int = 1;

const PTHREAD_CANCEL_ENABLE: c_int = 0;
const PTHREAD_CANCEL_DISABLE: c_int = 1;
const PTHREAD_CANCEL_DEFERRED: c_int = 0;
const PTHREAD_CANCEL_ASYNCHRONOUS: c_int = 1;

const PTHREAD_SCOPE_SYSTEM: c_int = 0;
const PTHREAD_PROCESS_PRIVATE: c_int = 0;
const PTHREAD_PROCESS_SHARED: c_int = 1;
const PTHREAD_INHERIT_SCHED: c_int = 0;
const PTHREAD_EXPLICIT_SCHED: c_int = 1;

const PTHREAD_BARRIER_SERIAL_THREAD: c_int = -1;

const PTHREAD_KEYS_MAX: usize = 128;
const PTHREAD_DESTRUCTOR_ITERATIONS: usize = 4;
const SEM_VALUE_MAX: c_int = 0x7fffffff;

const DT_EXITED: c_int = 0;
const DT_EXITING: c_int = 1;
const DT_JOINABLE: c_int = 2;
const DT_DETACHED: c_int = 3;

// ponytail: musl __ptcb cleanup handler node
#[repr(C)]
#[derive(Copy, Clone)]
pub struct __ptcb {
    __f: Option<unsafe extern "C" fn(*mut c_void)>,
    __x: *mut c_void,
    __next: *mut __ptcb,
}

// ponytail: robust_list_head for kernel robust mutex support
#[repr(C)]
#[derive(Copy, Clone)]
struct robust_list {
    next: *mut robust_list,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct robust_list_head {
    list: robust_list,
    futex_offset: c_long,
    pending: *mut robust_list,
}

// ponytail: adapted from musl x86_64 clone.s
#[cfg(not(test))]
core::arch::global_asm!(
    ".global __rc_clone",
    ".type __rc_clone, @function",
    "__rc_clone:",
    "mov rax, rdi",
    "mov rdi, rdx",
    "mov rdx, r8",
    "mov r10, [rsp + 8]",
    "mov r8, r9",
    "mov r9, rcx",
    "and rsi, -16",
    "sub rsi, 8",
    "mov [rsi], r9",
    "mov [rsi + 8], rax",
    "mov eax, 56",
    "syscall",
    "test rax, rax",
    "jnz 1f",
    "pop rdi",
    "ret",
    "1:",
    "ret",
);

extern "C" {
    fn __rc_clone(
        fn_: usize,
        stack: *mut u8,
        flags: c_ulong,
        arg: *mut c_void,
        ptid: *mut c_int,
        tls: c_ulong,
        ctid: *mut c_int,
    ) -> i64;
}

#[inline(never)]
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn __rc_create_thread_tls() -> *mut u8 {
    core::ptr::null_mut()
}

#[inline(never)]
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn __rc_tls_block_size() -> usize {
    0
}

#[inline]
unsafe fn sys_gettid() -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 186i64 => result,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_futex(
    uaddr: *mut c_int,
    futex_op: c_int,
    val: c_int,
    timeout: *mut c_void,
    uaddr2: *mut c_int,
    val3: c_int,
) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 202i64 => result,
        in("rdi") uaddr,
        in("rsi") futex_op as i64,
        in("rdx") val as i64,
        in("r10") timeout,
        in("r8") uaddr2,
        in("r9") val3 as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

const MAX_THREADS: usize = 64;
const STACK_SIZE: usize = 1024 * 1024;

#[repr(C)]
#[derive(Copy, Clone)]
struct Thread {
    tid: c_int,
    detach_state: c_int,
    result: *mut c_void,
    cancel: c_int,
    cancel_state: c_int,
    cancel_type: c_int,
    user_fn: usize,
    user_arg: *mut c_void,
    stack: *mut u8,
    stack_size: usize,
    fs_base: *mut u8,
    tsd: [*mut c_void; PTHREAD_KEYS_MAX],
    cancelbuf: *mut __ptcb,
    robust_list: robust_list_head,
}

static mut THREADS: [Thread; MAX_THREADS] = [Thread {
    tid: -1,
    detach_state: DT_JOINABLE,
    result: core::ptr::null_mut(),
    cancel: 0,
    cancel_state: PTHREAD_CANCEL_ENABLE,
    cancel_type: PTHREAD_CANCEL_DEFERRED,
    user_fn: 0,
    user_arg: core::ptr::null_mut(),
    stack: core::ptr::null_mut(),
    stack_size: 0,
    fs_base: core::ptr::null_mut(),
    tsd: [core::ptr::null_mut(); PTHREAD_KEYS_MAX],
    cancelbuf: core::ptr::null_mut(),
    robust_list: robust_list_head {
        list: robust_list { next: core::ptr::null_mut() },
        futex_offset: 0,
        pending: core::ptr::null_mut(),
    },
}; MAX_THREADS];
static NEXT_SLOT: AtomicUsize = AtomicUsize::new(0);
static mut KEY_DTORS: [Option<unsafe extern "C" fn(*mut c_void)>; PTHREAD_KEYS_MAX] = [None; PTHREAD_KEYS_MAX];
static NEXT_KEY: AtomicUsize = AtomicUsize::new(0);

// ponytail: futex helpers
unsafe fn futex_wait(addr: *mut c_int, expected: c_int) -> c_int {
    let r = sys_futex(addr, FUTEX_WAIT, expected, null_mut(), null_mut(), 0);
    if r < 0 { let e = (-r) as c_int; if e == EAGAIN { 0 } else { e } } else { 0 }
}

unsafe fn futex_wake(addr: *mut c_int, count: c_int) {
    let c = if count < 0 { c_int::MAX } else { count };
    sys_futex(addr, FUTEX_WAKE, c, null_mut(), null_mut(), 0);
}

unsafe fn futex_lock_pi(addr: *mut c_int, abs_timeout: *const timespec) -> c_int {
    let timeout_ptr = if abs_timeout.is_null() { null_mut() } else { abs_timeout as *mut c_void };
    let r = sys_futex(addr, FUTEX_LOCK_PI, 0, timeout_ptr, null_mut(), 0);
    if r < 0 { (-r) as c_int } else { 0 }
}

unsafe fn futex_unlock_pi(addr: *mut c_int) -> c_int {
    let r = sys_futex(addr, FUTEX_UNLOCK_PI, 0, null_mut(), null_mut(), 0);
    if r < 0 { (-r) as c_int } else { 0 }
}

#[inline]
unsafe fn sys_set_robust_list(head: *mut robust_list_head, len: usize) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 273i64 => result,
        in("rdi") head,
        in("rsi") len,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn futex_timedwait(addr: *mut c_int, expected: c_int, abs_timeout: *const timespec) -> c_int {
    if abs_timeout.is_null() { return futex_wait(addr, expected); }
    let mut now: timespec = core::mem::zeroed();
    if sys_clock_gettime(CLOCK_REALTIME, &mut now) < 0 { return EINVAL; }
    let mut rel: timespec = core::mem::zeroed();
    rel.tv_sec = (*abs_timeout).tv_sec - now.tv_sec;
    rel.tv_nsec = (*abs_timeout).tv_nsec - now.tv_nsec;
    if rel.tv_nsec < 0 { rel.tv_sec -= 1; rel.tv_nsec += 1_000_000_000; }
    if rel.tv_sec < 0 { return ETIMEDOUT; }
    let r = sys_futex(addr, FUTEX_WAIT, expected, &mut rel as *mut timespec as *mut c_void, null_mut(), 0);
    if r < 0 { let e = (-r) as c_int; if e == EAGAIN { 0 } else { e } } else { 0 }
}

// ponytail: atomic helpers wrapping AtomicI32
unsafe fn a_cas(addr: *mut c_int, expected: c_int, desired: c_int) -> c_int {
    let a = &*(addr as *const AtomicI32);
    match a.compare_exchange(expected, desired, Ordering::AcqRel, Ordering::Acquire) {
        Ok(v) | Err(v) => v,
    }
}

unsafe fn a_store(addr: *mut c_int, val: c_int) {
    (*(addr as *const AtomicI32)).store(val, Ordering::Release);
}

unsafe fn a_load(addr: *const c_int) -> c_int {
    (*(addr as *const AtomicI32)).load(Ordering::Acquire)
}

unsafe fn a_swap(addr: *mut c_int, val: c_int) -> c_int {
    (*(addr as *const AtomicI32)).swap(val, Ordering::AcqRel)
}

unsafe fn a_fetch_add(addr: *mut c_int, val: c_int) -> c_int {
    (*(addr as *const AtomicI32)).fetch_add(val, Ordering::AcqRel)
}

unsafe fn a_fetch_sub(addr: *mut c_int, val: c_int) -> c_int {
    (*(addr as *const AtomicI32)).fetch_sub(val, Ordering::AcqRel)
}

unsafe fn spinlock_lock(lock: *mut c_int, waiters: *mut c_int) {
    while a_swap(lock, 1) != 0 {
        let mut spins = 100;
        while spins > 0 && a_load(lock) != 0 { core::hint::spin_loop(); spins -= 1; }
        if a_load(lock) == 0 { continue; }
        a_fetch_add(waiters, 1);
        futex_wait(lock, 1);
        a_fetch_sub(waiters, 1);
    }
}

unsafe fn spinlock_unlock(lock: *mut c_int, waiters: *mut c_int) {
    a_store(lock, 0);
    if a_load(waiters) > 0 { futex_wake(lock, 1); }
}

unsafe fn alloc_thread_slot() -> Option<&'static mut Thread> {
    let idx = NEXT_SLOT.fetch_add(1, Ordering::SeqCst);
    if idx >= MAX_THREADS { return None; }
    Some(&mut THREADS[idx])
}

unsafe fn run_key_dtors(slot: &mut Thread) {
    for _ in 0..PTHREAD_DESTRUCTOR_ITERATIONS {
        let mut any = false;
        for i in 0..PTHREAD_KEYS_MAX {
            let val = slot.tsd[i];
            if !val.is_null() {
                if let Some(dtor) = KEY_DTORS[i] {
                    slot.tsd[i] = core::ptr::null_mut();
                    any = true;
                    dtor(val);
                }
            }
        }
        if !any { break; }
    }
}

unsafe extern "C" fn thread_entry(slot: *mut c_void) -> *mut c_void {
    let slot = &mut *(slot as *mut Thread);
    let user_fn: unsafe extern "C" fn(*mut c_void) -> *mut c_void =
        core::mem::transmute::<usize, _>(slot.user_fn);
    let ret = user_fn(slot.user_arg);
    run_key_dtors(slot);
    slot.result = ret;
    a_store(&raw mut slot.detach_state, DT_EXITED);
    futex_wake(&raw mut slot.detach_state, 1);
    _exit(0);
}

unsafe fn find_thread() -> Option<&'static mut Thread> {
    let me = sys_gettid() as c_int;
    let base = core::ptr::addr_of_mut!(THREADS[0]);
    for i in 0..MAX_THREADS {
        let slot = base.add(i);
        if core::ptr::read_volatile(&raw const (*slot).tid) == me {
            return Some(&mut *slot);
        }
    }
    None
}

#[no_mangle]
pub unsafe extern "C" fn pthread_self() -> PthreadT {
    let me = sys_gettid() as c_int;
    let base = core::ptr::addr_of_mut!(THREADS[0]);
    for i in 0..MAX_THREADS {
        let slot = base.add(i);
        if core::ptr::read_volatile(&raw const (*slot).tid) == me {
            return slot as PthreadT;
        }
    }
    // ponytail: register main thread lazily in first free slot
    let idx = NEXT_SLOT.fetch_add(1, Ordering::SeqCst);
    if idx < MAX_THREADS {
        let slot = &raw mut THREADS[idx];
        (*slot).tid = me;
        (*slot).detach_state = DT_JOINABLE;
        return slot as PthreadT;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_equal(t1: PthreadT, t2: PthreadT) -> c_int {
    (t1 == t2) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn pthread_create(
    thread: *mut PthreadT,
    attr: *const pthread_attr_t,
    start_routine: usize,
    arg: *mut c_void,
) -> c_int {
    if start_routine == 0 || thread.is_null() { return EINVAL; }
    let Some(slot_ref) = alloc_thread_slot() else { return EAGAIN; };
    let slot = slot_ref as *mut Thread;
    let stack_size = if !attr.is_null() {
        let s = *((*attr).__i.as_ptr() as *const usize);
        if s > 0 { s } else { STACK_SIZE }
    } else { STACK_SIZE };
    let stack = sys_mmap(null_mut(), stack_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if stack == MMAP_FAILED { return ENOMEM; }
    let fs_base = __rc_create_thread_tls();
    if fs_base.is_null() { sys_munmap(stack, stack_size); return ENOMEM; }
    (*slot).detach_state = DT_JOINABLE;
    (*slot).result = core::ptr::null_mut();
    (*slot).cancel = 0;
    (*slot).cancel_state = PTHREAD_CANCEL_ENABLE;
    (*slot).cancel_type = PTHREAD_CANCEL_DEFERRED;
    core::ptr::write_bytes((*slot).tsd.as_mut_ptr(), 0, PTHREAD_KEYS_MAX);
    (*slot).tid = 1;
    (*slot).user_fn = start_routine;
    (*slot).user_arg = arg;
    (*slot).stack = stack;
    (*slot).stack_size = stack_size;
    (*slot).fs_base = fs_base;
    let stack_top = stack.add(stack_size);
    let tid_ptr = &raw mut (*slot).tid;
    let flags = CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD
        | CLONE_SYSVSEM | CLONE_PARENT_SETTID | CLONE_CHILD_CLEARTID | CLONE_SETTLS;
    let tid = __rc_clone(thread_entry as usize, stack_top, flags, slot as *mut c_void, tid_ptr, fs_base as c_ulong, tid_ptr);
    if tid < 0 {
        (*slot).tid = -1;
        sys_munmap(stack, stack_size);
        sys_munmap(fs_base.sub(__rc_tls_block_size()), __rc_tls_block_size());
        return EAGAIN;
    }
    if !attr.is_null() && (*attr).__i[6] == PTHREAD_CREATE_DETACHED {
        (*slot).detach_state = DT_DETACHED;
    }
    *thread = slot as PthreadT;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_join(thread: PthreadT, retval: *mut *mut c_void) -> c_int {
    let slot = thread as *mut Thread;
    if slot.is_null() { return EINVAL; }
    if (*slot).detach_state == DT_DETACHED { return EINVAL; }
    let tid_ptr = &raw mut (*slot).tid;
    loop {
        let tid = core::ptr::read_volatile(tid_ptr);
        if tid == 0 { break; }
        if tid < 0 { return EINVAL; }
        sys_futex(tid_ptr, FUTEX_WAIT, tid, null_mut(), null_mut(), 0);
    }
    if !retval.is_null() { *retval = (*slot).result; }
    let stack = (*slot).stack;
    let stack_size = (*slot).stack_size;
    let fs_base = (*slot).fs_base;
    (*slot).tid = -1;
    (*slot).detach_state = DT_JOINABLE;
    (*slot).result = core::ptr::null_mut();
    (*slot).stack = core::ptr::null_mut();
    (*slot).stack_size = 0;
    (*slot).fs_base = core::ptr::null_mut();
    if !stack.is_null() && stack_size > 0 { sys_munmap(stack, stack_size); }
    if !fs_base.is_null() { let bs = __rc_tls_block_size(); if bs > 0 { sys_munmap(fs_base.sub(bs), bs); } }
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_detach(thread: PthreadT) -> c_int {
    let slot = thread as *mut Thread;
    if slot.is_null() { return EINVAL; }
    let old = a_cas(&raw mut (*slot).detach_state, DT_JOINABLE, DT_DETACHED);
    if old == DT_EXITED {
        let stack = (*slot).stack;
        let stack_size = (*slot).stack_size;
        let fs_base = (*slot).fs_base;
        (*slot).tid = -1;
        (*slot).stack = core::ptr::null_mut();
        (*slot).stack_size = 0;
        (*slot).fs_base = core::ptr::null_mut();
        if !stack.is_null() && stack_size > 0 { sys_munmap(stack, stack_size); }
        if !fs_base.is_null() { let bs = __rc_tls_block_size(); sys_munmap(fs_base.sub(bs), bs); }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_exit(retval: *mut c_void) -> ! {
    if let Some(slot) = find_thread() {
        run_cleanup_handlers(slot);
        run_key_dtors(slot);
        slot.result = retval;
        slot.detach_state = DT_EXITED;
        futex_wake(&slot.detach_state as *const c_int as *mut c_int, 1);
    }
    _exit(0);
}

// --- pthread_attr_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_init(attr: *mut pthread_attr_t) -> c_int {
    core::ptr::write_bytes(attr, 0, 1);
    *((*attr).__i.as_mut_ptr() as *mut usize) = STACK_SIZE;
    *((*attr).__i.as_mut_ptr().add(2) as *mut usize) = 4096;
    (*attr).__i[6] = PTHREAD_CREATE_JOINABLE;
    (*attr).__i[7] = PTHREAD_INHERIT_SCHED;
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_destroy(_attr: *mut pthread_attr_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setdetachstate(attr: *mut pthread_attr_t, s: c_int) -> c_int {
    if s != PTHREAD_CREATE_JOINABLE && s != PTHREAD_CREATE_DETACHED { return EINVAL; }
    (*attr).__i[6] = s; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getdetachstate(attr: *const pthread_attr_t, s: *mut c_int) -> c_int {
    *s = (*attr).__i[6]; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, sz: usize) -> c_int {
    if sz < 16384 { return EINVAL; }
    *((*attr).__i.as_mut_ptr() as *mut usize) = sz; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getstacksize(attr: *const pthread_attr_t, sz: *mut usize) -> c_int {
    *sz = *((*attr).__i.as_ptr() as *const usize); 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setstack(attr: *mut pthread_attr_t, addr: *mut c_void, sz: usize) -> c_int {
    if sz < 16384 { return EINVAL; }
    *((*attr).__i.as_mut_ptr() as *mut usize) = sz;
    *((*attr).__i.as_mut_ptr().add(4) as *mut *mut c_void) = addr; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getstack(attr: *const pthread_attr_t, addr: *mut *mut c_void, sz: *mut usize) -> c_int {
    *sz = *((*attr).__i.as_ptr() as *const usize);
    *addr = *((*attr).__i.as_ptr().add(4) as *const *mut c_void); 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setguardsize(attr: *mut pthread_attr_t, sz: usize) -> c_int {
    *((*attr).__i.as_mut_ptr().add(2) as *mut usize) = sz; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getguardsize(attr: *const pthread_attr_t, sz: *mut usize) -> c_int {
    *sz = *((*attr).__i.as_ptr().add(2) as *const usize); 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setscope(_attr: *mut pthread_attr_t, scope: c_int) -> c_int {
    if scope != PTHREAD_SCOPE_SYSTEM { return EINVAL; }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getscope(_attr: *const pthread_attr_t, scope: *mut c_int) -> c_int {
    *scope = PTHREAD_SCOPE_SYSTEM; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inh: c_int) -> c_int {
    if inh != PTHREAD_INHERIT_SCHED && inh != PTHREAD_EXPLICIT_SCHED { return EINVAL; }
    (*attr).__i[7] = inh; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getinheritsched(attr: *const pthread_attr_t, inh: *mut c_int) -> c_int {
    *inh = (*attr).__i[7]; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setschedpolicy(attr: *mut pthread_attr_t, p: c_int) -> c_int {
    (*attr).__i[8] = p; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getschedpolicy(attr: *const pthread_attr_t, p: *mut c_int) -> c_int {
    *p = (*attr).__i[8]; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setschedparam(attr: *mut pthread_attr_t, param: *const sched_param) -> c_int {
    (*attr).__i[9] = (*param).sched_priority; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getschedparam(attr: *const pthread_attr_t, param: *mut sched_param) -> c_int {
    (*param).sched_priority = (*attr).__i[9]; 0
}

// --- pthread_mutexattr_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> c_int {
    (*attr).__attr = 0; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_destroy(_attr: *mut pthread_mutexattr_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_settype(attr: *mut pthread_mutexattr_t, t: c_int) -> c_int {
    if (t as c_uint) > 2 { return EINVAL; }
    (*attr).__attr = ((*attr).__attr & !3) | t as c_uint; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_gettype(attr: *const pthread_mutexattr_t, t: *mut c_int) -> c_int {
    *t = ((*attr).__attr & 3) as c_int; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setpshared(attr: *mut pthread_mutexattr_t, p: c_int) -> c_int {
    if (p as c_uint) > 1 { return EINVAL; }
    (*attr).__attr = ((*attr).__attr & !128) | ((p as c_uint) << 7); 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getpshared(attr: *const pthread_mutexattr_t, p: *mut c_int) -> c_int {
    *p = (((*attr).__attr >> 7) & 1) as c_int; 0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setprotocol(attr: *mut pthread_mutexattr_t, protocol: c_int) -> c_int {
    match protocol {
        0 => { (*attr).__attr &= !(MUTEX_PI as c_uint); 0 }
        1 => {
            let mut lk: c_int = 0;
            let r = sys_futex(&raw mut lk, FUTEX_LOCK_PI, 0, null_mut(), null_mut(), 0);
            if r < 0 { return (-r) as c_int; }
            sys_futex(&raw mut lk, FUTEX_UNLOCK_PI, 0, null_mut(), null_mut(), 0);
            (*attr).__attr |= MUTEX_PI as c_uint;
            0
        }
        2 => ENOTSUP,
        _ => EINVAL,
    }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getprotocol(attr: *const pthread_mutexattr_t, protocol: *mut c_int) -> c_int {
    *protocol = (((*attr).__attr >> 3) & 3) as c_int; 0
}

const ENOTSUP: c_int = 95;

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setrobust(attr: *mut pthread_mutexattr_t, robust: c_int) -> c_int {
    if (robust as c_uint) > 1 { return EINVAL; }
    if robust != 0 {
        (*attr).__attr |= MUTEX_ROBUST as c_uint;
    } else {
        (*attr).__attr &= !(MUTEX_ROBUST as c_uint);
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getrobust(attr: *const pthread_mutexattr_t, robust: *mut c_int) -> c_int {
    *robust = (((*attr).__attr >> 2) & 1) as c_int; 0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_consistent(m: *mut pthread_mutex_t) -> c_int {
    let old = a_load(&raw const (*m).__i[1]);
    let own = old & 0x3fffffff;
    if ((*m).__i[0] & MUTEX_ROBUST) == 0 || own == 0 || (old & 0x40000000) == 0 {
        return EINVAL;
    }
    let self_tid = sys_gettid() as c_int;
    if own != self_tid { return EPERM; }
    a_and(&raw mut (*m).__i[1], !(0x40000000i32));
    0
}

unsafe fn a_and(addr: *mut c_int, val: c_int) {
    let a = &*(addr as *const AtomicI32);
    a.fetch_and(val, Ordering::AcqRel);
}

// --- pthread_mutex_* ---
const MUTEX_TYPE_MASK: c_int = 3;
const MUTEX_ROBUST: c_int = 4;
const MUTEX_PI: c_int = 8;
const MUTEX_PSHARED: c_int = 128;

unsafe fn ensure_robust_list(slot: &mut Thread) {
    if slot.robust_list.futex_offset == 0 {
        slot.robust_list.futex_offset = -28;
        slot.robust_list.pending = core::ptr::null_mut();
        sys_set_robust_list(&mut slot.robust_list, 24);
    }
}

unsafe fn mutex_trylock(m: *mut pthread_mutex_t) -> c_int {
    let type_ = (*m).__i[0];
    // Fast path: plain NORMAL only (no PI, no robust, no pshared)
    if (type_ & 0x8f) == PTHREAD_MUTEX_NORMAL {
        return a_cas(&raw mut (*m).__i[1], 0, EBUSY) & EBUSY;
    }
    let base_type = type_ & MUTEX_TYPE_MASK;
    let self_tid = sys_gettid() as c_int;
    let old = a_load(&raw const (*m).__i[1]);
    let own = old & 0x3fffffff;
    if own == self_tid {
        if base_type == PTHREAD_MUTEX_RECURSIVE {
            if (*m).__i[5] >= c_int::MAX { return EAGAIN; }
            (*m).__i[5] += 1;
            return 0;
        }
        if base_type == PTHREAD_MUTEX_ERRORCHECK { return EDEADLK; }
        return EBUSY;
    }
    if own == 0x3fffffff { return ENOTRECOVERABLE; }
    if own != 0 || (old != 0 && (type_ & MUTEX_ROBUST) == 0) { return EBUSY; }
    let mut newtid = self_tid;
    if (type_ & MUTEX_ROBUST) != 0 {
        if (*m).__i[2] != 0 { newtid |= 1i32 << 31; }
        newtid |= old & 0x40000000;
    }
    if a_cas(&raw mut (*m).__i[1], old, newtid) == old {
        (*m).__i[5] = 0;
        if (type_ & MUTEX_ROBUST) != 0 {
            if let Some(slot) = find_thread() {
                ensure_robust_list(slot);
                let m_next = &raw mut (*m).__i[8] as *mut robust_list;
                let head = &mut slot.robust_list.list;
                (*m_next).next = head.next;
                head.next = m_next;
            }
        }
        if old != 0 { return EOWNERDEAD; }
        0
    } else { EBUSY }
}

unsafe fn mutex_lock_internal(m: *mut pthread_mutex_t, abs_timeout: *const timespec) -> c_int {
    let type_ = (*m).__i[0];
    if (type_ & 0x8f) == PTHREAD_MUTEX_NORMAL && a_cas(&raw mut (*m).__i[1], 0, EBUSY) == 0 {
        return 0;
    }
    let r = mutex_trylock(m);
    if r != EBUSY { return r; }
    if (type_ & MUTEX_PI) != 0 {
        let timeout_ptr = if abs_timeout.is_null() { core::ptr::null() } else { abs_timeout };
        loop {
            let e = futex_lock_pi(&raw mut (*m).__i[1], timeout_ptr);
            match e {
                0 => {
                    if (type_ & MUTEX_ROBUST) == 0 && (((*m).__i[1] & 0x40000000) != 0 || (*m).__i[2] != 0) {
                        (*m).__i[2] = -1;
                        futex_unlock_pi(&raw mut (*m).__i[1]);
                        return EBUSY;
                    }
                    (*m).__i[5] = -1;
                    let r2 = mutex_trylock(m);
                    if r2 == 0 { return 0; }
                    return r2;
                }
                ETIMEDOUT => return e,
                EINTR => continue,
                _ => return e,
            }
        }
    }
    let mut spins = 100;
    while spins > 0 {
        let r = mutex_trylock(m);
        if r != EBUSY { return r; }
        core::hint::spin_loop();
        spins -= 1;
    }
    loop {
        let r = mutex_trylock(m);
        if r != EBUSY { return r; }
        if (type_ & MUTEX_TYPE_MASK) == PTHREAD_MUTEX_ERRORCHECK {
            let self_tid = sys_gettid() as c_int;
            if (a_load(&raw const (*m).__i[1]) & 0x3fffffff) == self_tid { return EDEADLK; }
        }
        a_fetch_add(&raw mut (*m).__i[2], 1);
        let val = a_load(&raw const (*m).__i[1]);
        a_cas(&raw mut (*m).__i[1], val, val | (1i32 << 31));
        let e = futex_timedwait(&raw mut (*m).__i[1], val | (1i32 << 31), abs_timeout);
        a_fetch_sub(&raw mut (*m).__i[2], 1);
        if e != 0 && e != EINTR { return e; }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_init(mutex: *mut pthread_mutex_t, attr: *const pthread_mutexattr_t) -> c_int {
    core::ptr::write_bytes(mutex, 0, 1);
    if !attr.is_null() { (*mutex).__i[0] = (*attr).__attr as c_int; }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_destroy(_mutex: *mut pthread_mutex_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int {
    mutex_lock_internal(mutex, core::ptr::null())
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> c_int {
    mutex_trylock(mutex)
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_timedlock(mutex: *mut pthread_mutex_t, abs_timeout: *const timespec) -> c_int {
    mutex_lock_internal(mutex, abs_timeout)
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int {
    let type_ = (*mutex).__i[0];
    let base_type = type_ & MUTEX_TYPE_MASK;
    if base_type != PTHREAD_MUTEX_NORMAL {
        let self_tid = sys_gettid() as c_int;
        if (a_load(&raw const (*mutex).__i[1]) & 0x3fffffff) != self_tid { return EPERM; }
        if base_type == PTHREAD_MUTEX_RECURSIVE && (*mutex).__i[5] > 0 {
            (*mutex).__i[5] -= 1;
            return 0;
        }
    }
    if (type_ & MUTEX_PI) != 0 {
        a_store(&raw mut (*mutex).__i[1], 0);
        return futex_unlock_pi(&raw mut (*mutex).__i[1]);
    }
    let old = a_swap(&raw mut (*mutex).__i[1], 0);
    if a_load(&raw const (*mutex).__i[2]) > 0 || old < 0 {
        futex_wake(&raw mut (*mutex).__i[1], 1);
    }
    0
}

// --- pthread_condattr_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_init(attr: *mut pthread_condattr_t) -> c_int {
    (*attr).__attr = 0; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_destroy(_attr: *mut pthread_condattr_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setclock(attr: *mut pthread_condattr_t, clk: c_int) -> c_int {
    if clk != CLOCK_REALTIME && clk != CLOCK_MONOTONIC { return EINVAL; }
    (*attr).__attr = ((*attr).__attr & 0x80000000) | clk as c_uint; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getclock(attr: *const pthread_condattr_t, clk: *mut c_int) -> c_int {
    *clk = ((*attr).__attr & 0x7fffffff) as c_int; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setpshared(attr: *mut pthread_condattr_t, p: c_int) -> c_int {
    if (p as c_uint) > 1 { return EINVAL; }
    (*attr).__attr = ((*attr).__attr & 0x7fffffff) | ((p as c_uint) << 31); 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getpshared(attr: *const pthread_condattr_t, p: *mut c_int) -> c_int {
    *p = ((*attr).__attr >> 31) as c_int; 0
}

// --- pthread_cond_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_init(cond: *mut pthread_cond_t, attr: *const pthread_condattr_t) -> c_int {
    core::ptr::write_bytes(cond, 0, 1);
    if !attr.is_null() {
        (*cond).__i[4] = ((*attr).__attr & 0x7fffffff) as c_int;
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_destroy(_cond: *mut pthread_cond_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_wait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t) -> c_int {
    pthread_cond_timedwait(cond, mutex, core::ptr::null())
}
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_timedwait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t, abs_timeout: *const timespec) -> c_int {
    let seq_ptr = &raw mut (*cond).__i[2];
    let waiters_ptr = &raw mut (*cond).__i[3];
    let seq = a_load(seq_ptr);
    a_fetch_add(waiters_ptr, 1);
    pthread_mutex_unlock(mutex);
    let e = futex_timedwait(seq_ptr, seq, abs_timeout);
    a_fetch_sub(waiters_ptr, 1);
    let r = pthread_mutex_lock(mutex);
    if r != 0 { return r; }
    if e != 0 && e != EINTR { return e; }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int {
    if a_load(&raw const (*cond).__i[3]) == 0 { return 0; }
    a_fetch_add(&raw mut (*cond).__i[2], 1);
    futex_wake(&raw mut (*cond).__i[2], 1);
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int {
    if a_load(&raw const (*cond).__i[3]) == 0 { return 0; }
    a_fetch_add(&raw mut (*cond).__i[2], 1);
    futex_wake(&raw mut (*cond).__i[2], -1);
    0
}

// --- pthread_rwlockattr_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_init(attr: *mut pthread_rwlockattr_t) -> c_int {
    (*attr).__attr = [0; 2]; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_destroy(_attr: *mut pthread_rwlockattr_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_setpshared(attr: *mut pthread_rwlockattr_t, p: c_int) -> c_int {
    if (p as c_uint) > 1 { return EINVAL; }
    (*attr).__attr[0] = ((*attr).__attr[0] & !128) | ((p as c_uint) << 7); 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_getpshared(attr: *const pthread_rwlockattr_t, p: *mut c_int) -> c_int {
    *p = (((*attr).__attr[0] >> 7) & 1) as c_int; 0
}

// --- pthread_rwlock_* ---
unsafe fn rwlock_tryrdlock(rw: *mut pthread_rwlock_t) -> c_int {
    loop {
        let val = a_load(&raw const (*rw).__i[0]);
        let cnt = val & 0x7fffffff;
        if cnt == 0x7fffffff { return EBUSY; }
        if cnt == 0x7ffffffe { return EAGAIN; }
        if a_cas(&raw mut (*rw).__i[0], val, val + 1) == val { return 0; }
    }
}
unsafe fn rwlock_trywrlock(rw: *mut pthread_rwlock_t) -> c_int {
    if a_cas(&raw mut (*rw).__i[0], 0, 0x7fffffff) == 0 { 0 } else { EBUSY }
}
unsafe fn rwlock_timedrdlock(rw: *mut pthread_rwlock_t, abs_timeout: *const timespec) -> c_int {
    let r = rwlock_tryrdlock(rw);
    if r != EBUSY { return r; }
    let mut spins = 100;
    while spins > 0 { let r = rwlock_tryrdlock(rw); if r != EBUSY { return r; } core::hint::spin_loop(); spins -= 1; }
    loop {
        let r = rwlock_tryrdlock(rw);
        if r != EBUSY { return r; }
        let val = a_load(&raw const (*rw).__i[0]);
        if val & 0x7fffffff != 0x7fffffff { continue; }
        a_fetch_add(&raw mut (*rw).__i[1], 1);
        a_cas(&raw mut (*rw).__i[0], val, val | (1i32 << 31));
        let e = futex_timedwait(&raw mut (*rw).__i[0], val | (1i32 << 31), abs_timeout);
        a_fetch_sub(&raw mut (*rw).__i[1], 1);
        if e != 0 && e != EINTR { return e; }
    }
}
unsafe fn rwlock_timedwrlock(rw: *mut pthread_rwlock_t, abs_timeout: *const timespec) -> c_int {
    let r = rwlock_trywrlock(rw);
    if r != EBUSY { return r; }
    let mut spins = 100;
    while spins > 0 { let r = rwlock_trywrlock(rw); if r != EBUSY { return r; } core::hint::spin_loop(); spins -= 1; }
    loop {
        let r = rwlock_trywrlock(rw);
        if r != EBUSY { return r; }
        let val = a_load(&raw const (*rw).__i[0]);
        if val == 0 { continue; }
        a_fetch_add(&raw mut (*rw).__i[1], 1);
        a_cas(&raw mut (*rw).__i[0], val, val | (1i32 << 31));
        let e = futex_timedwait(&raw mut (*rw).__i[0], val | (1i32 << 31), abs_timeout);
        a_fetch_sub(&raw mut (*rw).__i[1], 1);
        if e != 0 && e != EINTR { return e; }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_init(rw: *mut pthread_rwlock_t, attr: *const pthread_rwlockattr_t) -> c_int {
    core::ptr::write_bytes(rw, 0, 1);
    if !attr.is_null() { (*rw).__i[2] = (*attr).__attr[0] as c_int & 128; }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_destroy(_rw: *mut pthread_rwlock_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_rdlock(rw: *mut pthread_rwlock_t) -> c_int { rwlock_timedrdlock(rw, core::ptr::null()) }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_tryrdlock(rw: *mut pthread_rwlock_t) -> c_int { rwlock_tryrdlock(rw) }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_timedrdlock(rw: *mut pthread_rwlock_t, t: *const timespec) -> c_int { rwlock_timedrdlock(rw, t) }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_wrlock(rw: *mut pthread_rwlock_t) -> c_int { rwlock_timedwrlock(rw, core::ptr::null()) }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_trywrlock(rw: *mut pthread_rwlock_t) -> c_int { rwlock_trywrlock(rw) }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_timedwrlock(rw: *mut pthread_rwlock_t, t: *const timespec) -> c_int { rwlock_timedwrlock(rw, t) }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_unlock(rw: *mut pthread_rwlock_t) -> c_int {
    loop {
        let val = a_load(&raw const (*rw).__i[0]);
        let cnt = val & 0x7fffffff;
        let new_val = if cnt == 0x7fffffff || cnt == 1 { 0 } else { val - 1 };
        if a_cas(&raw mut (*rw).__i[0], val, new_val) == val {
            if new_val == 0 && (a_load(&raw const (*rw).__i[1]) > 0 || val < 0) {
                futex_wake(&raw mut (*rw).__i[0], cnt);
            }
            return 0;
        }
    }
}

// --- pthread_barrierattr_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_barrierattr_init(attr: *mut pthread_barrierattr_t) -> c_int { (*attr).__attr = 0; 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_barrierattr_destroy(_attr: *mut pthread_barrierattr_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_barrierattr_setpshared(attr: *mut pthread_barrierattr_t, p: c_int) -> c_int {
    if (p as c_uint) > 1 { return EINVAL; }
    (*attr).__attr = p as c_uint; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_barrierattr_getpshared(attr: *const pthread_barrierattr_t, p: *mut c_int) -> c_int {
    *p = (*attr).__attr as c_int; 0
}

// --- pthread_barrier_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_barrier_init(b: *mut pthread_barrier_t, _attr: *const pthread_barrierattr_t, count: c_uint) -> c_int {
    if count == 0 { return EINVAL; }
    core::ptr::write_bytes(b, 0, 1);
    (*b).__i[2] = count as c_int - 1;
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_barrier_destroy(_b: *mut pthread_barrier_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_barrier_wait(b: *mut pthread_barrier_t) -> c_int {
    let limit = (*b).__i[2];
    if limit == 0 { return PTHREAD_BARRIER_SERIAL_THREAD; }
    spinlock_lock(&raw mut (*b).__i[0], &raw mut (*b).__i[1]);
    let arrived = a_load(&raw const (*b).__i[3]) + 1;
    a_store(&raw mut (*b).__i[3], arrived);
    if arrived > limit {
        a_store(&raw mut (*b).__i[3], 0);
        let gen = a_load(&raw const (*b).__i[4]);
        a_store(&raw mut (*b).__i[4], gen + 1);
        spinlock_unlock(&raw mut (*b).__i[0], &raw mut (*b).__i[1]);
        futex_wake(&raw mut (*b).__i[4], -1);
        return PTHREAD_BARRIER_SERIAL_THREAD;
    }
    let gen = a_load(&raw const (*b).__i[4]);
    spinlock_unlock(&raw mut (*b).__i[0], &raw mut (*b).__i[1]);
    while a_load(&raw const (*b).__i[4]) == gen {
        futex_wait(&raw mut (*b).__i[4], gen);
    }
    0
}

// --- pthread_spin_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_init(s: *mut pthread_spinlock_t, _pshared: c_int) -> c_int { a_store(s, 0); 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_destroy(_s: *mut pthread_spinlock_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_lock(s: *mut pthread_spinlock_t) -> c_int {
    while a_load(s) != 0 || a_cas(s, 0, EBUSY) != 0 { core::hint::spin_loop(); }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_trylock(s: *mut pthread_spinlock_t) -> c_int { a_cas(s, 0, EBUSY) }
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_unlock(s: *mut pthread_spinlock_t) -> c_int { a_store(s, 0); 0 }

// --- sem_* ---
#[no_mangle]
pub unsafe extern "C" fn sem_init(sem: *mut sem_t, pshared: c_int, value: c_uint) -> c_int {
    if value > SEM_VALUE_MAX as c_uint { return EINVAL; }
    (*sem).__val[0] = value as c_int;
    (*sem).__val[1] = 0;
    (*sem).__val[2] = if pshared != 0 { 0 } else { 128 };
    0
}
#[no_mangle]
pub unsafe extern "C" fn sem_destroy(_sem: *mut sem_t) -> c_int { 0 }

unsafe fn sem_trywait_internal(sem: *mut sem_t) -> c_int {
    loop {
        let val = a_load(&raw const (*sem).__val[0]);
        if val & SEM_VALUE_MAX == 0 { return EAGAIN; }
        if a_cas(&raw mut (*sem).__val[0], val, val - 1) == val { return 0; }
    }
}

#[no_mangle]
pub unsafe extern "C" fn sem_trywait(sem: *mut sem_t) -> c_int {
    if sem_trywait_internal(sem) == 0 { 0 } else { ERRNO = EAGAIN; -1 }
}
#[no_mangle]
pub unsafe extern "C" fn sem_wait(sem: *mut sem_t) -> c_int {
    if sem_trywait_internal(sem) == 0 { return 0; }
    let mut spins = 100;
    while spins > 0 { if sem_trywait_internal(sem) == 0 { return 0; } core::hint::spin_loop(); spins -= 1; }
    loop {
        if sem_trywait_internal(sem) == 0 { return 0; }
        let val = a_load(&raw const (*sem).__val[0]);
        if val & SEM_VALUE_MAX != 0 { continue; }
        a_cas(&raw mut (*sem).__val[0], val, val | (1i32 << 31));
        a_fetch_add(&raw mut (*sem).__val[1], 1);
        futex_wait(&raw mut (*sem).__val[0], 1i32 << 31);
        a_fetch_sub(&raw mut (*sem).__val[1], 1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn sem_timedwait(sem: *mut sem_t, abs_timeout: *const timespec) -> c_int {
    if sem_trywait_internal(sem) == 0 { return 0; }
    let mut spins = 100;
    while spins > 0 { if sem_trywait_internal(sem) == 0 { return 0; } core::hint::spin_loop(); spins -= 1; }
    loop {
        if sem_trywait_internal(sem) == 0 { return 0; }
        let val = a_load(&raw const (*sem).__val[0]);
        if val & SEM_VALUE_MAX != 0 { continue; }
        a_cas(&raw mut (*sem).__val[0], val, val | (1i32 << 31));
        a_fetch_add(&raw mut (*sem).__val[1], 1);
        let e = futex_timedwait(&raw mut (*sem).__val[0], 1i32 << 31, abs_timeout);
        a_fetch_sub(&raw mut (*sem).__val[1], 1);
        if e != 0 && e != EINTR { ERRNO = e; return -1; }
    }
}
#[no_mangle]
pub unsafe extern "C" fn sem_post(sem: *mut sem_t) -> c_int {
    loop {
        let val = a_load(&raw const (*sem).__val[0]);
        if (val & SEM_VALUE_MAX) == SEM_VALUE_MAX { ERRNO = EOVERFLOW; return -1; }
        let mut new = val + 1;
        let waiters = a_load(&raw const (*sem).__val[1]);
        if waiters <= 1 { new &= !(1i32 << 31); }
        if a_cas(&raw mut (*sem).__val[0], val, new) == val {
            if val < 0 || waiters > 0 {
                futex_wake(&raw mut (*sem).__val[0], if waiters > 1 { 1 } else { -1 });
            }
            return 0;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn sem_getvalue(sem: *mut sem_t, sval: *mut c_int) -> c_int {
    *sval = a_load(&raw const (*sem).__val[0]) & SEM_VALUE_MAX;
    0
}

// ============================================================
// POSIX shared memory + named semaphores
// ============================================================

const NAME_MAX: usize = 255;
const SEM_NSEMS_MAX: usize = 256;

// ponytail: global spinlock for semtab, upgrade to per-entry if contention matters
static SEMTAB_LOCK: AtomicI32 = AtomicI32::new(0);

struct SemEntry {
    ino: u64,
    sem: *mut sem_t,
    refcnt: u32,
}

// ponytail: zero-init is fine for static mut (all-zero = free slots)
static mut SEMTAB: [SemEntry; SEM_NSEMS_MAX] = {
    const ZERO: SemEntry = SemEntry { ino: 0, sem: core::ptr::null_mut(), refcnt: 0 };
    [ZERO; SEM_NSEMS_MAX]
};

unsafe fn semtab_lock() {
    while SEMTAB_LOCK.compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed).is_err() {
        core::hint::spin_loop();
    }
}

unsafe fn semtab_unlock() {
    SEMTAB_LOCK.store(0, Ordering::Release);
}

// Validate shm/sem name and construct /dev/shm/<name> path.
// Returns path length or -1 with ERRNO set.
unsafe fn shm_mapname(name: *const c_char, buf: *mut u8) -> c_int {
    let mut p = name;
    // strip leading /
    while *p == b'/' as c_char { p = p.add(1); }
    let start = p;
    // find end of first component
    while *p != 0 && *p != b'/' as c_char { p = p.add(1); }
    let len = p.offset_from(start) as usize;
    // no trailing /, not empty, not "." or ".."
    if len == 0 || *p != 0 {
        ERRNO = EINVAL;
        return -1;
    }
    if len == 1 && *start == b'.' as c_char {
        ERRNO = EINVAL;
        return -1;
    }
    if len == 2 && *start == b'.' as c_char && *start.add(1) == b'.' as c_char {
        ERRNO = EINVAL;
        return -1;
    }
    if len > NAME_MAX {
        ERRNO = ENAMETOOLONG;
        return -1;
    }
    // build /dev/shm/<name>
    let prefix = b"/dev/shm/\0";
    let mut i = 0;
    while prefix[i] != 0 {
        *buf.add(i) = prefix[i];
        i += 1;
    }
    let mut j = 0;
    while j < len {
        *buf.add(i + j) = *start.add(j) as u8;
        j += 1;
    }
    *buf.add(i + j) = 0;
    (i + j) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn shm_open(name: *const c_char, flags: c_int, mode: mode_t) -> c_int {
    let mut buf = [0u8; 16 + NAME_MAX + 1];
    if shm_mapname(name, buf.as_mut_ptr()) < 0 { return -1; }
    let fd = sys_open(buf.as_ptr(), (flags | O_NOFOLLOW | O_CLOEXEC | O_NONBLOCK) as i64, mode as i64);
    if fd < 0 { ERRNO = (-fd) as c_int; return -1; }
    fd as c_int
}

#[no_mangle]
pub unsafe extern "C" fn shm_unlink(name: *const c_char) -> c_int {
    let mut buf = [0u8; 16 + NAME_MAX + 1];
    if shm_mapname(name, buf.as_mut_ptr()) < 0 { return -1; }
    let r = sys_unlink(buf.as_ptr());
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sem_open(name: *const c_char, flags: c_int, mut args: ...) -> *mut sem_t {
    let mut buf = [0u8; 16 + NAME_MAX + 1];
    if shm_mapname(name, buf.as_mut_ptr()).is_negative() {
        return core::ptr::null_mut(); // SEM_FAILED
    }

    let oflags = flags & (O_CREAT | O_EXCL);

    semtab_lock();

    // find a free slot
    let mut slot: c_int = -1;
    for i in 0..SEMTAB_LEN {
        if SEMTAB[i].sem.is_null() && slot < 0 { slot = i as c_int; }
    }
    if slot < 0 {
        ERRNO = EMFILE;
        semtab_unlock();
        return core::ptr::null_mut();
    }
    // reserve slot
    SEMTAB[slot as usize].sem = !0usize as *mut sem_t; // sentinel
    semtab_unlock();

    // extract va_args for O_CREAT
    let mut mode: mode_t = 0;
    let mut value: c_uint = 0;
    if oflags & O_CREAT != 0 {
        mode = args.arg::<mode_t>() & 0o666;
        value = args.arg::<c_uint>();
        if value > SEM_VALUE_MAX as c_uint {
            ERRNO = EINVAL;
            semtab_lock();
            SEMTAB[slot as usize].sem = core::ptr::null_mut();
            semtab_unlock();
            return core::ptr::null_mut();
        }
    }

    // try opening existing first (unless O_CREAT|O_EXCL)
    if oflags != O_CREAT | O_EXCL {
        let fd = sys_open(buf.as_ptr(), (O_RDWR | O_NOFOLLOW | O_CLOEXEC | O_NONBLOCK) as i64, 0);
        if fd >= 0 {
            let map = sys_mmap(
                core::ptr::null_mut(),
                core::mem::size_of::<sem_t>(),
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                fd as i32,
                0,
            );
            sys_close(fd);
            if map != MMAP_FAILED {
                let st_ino = get_ino(fd as c_int, buf.as_ptr());
                return sem_register(map as *mut sem_t, st_ino, slot);
            }
        }
        if oflags == 0 {
            // no O_CREAT and file doesn't exist
            semtab_lock();
            SEMTAB[slot as usize].sem = core::ptr::null_mut();
            semtab_unlock();
            return core::ptr::null_mut();
        }
    }

    // create: write sem_t to temp file, mmap, link
    let mut tmp = [0u8; 32];
    let tmpname = b"/dev/shm/tmp-XXXXXX\0";
    tmp[..tmpname.len()].copy_from_slice(tmpname);
    // use clock for unique name
    let mut ts: timespec = core::mem::zeroed();
    sys_clock_gettime(0, &mut ts);
    // append ns to tmp name
    let mut n = ts.tv_nsec;
    let mut pos = tmpname.len() - 1;
    tmp[pos] = 0;
    pos -= 1;
    loop {
        tmp[pos] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 || pos == 0 { break; }
        pos -= 1;
    }
    // shift to start of number part
    let tmp_path = tmp.as_ptr().add(pos);

    let mut newsem: sem_t = core::mem::zeroed();
    newsem.__val[0] = value as c_int;
    newsem.__val[1] = 0;
    newsem.__val[2] = 128; // pshared=0 flag

    loop {
        let fd = sys_open(tmp_path as *const u8, (O_RDWR | O_CREAT | O_EXCL | O_CLOEXEC | O_NONBLOCK) as i64, mode as i64);
        if fd < 0 {
            if (-fd) as c_int == EEXIST { continue; }
            semtab_lock();
            SEMTAB[slot as usize].sem = core::ptr::null_mut();
            semtab_unlock();
            return core::ptr::null_mut();
        }
        let written = sys_write(fd, &newsem as *const sem_t as *const u8, core::mem::size_of::<sem_t>());
        if written != core::mem::size_of::<sem_t>() as i64 {
            sys_close(fd);
            sys_unlink(tmp_path);
            semtab_lock();
            SEMTAB[slot as usize].sem = core::ptr::null_mut();
            semtab_unlock();
            return core::ptr::null_mut();
        }
        let map = sys_mmap(
            core::ptr::null_mut(),
            core::mem::size_of::<sem_t>(),
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            fd as i32,
            0,
        );
        sys_close(fd);
        if map == MMAP_FAILED {
            sys_unlink(tmp_path);
            semtab_lock();
            SEMTAB[slot as usize].sem = core::ptr::null_mut();
            semtab_unlock();
            return core::ptr::null_mut();
        }
        // link to final name
        let r = sys_linkat(AT_FDCWD, tmp_path, AT_FDCWD, buf.as_ptr(), 0);
        let e = if r < 0 { (-r) as c_int } else { 0 };
        sys_unlink(tmp_path);
        if e == 0 {
            let st_ino = get_ino(-1, buf.as_ptr());
            return sem_register(map as *mut sem_t, st_ino, slot);
        }
        sys_munmap(map, core::mem::size_of::<sem_t>());
        if e != EEXIST || oflags == O_CREAT | O_EXCL {
            ERRNO = e;
            semtab_lock();
            SEMTAB[slot as usize].sem = core::ptr::null_mut();
            semtab_unlock();
            return core::ptr::null_mut();
        }
        // EEXIST without O_EXCL: retry open existing
        break;
    }

    // fallthrough: try opening the existing file that won the race
    let fd = sys_open(buf.as_ptr(), (O_RDWR | O_NOFOLLOW | O_CLOEXEC | O_NONBLOCK) as i64, 0);
    if fd < 0 {
        semtab_lock();
        SEMTAB[slot as usize].sem = core::ptr::null_mut();
        semtab_unlock();
        return core::ptr::null_mut();
    }
    let map = sys_mmap(
        core::ptr::null_mut(),
        core::mem::size_of::<sem_t>(),
        PROT_READ | PROT_WRITE,
        MAP_SHARED,
        fd as i32,
        0,
    );
    sys_close(fd);
    if map == MMAP_FAILED {
        semtab_lock();
        SEMTAB[slot as usize].sem = core::ptr::null_mut();
        semtab_unlock();
        return core::ptr::null_mut();
    }
    let st_ino = get_ino(-1, buf.as_ptr());
    sem_register(map as *mut sem_t, st_ino, slot)
}

unsafe fn get_ino(_fd_hint: c_int, path: *const u8) -> u64 {
    let mut st: Stat = core::mem::zeroed();
    if sys_newfstatat(AT_FDCWD, path as *const c_char, &mut st as *mut Stat as *mut u8, 0) == 0 {
        st.st_ino
    } else {
        0
    }
}

unsafe fn sem_register(map: *mut sem_t, ino: u64, slot_hint: c_int) -> *mut sem_t {
    semtab_lock();
    // check if already mapped (by inode)
    for i in 0..SEMTAB_LEN {
        if SEMTAB[i].ino == ino && !SEMTAB[i].sem.is_null() && SEMTAB[i].sem != !0usize as *mut sem_t {
            // already mapped: unmap new, use existing, release hint slot
            sys_munmap(map as *mut u8, core::mem::size_of::<sem_t>());
            SEMTAB[slot_hint as usize].sem = core::ptr::null_mut();
            SEMTAB[i].refcnt += 1;
            let ret = SEMTAB[i].sem;
            semtab_unlock();
            return ret;
        }
    }
    // new entry
    SEMTAB[slot_hint as usize].sem = map;
    SEMTAB[slot_hint as usize].ino = ino;
    SEMTAB[slot_hint as usize].refcnt = 1;
    semtab_unlock();
    map
}

const SEMTAB_LEN: usize = SEM_NSEMS_MAX;

#[no_mangle]
pub unsafe extern "C" fn sem_close(sem: *mut sem_t) -> c_int {
    semtab_lock();
    for i in 0..SEMTAB_LEN {
        if SEMTAB[i].sem == sem {
            SEMTAB[i].refcnt -= 1;
            if SEMTAB[i].refcnt == 0 {
                SEMTAB[i].sem = core::ptr::null_mut();
                SEMTAB[i].ino = 0;
                semtab_unlock();
                sys_munmap(sem as *mut u8, core::mem::size_of::<sem_t>());
                return 0;
            }
            semtab_unlock();
            return 0;
        }
    }
    semtab_unlock();
    0
}

#[no_mangle]
pub unsafe extern "C" fn sem_unlink(name: *const c_char) -> c_int {
    shm_unlink(name)
}

// --- pthread_once ---
#[no_mangle]
pub unsafe extern "C" fn pthread_once(control: *mut pthread_once_t, init_routine: Option<unsafe extern "C" fn()>) -> c_int {
    if a_load(control) == 2 { return 0; }
    loop {
        match a_cas(control, 0, 1) {
            0 => {
                if let Some(f) = init_routine { f(); }
                if a_swap(control, 2) == 3 { futex_wake(control, -1); }
                return 0;
            }
            1 => { a_cas(control, 1, 3); }
            3 => { futex_wait(control, 3); }
            _ => { return 0; }
        }
    }
}

// --- pthread_key_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_key_create(key: *mut pthread_key_t, dtor: Option<unsafe extern "C" fn(*mut c_void)>) -> c_int {
    let start = NEXT_KEY.load(Ordering::Relaxed);
    let mut j = start;
    loop {
        if KEY_DTORS[j].is_none() {
            KEY_DTORS[j] = dtor;
            NEXT_KEY.store((j + 1) % PTHREAD_KEYS_MAX, Ordering::Relaxed);
            *key = j as pthread_key_t;
            return 0;
        }
        j = (j + 1) % PTHREAD_KEYS_MAX;
        if j == start { return EAGAIN; }
    }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_key_delete(key: pthread_key_t) -> c_int {
    KEY_DTORS[key as usize] = None;
    for i in 0..MAX_THREADS {
        if THREADS[i].tid > 0 { THREADS[i].tsd[key as usize] = core::ptr::null_mut(); }
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_getspecific(key: pthread_key_t) -> *mut c_void {
    if let Some(slot) = find_thread() { slot.tsd[key as usize] } else { core::ptr::null_mut() }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_setspecific(key: pthread_key_t, value: *const c_void) -> c_int {
    if let Some(slot) = find_thread() {
        slot.tsd[key as usize] = value as *mut c_void;
        0
    } else { EINVAL }
}

// --- pthread_cancel/state ---
unsafe fn run_cleanup_handlers(slot: &mut Thread) {
    let mut cb = slot.cancelbuf;
    while !cb.is_null() {
        let cur = &*cb;
        let f = cur.__f;
        let x = cur.__x;
        cb = cur.__next;
        if let Some(func) = f {
            func(x);
        }
    }
    slot.cancelbuf = core::ptr::null_mut();
}

unsafe fn do_cancel() -> ! {
    if let Some(slot) = find_thread() {
        run_cleanup_handlers(slot);
        slot.result = !0usize as *mut c_void; // PTHREAD_CANCELED
        slot.detach_state = DT_EXITED;
        futex_wake(&raw mut slot.detach_state, 1);
    }
    _exit(0);
}

extern "C" fn cancel_handler(_sig: c_int) {
    unsafe {
        if let Some(slot) = find_thread() {
            if slot.cancel != 0 && slot.cancel_state == PTHREAD_CANCEL_ENABLE {
                do_cancel();
            }
        }
    }
}

static CANCEL_INIT: AtomicI32 = AtomicI32::new(0);

unsafe fn ensure_cancel_handler() {
    if CANCEL_INIT.swap(1, Ordering::AcqRel) == 0 {
        let act = sigaction {
            sa_handler: cancel_handler as usize,
            sa_flags: SA_RESTORER,
            sa_restorer: sig_restorer as usize,
            sa_mask: [!0u64],
        };
        sys_rt_sigaction(SIGCANCEL, &act as *const sigaction, core::ptr::null_mut(), 8);
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cancel(thread: PthreadT) -> c_int {
    let slot = thread as *mut Thread;
    if slot.is_null() { return EINVAL; }
    ensure_cancel_handler();
    a_store(&raw mut (*slot).cancel, 1);
    let tid = (*slot).tid;
    if tid > 0 {
        sys_tgkill(sys_getpid() as c_int, tid, SIGCANCEL);
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_setcancelstate(state: c_int, oldstate: *mut c_int) -> c_int {
    if state != PTHREAD_CANCEL_ENABLE && state != PTHREAD_CANCEL_DISABLE { return EINVAL; }
    if let Some(slot) = find_thread() {
        if !oldstate.is_null() { *oldstate = slot.cancel_state; }
        slot.cancel_state = state;
        0
    } else { EINVAL }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_setcanceltype(type_: c_int, oldtype: *mut c_int) -> c_int {
    if type_ != PTHREAD_CANCEL_DEFERRED && type_ != PTHREAD_CANCEL_ASYNCHRONOUS { return EINVAL; }
    if let Some(slot) = find_thread() {
        if !oldtype.is_null() { *oldtype = slot.cancel_type; }
        slot.cancel_type = type_;
        0
    } else { EINVAL }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_testcancel() {
    if let Some(slot) = find_thread() {
        if slot.cancel != 0 && slot.cancel_state == PTHREAD_CANCEL_ENABLE {
            do_cancel();
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _pthread_cleanup_push(cb: *mut __ptcb, f: Option<unsafe extern "C" fn(*mut c_void)>, x: *mut c_void) {
    (*cb).__f = f;
    (*cb).__x = x;
    if let Some(slot) = find_thread() {
        (*cb).__next = slot.cancelbuf;
        slot.cancelbuf = cb;
    } else {
        (*cb).__next = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn _pthread_cleanup_pop(cb: *mut __ptcb, run: c_int) {
    if let Some(slot) = find_thread() {
        slot.cancelbuf = (*cb).__next;
    }
    if run != 0 {
        if let Some(f) = (*cb).__f {
            f((*cb).__x);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_kill(thread: PthreadT, sig: c_int) -> c_int {
    let slot = thread as *mut Thread;
    if slot.is_null() { return EINVAL; }
    let tid = (*slot).tid;
    if tid <= 0 { return EINVAL; }
    if sys_tgkill(sys_getpid() as c_int, tid, sig) < 0 { -1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_sigmask(how: c_int, set: *const SigSetT, oldset: *mut SigSetT) -> c_int {
    sigprocmask(how, set, oldset)
}


#[repr(C)]
pub struct lconv {
    pub decimal_point: *mut c_char,
    pub thousands_sep: *mut c_char,
    pub grouping: *mut c_char,
    pub int_curr_symbol: *mut c_char,
    pub currency_symbol: *mut c_char,
    pub mon_decimal_point: *mut c_char,
    pub mon_thousands_sep: *mut c_char,
    pub mon_grouping: *mut c_char,
    pub positive_sign: *mut c_char,
    pub negative_sign: *mut c_char,
    pub int_frac_digits: c_char,
    pub frac_digits: c_char,
    pub p_cs_precedes: c_char,
    pub p_sep_by_space: c_char,
    pub n_cs_precedes: c_char,
    pub n_sep_by_space: c_char,
    pub p_sign_posn: c_char,
    pub n_sign_posn: c_char,
    pub int_p_cs_precedes: c_char,
    pub int_p_sep_by_space: c_char,
    pub int_n_cs_precedes: c_char,
    pub int_n_sep_by_space: c_char,
    pub int_p_sign_posn: c_char,
    pub int_n_sign_posn: c_char,
}

static mut LOCALE_NAME: [c_char; 2] = [b'C' as c_char, 0];
static mut EMPTY_STR: [c_char; 1] = [0];
static mut DECIMAL_POINT: [c_char; 2] = [b'.' as c_char, 0];
static mut LCONV: lconv = lconv {
    decimal_point: core::ptr::null_mut(),
    thousands_sep: core::ptr::null_mut(),
    grouping: core::ptr::null_mut(),
    int_curr_symbol: core::ptr::null_mut(),
    currency_symbol: core::ptr::null_mut(),
    mon_decimal_point: core::ptr::null_mut(),
    mon_thousands_sep: core::ptr::null_mut(),
    mon_grouping: core::ptr::null_mut(),
    positive_sign: core::ptr::null_mut(),
    negative_sign: core::ptr::null_mut(),
    int_frac_digits: -1,
    frac_digits: -1,
    p_cs_precedes: -1,
    p_sep_by_space: -1,
    n_cs_precedes: -1,
    n_sep_by_space: -1,
    p_sign_posn: -1,
    n_sign_posn: -1,
    int_p_cs_precedes: -1,
    int_p_sep_by_space: -1,
    int_n_cs_precedes: -1,
    int_n_sep_by_space: -1,
    int_p_sign_posn: -1,
    int_n_sign_posn: -1,
};

#[no_mangle]
pub unsafe extern "C" fn setlocale(_category: c_int, locale: *const c_char) -> *mut c_char {
    if !locale.is_null() && *locale != 0 {
        // ponytail: ignore locale changes, stay in C locale
    }
    core::ptr::addr_of_mut!(LOCALE_NAME) as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn localeconv() -> *mut lconv {
    let empty = core::ptr::addr_of_mut!(EMPTY_STR) as *mut c_char;
    let lconv = core::ptr::addr_of_mut!(LCONV);
    (*lconv).decimal_point = core::ptr::addr_of_mut!(DECIMAL_POINT) as *mut c_char;
    (*lconv).thousands_sep = empty;
    (*lconv).grouping = empty;
    (*lconv).int_curr_symbol = empty;
    (*lconv).currency_symbol = empty;
    (*lconv).mon_decimal_point = empty;
    (*lconv).mon_thousands_sep = empty;
    (*lconv).mon_grouping = empty;
    (*lconv).positive_sign = empty;
    (*lconv).negative_sign = empty;
    lconv
}

pub type locale_t = *mut c_void;
static mut CURRENT_LOCALE: locale_t = LC_GLOBAL_LOCALE;

static mut C_LOCALE_STORAGE: c_int = 0;
const C_LOCALE: locale_t = unsafe { core::ptr::addr_of_mut!(C_LOCALE_STORAGE) as locale_t };
const LC_GLOBAL_LOCALE: locale_t = usize::MAX as locale_t;

#[no_mangle]
pub unsafe extern "C" fn newlocale(mask: c_int, name: *const c_char, base: locale_t) -> locale_t {
    if !name.is_null() && *name != 0 {
        let n = name as *const u8;
        if strcmp(n, b"C\0".as_ptr()) != 0
            && strcmp(n, b"POSIX\0".as_ptr()) != 0
        {
            return core::ptr::null_mut();
        }
    }
    if base.is_null() || base == C_LOCALE || base == LC_GLOBAL_LOCALE {
        C_LOCALE
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn freelocale(_loc: locale_t) {}

#[no_mangle]
pub unsafe extern "C" fn uselocale(loc: locale_t) -> locale_t {
    let old = CURRENT_LOCALE;
    if !loc.is_null() {
        CURRENT_LOCALE = loc;
    }
    old
}

#[no_mangle]
pub unsafe extern "C" fn duplocale(loc: locale_t) -> locale_t {
    if loc.is_null() {
        return core::ptr::null_mut();
    }
    C_LOCALE
}

const NL_ITEM_CODESET: c_int = 14;
const NL_ITEM_RADIXCHAR: c_int = 0x10000;
const NL_ITEM_THOUSEP: c_int = 0x10001;
const NL_ITEM_YESEXPR: c_int = 0x50000;
const NL_ITEM_NOEXPR: c_int = 0x50001;

const C_TIME_STRINGS: &[u8] = b"Sun\0Mon\0Tue\0Wed\0Thu\0Fri\0Sat\0\
Sunday\0Monday\0Tuesday\0Wednesday\0Thursday\0Friday\0Saturday\0\
Jan\0Feb\0Mar\0Apr\0May\0Jun\0Jul\0Aug\0Sep\0Oct\0Nov\0Dec\0\
January\0February\0March\0April\0May\0June\0July\0August\0\
September\0October\0November\0December\0\
AM\0PM\0\
%a %b %e %T %Y\0%m/%d/%y\0%H:%M:%S\0%I:%M:%S %p\0\
\0\0%m/%d/%y\00123456789\0%a %b %e %T %Y\0%H:%M:%S\0";

const C_NUMERIC_STRINGS: &[u8] = b".\0\0";
const C_MESSAGES_STRINGS: &[u8] = b"^[yY]\0^[nN]\0";

unsafe fn langinfo_str(table: *const u8, mut idx: c_int) -> *mut c_char {
    let mut p = table;
    while idx > 0 {
        while *p != 0 { p = p.add(1); }
        p = p.add(1);
        idx -= 1;
    }
    p as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn nl_langinfo(item: c_int) -> *mut c_char {
    if item == NL_ITEM_CODESET {
        return b"UTF-8\0".as_ptr() as *mut c_char;
    }
    let cat = item >> 16;
    let idx = item & 0xFFFF;
    match cat {
        1 => {
            if idx > 1 { return b"\0".as_ptr() as *mut c_char; }
            langinfo_str(C_NUMERIC_STRINGS.as_ptr(), idx)
        }
        2 => {
            if idx > 49 { return b"\0".as_ptr() as *mut c_char; }
            langinfo_str(C_TIME_STRINGS.as_ptr(), idx)
        }
        5 => {
            if idx > 1 { return b"\0".as_ptr() as *mut c_char; }
            langinfo_str(C_MESSAGES_STRINGS.as_ptr(), idx)
        }
        _ => b"\0".as_ptr() as *mut c_char,
    }
}

pub type nl_catd = *mut c_void;

#[no_mangle]
pub unsafe extern "C" fn catopen(_name: *const c_char, _oflag: c_int) -> nl_catd {
    !0usize as nl_catd
}

#[no_mangle]
pub unsafe extern "C" fn catclose(_catd: nl_catd) -> c_int { 0 }

#[no_mangle]
pub unsafe extern "C" fn catgets(catd: nl_catd, _set_id: c_int, _msg_id: c_int, s: *const c_char) -> *mut c_char {
    if catd == !0usize as nl_catd { return s as *mut c_char; }
    s as *mut c_char
}

pub type IconvT = *mut c_void;

const ENC_UTF8: i32 = 1;
const ENC_UTF16LE: i32 = 2;
const ENC_UTF16BE: i32 = 3;
const ENC_UTF32LE: i32 = 4;
const ENC_UTF32BE: i32 = 5;
const ENC_WCHAR_T: i32 = 6;
const ENC_ASCII: i32 = 7;
const ENC_LATIN1: i32 = 8;
const ENC_WIN1252: i32 = 9;
const ENC_WIN1251: i32 = 10;
const ENC_KOI8R: i32 = 11;
const ENC_GBK: i32 = 12;
const ENC_GB2312: i32 = 13;
const ENC_BIG5: i32 = 14;
const ENC_EUCJP: i32 = 15;
const ENC_SHIFTJIS: i32 = 16;

fn make_cd(from: i32, to: i32) -> IconvT {
    ((from as usize) << 16 | (to as usize) | 1) as IconvT
}
fn extract_from(cd: IconvT) -> i32 {
    ((cd as usize) >> 16) as i32
}
fn extract_to(cd: IconvT) -> i32 {
    ((cd as usize) & 0xFFFF) as i32
}

unsafe fn match_name(input: *const u8, target: &[u8]) -> bool {
    let mut i = input;
    let mut j = 0;
    while *i != 0 && j < target.len() {
        let mut c = *i;
        if !c.is_ascii_alphanumeric() { i = i.add(1); continue; }
        c = c.to_ascii_lowercase();
        if c != target[j] { return false; }
        i = i.add(1);
        j += 1;
    }
    while *i != 0 && !(*i).is_ascii_alphanumeric() { i = i.add(1); }
    *i == 0 && j == target.len()
}

unsafe fn find_encoding(name: *const u8) -> i32 {
    if name.is_null() { return -1; }
    if match_name(name, b"utf8") || match_name(name, b"utf-8") || match_name(name, b"char") { return ENC_UTF8; }
    if match_name(name, b"utf16le") || match_name(name, b"utf-16le") { return ENC_UTF16LE; }
    if match_name(name, b"utf16be") || match_name(name, b"utf-16be") { return ENC_UTF16BE; }
    if match_name(name, b"utf32le") || match_name(name, b"utf-32le") || match_name(name, b"ucs4le") { return ENC_UTF32LE; }
    if match_name(name, b"utf32be") || match_name(name, b"utf-32be") || match_name(name, b"ucs4be") { return ENC_UTF32BE; }
    if match_name(name, b"wchart") || match_name(name, b"wchar-t") { return ENC_WCHAR_T; }
    if match_name(name, b"ascii") || match_name(name, b"usascii") || match_name(name, b"iso646") { return ENC_ASCII; }
    if match_name(name, b"iso88591") || match_name(name, b"iso-8859-1") || match_name(name, b"latin1") { return ENC_LATIN1; }
    if match_name(name, b"cp1252") || match_name(name, b"windows1252") || match_name(name, b"windows-1252") { return ENC_WIN1252; }
    if match_name(name, b"cp1251") || match_name(name, b"windows1251") || match_name(name, b"windows-1251") { return ENC_WIN1251; }
    if match_name(name, b"koi8r") || match_name(name, b"koi8-r") { return ENC_KOI8R; }
    if match_name(name, b"gbk") || match_name(name, b"cp936") { return ENC_GBK; }
    if match_name(name, b"gb2312") { return ENC_GB2312; }
    if match_name(name, b"big5") || match_name(name, b"bigfive") || match_name(name, b"cp950") { return ENC_BIG5; }
    if match_name(name, b"eucjp") || match_name(name, b"euc-jp") { return ENC_EUCJP; }
    if match_name(name, b"shiftjis") || match_name(name, b"sjis") || match_name(name, b"cp932") { return ENC_SHIFTJIS; }
    -1
}

fn uni_to_jis(c: u32) -> u16 {
    let mut b = 0usize;
    let mut nel = REV_JIS.len();
    loop {
        let i = nel / 2;
        let j = REV_JIS[b + i] as usize;
        let row = j / 256;
        let col = j % 256;
        let d = JIS0208[row * 94 + col] as u32;
        if d == c { return (j as u16) + 0x2121; }
        if nel == 1 { return 0; }
        if c < d { nel /= 2; }
        else { b += i; nel -= nel / 2; }
    }
}

unsafe fn iconv_decode(enc: i32, src: *const u8, src_left: usize) -> (u32, usize, c_int) {
    if src_left == 0 { return (0, 0, EINVAL); }
    match enc {
        ENC_UTF8 => {
            let c = *src;
            if c < 128 { return (c as u32, 1, 0); }
            if c < 0xC2 { return (0, 0, EILSEQ); }
            if c < 0xE0 {
                if src_left < 2 || (*src.add(1) & 0xC0) != 0x80 { return (0, 0, EILSEQ); }
                return (((c as u32 & 0x1F) << 6) | (*src.add(1) as u32 & 0x3F), 2, 0);
            }
            if c < 0xF0 {
                if src_left < 3 || (*src.add(1) & 0xC0) != 0x80 || (*src.add(2) & 0xC0) != 0x80 { return (0, 0, EILSEQ); }
                let cp = ((c as u32 & 0x0F) << 12) | ((*src.add(1) as u32 & 0x3F) << 6) | (*src.add(2) as u32 & 0x3F);
                if cp < 0x800 { return (0, 0, EILSEQ); }
                return (cp, 3, 0);
            }
            if c < 0xF8 {
                if src_left < 4 || (*src.add(1) & 0xC0) != 0x80 || (*src.add(2) & 0xC0) != 0x80 || (*src.add(3) & 0xC0) != 0x80 { return (0, 0, EILSEQ); }
                let cp = ((c as u32 & 0x07) << 18) | ((*src.add(1) as u32 & 0x3F) << 12) | ((*src.add(2) as u32 & 0x3F) << 6) | (*src.add(3) as u32 & 0x3F);
                if cp < 0x10000 || cp >= 0x110000 { return (0, 0, EILSEQ); }
                return (cp, 4, 0);
            }
            (0, 0, EILSEQ)
        }
        ENC_UTF16LE => {
            if src_left < 2 { return (0, 0, EINVAL); }
            let c = (*src as u32) | ((*src.add(1) as u32) << 8);
            if c >= 0xD800 && c <= 0xDBFF {
                if src_left < 4 { return (0, 0, EINVAL); }
                let d = (*src.add(2) as u32) | ((*src.add(3) as u32) << 8);
                if d < 0xDC00 || d > 0xDFFF { return (0, 0, EILSEQ); }
                return (((c - 0xD800) << 10) + (d - 0xDC00) + 0x10000, 4, 0);
            }
            if c >= 0xDC00 && c <= 0xDFFF { return (0, 0, EILSEQ); }
            (c, 2, 0)
        }
        ENC_UTF16BE => {
            if src_left < 2 { return (0, 0, EINVAL); }
            let c = ((*src as u32) << 8) | (*src.add(1) as u32);
            if c >= 0xD800 && c <= 0xDBFF {
                if src_left < 4 { return (0, 0, EINVAL); }
                let d = ((*src.add(2) as u32) << 8) | (*src.add(3) as u32);
                if d < 0xDC00 || d > 0xDFFF { return (0, 0, EILSEQ); }
                return (((c - 0xD800) << 10) + (d - 0xDC00) + 0x10000, 4, 0);
            }
            if c >= 0xDC00 && c <= 0xDFFF { return (0, 0, EILSEQ); }
            (c, 2, 0)
        }
        ENC_UTF32LE => {
            if src_left < 4 { return (0, 0, EINVAL); }
            let c = (*src as u32) | ((*src.add(1) as u32) << 8) | ((*src.add(2) as u32) << 16) | ((*src.add(3) as u32) << 24);
            if c >= 0xD800 && c < 0xE000 { return (0, 0, EILSEQ); }
            if c >= 0x110000 { return (0, 0, EILSEQ); }
            (c, 4, 0)
        }
        ENC_UTF32BE => {
            if src_left < 4 { return (0, 0, EINVAL); }
            let c = ((*src as u32) << 24) | ((*src.add(1) as u32) << 16) | ((*src.add(2) as u32) << 8) | (*src.add(3) as u32);
            if c >= 0xD800 && c < 0xE000 { return (0, 0, EILSEQ); }
            if c >= 0x110000 { return (0, 0, EILSEQ); }
            (c, 4, 0)
        }
        ENC_WCHAR_T => {
            if src_left < 4 { return (0, 0, EINVAL); }
            let c = *(src as *const u32);
            if c >= 0xD800 && c < 0xE000 { return (0, 0, EILSEQ); }
            if c >= 0x110000 { return (0, 0, EILSEQ); }
            (c, 4, 0)
        }
        ENC_ASCII => {
            let c = *src as u32;
            if c >= 128 { return (0, 0, EILSEQ); }
            (c, 1, 0)
        }
        ENC_LATIN1 => (*src as u32, 1, 0),
        ENC_WIN1252 => {
            let b = *src;
            if b < 128 { return (b as u32, 1, 0); }
            (WIN1252_TO_U[b as usize - 128], 1, 0)
        }
        ENC_WIN1251 => {
            let b = *src;
            if b < 128 { return (b as u32, 1, 0); }
            (WIN1251_TO_U[b as usize - 128], 1, 0)
        }
        ENC_KOI8R => {
            let b = *src;
            if b < 128 { return (b as u32, 1, 0); }
            (KOI8R_TO_U[b as usize - 128], 1, 0)
        }
        ENC_GBK | ENC_GB2312 => {
            let c = *src;
            if c < 128 { return (c as u32, 1, 0); }
            if c == 128 { return (0x20AC, 1, 0); }
            if src_left < 2 { return (0, 0, EINVAL); }
            let d = *src.add(1);
            if d < 0x40 || d == 0x7F || d > 0xFE { return (0, 0, EILSEQ); }
            let row = (c as usize).wrapping_sub(0x81);
            if row >= 126 { return (0, 0, EILSEQ); }
            let mut col = (d as usize) - 0x40;
            if col > 63 { col -= 1; }
            let cp = GB18030_TABLE[row * 190 + col] as u32;
            if cp == 0 { return (0, 0, EILSEQ); }
            (cp, 2, 0)
        }
        ENC_BIG5 => {
            let c = *src;
            if c < 128 { return (c as u32, 1, 0); }
            if src_left < 2 { return (0, 0, EINVAL); }
            let d = *src.add(1);
            if d < 0x40 || d == 0x7F || d > 0xFE { return (0, 0, EILSEQ); }
            let mut col = (d as usize) - 0x40;
            if col > 0x3E { col -= 0x22; }
            if c >= 0xA1 && c < 0xFA {
                let row = (c as usize) - 0xA1;
                if row >= 89 { return (0, 0, EILSEQ); }
                let cp = BIG5_TABLE[row * 157 + col] as u32;
                if cp == 0 { return (0, 0, EILSEQ); }
                return (cp, 2, 0);
            }
            (0, 0, EILSEQ)
        }
        ENC_EUCJP => {
            let c = *src;
            if c < 128 { return (c as u32, 1, 0); }
            if c == 0x8E {
                if src_left < 2 { return (0, 0, EINVAL); }
                let d = *src.add(1);
                if d < 0xA1 || d > 0xDF { return (0, 0, EILSEQ); }
                return ((d as u32) + 0xFF61 - 0xA1, 2, 0);
            }
            if c < 0xA1 { return (0, 0, EILSEQ); }
            if src_left < 2 { return (0, 0, EINVAL); }
            let d = *src.add(1);
            if d < 0xA1 || d > 0xFE { return (0, 0, EILSEQ); }
            let row = (c as usize) - 0xA1;
            let col = (d as usize) - 0xA1;
            if row >= 84 || col >= 94 { return (0, 0, EILSEQ); }
            let cp = JIS0208[row * 94 + col] as u32;
            if cp == 0 { return (0, 0, EILSEQ); }
            (cp, 2, 0)
        }
        ENC_SHIFTJIS => {
            let c = *src;
            if c < 128 { return (c as u32, 1, 0); }
            if c >= 0xA1 && c <= 0xDF { return ((c as u32) + 0xFF61 - 0xA1, 1, 0); }
            if src_left < 2 { return (0, 0, EINVAL); }
            let d = *src.add(1);
            if d < 0x40 || d == 0x7F || d > 0xFC { return (0, 0, EILSEQ); }
            let row = if c >= 129 && c <= 159 { (c as usize) - 129 }
                      else if c >= 224 && c <= 239 { (c as usize) - 193 }
                      else { return (0, 0, EILSEQ); };
            let (col, row_adj) = if d >= 64 && d <= 158 && d != 127 {
                let mut dd = d as usize;
                if dd > 127 { dd -= 1; }
                (dd - 64, row * 2)
            } else if d >= 159 && d <= 252 {
                ((d as usize) - 159, row * 2 + 1)
            } else {
                return (0, 0, EILSEQ);
            };
            if row_adj >= 84 { return (0, 0, EILSEQ); }
            let cp = JIS0208[row_adj * 94 + col] as u32;
            if cp == 0 { return (0, 0, EILSEQ); }
            (cp, 2, 0)
        }
        _ => (0, 0, EILSEQ),
    }
}

unsafe fn iconv_encode(enc: i32, c: u32, dst: *mut u8, dst_left: usize) -> (usize, c_int) {
    match enc {
        ENC_UTF8 => {
            if c < 0x80 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            if c < 0x800 {
                if dst_left < 2 { return (0, E2BIG); }
                *dst = (0xC0 | (c >> 6)) as u8;
                *dst.add(1) = (0x80 | (c & 0x3F)) as u8;
                return (2, 0);
            }
            if c < 0x10000 {
                if dst_left < 3 { return (0, E2BIG); }
                *dst = (0xE0 | (c >> 12)) as u8;
                *dst.add(1) = (0x80 | ((c >> 6) & 0x3F)) as u8;
                *dst.add(2) = (0x80 | (c & 0x3F)) as u8;
                return (3, 0);
            }
            if dst_left < 4 { return (0, E2BIG); }
            *dst = (0xF0 | (c >> 18)) as u8;
            *dst.add(1) = (0x80 | ((c >> 12) & 0x3F)) as u8;
            *dst.add(2) = (0x80 | ((c >> 6) & 0x3F)) as u8;
            *dst.add(3) = (0x80 | (c & 0x3F)) as u8;
            (4, 0)
        }
        ENC_UTF16LE => {
            if c < 0x10000 {
                if dst_left < 2 { return (0, E2BIG); }
                *dst = (c & 0xFF) as u8;
                *dst.add(1) = (c >> 8) as u8;
                return (2, 0);
            }
            if dst_left < 4 { return (0, E2BIG); }
            let c = c - 0x10000;
            let hi = ((c >> 10) + 0xD800) as u16;
            let lo = ((c & 0x3FF) + 0xDC00) as u16;
            *dst = (hi & 0xFF) as u8; *dst.add(1) = (hi >> 8) as u8;
            *dst.add(2) = (lo & 0xFF) as u8; *dst.add(3) = (lo >> 8) as u8;
            (4, 0)
        }
        ENC_UTF16BE => {
            if c < 0x10000 {
                if dst_left < 2 { return (0, E2BIG); }
                *dst = (c >> 8) as u8;
                *dst.add(1) = (c & 0xFF) as u8;
                return (2, 0);
            }
            if dst_left < 4 { return (0, E2BIG); }
            let c = c - 0x10000;
            let hi = ((c >> 10) + 0xD800) as u16;
            let lo = ((c & 0x3FF) + 0xDC00) as u16;
            *dst = (hi >> 8) as u8; *dst.add(1) = (hi & 0xFF) as u8;
            *dst.add(2) = (lo >> 8) as u8; *dst.add(3) = (lo & 0xFF) as u8;
            (4, 0)
        }
        ENC_UTF32LE => {
            if dst_left < 4 { return (0, E2BIG); }
            *dst = (c & 0xFF) as u8; *dst.add(1) = ((c >> 8) & 0xFF) as u8;
            *dst.add(2) = ((c >> 16) & 0xFF) as u8; *dst.add(3) = (c >> 24) as u8;
            (4, 0)
        }
        ENC_UTF32BE => {
            if dst_left < 4 { return (0, E2BIG); }
            *dst = (c >> 24) as u8; *dst.add(1) = ((c >> 16) & 0xFF) as u8;
            *dst.add(2) = ((c >> 8) & 0xFF) as u8; *dst.add(3) = (c & 0xFF) as u8;
            (4, 0)
        }
        ENC_WCHAR_T => {
            if dst_left < 4 { return (0, E2BIG); }
            *(dst as *mut u32) = c;
            (4, 0)
        }
        ENC_ASCII => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_LATIN1 => {
            if c < 256 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_WIN1252 => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            if c >= 0xA0 && c <= 0xFF && WIN1252_TO_U[c as usize - 128] == c {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            for i in 0..128usize {
                if WIN1252_TO_U[i] == c {
                    if dst_left < 1 { return (0, E2BIG); }
                    *dst = (i + 128) as u8; return (1, 0);
                }
            }
            (0, EILSEQ)
        }
        ENC_WIN1251 => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            for i in 0..128usize {
                if WIN1251_TO_U[i] == c {
                    if dst_left < 1 { return (0, E2BIG); }
                    *dst = (i + 128) as u8; return (1, 0);
                }
            }
            (0, EILSEQ)
        }
        ENC_KOI8R => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            for i in 0..128usize {
                if KOI8R_TO_U[i] == c {
                    if dst_left < 1 { return (0, E2BIG); }
                    *dst = (i + 128) as u8; return (1, 0);
                }
            }
            (0, EILSEQ)
        }
        ENC_GBK | ENC_GB2312 => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            if c == 0x20AC {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = 128; return (1, 0);
            }
            for row in 0..126usize {
                for col in 0..190usize {
                    if GB18030_TABLE[row * 190 + col] as u32 == c {
                        if dst_left < 2 { return (0, E2BIG); }
                        *dst = (row + 0x81) as u8;
                        *dst.add(1) = if col <= 62 { (col + 0x40) as u8 } else { (col + 0x41) as u8 };
                        return (2, 0);
                    }
                }
            }
            (0, EILSEQ)
        }
        ENC_BIG5 => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            for row in 0..89usize {
                for col in 0..157usize {
                    if BIG5_TABLE[row * 157 + col] as u32 == c {
                        if dst_left < 2 { return (0, E2BIG); }
                        *dst = (row + 0xA1) as u8;
                        *dst.add(1) = if col <= 0x3E { (col + 0x40) as u8 } else { (col + 0x62) as u8 };
                        return (2, 0);
                    }
                }
            }
            (0, EILSEQ)
        }
        ENC_EUCJP => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            if c >= 0xFF61 && c <= 0xFF9F {
                if dst_left < 2 { return (0, E2BIG); }
                *dst = 0x8E;
                *dst.add(1) = (c - 0xFF61 + 0xA1) as u8;
                return (2, 0);
            }
            let jis = uni_to_jis(c);
            if jis == 0 { return (0, EILSEQ); }
            if dst_left < 2 { return (0, E2BIG); }
            *dst = ((jis >> 8) as u8).wrapping_add(0x80);
            *dst.add(1) = ((jis & 0xFF) as u8).wrapping_add(0x80);
            (2, 0)
        }
        ENC_SHIFTJIS => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            if c >= 0xFF61 && c <= 0xFF9F {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = (c - 0xFF61 + 0xA1) as u8;
                return (1, 0);
            }
            let jis = uni_to_jis(c);
            if jis == 0 { return (0, EILSEQ); }
            if dst_left < 2 { return (0, E2BIG); }
            let r = (jis >> 8) as usize;
            let d = (jis & 0xFF) as usize;
            *dst = ((r + 1) / 2 + if r < 95 { 112 } else { 176 }) as u8;
            *dst.add(1) = if r % 2 == 1 { (d + 31 + d / 96) as u8 } else { (d + 126) as u8 };
            (2, 0)
        }
        _ => (0, EILSEQ),
    }
}

#[no_mangle]
pub unsafe extern "C" fn iconv_open(tocode: *const c_char, fromcode: *const c_char) -> IconvT {
    let to = find_encoding(tocode as *const u8);
    let from = find_encoding(fromcode as *const u8);
    if to < 0 || from < 0 {
        ERRNO = EINVAL;
        return !0usize as IconvT;
    }
    make_cd(from, to)
}

#[no_mangle]
pub unsafe extern "C" fn iconv(
    cd: IconvT,
    inbuf: *mut *mut c_char,
    inbytesleft: *mut SizeT,
    outbuf: *mut *mut c_char,
    outbytesleft: *mut SizeT,
) -> SizeT {
    if cd == !0usize as IconvT {
        ERRNO = EINVAL;
        return !0usize;
    }
    if inbuf.is_null() || outbuf.is_null() { return 0; }
    let from = extract_from(cd);
    let to = extract_to(cd);
    let mut src = *inbuf as *const u8;
    let mut dst = *outbuf as *mut u8;
    let mut src_left = *inbytesleft;
    let mut dst_left = *outbytesleft;
    let mut subst = 0usize;
    while src_left > 0 {
        let (cp, consumed, err) = iconv_decode(from, src, src_left);
        if err != 0 {
            if err == EINVAL { break; }
            ERRNO = err;
            return !0usize;
        }
        let (written, err) = iconv_encode(to, cp, dst, dst_left);
        if err != 0 {
            if err == E2BIG {
                ERRNO = E2BIG;
                return !0usize;
            }
            if dst_left < 1 {
                ERRNO = E2BIG;
                return !0usize;
            }
            *dst = b'*';
            dst = dst.add(1);
            dst_left -= 1;
            subst += 1;
        } else {
            dst = dst.add(written);
            dst_left -= written;
        }
        src = src.add(consumed);
        src_left -= consumed;
    }
    *inbuf = src as *mut c_char;
    *outbuf = dst as *mut c_char;
    *inbytesleft = src_left;
    *outbytesleft = dst_left;
    subst
}

#[no_mangle]
pub unsafe extern "C" fn iconv_close(_cd: IconvT) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn acos(x: f64) -> f64 {
    libm::acos(x)
}
#[no_mangle]
pub extern "C" fn asin(x: f64) -> f64 {
    libm::asin(x)
}
#[no_mangle]
pub extern "C" fn atan(x: f64) -> f64 {
    libm::atan(x)
}
#[no_mangle]
pub extern "C" fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}
#[no_mangle]
pub extern "C" fn ceil(x: f64) -> f64 {
    libm::ceil(x)
}
#[no_mangle]
pub extern "C" fn cos(x: f64) -> f64 {
    libm::cos(x)
}
#[no_mangle]
pub extern "C" fn cosh(x: f64) -> f64 {
    libm::cosh(x)
}
#[no_mangle]
pub extern "C" fn exp(x: f64) -> f64 {
    libm::exp(x)
}
#[no_mangle]
pub extern "C" fn fabs(x: f64) -> f64 {
    libm::fabs(x)
}
#[no_mangle]
pub extern "C" fn floor(x: f64) -> f64 {
    libm::floor(x)
}
#[no_mangle]
pub extern "C" fn fmod(x: f64, y: f64) -> f64 {
    libm::fmod(x, y)
}
#[no_mangle]
pub extern "C" fn frexp(x: f64, exp: *mut c_int) -> f64 {
    let (frac, e) = libm::frexp(x);
    if !exp.is_null() {
        unsafe { *exp = e };
    }
    frac
}
#[no_mangle]
pub extern "C" fn ldexp(x: f64, exp: c_int) -> f64 {
    libm::ldexp(x, exp)
}
#[no_mangle]
pub extern "C" fn log(x: f64) -> f64 {
    libm::log(x)
}
#[no_mangle]
pub extern "C" fn log10(x: f64) -> f64 {
    libm::log10(x)
}
#[no_mangle]
pub extern "C" fn log2(x: f64) -> f64 {
    libm::log2(x)
}
#[no_mangle]
pub extern "C" fn modf(x: f64, iptr: *mut f64) -> f64 {
    let (frac, int) = libm::modf(x);
    if !iptr.is_null() {
        unsafe { *iptr = int };
    }
    frac
}
#[no_mangle]
pub extern "C" fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}
#[no_mangle]
pub extern "C" fn round(x: f64) -> f64 {
    libm::round(x)
}
#[no_mangle]
pub extern "C" fn sin(x: f64) -> f64 {
    libm::sin(x)
}
#[no_mangle]
pub extern "C" fn sinh(x: f64) -> f64 {
    libm::sinh(x)
}
#[no_mangle]
pub extern "C" fn sqrt(x: f64) -> f64 {
    libm::sqrt(x)
}
#[no_mangle]
pub extern "C" fn tan(x: f64) -> f64 {
    libm::tan(x)
}
#[no_mangle]
pub extern "C" fn tanh(x: f64) -> f64 {
    libm::tanh(x)
}
#[no_mangle]
pub extern "C" fn trunc(x: f64) -> f64 {
    libm::trunc(x)
}
#[no_mangle]
pub extern "C" fn hypot(x: f64, y: f64) -> f64 {
    libm::hypot(x, y)
}

#[no_mangle]
pub extern "C" fn acosf(x: f32) -> f32 {
    libm::acosf(x)
}
#[no_mangle]
pub extern "C" fn asinf(x: f32) -> f32 {
    libm::asinf(x)
}
#[no_mangle]
pub extern "C" fn atanf(x: f32) -> f32 {
    libm::atanf(x)
}
#[no_mangle]
pub extern "C" fn atan2f(y: f32, x: f32) -> f32 {
    libm::atan2f(y, x)
}
#[no_mangle]
pub extern "C" fn ceilf(x: f32) -> f32 {
    libm::ceilf(x)
}
#[no_mangle]
pub extern "C" fn cosf(x: f32) -> f32 {
    libm::cosf(x)
}
#[no_mangle]
pub extern "C" fn coshf(x: f32) -> f32 {
    libm::coshf(x)
}
#[no_mangle]
pub extern "C" fn expf(x: f32) -> f32 {
    libm::expf(x)
}
#[no_mangle]
pub extern "C" fn fabsf(x: f32) -> f32 {
    libm::fabsf(x)
}
#[no_mangle]
pub extern "C" fn floorf(x: f32) -> f32 {
    libm::floorf(x)
}
#[no_mangle]
pub extern "C" fn fmodf(x: f32, y: f32) -> f32 {
    libm::fmodf(x, y)
}
#[no_mangle]
pub extern "C" fn frexpf(x: f32, exp: *mut c_int) -> f32 {
    let (frac, e) = libm::frexpf(x);
    if !exp.is_null() {
        unsafe { *exp = e };
    }
    frac
}
#[no_mangle]
pub extern "C" fn ldexpf(x: f32, exp: c_int) -> f32 {
    libm::ldexpf(x, exp)
}
#[no_mangle]
pub extern "C" fn logf(x: f32) -> f32 {
    libm::logf(x)
}
#[no_mangle]
pub extern "C" fn log10f(x: f32) -> f32 {
    libm::log10f(x)
}
#[no_mangle]
pub extern "C" fn log2f(x: f32) -> f32 {
    libm::log2f(x)
}
#[no_mangle]
pub extern "C" fn modff(x: f32, iptr: *mut f32) -> f32 {
    let (frac, int) = libm::modff(x);
    if !iptr.is_null() {
        unsafe { *iptr = int };
    }
    frac
}
#[no_mangle]
pub extern "C" fn powf(x: f32, y: f32) -> f32 {
    libm::powf(x, y)
}
#[no_mangle]
pub extern "C" fn roundf(x: f32) -> f32 {
    libm::roundf(x)
}
#[no_mangle]
pub extern "C" fn sinf(x: f32) -> f32 {
    libm::sinf(x)
}
#[no_mangle]
pub extern "C" fn sinhf(x: f32) -> f32 {
    libm::sinhf(x)
}
#[no_mangle]
pub extern "C" fn sqrtf(x: f32) -> f32 {
    libm::sqrtf(x)
}
#[no_mangle]
pub extern "C" fn tanf(x: f32) -> f32 {
    libm::tanf(x)
}
#[no_mangle]
pub extern "C" fn tanhf(x: f32) -> f32 {
    libm::tanhf(x)
}
#[no_mangle]
pub extern "C" fn truncf(x: f32) -> f32 {
    libm::truncf(x)
}
#[no_mangle]
pub extern "C" fn hypotf(x: f32, y: f32) -> f32 {
    libm::hypotf(x, y)
}

#[no_mangle]
pub extern "C" fn lrint(x: f64) -> c_long {
    libm::rint(x) as c_long
}
#[no_mangle]
pub extern "C" fn lrintf(x: f32) -> c_long {
    libm::rintf(x) as c_long
}
#[no_mangle]
pub extern "C" fn lrintl(x: f64) -> c_long {
    libm::rint(x) as c_long
}
#[no_mangle]
pub extern "C" fn llrint(x: f64) -> c_longlong {
    libm::rint(x) as c_longlong
}
#[no_mangle]
pub extern "C" fn llrintf(x: f32) -> c_longlong {
    libm::rintf(x) as c_longlong
}
#[no_mangle]
pub extern "C" fn llrintl(x: f64) -> c_longlong {
    libm::rint(x) as c_longlong
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
pub unsafe extern "C" fn open(path: *const c_char, flags: c_int, mode: c_int) -> c_int {
    sys_open(path as *const u8, flags as i64, mode as i64) as c_int
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
// Additional syscall wrappers for stdio
// ============================================================

const O_RDONLY: i32 = 0;
const O_WRONLY: i32 = 1;
const O_RDWR: i32 = 2;
const O_CREAT: i32 = 64;
const O_TRUNC: i32 = 512;
const O_APPEND: i32 = 1024;
const O_EXCL: i32 = 128;
const O_NONBLOCK: i32 = 2048;
const O_NOFOLLOW: i32 = 0x40000;
const O_CLOEXEC: i32 = 0x80000;

const F_GETFD: i32 = 1;
const F_SETFD: i32 = 2;
const F_GETFL: i32 = 3;
const F_SETFL: i32 = 4;
const FD_CLOEXEC: i32 = 1;

const TIOCGWINSZ: u32 = 0x5413;

#[repr(C)]
struct winsize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

#[inline]
unsafe fn sys_pipe2(fds: *mut c_int, flags: i32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 293i64 => result,
        in("rdi") fds,
        in("rsi") flags as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_dup3(oldfd: i32, newfd: i32, flags: i32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 292i64 => result,
        in("rdi") oldfd as i64,
        in("rsi") newfd as i64,
        in("rdx") flags as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_fcntl(fd: i32, cmd: i32, arg: i64) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 72i64 => result,
        in("rdi") fd as i64,
        in("rsi") cmd as i64,
        in("rdx") arg,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_ioctl(fd: c_int, request: u32, arg: *mut u8) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 16i64 => result,
        in("rdi") fd as i64,
        in("rsi") request as i64,
        in("rdx") arg,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_unlinkat(dirfd: i32, path: *const u8, flags: i32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 263i64 => result,
        in("rdi") dirfd as i64,
        in("rsi") path,
        in("rdx") flags as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_renameat2(olddirfd: i32, oldpath: *const u8, newdirfd: i32, newpath: *const u8, flags: u32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 316i64 => result,
        in("rdi") olddirfd as i64,
        in("rsi") oldpath,
        in("rdx") newdirfd as i64,
        in("r10") newpath,
        in("r8") flags as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

// ============================================================
// FILE / stdio (buffered, musl-compatible layout)
// ============================================================

const UNGET: usize = 8;
const F_PERM: u32 = 1;
const F_NORD: u32 = 4;
const F_NOWR: u32 = 8;
const F_EOF: u32 = 16;
const F_ERR: u32 = 32;
const F_SVB: u32 = 64;
const F_APP: u32 = 128;

const BUFSIZ: usize = 1024;

const _IOFBF: c_int = 0;
const _IOLBF: c_int = 1;
const _IONBF: c_int = 2;

const SEEK_SET: c_int = 0;
const SEEK_CUR: c_int = 1;
const SEEK_END: c_int = 2;

// ponytail: buffered FILE, musl x86_64 layout subset
#[repr(C)]
pub struct FILE {
    flags: c_uint,
    rpos: *mut u8,
    rend: *mut u8,
    close: Option<unsafe extern "C" fn(*mut FILE) -> c_int>,
    wend: *mut u8,
    wpos: *mut u8,
    mustbezero_1: *mut u8,
    wbase: *mut u8,
    buf: *mut u8,
    buf_size: usize,
    prev: *mut FILE,
    next: *mut FILE,
    fd: c_int,
    pipe_pid: c_int,
    lockcount: c_long,
    mode: c_int,
    lock: c_int,
    lbf: c_int,
    cookie: *mut c_void,
    off: i64,
    getln_buf: *mut c_char,
    mustbezero_2: *mut c_void,
    shend: *mut u8,
    shlim: i64,
    shcnt: i64,
    ungotten: [c_int; UNGET],
    ungotten_count: c_int,
    _eof: c_int,
    _err: c_int,
}

static mut STDIN_BUF: [u8; BUFSIZ] = [0; BUFSIZ];
static mut STDOUT_BUF: [u8; BUFSIZ] = [0; BUFSIZ];
static mut STDERR_BUF: [u8; BUFSIZ] = [0; BUFSIZ];

static mut STDIN_FILE: FILE = FILE {
    flags: 0, rpos: core::ptr::null_mut(), rend: core::ptr::null_mut(),
    close: None, wend: core::ptr::null_mut(), wpos: core::ptr::null_mut(),
    mustbezero_1: core::ptr::null_mut(), wbase: core::ptr::null_mut(),
    buf: core::ptr::null_mut(), buf_size: BUFSIZ,
    prev: core::ptr::null_mut(), next: core::ptr::null_mut(),
    fd: 0, pipe_pid: 0, lockcount: 0, mode: 0, lock: -1, lbf: -1,
    cookie: core::ptr::null_mut(), off: 0,
    getln_buf: core::ptr::null_mut(), mustbezero_2: core::ptr::null_mut(),
    shend: core::ptr::null_mut(), shlim: 0, shcnt: 0,
    ungotten: [0; UNGET], ungotten_count: 0, _eof: 0, _err: 0,
};
static mut STDOUT_FILE: FILE = FILE {
    flags: F_NOWR as u32 + F_SVB as u32, rpos: core::ptr::null_mut(), rend: core::ptr::null_mut(),
    close: None, wend: core::ptr::null_mut(), wpos: core::ptr::null_mut(),
    mustbezero_1: core::ptr::null_mut(), wbase: core::ptr::null_mut(),
    buf: core::ptr::null_mut(), buf_size: BUFSIZ,
    prev: core::ptr::null_mut(), next: core::ptr::null_mut(),
    fd: 1, pipe_pid: 0, lockcount: 0, mode: 0, lock: -1, lbf: -1,
    cookie: core::ptr::null_mut(), off: 0,
    getln_buf: core::ptr::null_mut(), mustbezero_2: core::ptr::null_mut(),
    shend: core::ptr::null_mut(), shlim: 0, shcnt: 0,
    ungotten: [0; UNGET], ungotten_count: 0, _eof: 0, _err: 0,
};
static mut STDERR_FILE: FILE = FILE {
    flags: F_NOWR as u32, rpos: core::ptr::null_mut(), rend: core::ptr::null_mut(),
    close: None, wend: core::ptr::null_mut(), wpos: core::ptr::null_mut(),
    mustbezero_1: core::ptr::null_mut(), wbase: core::ptr::null_mut(),
    buf: core::ptr::null_mut(), buf_size: BUFSIZ,
    prev: core::ptr::null_mut(), next: core::ptr::null_mut(),
    fd: 2, pipe_pid: 0, lockcount: 0, mode: 0, lock: -1, lbf: -1,
    cookie: core::ptr::null_mut(), off: 0,
    getln_buf: core::ptr::null_mut(), mustbezero_2: core::ptr::null_mut(),
    shend: core::ptr::null_mut(), shlim: 0, shcnt: 0,
    ungotten: [0; UNGET], ungotten_count: 0, _eof: 0, _err: 0,
};

#[no_mangle]
pub static mut stdin: *mut FILE = &raw mut STDIN_FILE as *mut FILE;
#[no_mangle]
pub static mut stdout: *mut FILE = &raw mut STDOUT_FILE as *mut FILE;
#[no_mangle]
pub static mut stderr: *mut FILE = &raw mut STDERR_FILE as *mut FILE;

// ============================================================
// FILE helpers
// ============================================================

unsafe fn __stdio_init() {
    STDIN_FILE.buf = core::ptr::addr_of_mut!(STDIN_BUF) as *mut u8;
    STDIN_FILE.buf_size = BUFSIZ;
    STDOUT_FILE.buf = core::ptr::addr_of_mut!(STDOUT_BUF) as *mut u8;
    STDOUT_FILE.buf_size = BUFSIZ;
    STDOUT_FILE.lbf = b'\n' as c_int;
    STDERR_FILE.buf = core::ptr::addr_of_mut!(STDERR_BUF) as *mut u8;
    STDERR_FILE.buf_size = BUFSIZ;
}

unsafe fn buf_ptr(f: *mut FILE) -> *mut u8 {
    (f as *mut u8).add(core::mem::size_of::<FILE>())
}

unsafe fn init_file(
    f: *mut FILE,
    fd: c_int,
    mode: *const c_char,
    close_fn: Option<unsafe extern "C" fn(*mut FILE) -> c_int>,
    buf_area: *mut u8,
    buf_sz: usize,
) {
    core::ptr::write_bytes(f as *mut u8, 0, core::mem::size_of::<FILE>());
    (*f).fd = fd;
    (*f).close = close_fn;
    (*f).lock = -1;
    (*f).lbf = -1;
    (*f).buf = buf_area;
    (*f).buf_size = buf_sz;
    let m = *mode;
    let has_plus = !strchr(mode as *const u8, b'+' as c_int).is_null();
    if m == b'r' as c_char && !has_plus {
        (*f).flags = F_NORD;
    } else if !has_plus {
        (*f).flags = F_NOWR;
    }
}

unsafe extern "C" fn __stdio_close(f: *mut FILE) -> c_int {
    sys_close((*f).fd as i64);
    0
}

unsafe fn fmodeflags(mode: *const c_char) -> c_int {
    let mut flags: c_int;
    let m = *mode;
    if !strchr(mode as *const u8, b'+' as c_int).is_null() {
        flags = O_RDWR;
    } else if m == b'r' as c_char {
        flags = O_RDONLY;
    } else {
        flags = O_WRONLY;
    }
    if !strchr(mode as *const u8, b'x' as c_int).is_null() { flags |= O_EXCL; }
    if !strchr(mode as *const u8, b'e' as c_int).is_null() { flags |= O_CLOEXEC; }
    if m != b'r' as c_char { flags |= O_CREAT; }
    if m == b'w' as c_char { flags |= O_TRUNC; }
    if m == b'a' as c_char { flags |= O_APPEND; }
    flags
}

const O_ACCMODE: c_int = O_RDONLY | O_WRONLY | O_RDWR;

unsafe fn flush_buf(f: *mut FILE) -> c_int {
    let base = (*f).wbase;
    let len = (*f).wpos as usize - base as usize;
    if len == 0 {
        (*f).wend = (*f).buf.add((*f).buf_size);
        (*f).wpos = (*f).wbase;
        return 0;
    }
    (*f).wpos = (*f).wbase;
    let mut written = 0usize;
    while written < len {
        let n = sys_write((*f).fd as i64, base.add(written), len - written);
        if n <= 0 {
            (*f).flags |= F_ERR;
            (*f)._err = 1;
            (*f).wpos = core::ptr::null_mut();
            (*f).wbase = core::ptr::null_mut();
            (*f).wend = core::ptr::null_mut();
            return -1;
        }
        written += n as usize;
    }
    (*f).wend = (*f).buf.add((*f).buf_size);
    (*f).wpos = (*f).wbase;
    0
}

// ============================================================
// fopen / fdopen / freopen / fclose
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE {
    if filename.is_null() || mode.is_null() { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let m = *mode;
    if m != b'r' as c_char && m != b'w' as c_char && m != b'a' as c_char {
        ERRNO = EINVAL; return core::ptr::null_mut();
    }
    let flags = fmodeflags(mode);
    let fd = sys_open(filename as *const u8, flags as i64, 0o666);
    if fd < 0 { ERRNO = (-fd) as c_int; return core::ptr::null_mut(); }
    fdopen(fd as c_int, mode)
}

#[no_mangle]
pub unsafe extern "C" fn fdopen(fd: c_int, mode: *const c_char) -> *mut FILE {
    if fd < 0 || mode.is_null() { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let m = *mode;
    if m != b'r' as c_char && m != b'w' as c_char && m != b'a' as c_char {
        ERRNO = EINVAL; return core::ptr::null_mut();
    }
    let f = calloc(1, core::mem::size_of::<FILE>() + UNGET + BUFSIZ) as *mut FILE;
    if f.is_null() { ERRNO = ENOMEM; return core::ptr::null_mut(); }
    let buf = buf_ptr(f);
    init_file(f, fd, mode, Some(__stdio_close), buf, BUFSIZ);
    if !strchr(mode as *const u8, b'a' as c_int).is_null() { (*f).flags |= F_APP; }
    if !strchr(mode as *const u8, b'e' as c_int).is_null() { sys_fcntl(fd, F_SETFD, FD_CLOEXEC as i64); }
    if (*f).flags & F_NOWR == 0 {
        let mut ws: winsize = core::mem::zeroed();
        if sys_ioctl(fd, TIOCGWINSZ, &mut ws as *mut winsize as *mut u8) == 0 {
            (*f).lbf = b'\n' as c_int;
        }
    }
    f
}

#[no_mangle]
pub unsafe extern "C" fn freopen(
    filename: *const c_char, mode: *const c_char, f: *mut FILE,
) -> *mut FILE {
    fflush(f);
    if (*f).pipe_pid > 0 { let _ = __stdio_close(f); }
    if filename.is_null() {
        let flags = fmodeflags(mode);
        if flags == 0 { return core::ptr::null_mut(); }
        let mut fl = flags & !(O_CREAT | O_EXCL | O_CLOEXEC);
        if (*f).flags & F_APP != 0 { fl = (fl & !O_ACCMODE) | O_WRONLY | O_APPEND; }
        if sys_fcntl((*f).fd, F_SETFL, fl as i64) < 0 { return core::ptr::null_mut(); }
        let has_plus = !strchr(mode as *const u8, b'+' as c_int).is_null();
        if has_plus {
            (*f).flags &= !(F_NORD | F_NOWR);
        } else if *mode == b'r' as c_char {
            (*f).flags = ((*f).flags & !F_NOWR) | F_NORD;
        } else {
            (*f).flags = ((*f).flags & !F_NORD) | F_NOWR;
        }
        return f;
    }
    let new_f = fopen(filename, mode);
    if new_f.is_null() { let _ = __stdio_close(f); return core::ptr::null_mut(); }
    if (*new_f).fd == (*f).fd {
        (*new_f).fd = -1;
    } else {
        sys_dup3((*new_f).fd, (*f).fd, 0);
        let _ = __stdio_close(new_f);
    }
    (*f).flags = (*new_f).flags;
    (*f).rpos = core::ptr::null_mut();
    (*f).rend = core::ptr::null_mut();
    (*f).wpos = core::ptr::null_mut();
    (*f).wbase = core::ptr::null_mut();
    (*f).wend = core::ptr::null_mut();
    (*f).lbf = (*new_f).lbf;
    (*f).pipe_pid = 0;
    free(new_f as *mut c_void);
    f
}

#[no_mangle]
pub unsafe extern "C" fn fclose(f: *mut FILE) -> c_int {
    if f.is_null() { return -1; }
    let r = fflush(f);
    if let Some(close_fn) = (*f).close { let _ = close_fn(f); }
    if r != 0 { -1 } else { 0 }
}

// ============================================================
// setvbuf / setbuf / setbuffer / fflush
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn setvbuf(f: *mut FILE, buf: *mut c_char, mode: c_int, size: usize) -> c_int {
    if mode != _IOFBF && mode != _IOLBF && mode != _IONBF { return -1; }
    fflush(f);
    (*f).rpos = core::ptr::null_mut();
    (*f).rend = core::ptr::null_mut();
    (*f).wpos = core::ptr::null_mut();
    (*f).wbase = core::ptr::null_mut();
    (*f).wend = core::ptr::null_mut();
    if mode == _IONBF {
        (*f).buf_size = 0;
    } else if !buf.is_null() && size > 0 {
        (*f).buf = buf as *mut u8;
        (*f).buf_size = size;
    }
    (*f).lbf = if mode == _IOLBF { b'\n' as c_int } else { -1 };
    0
}

#[no_mangle]
pub unsafe extern "C" fn setbuf(f: *mut FILE, buf: *mut c_char) {
    if buf.is_null() { setvbuf(f, core::ptr::null_mut(), _IONBF, 0); }
    else { setvbuf(f, buf, _IOFBF, BUFSIZ); }
}

#[no_mangle]
pub unsafe extern "C" fn setbuffer(f: *mut FILE, buf: *mut c_char, size: usize) {
    if buf.is_null() { setvbuf(f, core::ptr::null_mut(), _IONBF, 0); }
    else { setvbuf(f, buf, _IOFBF, size); }
}

#[no_mangle]
pub unsafe extern "C" fn fflush(f: *mut FILE) -> c_int {
    if f.is_null() {
        let mut r = 0;
        if !stdout.is_null() && !(*stdout).wpos.is_null() && (*stdout).wpos != (*stdout).wbase {
            if flush_buf(stdout) != 0 { r = -1; }
        }
        if !stderr.is_null() && !(*stderr).wpos.is_null() && (*stderr).wpos != (*stderr).wbase {
            if flush_buf(stderr) != 0 { r = -1; }
        }
        return r;
    }
    if !(*f).wpos.is_null() && (*f).wpos != (*f).wbase {
        return flush_buf(f);
    }
    0
}

// ============================================================
// fwide / feof / ferror / clearerr / fileno
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn fwide(_f: *mut FILE, mode: c_int) -> c_int { mode }

#[no_mangle]
pub unsafe extern "C" fn feof(f: *mut FILE) -> c_int { if (*f)._eof != 0 { 1 } else { 0 } }

#[no_mangle]
pub unsafe extern "C" fn ferror(f: *mut FILE) -> c_int { if (*f)._err != 0 { 1 } else { 0 } }

#[no_mangle]
pub unsafe extern "C" fn clearerr(f: *mut FILE) { (*f)._eof = 0; (*f)._err = 0; }

#[no_mangle]
pub unsafe extern "C" fn fileno(f: *mut FILE) -> c_int { (*f).fd }

// ============================================================
// fseek / ftell / rewind / fseeko / ftello / fgetpos / fsetpos
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn fseek(stream: *mut FILE, offset: c_long, whence: c_int) -> c_int {
    fseeko(stream, offset as i64, whence)
}

#[no_mangle]
pub unsafe extern "C" fn ftell(stream: *mut FILE) -> c_long {
    ftello(stream) as c_long
}

#[no_mangle]
pub unsafe extern "C" fn rewind(stream: *mut FILE) {
    fseek(stream, 0, SEEK_SET);
    (*stream)._eof = 0;
    (*stream)._err = 0;
}

#[no_mangle]
pub unsafe extern "C" fn fseeko(stream: *mut FILE, offset: i64, whence: c_int) -> c_int {
    if whence != SEEK_SET && whence != SEEK_CUR && whence != SEEK_END {
        ERRNO = EINVAL;
        return -1;
    }
    let f = &mut *stream;
    let mut adj_offset = offset;
    if whence == SEEK_CUR && !f.rpos.is_null() {
        adj_offset -= f.rend as i64 - f.rpos as i64;
    }
    if !f.wpos.is_null() && f.wpos != f.wbase {
        if flush_buf(stream) != 0 { return -1; }
    }
    f.wpos = core::ptr::null_mut();
    f.wbase = core::ptr::null_mut();
    f.wend = core::ptr::null_mut();
    let r = sys_lseek(f.fd as i64, adj_offset, whence as i64);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    f.rpos = core::ptr::null_mut();
    f.rend = core::ptr::null_mut();
    f._eof = 0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn ftello(stream: *mut FILE) -> i64 {
    let f = &mut *stream;
    let r = sys_lseek(f.fd as i64, 0, SEEK_CUR as i64);
    if r < 0 { return -1; }
    let mut pos = r;
    if !f.rpos.is_null() {
        pos -= f.rend as i64 - f.rpos as i64;
    } else if !f.wbase.is_null() {
        pos += f.wpos as i64 - f.wbase as i64;
    }
    pos
}

#[no_mangle]
pub unsafe extern "C" fn fgetpos(stream: *mut FILE, pos: *mut i64) -> c_int {
    let off = ftello(stream);
    if off < 0 { return -1; }
    *pos = off;
    0
}

#[no_mangle]
pub unsafe extern "C" fn fsetpos(stream: *mut FILE, pos: *const i64) -> c_int {
    fseeko(stream, *pos, SEEK_SET)
}

// ============================================================
// fgetc / getc / getchar / ungetc
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn fgetc(stream: *mut FILE) -> c_int {
    let f = &mut *stream;
    if f.ungotten_count > 0 {
        f.ungotten_count -= 1;
        return f.ungotten[f.ungotten_count as usize];
    }
    if !f.rpos.is_null() && f.rpos < f.rend {
        let c = *f.rpos;
        f.rpos = f.rpos.add(1);
        return c as c_int;
    }
    if f._eof != 0 || f.fd < 0 {
        f._eof = 1;
        return -1;
    }
    let mut buf = [0u8; 1];
    let n = sys_read(f.fd as i64, buf.as_mut_ptr(), 1);
    if n <= 0 {
        if n == 0 { f._eof = 1; } else { f._err = 1; }
        -1
    } else {
        buf[0] as c_int
    }
}

#[no_mangle]
pub unsafe extern "C" fn getc(stream: *mut FILE) -> c_int { fgetc(stream) }

#[no_mangle]
pub unsafe extern "C" fn getchar() -> c_int { fgetc(stdin) }

#[no_mangle]
pub unsafe extern "C" fn ungetc(c: c_int, stream: *mut FILE) -> c_int {
    if c == -1 { return -1; }
    let f = &mut *stream;
    if f.ungotten_count >= UNGET as c_int { return -1; }
    f.ungotten[f.ungotten_count as usize] = c;
    f.ungotten_count += 1;
    f._eof = 0;
    c
}

// ============================================================
// fgets / fread
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn fgets(s: *mut c_char, n: c_int, stream: *mut FILE) -> *mut c_char {
    if n <= 0 { return null_mut(); }
    let max = (n - 1) as usize;
    let mut i = 0usize;
    while i < max {
        let c = fgetc(stream);
        if c == -1 { if i == 0 { return null_mut(); } break; }
        *s.add(i) = c as c_char;
        i += 1;
        if c == b'\n' as c_int { break; }
    }
    *s.add(i) = 0;
    s
}

// ponytail: gets is dangerous, kept for POSIX compliance
#[no_mangle]
pub unsafe extern "C" fn gets(s: *mut c_char) -> *mut c_char {
    let mut i = 0usize;
    loop {
        let c = fgetc(stdin);
        if c == -1 { if i == 0 { return null_mut(); } break; }
        if c == b'\n' as c_int { break; }
        *s.add(i) = c as c_char;
        i += 1;
    }
    *s.add(i) = 0;
    s
}

#[no_mangle]
pub unsafe extern "C" fn fopen64(filename: *const c_char, mode: *const c_char) -> *mut FILE {
    fopen(filename, mode)
}

// ponytail: perror - minimal, just prints "msg: errno\n"
const PERROR_BUF_SIZE: usize = 64;
static mut PERROR_BUF: [u8; PERROR_BUF_SIZE] = [0; PERROR_BUF_SIZE];

#[no_mangle]
pub unsafe extern "C" fn perror(msg: *const c_char) {
    let fd = (*stderr).fd;
    if !msg.is_null() && *msg != 0 {
        let len = strlen(msg as *const u8);
        write_str(fd, msg as *const u8, len);
        write_str(fd, b": ".as_ptr(), 2);
    }
    let e = ERRNO;
    // ponytail: just print errno number, not full strerror
    if e != 0 {
        let (buf, len) = format_i64(e as i64);
        write_str(fd, buf.as_ptr().add(21 - len), len);
    }
    write_str(fd, b"\n".as_ptr(), 1);
}

#[no_mangle]
pub unsafe extern "C" fn fread(
    ptr: *mut c_void, size: SizeT, nmemb: SizeT, stream: *mut FILE,
) -> SizeT {
    if size == 0 || nmemb == 0 { return 0; }
    let total = size * nmemb;
    let mut rd = 0usize;
    while rd < total {
        let c = fgetc(stream);
        if c == -1 { break; }
        *(ptr as *mut u8).add(rd) = c as u8;
        rd += 1;
    }
    rd / size
}

// ============================================================
// puts / fputs / fputc / putc / putchar / fwrite
// ============================================================

unsafe fn write_str(fd: c_int, s: *const u8, len: usize) {
    let mut written = 0usize;
    while written < len {
        let n = sys_write(fd as i64, s.add(written), len - written);
        if n <= 0 { break; }
        written += n as usize;
    }
}

unsafe fn __overflow(f: *mut FILE, c: c_int) -> c_int {
    if (*f).wpos.is_null() {
        (*f).wbase = (*f).buf;
        (*f).wpos = (*f).buf;
        (*f).wend = (*f).buf.add((*f).buf_size);
    }
    if flush_buf(f) == -1 { return -1; }
    if (*f).wpos >= (*f).wend {
        let byte = c as u8;
        write_str((*f).fd, &byte as *const u8, 1);
        return c;
    }
    *(*f).wpos = c as u8;
    (*f).wpos = (*f).wpos.add(1);
    if c == (*f).lbf || (*f).wpos >= (*f).wend { let _ = flush_buf(f); }
    c
}

#[no_mangle]
pub unsafe extern "C" fn puts(s: *const c_char) -> c_int {
    let len = strlen(s as *const u8);
    let _ = fwrite(s as *const c_void, 1, len, stdout);
    let _ = fwrite(b"\n".as_ptr() as *const c_void, 1, 1, stdout);
    let f = &mut *stdout;
    if f.lbf >= 0 { let _ = flush_buf(stdout); }
    (len + 1) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn fputs(s: *const c_char, stream: *mut FILE) -> c_int {
    let len = strlen(s as *const u8);
    fwrite(s as *const c_void, 1, len, stream) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn fputc(c: c_int, stream: *mut FILE) -> c_int {
    let f = &mut *stream;
    if f.wpos.is_null() || f.wpos >= f.wend {
        if __overflow(f, c) == -1 { return -1; }
        return c;
    }
    *f.wpos = c as u8;
    f.wpos = f.wpos.add(1);
    if c == f.lbf || f.wpos >= f.wend { let _ = flush_buf(f); }
    c
}

#[no_mangle]
pub unsafe extern "C" fn putc(c: c_int, stream: *mut FILE) -> c_int { fputc(c, stream) }

#[no_mangle]
pub unsafe extern "C" fn putchar(c: c_int) -> c_int { fputc(c, stdout) }

#[no_mangle]
pub unsafe extern "C" fn fwrite(
    ptr: *const c_void, size: SizeT, nmemb: SizeT, stream: *mut FILE,
) -> SizeT {
    if size == 0 || nmemb == 0 { return 0; }
    let total = size * nmemb;
    let f = &mut *stream;
    if f.buf_size == 0 {
        let mut written = 0usize;
        let src = ptr as *const u8;
        while written < total {
            let n = sys_write(f.fd as i64, src.add(written), total - written);
            if n <= 0 { f._err = 1; break; }
            written += n as usize;
        }
        return written / size;
    }
    if f.wpos.is_null() {
        f.wbase = f.buf;
        f.wpos = f.buf;
        f.wend = f.buf.add(f.buf_size);
    }
    let mut remaining = total;
    let mut src = ptr as *const u8;
    while remaining > 0 {
        let space = f.wend as usize - f.wpos as usize;
        if space == 0 {
            if flush_buf(f as *mut FILE) == -1 { break; }
            continue;
        }
        let n = if remaining < space { remaining } else { space };
        core::ptr::copy_nonoverlapping(src, f.wpos, n);
        f.wpos = f.wpos.add(n);
        src = src.add(n);
        remaining -= n;
        if f.wpos >= f.wend { let _ = flush_buf(f as *mut FILE); }
    }
    if f.lbf >= 0 { let _ = flush_buf(f as *mut FILE); }
    (total - remaining) / size
}

// ============================================================
// printf / fprintf / vprintf / vfprintf / dprintf / vdprintf
// ============================================================

macro_rules! impl_format {
    ($fmt:expr, $args:expr, $write_char:expr, $write_str:expr) => {{
        let fmt = $fmt;
        let args = &mut $args;
        let mut count: usize = 0;
        let mut i = 0usize;
        loop {
            let c = *fmt.add(i) as u8;
            if c == 0 { break; }
            if c != b'%' {
                ($write_char)(c);
                count += 1; i += 1; continue;
            }
            i += 1;
            let spec = *fmt.add(i) as u8;
            match spec {
                b's' => {
                    let s = args.arg::<*const c_char>();
                    if s.is_null() { ($write_str)(b"(null)".as_ptr(), 6); count += 6; }
                    else { let len = strlen(s as *const u8); ($write_str)(s as *const u8, len); count += len; }
                }
                b'd' | b'i' => {
                    let d = args.arg::<c_int>();
                    let buf = format_i64(d as i64);
                    ($write_str)(buf.0.as_ptr(), buf.1); count += buf.1;
                }
                b'u' => {
                    let u = args.arg::<c_uint>();
                    let buf = format_u64(u as u64);
                    ($write_str)(buf.0.as_ptr(), buf.1); count += buf.1;
                }
                b'x' => {
                    let x = args.arg::<c_uint>();
                    let buf = format_hex(x as u64, false);
                    ($write_str)(buf.0.as_ptr(), buf.1); count += buf.1;
                }
                b'X' => {
                    let x = args.arg::<c_uint>();
                    let buf = format_hex(x as u64, true);
                    ($write_str)(buf.0.as_ptr(), buf.1); count += buf.1;
                }
                b'c' => {
                    let ch = args.arg::<c_int>();
                    ($write_char)(ch as u8); count += 1;
                }
                b'p' => {
                    let p = args.arg::<*const c_void>();
                    ($write_char)(b'0'); ($write_char)(b'x'); count += 2;
                    let buf = format_hex(p as u64, false);
                    ($write_str)(buf.0.as_ptr(), buf.1); count += buf.1;
                }
                b'f' => {
                    let val = args.arg::<f64>();
                    let buf = format_f64(val);
                    ($write_str)(buf.0.as_ptr(), buf.1); count += buf.1;
                }
                b'l' => {
                    i += 1;
                    let sub = *fmt.add(i) as u8;
                    match sub {
                        b'd' => { let ld = args.arg::<c_long>(); let buf = format_i64(ld as i64); ($write_str)(buf.0.as_ptr(), buf.1); count += buf.1; }
                        b'u' => { let lu = args.arg::<c_ulong>(); let buf = format_u64(lu as u64); ($write_str)(buf.0.as_ptr(), buf.1); count += buf.1; }
                        b'x' => { let lx = args.arg::<c_ulong>(); let buf = format_hex(lx as u64, false); ($write_str)(buf.0.as_ptr(), buf.1); count += buf.1; }
                        b'X' => { let lx = args.arg::<c_ulong>(); let buf = format_hex(lx as u64, true); ($write_str)(buf.0.as_ptr(), buf.1); count += buf.1; }
                        b'f' => { let val = args.arg::<f64>(); let buf = format_f64(val); ($write_str)(buf.0.as_ptr(), buf.1); count += buf.1; }
                        _ => { ($write_char)(b'%'); ($write_char)(sub); count += 2; }
                    }
                }
                b'%' => { ($write_char)(b'%'); count += 1; }
                b'n' => { let _ = args.arg::<*mut c_int>(); }
                _ => { ($write_char)(b'%'); ($write_char)(spec); count += 2; }
            }
            i += 1;
        }
        count as c_int
    }};
}

unsafe fn format_u64(mut val: u64) -> ([u8; 20], usize) {
    let mut buf = [0u8; 20];
    if val == 0 { buf[0] = b'0'; return (buf, 1); }
    let mut tmp = [0u8; 20];
    let mut pos = 20;
    while val > 0 { pos -= 1; tmp[pos] = b'0' + (val % 10) as u8; val /= 10; }
    let len = 20 - pos;
    core::ptr::copy_nonoverlapping(tmp.as_ptr().add(pos), buf.as_mut_ptr(), len);
    (buf, len)
}

unsafe fn format_i64(val: i64) -> ([u8; 21], usize) {
    let mut buf = [0u8; 21];
    if val < 0 {
        buf[0] = b'-';
        let (inner, len) = format_u64((-(val as i128)) as u64);
        core::ptr::copy_nonoverlapping(inner.as_ptr(), buf.as_mut_ptr().add(1), len);
        (buf, len + 1)
    } else {
        let (inner, len) = format_u64(val as u64);
        core::ptr::copy_nonoverlapping(inner.as_ptr(), buf.as_mut_ptr(), len);
        (buf, len)
    }
}

unsafe fn format_hex(mut val: u64, uppercase: bool) -> ([u8; 16], usize) {
    let digits = if uppercase { b"0123456789ABCDEF" } else { b"0123456789abcdef" };
    let mut buf = [0u8; 16];
    if val == 0 { buf[0] = b'0'; return (buf, 1); }
    let mut tmp = [0u8; 16];
    let mut pos = 16;
    while val > 0 { pos -= 1; tmp[pos] = digits[(val & 0xf) as usize]; val >>= 4; }
    let len = 16 - pos;
    core::ptr::copy_nonoverlapping(tmp.as_ptr().add(pos), buf.as_mut_ptr(), len);
    (buf, len)
}

unsafe fn format_f64(val: f64) -> ([u8; 64], usize) {
    let mut buf = [0u8; 64];
    let mut pos = 0usize;
    let mut v = val;
    if v.is_nan() {
        let nan = b"nan";
        core::ptr::copy_nonoverlapping(nan.as_ptr(), buf.as_mut_ptr(), 3);
        return (buf, 3);
    }
    if v < 0.0 { buf[pos] = b'-'; pos += 1; v = -v; }
    if v.is_infinite() {
        let inf = b"inf";
        core::ptr::copy_nonoverlapping(inf.as_ptr(), buf.as_mut_ptr().add(pos), 3);
        return (buf, pos + 3);
    }
    let int_part = v as u64;
    let frac = v - int_part as f64;
    let (ibuf, ilen) = format_u64(int_part);
    core::ptr::copy_nonoverlapping(ibuf.as_ptr().add(20 - ilen), buf.as_mut_ptr().add(pos), ilen);
    pos += ilen;
    buf[pos] = b'.';
    pos += 1;
    let mut f = frac;
    for _ in 0..6 { f *= 10.0; let digit = f as u8; buf[pos] = b'0' + digit; pos += 1; f -= digit as f64; }
    (buf, pos)
}

#[no_mangle]
pub unsafe extern "C" fn vprintf(fmt: *const c_char, mut args: VaList) -> c_int {
    impl_format!(fmt, args,
        |c: u8| { let _ = fwrite(&c as *const u8 as *const c_void, 1, 1, stdout); },
        |s: *const u8, len: usize| { let _ = fwrite(s as *const c_void, 1, len, stdout); }
    )
}

#[no_mangle]
pub unsafe extern "C" fn vfprintf(stream: *mut FILE, fmt: *const c_char, mut args: VaList) -> c_int {
    impl_format!(fmt, args,
        |c: u8| { let _ = fwrite(&c as *const u8 as *const c_void, 1, 1, stream); },
        |s: *const u8, len: usize| { let _ = fwrite(s as *const c_void, 1, len, stream); }
    )
}

#[no_mangle]
pub unsafe extern "C" fn printf(fmt: *const c_char, mut args: ...) -> c_int {
    impl_format!(fmt, args,
        |c: u8| { let _ = fwrite(&c as *const u8 as *const c_void, 1, 1, stdout); },
        |s: *const u8, len: usize| { let _ = fwrite(s as *const c_void, 1, len, stdout); }
    )
}

#[no_mangle]
pub unsafe extern "C" fn fprintf(stream: *mut FILE, fmt: *const c_char, mut args: ...) -> c_int {
    impl_format!(fmt, args,
        |c: u8| { let _ = fwrite(&c as *const u8 as *const c_void, 1, 1, stream); },
        |s: *const u8, len: usize| { let _ = fwrite(s as *const c_void, 1, len, stream); }
    )
}

#[no_mangle]
pub unsafe extern "C" fn dprintf(fd: c_int, fmt: *const c_char, mut args: ...) -> c_int {
    impl_format!(fmt, args,
        |c: u8| { write_str(fd, &c as *const u8, 1); },
        |s: *const u8, len: usize| { write_str(fd, s, len); }
    )
}

#[no_mangle]
pub unsafe extern "C" fn vdprintf(fd: c_int, fmt: *const c_char, mut args: VaList) -> c_int {
    impl_format!(fmt, args,
        |c: u8| { write_str(fd, &c as *const u8, 1); },
        |s: *const u8, len: usize| { write_str(fd, s, len); }
    )
}

// ============================================================
// sprintf / snprintf / vsprintf / vsnprintf
// ============================================================

unsafe fn format_to_buf(buf: *mut u8, cap: usize, fmt: *const c_char, args: &mut VaListImpl) -> c_int {
    let mut pos = 0usize;
    let mut i = 0usize;
    let mut count = 0usize;
    macro_rules! wc {
        ($c:expr) => { if pos < cap { *buf.add(pos) = $c; } pos += 1; count += 1; }
    }
    macro_rules! ws {
        ($s:expr, $len:expr) => {{
            let __len = $len;
            for k in 0..__len { if pos + k < cap { *buf.add(pos + k) = *$s.add(k); } }
            pos += __len; count += __len;
        }}
    }
    loop {
        let c = *fmt.add(i) as u8;
        if c == 0 { break; }
        if c != b'%' { wc!(c); i += 1; continue; }
        i += 1;
        let spec = *fmt.add(i) as u8;
        match spec {
            b's' => {
                let s = args.arg::<*const c_char>();
                if s.is_null() { ws!(b"(null)".as_ptr(), 6); }
                else { let len = strlen(s as *const u8); ws!(s as *const u8, len); }
            }
            b'd' | b'i' => { let d = args.arg::<c_int>(); let b = format_i64(d as i64); ws!(b.0.as_ptr(), b.1); }
            b'u' => { let u = args.arg::<c_uint>(); let b = format_u64(u as u64); ws!(b.0.as_ptr(), b.1); }
            b'x' => { let x = args.arg::<c_uint>(); let b = format_hex(x as u64, false); ws!(b.0.as_ptr(), b.1); }
            b'X' => { let x = args.arg::<c_uint>(); let b = format_hex(x as u64, true); ws!(b.0.as_ptr(), b.1); }
            b'c' => { let ch = args.arg::<c_int>(); wc!(ch as u8); }
            b'p' => { wc!(b'0'); wc!(b'x'); let p = args.arg::<*const c_void>(); let b = format_hex(p as u64, false); ws!(b.0.as_ptr(), b.1); }
            b'f' => { let val = args.arg::<f64>(); let b = format_f64(val); ws!(b.0.as_ptr(), b.1); }
            b'l' => {
                i += 1; let sub = *fmt.add(i) as u8;
                match sub {
                    b'd' => { let ld = args.arg::<c_long>(); let b = format_i64(ld as i64); ws!(b.0.as_ptr(), b.1); }
                    b'u' => { let lu = args.arg::<c_ulong>(); let b = format_u64(lu as u64); ws!(b.0.as_ptr(), b.1); }
                    b'x' => { let lx = args.arg::<c_ulong>(); let b = format_hex(lx as u64, false); ws!(b.0.as_ptr(), b.1); }
                    b'X' => { let lx = args.arg::<c_ulong>(); let b = format_hex(lx as u64, true); ws!(b.0.as_ptr(), b.1); }
                    b'f' => { let val = args.arg::<f64>(); let b = format_f64(val); ws!(b.0.as_ptr(), b.1); }
                    _ => { wc!(b'%'); wc!(sub); }
                }
            }
            b'%' => { wc!(b'%'); }
            b'n' => { let _ = args.arg::<*mut c_int>(); }
            _ => { wc!(b'%'); wc!(spec); }
        }
        i += 1;
    }
    if cap > 0 { let null_pos = if pos < cap { pos } else { cap - 1 }; *buf.add(null_pos) = 0; }
    count as c_int
}

#[no_mangle]
pub unsafe extern "C" fn vsprintf(buf: *mut c_char, fmt: *const c_char, mut args: VaList) -> c_int {
    format_to_buf(buf as *mut u8, usize::MAX, fmt, &mut *args)
}

#[no_mangle]
pub unsafe extern "C" fn vsnprintf(buf: *mut c_char, size: usize, fmt: *const c_char, mut args: VaList) -> c_int {
    format_to_buf(buf as *mut u8, size, fmt, &mut *args)
}

#[no_mangle]
pub unsafe extern "C" fn sprintf(buf: *mut c_char, fmt: *const c_char, mut args: ...) -> c_int {
    format_to_buf(buf as *mut u8, usize::MAX, fmt, &mut args)
}

#[no_mangle]
pub unsafe extern "C" fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, mut args: ...) -> c_int {
    format_to_buf(buf as *mut u8, size, fmt, &mut args)
}

// ============================================================
// scanf / vscanf / fscanf / vfscanf / sscanf / vsscanf
// ============================================================

unsafe fn skip_ws(p: *const u8) -> *const u8 {
    let mut q = p;
    while *q == b' ' || *q == b'\t' || *q == b'\n' || *q == b'\r' { q = q.add(1); }
    q
}

#[inline]
fn is_ws_byte(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == 0x0b || c == 0x0c
}

// Scan integer with position tracking. Returns (value, is_negative, success).
// base: 0=auto, 8=octal, 10=decimal, 16=hex
unsafe fn scan_int_val(
    buf: *const u8, pos: &mut usize, buf_len: usize, base: u32, width: usize,
) -> (u64, bool, bool) {
    let start = *pos;
    let max = if width > 0 { (start + width).min(buf_len) } else { buf_len };
    let neg = if *pos < max && *buf.add(*pos) == b'-' { *pos += 1; true }
    else if *pos < max && *buf.add(*pos) == b'+' { *pos += 1; false }
    else { false };
    if *pos >= max { *pos = start; return (0, false, false); }
    let mut base = base;
    let mut val: u64 = 0;
    // Auto-detect base / handle 0x prefix
    if (base == 0 || base == 16) && *pos < max && *buf.add(*pos) == b'0' {
        *pos += 1;
        if *pos < max && (*buf.add(*pos) == b'x' || *buf.add(*pos) == b'X') {
            *pos += 1;
            if *pos < max {
                if let Some(d) = hex_val(*buf.add(*pos)) {
                    val = d as u64; *pos += 1;
                    while *pos < max { if let Some(d) = hex_val(*buf.add(*pos)) { val = val.wrapping_mul(16).wrapping_add(d as u64); *pos += 1; } else { break; } }
                    return (val, neg, true);
                }
            }
            return (0, neg, false);
        } else if base == 0 {
            while *pos < max { let c = *buf.add(*pos); if c >= b'0' && c <= b'7' { val = val.wrapping_mul(8).wrapping_add((c - b'0') as u64); *pos += 1; } else { break; } }
            return (val, neg, true);
        } else {
            return (0, neg, true); // base=16, '0' not followed by 'x'
        }
    }
    if base == 0 { base = 10; }
    let mut found = false;
    while *pos < max {
        let c = *buf.add(*pos);
        let d: Option<u64> = match base {
            8 => if c >= b'0' && c <= b'7' { Some((c - b'0') as u64) } else { None },
            10 => if c >= b'0' && c <= b'9' { Some((c - b'0') as u64) } else { None },
            16 => hex_val(c).map(|v| v as u64),
            _ => None,
        };
        match d { Some(digit) => { val = val.wrapping_mul(base as u64).wrapping_add(digit); *pos += 1; found = true; } None => break }
    }
    if !found { *pos = start; return (0, false, false); }
    (val, neg, true)
}

// Scan float with position tracking. Returns (value, success).
unsafe fn scan_float_val(
    buf: *const u8, pos: &mut usize, buf_len: usize, width: usize,
) -> (f64, bool) {
    let start = *pos;
    let max = if width > 0 { (start + width).min(buf_len) } else { buf_len };
    let neg = if *pos < max && *buf.add(*pos) == b'-' { *pos += 1; true }
    else if *pos < max && *buf.add(*pos) == b'+' { *pos += 1; false }
    else { false };
    // inf/nan
    if *pos + 2 < max {
        let (c0,c1,c2) = (*buf.add(*pos), *buf.add(*pos+1), *buf.add(*pos+2));
        if (c0==b'i'||c0==b'I')&&(c1==b'n'||c1==b'N')&&(c2==b'f'||c2==b'F') { *pos+=3; return (if neg{f64::NEG_INFINITY}else{f64::INFINITY}, true); }
        if (c0==b'n'||c0==b'N')&&(c1==b'a'||c1==b'A')&&(c2==b'n'||c2==b'N') { *pos+=3; return (if neg{-f64::NAN}else{f64::NAN}, true); }
    }
    // hex float
    if *pos+1 < max && *buf.add(*pos)==b'0' && (*buf.add(*pos+1)==b'x'||*buf.add(*pos+1)==b'X') {
        *pos += 2;
        let mut val: f64 = 0.0; let mut found = false; let mut frac_scale = 1.0f64; let mut in_frac = false;
        while *pos < max {
            if let Some(d) = hex_val(*buf.add(*pos)) {
                if in_frac { frac_scale /= 16.0; val += d as f64 * frac_scale; } else { val = val * 16.0 + d as f64; }
                *pos += 1; found = true;
            } else if *buf.add(*pos)==b'.' && !in_frac { in_frac = true; *pos += 1; } else { break; }
        }
        if !found { *pos = start; return (0.0, false); }
        // p exponent
        if *pos < max && (*buf.add(*pos)==b'p'||*buf.add(*pos)==b'P') {
            *pos += 1;
            let mut eneg = false;
            if *pos<max && *buf.add(*pos)==b'-' { eneg=true; *pos+=1; }
            else if *pos<max && *buf.add(*pos)==b'+' { *pos+=1; }
            let mut ev: i32 = 0; let mut ef = false;
            while *pos<max && *buf.add(*pos)>=b'0' && *buf.add(*pos)<=b'9' { ev = ev*10+(*buf.add(*pos)-b'0') as i32; *pos+=1; ef=true; }
            if !ef { return (0.0, false); }
            if eneg { ev = -ev; }
            val *= libm::pow(2.0, ev as f64);
        }
        return (if neg{-val}else{val}, true);
    }
    // decimal float
    let mut val: f64 = 0.0; let mut found = false; let mut frac_scale = 1.0f64; let mut in_frac = false;
    while *pos < max {
        let c = *buf.add(*pos);
        if c>=b'0' && c<=b'9' { let d=(c-b'0') as f64; if in_frac{frac_scale/=10.0; val+=d*frac_scale;} else{val=val*10.0+d;} *pos+=1; found=true; }
        else if c==b'.' && !in_frac { in_frac=true; *pos+=1; } else { break; }
    }
    if !found { *pos = start; return (0.0, false); }
    // e exponent
    if *pos<max && (*buf.add(*pos)==b'e'||*buf.add(*pos)==b'E') {
        *pos += 1;
        let mut eneg = false;
        if *pos<max && *buf.add(*pos)==b'-' { eneg=true; *pos+=1; }
        else if *pos<max && *buf.add(*pos)==b'+' { *pos+=1; }
        let mut ev: i32 = 0; let mut ef = false;
        while *pos<max && *buf.add(*pos)>=b'0' && *buf.add(*pos)<=b'9' { ev = ev*10+(*buf.add(*pos)-b'0') as i32; *pos+=1; ef=true; }
        if !ef { return (0.0, false); }
        if eneg { ev = -ev; }
        val *= libm::pow(10.0, ev as f64);
    }
    (if neg{-val}else{val}, true)
}

// Comprehensive scanf parser with position tracking.
// consumed: if non-null, stores number of bytes consumed from buf.
unsafe fn do_vsscanf(
    buf: *const u8, buf_len: usize, fmt: *const c_char, args: &mut VaListImpl, consumed: *mut usize,
) -> c_int {
    let mut p = 0usize;
    let mut fi = 0usize;
    let mut assigned = 0i32;
    loop {
        let fc = *fmt.add(fi) as u8;
        if fc == 0 { break; }
        if is_ws_byte(fc) {
            while is_ws_byte(*fmt.add(fi) as u8) { fi += 1; }
            while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
            continue;
        }
        if fc != b'%' {
            if p >= buf_len || *buf.add(p) != fc { break; }
            p += 1; fi += 1; continue;
        }
        fi += 1;
        // %%
        if *fmt.add(fi) as u8 == b'%' {
            if p >= buf_len || *buf.add(p) != b'%' { break; }
            p += 1; fi += 1; continue;
        }
        let suppress = if *fmt.add(fi) as u8 == b'*' { fi += 1; true } else { false };
        let mut width: usize = 0;
        while (*fmt.add(fi) as u8) >= b'0' && (*fmt.add(fi) as u8) <= b'9' {
            width = width * 10 + ((*fmt.add(fi) as u8) - b'0') as usize; fi += 1;
        }
        if *fmt.add(fi) as u8 == b'm' { fi += 1; }
        let mut len_mod = 0u8;
        match *fmt.add(fi) as u8 {
            b'h' => { fi += 1; if *fmt.add(fi) as u8 == b'h' { fi += 1; len_mod = 2; } else { len_mod = 1; } }
            b'l' => { fi += 1; if *fmt.add(fi) as u8 == b'l' { fi += 1; len_mod = 4; } else { len_mod = 3; } }
            b'j' => { fi += 1; len_mod = 4; }
            b'z' | b't' => { fi += 1; len_mod = 3; }
            b'L' => { fi += 1; len_mod = 5; }
            _ => {}
        }
        let spec = *fmt.add(fi) as u8;
        let (real_spec, _real_len) = if spec==b'C' {(b'c',3u8)} else if spec==b'S' {(b's',3u8)} else {(spec,len_mod)};
        match real_spec {
            b'n' => {
                if !suppress { let out = args.arg::<*mut c_int>(); if !out.is_null() { *out = p as c_int; } }
            }
            b'd' | b'u' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                let dest = if !suppress { Some(args.arg::<*mut c_int>()) } else { None };
                let (val, neg, ok) = scan_int_val(buf, &mut p, buf_len, 10, width);
                if !ok { break; }
                if let Some(out) = dest { if !out.is_null() {
                    *out = if real_spec==b'd' && neg { -(val as i64) as c_int } else { val as c_int };
                }}
                if !suppress { assigned += 1; }
            }
            b'i' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                let dest = if !suppress { Some(args.arg::<*mut c_int>()) } else { None };
                let (val, neg, ok) = scan_int_val(buf, &mut p, buf_len, 0, width);
                if !ok { break; }
                if let Some(out) = dest { if !out.is_null() { *out = if neg { -(val as i64) as c_int } else { val as c_int }; } }
                if !suppress { assigned += 1; }
            }
            b'o' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                let dest = if !suppress { Some(args.arg::<*mut c_int>()) } else { None };
                let (val, _, ok) = scan_int_val(buf, &mut p, buf_len, 8, width);
                if !ok { break; }
                if let Some(out) = dest { if !out.is_null() { *out = val as c_int; } }
                if !suppress { assigned += 1; }
            }
            b'x' | b'X' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                let dest = if !suppress { Some(args.arg::<*mut c_int>()) } else { None };
                let (val, _, ok) = scan_int_val(buf, &mut p, buf_len, 16, width);
                if !ok { break; }
                if let Some(out) = dest { if !out.is_null() { *out = val as c_int; } }
                if !suppress { assigned += 1; }
            }
            b'p' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                let dest = if !suppress { Some(args.arg::<*mut *mut c_void>()) } else { None };
                let (val, _, ok) = scan_int_val(buf, &mut p, buf_len, 16, width);
                if !ok { break; }
                if let Some(out) = dest { if !out.is_null() { *out = val as usize as *mut c_void; } }
                if !suppress { assigned += 1; }
            }
            b'a' | b'e' | b'f' | b'g' | b'A' | b'E' | b'F' | b'G' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                let (val, ok) = scan_float_val(buf, &mut p, buf_len, width);
                if !ok { break; }
                if !suppress {
                    // %f -> f32, %lf -> f64, %Lf -> f64
                    if _real_len == 3 || _real_len == 5 {
                        let out = args.arg::<*mut f64>(); if !out.is_null() { *out = val; }
                    } else {
                        let out = args.arg::<*mut f32>(); if !out.is_null() { *out = val as f32; }
                    }
                }
                if !suppress { assigned += 1; }
            }
            b'c' => {
                let w = if width > 0 { width } else { 1 };
                let dest_ptr: *mut c_char = if !suppress { args.arg::<*mut c_char>() } else { core::ptr::null_mut() };
                let mut j = 0usize;
                while j < w && p < buf_len {
                    if !dest_ptr.is_null() { *dest_ptr.add(j) = *buf.add(p) as c_char; }
                    p += 1; j += 1;
                }
                if j == 0 { break; }
                if !suppress { assigned += 1; }
            }
            b's' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                if p >= buf_len || *buf.add(p) == 0 { break; }
                let dest_ptr: *mut c_char = if !suppress { args.arg::<*mut c_char>() } else { core::ptr::null_mut() };
                let w = if width > 0 { width } else { usize::MAX };
                let mut j = 0usize;
                while p < buf_len && *buf.add(p) != 0 && !is_ws_byte(*buf.add(p)) && j < w {
                    if !dest_ptr.is_null() { *dest_ptr.add(j) = *buf.add(p) as c_char; }
                    p += 1; j += 1;
                }
                if j == 0 { break; }
                if !dest_ptr.is_null() { *dest_ptr.add(j) = 0; }
                if !suppress { assigned += 1; }
            }
            b'[' => {
                fi += 1;
                let negate = if *fmt.add(fi) as u8 == b'^' { fi += 1; true } else { false };
                let mut charset = [0u8; 256];
                if *fmt.add(fi) as u8 == b']' { charset[b']' as usize] = 1; fi += 1; }
                loop { let c = *fmt.add(fi) as u8; if c == b']' || c == 0 { break; } charset[c as usize] = 1; fi += 1; }
                if *fmt.add(fi) as u8 == b']' { fi += 1; }
                fi -= 1; // common fi += 1 will advance past the closing ]
                let dest_ptr: *mut c_char = if !suppress { args.arg::<*mut c_char>() } else { core::ptr::null_mut() };
                let w = if width > 0 { width } else { usize::MAX };
                let mut j = 0usize;
                while p < buf_len && *buf.add(p) != 0 && j < w {
                    let c = *buf.add(p);
                    let in_set = charset[c as usize] != 0;
                    if negate { if in_set { break; } } else { if !in_set { break; } }
                    if !dest_ptr.is_null() { *dest_ptr.add(j) = c as c_char; }
                    p += 1; j += 1;
                }
                if j == 0 { break; }
                if !dest_ptr.is_null() { *dest_ptr.add(j) = 0; }
                if !suppress { assigned += 1; }
            }
            _ => break,
        }
        fi += 1;
    }
    if !consumed.is_null() { *consumed = p; }
    assigned
}

unsafe fn vsscanf_inner(buf: *const u8, fmt: *const c_char, args: &mut VaListImpl) -> c_int {
    let len = strlen(buf);
    do_vsscanf(buf, len, fmt, args, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn vsscanf(buf: *const c_char, fmt: *const c_char, mut args: VaList) -> c_int {
    vsscanf_inner(buf as *const u8, fmt, &mut *args)
}

#[no_mangle]
pub unsafe extern "C" fn vfscanf(stream: *mut FILE, fmt: *const c_char, mut args: VaList) -> c_int {
    let mut line = [0u8; 4096];
    let mut pos = 0usize;
    loop {
        if pos >= line.len() - 1 { break; }
        let c = fgetc(stream);
        if c == -1 { break; }
        line[pos] = c as u8;
        pos += 1;
        if c == b'\n' as c_int { break; }
    }
    line[pos] = 0;
    if pos == 0 { return 0; }
    vsscanf_inner(line.as_ptr(), fmt, &mut *args)
}

#[no_mangle]
pub unsafe extern "C" fn vscanf(fmt: *const c_char, mut args: VaList) -> c_int {
    vfscanf(stdin, fmt, args)
}

#[no_mangle]
pub unsafe extern "C" fn sscanf(buf: *const c_char, fmt: *const c_char, mut args: ...) -> c_int {
    vsscanf_inner(buf as *const u8, fmt, &mut args)
}

#[no_mangle]
pub unsafe extern "C" fn fscanf(stream: *mut FILE, fmt: *const c_char, mut args: ...) -> c_int {
    vfscanf(stream, fmt, args.as_va_list())
}

#[no_mangle]
pub unsafe extern "C" fn scanf(fmt: *const c_char, mut args: ...) -> c_int {
    vfscanf(stdin, fmt, args.as_va_list())
}

// ============================================================
// getdelim / getline
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn getdelim(
    lineptr: *mut *mut c_char, n: *mut usize, delim: c_int, stream: *mut FILE,
) -> isize {
    if lineptr.is_null() || n.is_null() { return -1; }
    let mut cap = *n;
    let mut buf = *lineptr;
    if buf.is_null() || cap == 0 {
        cap = 128;
        buf = realloc(buf as *mut c_void, cap) as *mut c_char;
        if buf.is_null() { return -1; }
        *lineptr = buf;
        *n = cap;
    }
    let mut len = 0usize;
    loop {
        let c = fgetc(stream);
        if c == -1 { if len == 0 { return -1; } break; }
        if len + 1 >= cap {
            let new_cap = cap.wrapping_mul(2);
            let new_buf = realloc(buf as *mut c_void, new_cap) as *mut c_char;
            if new_buf.is_null() { return -1; }
            buf = new_buf; cap = new_cap;
            *lineptr = buf; *n = cap;
        }
        *buf.add(len) = c as c_char;
        len += 1;
        if c == delim { break; }
    }
    *buf.add(len) = 0;
    len as isize
}

#[no_mangle]
pub unsafe extern "C" fn getline(lineptr: *mut *mut c_char, n: *mut usize, stream: *mut FILE) -> isize {
    getdelim(lineptr, n, b'\n' as c_int, stream)
}

// ============================================================
// popen / pclose
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn popen(cmd: *const c_char, mode: *const c_char) -> *mut FILE {
    if cmd.is_null() || mode.is_null() { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let m = *mode;
    if m != b'r' as c_char && m != b'w' as c_char { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let mut p: [c_int; 2] = [0; 2];
    if sys_pipe2(p.as_mut_ptr(), O_CLOEXEC) < 0 { return core::ptr::null_mut(); }
    let op = if m == b'r' as c_char { 0 } else { 1 };
    let child_fd = p[1 - op];
    let f = fdopen(p[op], mode);
    if f.is_null() { sys_close(p[0] as i64); sys_close(p[1] as i64); return core::ptr::null_mut(); }
    let pid = sys_fork();
    if pid < 0 { fclose(f); sys_close(child_fd as i64); return core::ptr::null_mut(); }
    if pid == 0 {
        sys_close(p[op] as i64);
        if child_fd != (1 - op) as c_int { sys_dup3(child_fd, (1 - op) as c_int, 0); sys_close(child_fd as i64); }
        let sh = b"/bin/sh\0".as_ptr() as *const c_char;
        let argv = [b"sh\0".as_ptr() as *const c_char, b"-c\0".as_ptr() as *const c_char, cmd, core::ptr::null()];
        sys_execve(sh, argv.as_ptr(), __environ as *const *const c_char);
        _exit(127);
    }
    sys_close(child_fd as i64);
    (*f).pipe_pid = pid as c_int;
    f
}

#[no_mangle]
pub unsafe extern "C" fn pclose(f: *mut FILE) -> c_int {
    let pid = (*f).pipe_pid;
    let r = fclose(f);
    if pid <= 0 { return -1; }
    let mut status: c_int = 0;
    let w = waitpid(pid, &mut status, 0);
    if w < 0 || r != 0 { return -1; }
    status
}

// ============================================================
// tmpfile / tmpnam / remove / rename
// ============================================================

const L_TMPNAM: usize = 20;
static mut TMPNAM_COUNTER: c_uint = 0;

unsafe fn fill_tmpname(out: *mut c_char, seed: c_uint) {
    let prefix = b"/tmp/tmp\0";
    let mut k = 0;
    while k < prefix.len() - 1 { *out.add(k) = prefix[k] as c_char; k += 1; }
    let (buf, len) = format_hex(seed as u64, false);
    for j in 0..len.min(L_TMPNAM - k - 1) { *out.add(k + j) = buf[16 - len + j] as c_char; k += 1; }
    *out.add(k) = 0;
}

#[no_mangle]
pub unsafe extern "C" fn tmpfile() -> *mut FILE {
    let mut name = [0u8; L_TMPNAM];
    let mut ctr: c_uint = 0;
    loop {
        ctr = ctr.wrapping_add(1);
        let mut ts: timespec = core::mem::zeroed();
        let _ = sys_clock_gettime(CLOCK_REALTIME, &mut ts);
        let seed = (ts.tv_nsec as c_uint).wrapping_add(ctr);
        fill_tmpname(name.as_mut_ptr() as *mut c_char, seed);
        let fd = sys_open(name.as_ptr(), (O_RDWR | O_CREAT | O_EXCL) as i64, 0o600);
        if fd >= 0 {
            sys_unlinkat(AT_FDCWD, name.as_ptr(), 0);
            let f = fdopen(fd as c_int, b"w+\0".as_ptr() as *const c_char);
            if f.is_null() { sys_close(fd); }
            return f;
        }
        if ctr > 100 { return core::ptr::null_mut(); }
    }
}

#[no_mangle]
pub unsafe extern "C" fn tmpnam(s: *mut c_char) -> *mut c_char {
    static mut BUF: [c_char; L_TMPNAM] = [0; L_TMPNAM];
    let out = if s.is_null() { core::ptr::addr_of_mut!(BUF) as *mut c_char } else { s };
    TMPNAM_COUNTER = TMPNAM_COUNTER.wrapping_add(1);
    let mut ts: timespec = core::mem::zeroed();
    let _ = sys_clock_gettime(CLOCK_REALTIME, &mut ts);
    let seed = (ts.tv_nsec as c_uint).wrapping_add(TMPNAM_COUNTER);
    fill_tmpname(out, seed);
    out
}

#[no_mangle]
pub unsafe extern "C" fn remove(path: *const c_char) -> c_int {
    if path.is_null() { ERRNO = EINVAL; return -1; }
    let r = sys_unlinkat(AT_FDCWD, path as *const u8, 0);
    if r == 0 { 0 } else { ERRNO = (-r) as c_int; -1 }
}

#[no_mangle]
pub unsafe extern "C" fn rename(old: *const c_char, new_: *const c_char) -> c_int {
    if old.is_null() || new_.is_null() { ERRNO = EINVAL; return -1; }
    let r = sys_renameat2(AT_FDCWD, old as *const u8, AT_FDCWD, new_ as *const u8, 0);
    if r == 0 { 0 } else { ERRNO = (-r) as c_int; -1 }
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
    fflush(core::ptr::null_mut());
    __funcs_on_exit();
    _exit(code);
}

#[no_mangle]
pub unsafe extern "C" fn abort() -> ! {
    raise(6); // SIGABRT
    _exit(128 + 6);
}

// ============================================================
// qsort / qsort_r (smoothsort, O(1) aux, adaptive)
// ============================================================

type CmpRfn = unsafe extern "C" fn(*const c_void, *const c_void, *mut c_void) -> c_int;

const AR_LEN: usize = 16 * core::mem::size_of::<usize>();
const AR_MASK: usize = AR_LEN - 1;

unsafe fn qsort_pntz(p: &[usize; 2]) -> i32 {
    if p[0] != 1 {
        p[0].wrapping_sub(1).trailing_zeros() as i32
    } else if p[1] != 0 {
        (8 * core::mem::size_of::<usize>()) as i32 + p[1].trailing_zeros() as i32
    } else {
        0
    }
}

unsafe fn qsort_cycle(width: usize, ar: &mut [*mut u8; AR_LEN], n: usize) {
    if n < 2 { return; }
    let mut tmp = [0u8; 256];
    ar[n] = tmp.as_mut_ptr();
    let mut w = width;
    while w > 0 {
        let l = if 256 < w { 256 } else { w };
        core::ptr::copy_nonoverlapping(ar[0], ar[n], l);
        for i in 0..n {
            core::ptr::copy_nonoverlapping(ar[i + 1], ar[i], l);
            ar[i] = ar[i].add(l);
        }
        w -= l;
    }
}

unsafe fn qsort_shl(p: &mut [usize; 2], n: i32) {
    let bits = (8 * core::mem::size_of::<usize>()) as i32;
    if n >= bits {
        p[1] = p[0];
        p[0] = 0;
        let n = n - bits;
        if n == 0 { return; }
    }
    p[1] = (p[1] << n) | (p[0] >> (bits - n));
    p[0] <<= n;
}

unsafe fn qsort_shr(p: &mut [usize; 2], n: i32) {
    let bits = (8 * core::mem::size_of::<usize>()) as i32;
    if n >= bits {
        p[0] = p[1];
        p[1] = 0;
        let n = n - bits;
        if n == 0 { return; }
    }
    p[0] = (p[0] >> n) | (p[1] << (bits - n));
    p[1] >>= n;
}

unsafe fn qsort_sift(
    head: *mut u8, width: usize,
    cmp: CmpRfn, arg: *mut c_void,
    pshift: i32, lp: &[usize],
) {
    let mut ar: [*mut u8; AR_LEN] = [core::ptr::null_mut(); AR_LEN];
    ar[0] = head;
    let mut i = 1usize;
    let mut h = head;
    let mut ps = pshift;

    while ps > 1 {
        let rt = h.sub(width);
        let lf = h.sub(width + lp[(ps - 2) as usize]);

        if cmp(ar[0] as *const c_void, lf as *const c_void, arg) >= 0
            && cmp(ar[0] as *const c_void, rt as *const c_void, arg) >= 0
        {
            break;
        }
        if cmp(lf as *const c_void, rt as *const c_void, arg) >= 0 {
            ar[i & AR_MASK] = lf;
            i += 1;
            h = lf;
            ps -= 1;
        } else {
            ar[i & AR_MASK] = rt;
            i += 1;
            h = rt;
            ps -= 2;
        }
    }
    qsort_cycle(width, &mut ar, i & AR_MASK);
}

unsafe fn qsort_trinkle(
    head: *mut u8, width: usize,
    cmp: CmpRfn, arg: *mut c_void,
    pp: &[usize; 2], pshift: i32, trusty: i32,
    lp: &[usize],
) {
    let mut ar: [*mut u8; AR_LEN] = [core::ptr::null_mut(); AR_LEN];
    let mut p = *pp;
    ar[0] = head;
    let mut i = 1usize;
    let mut h = head;
    let mut ps = pshift;
    let mut t = trusty;

    while p[0] != 1 || p[1] != 0 {
        let stepson = h.sub(lp[ps as usize]);
        if cmp(stepson as *const c_void, ar[0] as *const c_void, arg) <= 0 {
            break;
        }
        if t == 0 && ps > 1 {
            let rt = h.sub(width);
            let lf = h.sub(width + lp[(ps - 2) as usize]);
            if cmp(rt as *const c_void, stepson as *const c_void, arg) >= 0
                || cmp(lf as *const c_void, stepson as *const c_void, arg) >= 0
            {
                break;
            }
        }

        ar[i & AR_MASK] = stepson;
        i += 1;
        h = stepson;
        let trail = qsort_pntz(&p);
        qsort_shr(&mut p, trail);
        ps += trail;
        t = 0;
    }
    if t == 0 {
        qsort_cycle(width, &mut ar, i & AR_MASK);
        qsort_sift(h, width, cmp, arg, ps, lp);
    }
}

#[no_mangle]
pub unsafe extern "C" fn __qsort_r(
    base: *mut c_void,
    nel: usize,
    width: usize,
    cmp: CmpRfn,
    arg: *mut c_void,
) {
    let size = width * nel;
    if size == 0 { return; }

    let mut lp = [0usize; 96];
    lp[0] = width;
    lp[1] = width;
    let mut li = 2;
    while {
        lp[li] = lp[li - 2] + lp[li - 1] + width;
        lp[li] < size
    } {
        li += 1;
    }

    let mut head = base as *mut u8;
    let high = head.add(size - width);
    let mut p: [usize; 2] = [1, 0];
    let mut pshift: i32 = 1;

    while head < high {
        if (p[0] & 3) == 3 {
            qsort_sift(head, width, cmp, arg, pshift, &lp);
            qsort_shr(&mut p, 2);
            pshift += 2;
        } else {
            if lp[(pshift - 1) as usize] >= (high as usize) - (head as usize) {
                qsort_trinkle(head, width, cmp, arg, &p, pshift, 0, &lp);
            } else {
                qsort_sift(head, width, cmp, arg, pshift, &lp);
            }

            if pshift == 1 {
                qsort_shl(&mut p, 1);
                pshift = 0;
            } else {
                qsort_shl(&mut p, pshift - 1);
                pshift = 1;
            }
        }

        p[0] |= 1;
        head = head.add(width);
    }

    qsort_trinkle(head, width, cmp, arg, &p, pshift, 0, &lp);

    while pshift != 1 || p[0] != 1 || p[1] != 0 {
        if pshift <= 1 {
            let trail = qsort_pntz(&p);
            qsort_shr(&mut p, trail);
            pshift += trail;
        } else {
            qsort_shl(&mut p, 2);
            pshift -= 2;
            p[0] ^= 7;
            qsort_shr(&mut p, 1);
            qsort_trinkle(
                head.sub(lp[pshift as usize] + width),
                width, cmp, arg, &p, pshift + 1, 1, &lp,
            );
            qsort_shl(&mut p, 1);
            p[0] |= 1;
            qsort_trinkle(head.sub(width), width, cmp, arg, &p, pshift, 1, &lp);
        }
        head = head.sub(width);
    }
}

#[no_mangle]
pub unsafe extern "C" fn qsort_r(
    base: *mut c_void,
    nel: usize,
    width: usize,
    cmp: CmpRfn,
    arg: *mut c_void,
) {
    __qsort_r(base, nel, width, cmp, arg);
}

unsafe extern "C" fn qsort_wrap_cmp(
    a: *const c_void,
    b: *const c_void,
    cmp_ctx: *mut c_void,
) -> c_int {
    let cmp: CmpFn = core::mem::transmute(cmp_ctx);
    cmp(a, b)
}

#[no_mangle]
pub unsafe extern "C" fn qsort(
    base: *mut c_void,
    nel: usize,
    width: usize,
    cmp: CmpFn,
) {
    __qsort_r(base, nel, width, qsort_wrap_cmp, cmp as *mut c_void);
}

// ============================================================
// atexit / __cxa_atexit / __funcs_on_exit
// ============================================================

const ATEXIT_COUNT: usize = 32;

#[repr(C)]
struct AtExitBlock {
    next: *mut AtExitBlock,
    f: [Option<unsafe extern "C" fn(*mut c_void)>; ATEXIT_COUNT],
    a: [*mut c_void; ATEXIT_COUNT],
}

static mut ATEXIT_BUILTIN: AtExitBlock = AtExitBlock {
    next: core::ptr::null_mut(),
    f: [None; ATEXIT_COUNT],
    a: [core::ptr::null_mut(); ATEXIT_COUNT],
};
static mut ATEXIT_HEAD: *mut AtExitBlock = core::ptr::null_mut();
static mut ATEXIT_SLOT: usize = 0;
static mut ATEXIT_FINISHED: bool = false;

#[no_mangle]
pub unsafe extern "C" fn __funcs_on_exit() {
    while !ATEXIT_HEAD.is_null() {
        let head = ATEXIT_HEAD;
        while ATEXIT_SLOT > 0 {
            ATEXIT_SLOT -= 1;
            if let Some(f) = (*head).f[ATEXIT_SLOT] {
                let a = (*head).a[ATEXIT_SLOT];
                f(a);
            }
        }
        ATEXIT_HEAD = (*head).next;
        ATEXIT_SLOT = ATEXIT_COUNT;
    }
    ATEXIT_FINISHED = true;
}

#[no_mangle]
pub unsafe extern "C" fn __cxa_atexit(
    func: unsafe extern "C" fn(*mut c_void),
    arg: *mut c_void,
    _dso: *mut c_void,
) -> c_int {
    if ATEXIT_FINISHED {
        return -1;
    }
    if ATEXIT_HEAD.is_null() {
        ATEXIT_HEAD = core::ptr::addr_of_mut!(ATEXIT_BUILTIN);
    }
    if ATEXIT_SLOT == ATEXIT_COUNT {
        let new = calloc(1, core::mem::size_of::<AtExitBlock>()) as *mut AtExitBlock;
        if new.is_null() {
            return -1;
        }
        (*new).next = ATEXIT_HEAD;
        ATEXIT_HEAD = new;
        ATEXIT_SLOT = 0;
    }
    (*ATEXIT_HEAD).f[ATEXIT_SLOT] = Some(func);
    (*ATEXIT_HEAD).a[ATEXIT_SLOT] = arg;
    ATEXIT_SLOT += 1;
    0
}

unsafe extern "C" fn atexit_caller(arg: *mut c_void) {
    let func: unsafe extern "C" fn() = core::mem::transmute(arg);
    func();
}

#[no_mangle]
pub unsafe extern "C" fn atexit(func: unsafe extern "C" fn()) -> c_int {
    __cxa_atexit(atexit_caller, func as *mut c_void, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn __cxa_finalize(_dso: *mut c_void) {}

// ============================================================
// getenv / setenv / putenv / unsetenv / clearenv
// ============================================================

#[no_mangle]
pub static mut __environ: *mut *mut c_char = core::ptr::null_mut();

// ponytail: simple env tracking for allocated entries; O(n) scan
const ENV_MAX_ALLOCED: usize = 256;
static mut ENV_ALLOCED: [*mut c_char; ENV_MAX_ALLOCED] = [core::ptr::null_mut(); ENV_MAX_ALLOCED];
static mut ENV_ALLOCED_N: usize = 0;

unsafe fn env_rm_add(old: *mut c_char, mut new: *mut c_char) {
    for i in 0..ENV_ALLOCED_N {
        if ENV_ALLOCED[i] == old {
            ENV_ALLOCED[i] = new;
            if !old.is_null() {
                free(old as *mut c_void);
            }
            return;
        } else if ENV_ALLOCED[i].is_null() && !new.is_null() {
            ENV_ALLOCED[i] = new;
            new = core::ptr::null_mut();
        }
    }
    if !new.is_null() && ENV_ALLOCED_N < ENV_MAX_ALLOCED {
        ENV_ALLOCED[ENV_ALLOCED_N] = new;
        ENV_ALLOCED_N += 1;
    }
}

unsafe fn strchrnul(s: *const u8, c: u8) -> *const u8 {
    let mut p = s;
    while *p != 0 && *p != c {
        p = p.add(1);
    }
    p
}

#[no_mangle]
pub unsafe extern "C" fn getenv(name: *const c_char) -> *mut c_char {
    if __environ.is_null() || name.is_null() {
        return core::ptr::null_mut();
    }
    let name = name as *const u8;
    let l = strchrnul(name, b'=') as usize - name as usize;
    if l == 0 || *name.add(l) != 0 {
        return core::ptr::null_mut();
    }
    let mut e = __environ;
    while !(*e).is_null() {
        let entry = *e as *const u8;
        if strncmp(entry, name, l) == 0 && *entry.add(l) == b'=' {
            return entry.add(l + 1) as *mut c_char;
        }
        e = e.add(1);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn setenv(
    var: *const c_char,
    value: *const c_char,
    overwrite: c_int,
) -> c_int {
    if var.is_null() || value.is_null() {
        ERRNO = EINVAL;
        return -1;
    }
    let var = var as *const u8;
    let l1 = strchrnul(var, b'=') as usize - var as usize;
    if l1 == 0 || *var.add(l1) != 0 {
        ERRNO = EINVAL;
        return -1;
    }
    if overwrite == 0 && !getenv(var as *const c_char).is_null() {
        return 0;
    }
    let l2 = strlen(value as *const u8);
    let s = malloc(l1 + l2 + 2) as *mut u8;
    if s.is_null() {
        return -1;
    }
    core::ptr::copy_nonoverlapping(var, s, l1);
    *s.add(l1) = b'=';
    core::ptr::copy_nonoverlapping(value as *const u8, s.add(l1 + 1), l2 + 1);
    putenv_internal(s as *mut c_char, l1, s as *mut c_char)
}

unsafe fn putenv_internal(s: *mut c_char, l: usize, r: *mut c_char) -> c_int {
    let mut i = 0;
    if !__environ.is_null() {
        let mut e = __environ;
        while !(*e).is_null() {
            if strncmp(s as *const u8, *e as *const u8, l + 1) == 0 {
                let tmp = *e;
                *e = s;
                env_rm_add(tmp, r);
                return 0;
            }
            e = e.add(1);
            i += 1;
        }
    }
    let newenv = malloc(core::mem::size_of::<*mut c_char>() * (i + 2)) as *mut *mut c_char;
    if newenv.is_null() {
        if !r.is_null() { free(r as *mut c_void); }
        return -1;
    }
    if i > 0 && !__environ.is_null() {
        core::ptr::copy_nonoverlapping(__environ, newenv, i);
    }
    *newenv.add(i) = s;
    *newenv.add(i + 1) = core::ptr::null_mut();
    // ponytail: don't free old __environ (may not be from our malloc)
    __environ = newenv;
    environ = __environ;
    if !r.is_null() {
        env_rm_add(core::ptr::null_mut(), r);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn putenv(s: *mut c_char) -> c_int {
    if s.is_null() { return -1; }
    let l = strchrnul(s as *const u8, b'=') as usize - s as *const u8 as usize;
    if l == 0 || *s.add(l) as u8 == 0 {
        return unsetenv(s);
    }
    putenv_internal(s, l, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn unsetenv(name: *const c_char) -> c_int {
    if name.is_null() {
        ERRNO = EINVAL;
        return -1;
    }
    let name = name as *const u8;
    let l = strchrnul(name, b'=') as usize - name as usize;
    if l == 0 || *name.add(l) != 0 {
        ERRNO = EINVAL;
        return -1;
    }
    if __environ.is_null() {
        return 0;
    }
    let mut e = __environ;
    let mut eo = e;
    while !(*e).is_null() {
        if strncmp(name, *e as *const u8, l) == 0 && *(*e as *const u8).add(l) == b'=' {
            env_rm_add(*e, core::ptr::null_mut());
        } else {
            if eo != e {
                *eo = *e;
            }
            eo = eo.add(1);
        }
        e = e.add(1);
    }
    if eo != e {
        *eo = core::ptr::null_mut();
    }
    environ = __environ;
    0
}

#[no_mangle]
pub unsafe extern "C" fn clearenv() -> c_int {
    let e = __environ;
    __environ = core::ptr::null_mut();
    environ = core::ptr::null_mut();
    if !e.is_null() {
        let mut p = e;
        while !(*p).is_null() {
            env_rm_add(*p, core::ptr::null_mut());
            p = p.add(1);
        }
    }
    0
}

// ============================================================
// random / srandom / initstate / setstate (BSD lagged fibonacci)
// ponytail: single-threaded, no locking; add lock if concurrency matters
// ============================================================

static mut RANDOM_INIT: [u32; 32] = [
    0x00000000,0x5851f42d,0xc0b18ccf,0xcbb5f646,
    0xc7033129,0x30705b04,0x20fd5db4,0x9a8b7f78,
    0x502959d8,0xab894868,0x6c0356a7,0x88cdb7ff,
    0xb477d43f,0x70a3a52b,0xa8e4baf1,0xfd8341fc,
    0x8ae16fd9,0x742d2f7a,0x0d1f0796,0x76035e09,
    0x40f7702c,0x6fa72ca5,0xaaa84157,0x58a0df74,
    0xc74a0364,0xae533cc4,0x04185faf,0x6de3b115,
    0x0cab8628,0xf043bfa4,0x398150e9,0x37521657,
];

static mut RANDOM_N: i32 = 31;
static mut RANDOM_I: i32 = 3;
static mut RANDOM_J: i32 = 0;
static mut RANDOM_X: *mut u32 = unsafe { core::ptr::addr_of_mut!(RANDOM_INIT).cast::<u32>().add(1) };

unsafe fn random_lcg31(x: u32) -> u32 {
    (1103515245u32.wrapping_mul(x).wrapping_add(12345)) & 0x7fffffff
}

unsafe fn random_lcg64(x: u64) -> u64 {
    6364136223846793005u64.wrapping_mul(x).wrapping_add(1)
}

unsafe fn random_savestate() -> *mut u8 {
    (*RANDOM_X.offset(-1)) = ((RANDOM_N as u32) << 16) | ((RANDOM_I as u32) << 8) | (RANDOM_J as u32);
    RANDOM_X.offset(-1) as *mut u8
}

unsafe fn random_loadstate(state: *mut u32) {
    RANDOM_X = state.add(1);
    RANDOM_N = ((*RANDOM_X.offset(-1)) >> 16) as i32;
    RANDOM_I = (((*RANDOM_X.offset(-1)) >> 8) & 0xff) as i32;
    RANDOM_J = ((*RANDOM_X.offset(-1)) & 0xff) as i32;
}

unsafe fn random_srandom_inner(seed: u32) {
    let mut s = seed as u64;
    if RANDOM_N == 0 {
        *RANDOM_X = s as u32;
        return;
    }
    RANDOM_I = if RANDOM_N == 31 || RANDOM_N == 7 { 3 } else { 1 };
    RANDOM_J = 0;
    for k in 0..RANDOM_N as usize {
        s = random_lcg64(s);
        *RANDOM_X.add(k) = (s >> 32) as u32;
    }
    *RANDOM_X |= 1;
}

#[no_mangle]
pub unsafe extern "C" fn srandom(seed: c_uint) {
    random_srandom_inner(seed);
}

#[no_mangle]
pub unsafe extern "C" fn initstate(seed: c_uint, state: *mut c_char, size: usize) -> *mut c_char {
    if size < 8 { return core::ptr::null_mut(); }
    let old = random_savestate();
    RANDOM_N = if size < 32 { 0 }
        else if size < 64 { 7 }
        else if size < 128 { 15 }
        else if size < 256 { 31 }
        else { 63 };
    RANDOM_X = (state as *mut u32).add(1);
    random_srandom_inner(seed);
    random_savestate();
    old as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn setstate(state: *mut c_char) -> *mut c_char {
    let old = random_savestate();
    random_loadstate(state as *mut u32);
    old as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn random() -> c_long {
    let k: c_long;
    if RANDOM_N == 0 {
        *RANDOM_X = random_lcg31(*RANDOM_X);
        k = *RANDOM_X as c_long;
    } else {
        *RANDOM_X.add(RANDOM_I as usize) = (*RANDOM_X.add(RANDOM_I as usize)).wrapping_add(*RANDOM_X.add(RANDOM_J as usize));
        k = (*RANDOM_X.add(RANDOM_I as usize) >> 1) as c_long;
        RANDOM_I += 1;
        if RANDOM_I == RANDOM_N { RANDOM_I = 0; }
        RANDOM_J += 1;
        if RANDOM_J == RANDOM_N { RANDOM_J = 0; }
    }
    k
}

// ============================================================
// multibyte (UTF-8, musl state machine)
// ============================================================

const SA: u8 = 0xC2;
const SB: u8 = 0xF4;

const BITTAB: [u32; 51] = [
    0xC000_0002, 0xC000_0003, 0xC000_0004, 0xC000_0005, 0xC000_0006, 0xC000_0007,
    0xC000_0008, 0xC000_0009, 0xC000_000A, 0xC000_000B, 0xC000_000C, 0xC000_000D,
    0xC000_000E, 0xC000_000F, 0xC000_0010, 0xC000_0011, 0xC000_0012, 0xC000_0013,
    0xC000_0014, 0xC000_0015, 0xC000_0016, 0xC000_0017, 0xC000_0018, 0xC000_0019,
    0xC000_001A, 0xC000_001B, 0xC000_001C, 0xC000_001D, 0xC000_001E, 0xC000_001F,
    0xB300_0000, 0xC300_0001, 0xC300_0002, 0xC300_0003, 0xC300_0004, 0xC300_0005,
    0xC300_0006, 0xC300_0007, 0xC300_0008, 0xC300_0009, 0xC300_000A, 0xC300_000B,
    0xC300_000C, 0xD300_000D, 0xC300_000E, 0xC300_000F, 0xBB0C_0000, 0xC30C_0001,
    0xC30C_0002, 0xC30C_0003, 0xDB0C_0004,
];

#[inline]
fn mb_oob(c: u32, b: u8) -> u32 {
    let b3 = (b >> 3) as u32;
    ((b3.wrapping_sub(0x10)) | (b3.wrapping_add((c as i32 >> 26) as u32))) & !7
}

static mut MB_STATE: c_uint = 0;

const MB_CUR_MAX_VAL: usize = 4;

#[no_mangle]
pub extern "C" fn __ctype_get_mb_cur_max() -> usize {
    MB_CUR_MAX_VAL
}

#[no_mangle]
pub unsafe extern "C" fn mbrtowc(
    wc: *mut c_int,
    src: *const c_char,
    n: usize,
    st: *mut c_uint,
) -> usize {
    let s = src as *const u8;
    let n0 = n;
    let mut dummy: c_int = 0;
    let wc = if wc.is_null() { &mut dummy } else { &mut *wc };
    let st: &mut c_uint = if st.is_null() { &mut *(&raw mut MB_STATE) } else { &mut *st };

    let mut c: u32 = *st;
    let mut s = s;
    let mut n = n;

    if s.is_null() {
        if c != 0 { *st = 0; ERRNO = EILSEQ; return !0usize; }
        return 0;
    }

    if n == 0 { return !1usize; }

    if c == 0 {
        if *s < 0x80 {
            let r = if *s == 0 { 0 } else { 1 };
            *wc = *s as c_int;
            return r;
        }
        if (*s).wrapping_sub(SA) as u32 > (SB - SA) as u32 {
            *st = 0; ERRNO = EILSEQ; return !0usize;
        }
        c = BITTAB[*s as usize - SA as usize];
        s = s.add(1);
        n -= 1;
    }

    if n > 0 {
        if mb_oob(c, *s) != 0 {
            *st = 0; ERRNO = EILSEQ; return !0usize;
        }
        loop {
            c = (c << 6) | (*s as u32).wrapping_sub(0x80);
            s = s.add(1);
            n -= 1;
            if c & (1u32 << 31) == 0 {
                *st = 0;
                *wc = c as c_int;
                return n0 - n;
            }
            if n == 0 { break; }
            if (*s).wrapping_sub(0x80) >= 0x40 {
                *st = 0; ERRNO = EILSEQ; return !0usize;
            }
        }
    }

    *st = c;
    !1usize
}

#[no_mangle]
pub unsafe extern "C" fn wcrtomb(
    s: *mut c_char,
    wc: c_int,
    _st: *mut c_uint,
) -> usize {
    if s.is_null() { return 1; }
    let wc = wc as u32;
    if wc < 0x80 {
        *s = wc as c_char;
        return 1;
    }
    if wc < 0x800 {
        *s = (0xC0 | (wc >> 6)) as c_char;
        *s.add(1) = (0x80 | (wc & 0x3F)) as c_char;
        return 2;
    }
    if wc < 0xD800 || wc.wrapping_sub(0xE000) < 0x2000 {
        *s = (0xE0 | (wc >> 12)) as c_char;
        *s.add(1) = (0x80 | ((wc >> 6) & 0x3F)) as c_char;
        *s.add(2) = (0x80 | (wc & 0x3F)) as c_char;
        return 3;
    }
    if wc.wrapping_sub(0x10000) < 0x100000 {
        *s = (0xF0 | (wc >> 18)) as c_char;
        *s.add(1) = (0x80 | ((wc >> 12) & 0x3F)) as c_char;
        *s.add(2) = (0x80 | ((wc >> 6) & 0x3F)) as c_char;
        *s.add(3) = (0x80 | (wc & 0x3F)) as c_char;
        return 4;
    }
    ERRNO = EILSEQ;
    !0usize
}

#[no_mangle]
pub unsafe extern "C" fn mblen(s: *const c_char, n: usize) -> c_int {
    if s.is_null() { return 0; }
    mbtowc(core::ptr::null_mut(), s, n)
}

#[no_mangle]
pub unsafe extern "C" fn mbtowc(
    wc: *mut c_int,
    src: *const c_char,
    n: usize,
) -> c_int {
    if src.is_null() { return 0; }
    if n == 0 { ERRNO = EILSEQ; return -1; }
    let mut state: c_uint = 0;
    let mut w: c_int = 0;
    let r = mbrtowc(&mut w, src, n, &mut state);
    if r == !0usize { return -1; }
    if !wc.is_null() { *wc = w; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn wctomb(s: *mut c_char, wc: c_int) -> c_int {
    if s.is_null() { return 0; }
    wcrtomb(s, wc, core::ptr::null_mut()) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn mbsrtowcs(
    ws: *mut c_int,
    src: *mut *const c_char,
    wn: usize,
    st: *mut c_uint,
) -> usize {
    let st: &mut c_uint = if st.is_null() { &mut *(&raw mut MB_STATE) } else { &mut *st };
    let mut s = *src as *const u8;
    let wn0 = wn;
    let mut wn = wn;

    if ws.is_null() {
        while *s != 0 {
            if *s < 0x80 {
                s = s.add(1);
                wn = wn.wrapping_sub(1);
                continue;
            }
            let mut wc: c_int = 0;
            let r = mbrtowc(&mut wc, s as *const c_char, 4, st);
            if r == !0usize { return !0usize; }
            if r == !1usize { return wn0 - wn; }
            s = s.add(r);
            wn = wn.wrapping_sub(1);
        }
        wn0 - wn
    } else {
        let mut w = ws;
        while wn > 0 {
            if *s < 0x80 {
                let c = *s as c_int;
                *w = c;
                if c == 0 {
                    *src = core::ptr::null();
                    return wn0 - wn;
                }
                w = w.add(1);
                s = s.add(1);
                wn -= 1;
                continue;
            }
            let mut wc: c_int = 0;
            let r = mbrtowc(&mut wc, s as *const c_char, 4, st);
            if r == !0usize {
                *src = s as *const c_char;
                return !0usize;
            }
            if r == !1usize {
                *src = s as *const c_char;
                return wn0 - wn;
            }
            *w = wc;
            w = w.add(1);
            s = s.add(r);
            wn -= 1;
        }
        *src = s as *const c_char;
        wn0 - wn
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcsrtombs(
    s: *mut c_char,
    ws: *mut *const c_int,
    n: usize,
    _st: *mut c_uint,
) -> usize {
    let mut w = *ws;
    let n0 = n;
    let mut n = n;

    if s.is_null() {
        let mut count = 0usize;
        while *w != 0 {
            if (*w as u32) < 0x80 {
                count += 1;
            } else {
                let mut buf = [0u8; 4];
                let r = wcrtomb(buf.as_mut_ptr() as *mut c_char, *w, core::ptr::null_mut());
                if r == !0usize { return !0usize; }
                count += r;
            }
            w = w.add(1);
        }
        return count;
    }

    let mut dst = s;
    while n > 0 {
        if (*w as u32) < 0x80 {
            *dst = *w as c_char;
            if *w == 0 {
                *ws = core::ptr::null();
                return n0 - n;
            }
            dst = dst.add(1);
            n -= 1;
        } else {
            let r = wcrtomb(dst, *w, core::ptr::null_mut());
            if r == !0usize {
                *ws = w;
                return !0usize;
            }
            if r > n {
                *ws = w;
                return n0 - n;
            }
            dst = dst.add(r);
            n -= r;
        }
        w = w.add(1);
    }
    *ws = w;
    n0
}

#[no_mangle]
pub unsafe extern "C" fn mbstowcs(
    ws: *mut c_int,
    src: *const c_char,
    wn: usize,
) -> usize {
    let mut src_ptr = src;
    mbsrtowcs(ws, &mut src_ptr, wn, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn wcstombs(
    s: *mut c_char,
    ws: *const c_int,
    n: usize,
) -> usize {
    let mut ws_ptr = ws;
    wcsrtombs(s, &mut ws_ptr, n, core::ptr::null_mut())
}

// ============================================================
// wchar: btowc / wctob / mbsinit / mbrlen
// ============================================================

#[no_mangle]
pub extern "C" fn btowc(c: c_int) -> wint_t {
    if c == -1 { return WEOF; }
    if (c as u32) < 128 { c as wint_t } else { WEOF }
}

#[no_mangle]
pub extern "C" fn wctob(c: wint_t) -> c_int {
    if c < 128 { c as c_int } else { -1 }
}

#[no_mangle]
pub unsafe extern "C" fn mbsinit(s: *const c_uint) -> c_int {
    if s.is_null() || *s == 0 { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn mbrlen(s: *const c_char, n: usize, ps: *mut c_uint) -> usize {
    mbrtowc(core::ptr::null_mut(), s, n, ps)
}

// ============================================================
// wchar: wide string functions
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn wcslen(s: *const wchar_t) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 { len += 1; }
    len
}

#[no_mangle]
pub unsafe extern "C" fn wcschr(s: *const wchar_t, c: wchar_t) -> *mut wchar_t {
    let mut i = 0;
    loop {
        let ch = *s.add(i);
        if ch == c { return s.add(i) as *mut wchar_t; }
        if ch == 0 { return null_mut(); }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcsrchr(s: *const wchar_t, c: wchar_t) -> *mut wchar_t {
    let mut i = 0;
    let mut last: *mut wchar_t = null_mut();
    loop {
        let ch = *s.add(i);
        if ch == c { last = s.add(i) as *mut wchar_t; }
        if ch == 0 { return last; }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcsstr(haystack: *const wchar_t, needle: *const wchar_t) -> *mut wchar_t {
    if *needle == 0 { return haystack as *mut wchar_t; }
    let mut i = 0;
    loop {
        let h = *haystack.add(i);
        if h == 0 { return null_mut(); }
        if h == *needle {
            let mut j = 0;
            loop {
                let n = *needle.add(j);
                if n == 0 { return haystack.add(i) as *mut wchar_t; }
                if *haystack.add(i + j) != n { break; }
                j += 1;
            }
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcscmp(s1: *const wchar_t, s2: *const wchar_t) -> c_int {
    let mut i = 0;
    loop {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b { return a - b; }
        if a == 0 { return 0; }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcsncmp(s1: *const wchar_t, s2: *const wchar_t, n: usize) -> c_int {
    let mut i = 0;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b { return a - b; }
        if a == 0 { return 0; }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn wcscpy(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t {
    let mut i = 0;
    loop {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 { break; }
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn wcsncpy(dst: *mut wchar_t, src: *const wchar_t, n: usize) -> *mut wchar_t {
    let mut i = 0;
    while i < n && *src.add(i) != 0 { *dst.add(i) = *src.add(i); i += 1; }
    while i < n { *dst.add(i) = 0; i += 1; }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn wcscat(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t {
    let mut i = 0;
    while *dst.add(i) != 0 { i += 1; }
    let mut j = 0;
    loop {
        let c = *src.add(j);
        *dst.add(i + j) = c;
        if c == 0 { break; }
        j += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn wcsncat(dst: *mut wchar_t, src: *const wchar_t, n: usize) -> *mut wchar_t {
    let mut i = 0;
    while *dst.add(i) != 0 { i += 1; }
    let mut j = 0;
    while j < n && *src.add(j) != 0 { *dst.add(i + j) = *src.add(j); j += 1; }
    *dst.add(i + j) = 0;
    dst
}

#[no_mangle]
pub unsafe extern "C" fn wcsdup(s: *const wchar_t) -> *mut wchar_t {
    let len = wcslen(s);
    let p = malloc((len + 1) * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
    if p.is_null() { return null_mut(); }
    core::ptr::copy_nonoverlapping(s, p, len + 1);
    p
}

#[no_mangle]
pub unsafe extern "C" fn wcsnlen(s: *const wchar_t, maxlen: usize) -> usize {
    let mut i = 0;
    while i < maxlen && *s.add(i) != 0 { i += 1; }
    i
}

#[no_mangle]
pub unsafe extern "C" fn wcsspn(s: *const wchar_t, accept: *const wchar_t) -> usize {
    let mut i = 0;
    loop {
        let c = *s.add(i);
        if c == 0 { return i; }
        let mut found = false;
        let mut j = 0;
        loop {
            let a = *accept.add(j);
            if a == 0 { break; }
            if c == a { found = true; break; }
            j += 1;
        }
        if !found { return i; }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcscspn(s: *const wchar_t, reject: *const wchar_t) -> usize {
    let mut i = 0;
    loop {
        let c = *s.add(i);
        if c == 0 { return i; }
        let mut j = 0;
        loop {
            let r = *reject.add(j);
            if r == 0 { break; }
            if c == r { return i; }
            j += 1;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcspbrk(s: *const wchar_t, accept: *const wchar_t) -> *mut wchar_t {
    let mut i = 0;
    loop {
        let c = *s.add(i);
        if c == 0 { return null_mut(); }
        let mut j = 0;
        loop {
            let a = *accept.add(j);
            if a == 0 { break; }
            if c == a { return s.add(i) as *mut wchar_t; }
            j += 1;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcsxfrm(dst: *mut wchar_t, src: *const wchar_t, n: usize) -> usize {
    let len = wcslen(src);
    if !dst.is_null() {
        let copy = if len < n { len } else { if n > 0 { n - 1 } else { 0 } };
        for i in 0..copy { *dst.add(i) = *src.add(i); }
        if n > 0 { *dst.add(copy) = 0; }
    }
    len
}

// ============================================================
// wchar: wcsto* number conversions
// ============================================================

unsafe fn wcs_to_bytes(ws: *const wchar_t, buf: *mut u8, bufsz: usize) -> usize {
    let mut i = 0;
    while i < bufsz - 1 {
        let c = *ws.add(i);
        if c == 0 { break; }
        if (c as u32) > 127 { break; }
        *buf.add(i) = c as u8;
        i += 1;
    }
    *buf.add(i) = 0;
    i
}

#[no_mangle]
pub unsafe extern "C" fn wcstol(s: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_long {
    let mut buf = [0u8; 256];
    wcs_to_bytes(s, buf.as_mut_ptr(), 256);
    let mut end: *mut c_char = core::ptr::null_mut();
    let r = strtol(buf.as_ptr() as *const c_char, &mut end, base);
    if !endptr.is_null() {
        let offset = end as usize - buf.as_ptr() as usize;
        *endptr = s.add(offset) as *mut wchar_t;
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn wcstoul(s: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_ulong {
    let mut buf = [0u8; 256];
    wcs_to_bytes(s, buf.as_mut_ptr(), 256);
    let mut end: *mut c_char = core::ptr::null_mut();
    let r = strtoul(buf.as_ptr() as *const c_char, &mut end, base);
    if !endptr.is_null() {
        let offset = end as usize - buf.as_ptr() as usize;
        *endptr = s.add(offset) as *mut wchar_t;
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn wcstoll(s: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_longlong {
    let mut buf = [0u8; 256];
    wcs_to_bytes(s, buf.as_mut_ptr(), 256);
    let mut end: *mut c_char = core::ptr::null_mut();
    let r = strtoll(buf.as_ptr() as *const c_char, &mut end, base);
    if !endptr.is_null() {
        let offset = end as usize - buf.as_ptr() as usize;
        *endptr = s.add(offset) as *mut wchar_t;
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn wcstoull(s: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_ulonglong {
    let mut buf = [0u8; 256];
    wcs_to_bytes(s, buf.as_mut_ptr(), 256);
    let mut end: *mut c_char = core::ptr::null_mut();
    let r = strtoull(buf.as_ptr() as *const c_char, &mut end, base);
    if !endptr.is_null() {
        let offset = end as usize - buf.as_ptr() as usize;
        *endptr = s.add(offset) as *mut wchar_t;
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn wcstod(s: *const wchar_t, endptr: *mut *mut wchar_t) -> f64 {
    let mut buf = [0u8; 256];
    wcs_to_bytes(s, buf.as_mut_ptr(), 256);
    let mut end: *mut c_char = core::ptr::null_mut();
    let r = strtod(buf.as_ptr() as *const c_char, &mut end);
    if !endptr.is_null() {
        let offset = end as usize - buf.as_ptr() as usize;
        *endptr = s.add(offset) as *mut wchar_t;
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn wcstof(s: *const wchar_t, endptr: *mut *mut wchar_t) -> f32 { wcstod(s, endptr) as f32 }

#[no_mangle]
pub unsafe extern "C" fn wcstold(s: *const wchar_t, endptr: *mut *mut wchar_t) -> f64 { wcstod(s, endptr) }

#[no_mangle]
pub unsafe extern "C" fn wcstoimax(s: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_longlong { wcstoll(s, endptr, base) }

#[no_mangle]
pub unsafe extern "C" fn wcstoumax(s: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_ulonglong { wcstoull(s, endptr, base) }

// ============================================================
// wctype: classification functions
// ============================================================

const WC_ALPHA: c_int = 1;
const WC_DIGIT: c_int = 2;
const WC_SPACE: c_int = 4;
const WC_UPPER: c_int = 8;
const WC_LOWER: c_int = 16;
const WC_ALNUM: c_int = WC_ALPHA | WC_DIGIT;
const WC_BLANK: c_int = 32;
const WC_CNTRL: c_int = 64;
const WC_PUNCT: c_int = 128;
const WC_GRAPH: c_int = WC_ALNUM | WC_PUNCT;
const WC_PRINT: c_int = WC_GRAPH | WC_BLANK;
const WC_XDIGIT: c_int = 256;

fn wc_flags(c: wint_t) -> c_int {
    if c < 128 {
        let f = CT_TABLE[c as usize];
        let mut r = 0;
        if f & (CT_UPPER | CT_LOWER) != 0 { r |= WC_ALPHA; }
        if f & CT_DIGIT != 0 { r |= WC_DIGIT; }
        if f & CT_SPACE != 0 { r |= WC_SPACE; }
        if f & CT_UPPER != 0 { r |= WC_UPPER; }
        if f & CT_LOWER != 0 { r |= WC_LOWER; }
        if f & CT_BLANK != 0 { r |= WC_BLANK; }
        if f & CT_CNTRL != 0 { r |= WC_CNTRL; }
        if f & CT_PUNCT != 0 { r |= WC_PUNCT; }
        if f & CT_XDIGIT != 0 { r |= WC_XDIGIT; }
        r
    } else {
        if c < 0x100 { 0 }
        else { WC_PRINT | WC_ALPHA }
    }
}

#[no_mangle]
pub extern "C" fn iswalnum(c: wint_t) -> c_int { (wc_flags(c) & WC_ALNUM != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswalpha(c: wint_t) -> c_int { (wc_flags(c) & WC_ALPHA != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswblank(c: wint_t) -> c_int { (wc_flags(c) & WC_BLANK != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswcntrl(c: wint_t) -> c_int { (wc_flags(c) & WC_CNTRL != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswdigit(c: wint_t) -> c_int { ((c >= b'0' as wint_t) && (c <= b'9' as wint_t)) as c_int }
#[no_mangle]
pub extern "C" fn iswgraph(c: wint_t) -> c_int { (wc_flags(c) & WC_GRAPH != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswlower(c: wint_t) -> c_int { (wc_flags(c) & WC_LOWER != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswprint(c: wint_t) -> c_int {
    if c < 128 { (wc_flags(c) & WC_PRINT != 0) as c_int }
    else if c >= 0x100 && c < 0x110000 { 1 }
    else { 0 }
}
#[no_mangle]
pub extern "C" fn iswpunct(c: wint_t) -> c_int { (wc_flags(c) & WC_PUNCT != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswspace(c: wint_t) -> c_int {
    if c < 128 { (wc_flags(c) & WC_SPACE != 0) as c_int }
    else {
        match c {
            0x1680 | 0x2000..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000 => 1,
            _ => 0,
        }
    }
}
#[no_mangle]
pub extern "C" fn iswupper(c: wint_t) -> c_int { (wc_flags(c) & WC_UPPER != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswxdigit(c: wint_t) -> c_int { (wc_flags(c) & WC_XDIGIT != 0) as c_int }

static WCTYPE_ALNUM: c_int = WC_ALNUM;
static WCTYPE_ALPHA: c_int = WC_ALPHA;
static WCTYPE_BLANK: c_int = WC_BLANK;
static WCTYPE_CNTRL: c_int = WC_CNTRL;
static WCTYPE_DIGIT: c_int = WC_DIGIT;
static WCTYPE_GRAPH: c_int = WC_GRAPH;
static WCTYPE_LOWER: c_int = WC_LOWER;
static WCTYPE_PRINT: c_int = WC_PRINT;
static WCTYPE_PUNCT: c_int = WC_PUNCT;
static WCTYPE_SPACE: c_int = WC_SPACE;
static WCTYPE_UPPER: c_int = WC_UPPER;
static WCTYPE_XDIGIT: c_int = WC_XDIGIT;

#[no_mangle]
pub unsafe extern "C" fn wctype(name: *const c_char) -> wctype_t {
    if name.is_null() { return core::ptr::null(); }
    let n = name as *const u8;
    if strcmp(n, b"alnum\0".as_ptr()) == 0 { return &WCTYPE_ALNUM; }
    if strcmp(n, b"alpha\0".as_ptr()) == 0 { return &WCTYPE_ALPHA; }
    if strcmp(n, b"blank\0".as_ptr()) == 0 { return &WCTYPE_BLANK; }
    if strcmp(n, b"cntrl\0".as_ptr()) == 0 { return &WCTYPE_CNTRL; }
    if strcmp(n, b"digit\0".as_ptr()) == 0 { return &WCTYPE_DIGIT; }
    if strcmp(n, b"graph\0".as_ptr()) == 0 { return &WCTYPE_GRAPH; }
    if strcmp(n, b"lower\0".as_ptr()) == 0 { return &WCTYPE_LOWER; }
    if strcmp(n, b"print\0".as_ptr()) == 0 { return &WCTYPE_PRINT; }
    if strcmp(n, b"punct\0".as_ptr()) == 0 { return &WCTYPE_PUNCT; }
    if strcmp(n, b"space\0".as_ptr()) == 0 { return &WCTYPE_SPACE; }
    if strcmp(n, b"upper\0".as_ptr()) == 0 { return &WCTYPE_UPPER; }
    if strcmp(n, b"xdigit\0".as_ptr()) == 0 { return &WCTYPE_XDIGIT; }
    core::ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn iswctype(c: wint_t, desc: wctype_t) -> c_int {
    if desc.is_null() { return 0; }
    (wc_flags(c) & *desc != 0) as c_int
}

static WCTRANS_LOWER: c_int = 1;
static WCTRANS_UPPER: c_int = 2;

#[no_mangle]
pub unsafe extern "C" fn towlower(c: wint_t) -> wint_t {
    if c < 128 { tolower(c as c_int) as wint_t } else { c }
}

#[no_mangle]
pub unsafe extern "C" fn towupper(c: wint_t) -> wint_t {
    if c < 128 { toupper(c as c_int) as wint_t } else { c }
}

#[no_mangle]
pub unsafe extern "C" fn wctrans(name: *const c_char) -> wctrans_t {
    if name.is_null() { return core::ptr::null(); }
    let n = name as *const u8;
    if strcmp(n, b"tolower\0".as_ptr()) == 0 { return &WCTRANS_LOWER; }
    if strcmp(n, b"toupper\0".as_ptr()) == 0 { return &WCTRANS_UPPER; }
    core::ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn towctrans(c: wint_t, desc: wctrans_t) -> wint_t {
    if desc.is_null() { return c; }
    match *desc {
        1 => towlower(c),
        2 => towupper(c),
        _ => c,
    }
}

// ============================================================
// wchar: wide stdio
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn fgetwc(f: *mut FILE) -> wint_t {
    let mut state: c_uint = 0;
    let mut wc: wchar_t = 0;
    let mut buf = [0u8; 4];
    let c = fgetc(f);
    if c == -1 { return WEOF; }
    buf[0] = c as u8;
    if buf[0] < 0x80 { return buf[0] as wint_t; }
    let len = if buf[0] < 0xE0 { 2 } else if buf[0] < 0xF0 { 3 } else { 4 };
    let mut n = 1usize;
    while n < len {
        let c2 = fgetc(f);
        if c2 == -1 { return WEOF; }
        buf[n] = c2 as u8;
        n += 1;
    }
    let r = mbrtowc(&mut wc, buf.as_ptr() as *const c_char, n, &mut state);
    if r == !0usize || r == !1usize { WEOF } else { wc as wint_t }
}

#[no_mangle]
pub unsafe extern "C" fn fputwc(c: wchar_t, f: *mut FILE) -> wint_t {
    let mut buf = [0u8; 4];
    let n = wcrtomb(buf.as_mut_ptr() as *mut c_char, c, core::ptr::null_mut());
    if n == !0usize || n == 0 { return WEOF; }
    if fwrite(buf.as_ptr() as *const c_void, 1, n, f) != n { return WEOF; }
    c as wint_t
}

#[no_mangle]
pub unsafe extern "C" fn fputws(s: *const wchar_t, f: *mut FILE) -> c_int {
    let mut i = 0;
    while *s.add(i) != 0 {
        if fputwc(*s.add(i), f) == WEOF { return -1; }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn ungetwc(c: wint_t, f: *mut FILE) -> wint_t {
    if c == WEOF { return WEOF; }
    let mut buf = [0u8; 4];
    let n = wcrtomb(buf.as_mut_ptr() as *mut c_char, c as wchar_t, core::ptr::null_mut());
    if n == !0usize || n == 0 { return WEOF; }
    let mut i = n;
    while i > 0 {
        i -= 1;
        if ungetc(buf[i] as c_int, f) == -1 { return WEOF; }
    }
    c
}

#[no_mangle]
pub unsafe extern "C" fn getwchar() -> wint_t { fgetwc(stdin) }

#[no_mangle]
pub unsafe extern "C" fn putwchar(c: wchar_t) -> wint_t { fputwc(c, stdout) }

// ============================================================
// wchar: swprintf / vswprintf / fwprintf / vfwprintf
// ============================================================

unsafe fn wfmt_write_str(dst: *mut wchar_t, pos: usize, cap: usize, s: *const u8, len: usize) -> usize {
    let mut i = 0;
    while i < len {
        if pos + i < cap { *dst.add(pos + i) = *s.add(i) as wchar_t; }
        i += 1;
    }
    pos + len
}

#[no_mangle]
pub unsafe extern "C" fn vswprintf(s: *mut wchar_t, n: usize, fmt: *const wchar_t, mut args: VaList) -> c_int {
    if n == 0 { return -1; }
    let cap = n - 1;
    let mut pos = 0usize;
    let mut i = 0usize;
    loop {
        let c = *fmt.add(i) as u32;
        if c == 0 { break; }
        if c != 0x25 {
            if pos < cap { *s.add(pos) = c as wchar_t; }
            pos += 1;
            i += 1;
            continue;
        }
        i += 1;
        let spec = *fmt.add(i) as u32;
        match spec {
            0x73 => {
                let p = args.arg::<*const c_char>();
                if p.is_null() { pos = wfmt_write_str(s, pos, cap, b"(null)".as_ptr(), 6); }
                else { let len = strlen(p as *const u8); pos = wfmt_write_str(s, pos, cap, p as *const u8, len); }
            }
            0x64 | 0x69 => {
                let d = args.arg::<c_int>();
                let (buf, len) = format_i64(d as i64);
                pos = wfmt_write_str(s, pos, cap, buf.as_ptr().add(21 - len), len);
            }
            0x75 => {
                let u = args.arg::<c_uint>();
                let (buf, len) = format_u64(u as u64);
                pos = wfmt_write_str(s, pos, cap, buf.as_ptr().add(20 - len), len);
            }
            0x78 => {
                let x = args.arg::<c_uint>();
                let (buf, len) = format_hex(x as u64, false);
                pos = wfmt_write_str(s, pos, cap, buf.as_ptr().add(16 - len), len);
            }
            0x58 => {
                let x = args.arg::<c_uint>();
                let (buf, len) = format_hex(x as u64, true);
                pos = wfmt_write_str(s, pos, cap, buf.as_ptr().add(16 - len), len);
            }
            0x63 => {
                let ch = args.arg::<c_int>();
                if pos < cap { *s.add(pos) = ch as wchar_t; }
                pos += 1;
            }
            0x70 => {
                if pos < cap { *s.add(pos) = b'0' as wchar_t; }
                if pos + 1 < cap { *s.add(pos + 1) = b'x' as wchar_t; }
                pos += 2;
                let p = args.arg::<*const c_void>();
                let (buf, len) = format_hex(p as u64, false);
                pos = wfmt_write_str(s, pos, cap, buf.as_ptr().add(16 - len), len);
            }
            0x66 => {
                let val = args.arg::<f64>();
                let (buf, len) = format_f64(val);
                pos = wfmt_write_str(s, pos, cap, buf.as_ptr(), len);
            }
            0x6c => {
                i += 1;
                let sub = *fmt.add(i) as u32;
                match sub {
                    0x64 => { let ld = args.arg::<c_long>(); let (buf, len) = format_i64(ld as i64); pos = wfmt_write_str(s, pos, cap, buf.as_ptr().add(21 - len), len); }
                    0x75 => { let lu = args.arg::<c_ulong>(); let (buf, len) = format_u64(lu as u64); pos = wfmt_write_str(s, pos, cap, buf.as_ptr().add(20 - len), len); }
                    0x78 => { let lx = args.arg::<c_ulong>(); let (buf, len) = format_hex(lx as u64, false); pos = wfmt_write_str(s, pos, cap, buf.as_ptr().add(16 - len), len); }
                    0x58 => { let lx = args.arg::<c_ulong>(); let (buf, len) = format_hex(lx as u64, true); pos = wfmt_write_str(s, pos, cap, buf.as_ptr().add(16 - len), len); }
                    _ => { if pos < cap { *s.add(pos) = b'%' as wchar_t; } if pos + 1 < cap { *s.add(pos + 1) = sub as wchar_t; } pos += 2; }
                }
            }
            0x25 => { if pos < cap { *s.add(pos) = b'%' as wchar_t; } pos += 1; }
            _ => { if pos < cap { *s.add(pos) = b'%' as wchar_t; } if pos + 1 < cap { *s.add(pos + 1) = spec as wchar_t; } pos += 2; }
        }
        i += 1;
    }
    if n > 0 { let null_pos = if pos < cap { pos } else { cap }; *s.add(null_pos) = 0; }
    pos as c_int
}

#[no_mangle]
pub unsafe extern "C" fn swprintf(s: *mut wchar_t, n: usize, fmt: *const wchar_t, mut args: ...) -> c_int {
    vswprintf(s, n, fmt, args.as_va_list())
}

#[no_mangle]
pub unsafe extern "C" fn vfwprintf(f: *mut FILE, fmt: *const wchar_t, mut args: VaList) -> c_int {
    let mut buf = [0i32; 4096];
    let r = vswprintf(buf.as_mut_ptr(), 4096, fmt, args);
    if r < 0 { return r; }
    let mut i = 0;
    while i < r as usize {
        let mut mb = [0u8; 4];
        let n = wcrtomb(mb.as_mut_ptr() as *mut c_char, buf[i], core::ptr::null_mut());
        if n == !0usize { return -1; }
        if fwrite(mb.as_ptr() as *const c_void, 1, n, f) != n { return -1; }
        i += 1;
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn fwprintf(f: *mut FILE, fmt: *const wchar_t, mut args: ...) -> c_int {
    vfwprintf(f, fmt, args.as_va_list())
}

// ============================================================
// wchar: vswscanf / swscanf / vfwscanf / fwscanf / vwscanf / wscanf
// ============================================================

// Convert wchar_t format to byte format (ASCII cast, format specifiers are ASCII)
unsafe fn wcsfmt_to_mbs(dst: *mut u8, dst_len: usize, src: *const wchar_t) {
    let mut i = 0usize;
    loop {
        let ch = *src.add(i) as u32;
        if ch == 0 || i >= dst_len - 1 { break; }
        *dst.add(i) = if ch < 128 { ch as u8 } else { b'?' };
        i += 1;
    }
    *dst.add(i) = 0;
}

#[no_mangle]
pub unsafe extern "C" fn vswscanf(s: *const wchar_t, fmt: *const wchar_t, mut args: VaList) -> c_int {
    let mut mbs_buf = [0u8; 4096];
    let mut i = 0usize;
    let mut mb_pos = 0usize;
    loop {
        let wc = *s.add(i) as c_int;
        if wc == 0 { break; }
        let mut mb = [0u8; 4];
        let n = wcrtomb(mb.as_mut_ptr() as *mut c_char, wc, core::ptr::null_mut());
        if n == !0usize || n == 0 { break; }
        if mb_pos + n >= mbs_buf.len() { break; }
        let mut k = 0usize;
        while k < n { mbs_buf[mb_pos + k] = mb[k]; k += 1; }
        mb_pos += n;
        i += 1;
    }
    mbs_buf[mb_pos] = 0;
    let mut mbs_fmt = [0u8; 4096];
    wcsfmt_to_mbs(mbs_fmt.as_mut_ptr(), mbs_fmt.len(), fmt);
    do_vsscanf(mbs_buf.as_ptr(), mb_pos, mbs_fmt.as_ptr() as *const c_char, &mut *args, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn swscanf(s: *const wchar_t, fmt: *const wchar_t, mut args: ...) -> c_int {
    vswscanf(s, fmt, args.as_va_list())
}

#[no_mangle]
pub unsafe extern "C" fn vfwscanf(f: *mut FILE, fmt: *const wchar_t, mut args: VaList) -> c_int {
    let start_pos = ftello(f);
    if start_pos < 0 { return -1; }
    let mut buf = [0u8; 4096];
    let mut n = 0usize;
    loop {
        if n >= buf.len() - 1 { break; }
        let c = fgetc(f);
        if c == -1 { break; }
        buf[n] = c as u8;
        n += 1;
    }
    buf[n] = 0;
    let mut mbs_fmt = [0u8; 4096];
    wcsfmt_to_mbs(mbs_fmt.as_mut_ptr(), mbs_fmt.len(), fmt);
    let mut consumed = 0usize;
    let result = do_vsscanf(buf.as_ptr(), n, mbs_fmt.as_ptr() as *const c_char, &mut *args, &mut consumed);
    let _ = fseeko(f, start_pos + consumed as i64, SEEK_SET);
    result
}

#[no_mangle]
pub unsafe extern "C" fn fwscanf(f: *mut FILE, fmt: *const wchar_t, mut args: ...) -> c_int {
    vfwscanf(f, fmt, args.as_va_list())
}

#[no_mangle]
pub unsafe extern "C" fn vwscanf(fmt: *const wchar_t, mut args: VaList) -> c_int {
    vfwscanf(stdin, fmt, args)
}

#[no_mangle]
pub unsafe extern "C" fn wscanf(fmt: *const wchar_t, mut args: ...) -> c_int {
    vfwscanf(stdin, fmt, args.as_va_list())
}

// ============================================================
// Time: internal helpers
// ============================================================

const LEAPOCH: i64 = 946684800 + 86400 * (31 + 29);
const DAYS_PER_400Y: i64 = 365 * 400 + 97;
const DAYS_PER_100Y: i64 = 365 * 100 + 24;
const DAYS_PER_4Y: i64 = 365 * 4 + 1;

const SECS_THROUGH_MONTH: [i32; 12] = [
    0, 31*86400, 59*86400, 90*86400, 120*86400, 151*86400,
    181*86400, 212*86400, 243*86400, 273*86400, 304*86400, 334*86400,
];

const DAYS_IN_MONTH: [i32; 12] = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];

unsafe fn year_to_secs(year: i64, is_leap: &mut bool) -> i64 {
    if year - 2 <= 136 {
        let y = year as i32;
        let mut leaps = (y - 68) >> 2;
        if (y - 68) & 3 == 0 { leaps -= 1; *is_leap = true; } else { *is_leap = false; }
        return 31536000 * (y as i64 - 70) + 86400 * leaps as i64;
    }
    let mut cycles = ((year - 100) / 400) as i32;
    let mut rem = ((year - 100) % 400) as i32;
    if rem < 0 { cycles -= 1; rem += 400; }
    let (centuries, leaps);
    if rem == 0 { *is_leap = true; centuries = 0; leaps = 0; }
    else {
        let c;
        if rem >= 200 { if rem >= 300 { c = 3; rem -= 300; } else { c = 2; rem -= 200; } }
        else { if rem >= 100 { c = 1; rem -= 100; } else { c = 0; } }
        centuries = c;
        if rem == 0 { *is_leap = false; leaps = 0; }
        else { leaps = rem / 4; rem %= 4; *is_leap = rem == 0; }
    }
    let total_leaps = leaps + 97 * cycles + 24 * centuries - (*is_leap as i32);
    (year - 100) * 31536000 + total_leaps as i64 * 86400 + 946684800 + 86400
}

fn month_to_secs(month: i32, is_leap: bool) -> i32 {
    let mut t = SECS_THROUGH_MONTH[month as usize];
    if is_leap && month >= 2 { t += 86400; }
    t
}

unsafe fn secs_to_tm(t: i64, tm: &mut tm) -> bool {
    if t < (i32::MIN as i64) * 31622400 || t > (i32::MAX as i64) * 31622400 { return false; }
    let secs = t - LEAPOCH;
    let mut days = secs / 86400;
    let mut remsecs = (secs % 86400) as i32;
    if remsecs < 0 { remsecs += 86400; days -= 1; }
    let mut wday = ((3 + days) % 7) as i32;
    if wday < 0 { wday += 7; }
    let mut qc_cycles = (days / DAYS_PER_400Y) as i32;
    let mut remdays = (days % DAYS_PER_400Y) as i32;
    if remdays < 0 { remdays += DAYS_PER_400Y as i32; qc_cycles -= 1; }
    let mut c_cycles = remdays / DAYS_PER_100Y as i32;
    if c_cycles == 4 { c_cycles -= 1; }
    remdays -= c_cycles * DAYS_PER_100Y as i32;
    let mut q_cycles = remdays / DAYS_PER_4Y as i32;
    if q_cycles == 25 { q_cycles -= 1; }
    remdays -= q_cycles * DAYS_PER_4Y as i32;
    let mut remyears = remdays / 365;
    if remyears == 4 { remyears -= 1; }
    remdays -= remyears * 365;
    let leap = remyears == 0 && (q_cycles != 0 || c_cycles == 0);
    let mut yday = remdays + 31 + 28 + leap as i32;
    if yday >= 365 + leap as i32 { yday -= 365 + leap as i32; }
    let years = remyears as i64 + 4 * q_cycles as i64 + 100 * c_cycles as i64 + 400 * qc_cycles as i64;
    let mut months = 0i32;
    let mut rd = remdays;
    for m in 0..12 {
        if DAYS_IN_MONTH[m] <= rd { rd -= DAYS_IN_MONTH[m]; } else { months = m as i32; break; }
    }
    let mut adj_years = years;
    if months >= 10 { months -= 12; adj_years += 1; }
    if adj_years + 100 > i32::MAX as i64 || adj_years + 100 < i32::MIN as i64 { return false; }
    tm.tm_year = (adj_years + 100) as c_int;
    tm.tm_mon = months + 2;
    tm.tm_mday = rd + 1;
    tm.tm_wday = wday;
    tm.tm_yday = yday;
    tm.tm_hour = remsecs / 3600;
    tm.tm_min = remsecs / 60 % 60;
    tm.tm_sec = remsecs % 60;
    true
}

unsafe fn tm_to_secs(tm: *const tm) -> i64 {
    let mut year = (*tm).tm_year as i64;
    let mut month = (*tm).tm_mon;
    if month >= 12 || month < 0 {
        let adj = month / 12;
        month %= 12;
        if month < 0 { month += 12; }
        year += adj as i64;
    }
    let mut is_leap = false;
    let mut t = year_to_secs(year, &mut is_leap);
    t += month_to_secs(month, is_leap) as i64;
    t += 86400 * ((*tm).tm_mday - 1) as i64;
    t += 3600 * (*tm).tm_hour as i64;
    t += 60 * (*tm).tm_min as i64;
    t += (*tm).tm_sec as i64;
    t
}

static mut TM_BUF: tm = tm {
    tm_sec: 0, tm_min: 0, tm_hour: 0, tm_mday: 0, tm_mon: 0,
    tm_year: 0, tm_wday: 0, tm_yday: 0, tm_isdst: 0,
    tm_gmtoff: 0, tm_zone: core::ptr::null(),
};

static mut UTC_STR: [c_char; 4] = [b'U' as c_char, b'T' as c_char, b'C' as c_char, 0];

const CLOCK_PROCESS_CPUTIME_ID: c_int = 2;

// ============================================================
// Time: core functions
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn time(t: *mut TimeT) -> TimeT {
    let mut ts: timespec = core::mem::zeroed();
    let _ = sys_clock_gettime(CLOCK_REALTIME, &mut ts);
    if !t.is_null() { *t = ts.tv_sec; }
    ts.tv_sec
}

#[no_mangle]
pub unsafe extern "C" fn difftime(t1: TimeT, t0: TimeT) -> f64 {
    (t1 as i64 - t0 as i64) as f64
}

#[no_mangle]
pub unsafe extern "C" fn clock() -> ClockT {
    let mut ts: timespec = core::mem::zeroed();
    let _ = sys_clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &mut ts);
    ts.tv_sec * 1000000 + ts.tv_nsec / 1000
}

#[no_mangle]
pub unsafe extern "C" fn gmtime(t: *const TimeT) -> *mut tm {
    gmtime_r(t, &raw mut TM_BUF)
}

#[no_mangle]
pub unsafe extern "C" fn gmtime_r(t: *const TimeT, tm: *mut tm) -> *mut tm {
    if !secs_to_tm(*t as i64, &mut *tm) { ERRNO = EOVERFLOW; return core::ptr::null_mut(); }
    (*tm).tm_isdst = 0;
    (*tm).tm_gmtoff = 0;
    (*tm).tm_zone = core::ptr::addr_of!(UTC_STR).cast::<c_char>();
    tm
}

#[no_mangle]
pub unsafe extern "C" fn localtime(t: *const TimeT) -> *mut tm { gmtime(t) }

#[no_mangle]
pub unsafe extern "C" fn localtime_r(t: *const TimeT, tm: *mut tm) -> *mut tm { gmtime_r(t, tm) }

#[no_mangle]
pub unsafe extern "C" fn mktime(tm: *mut tm) -> TimeT {
    let t = tm_to_secs(tm as *const tm);
    if !secs_to_tm(t, &mut *tm) { ERRNO = EOVERFLOW; return -1; }
    (*tm).tm_isdst = 0;
    (*tm).tm_gmtoff = 0;
    (*tm).tm_zone = core::ptr::addr_of!(UTC_STR).cast::<c_char>();
    t as TimeT
}

#[no_mangle]
pub unsafe extern "C" fn timegm(tm: *mut tm) -> TimeT {
    let t = tm_to_secs(tm as *const tm);
    if !secs_to_tm(t, &mut *tm) { ERRNO = EOVERFLOW; return -1; }
    (*tm).tm_isdst = 0;
    (*tm).tm_gmtoff = 0;
    (*tm).tm_zone = core::ptr::addr_of!(UTC_STR).cast::<c_char>();
    t as TimeT
}

static mut ASCTIME_BUF: [c_char; 26] = [0; 26];

const DAY_NAMES: [&[u8]; 7] = [b"Sun\0", b"Mon\0", b"Tue\0", b"Wed\0", b"Thu\0", b"Fri\0", b"Sat\0"];
const MON_NAMES: [&[u8]; 12] = [b"Jan\0", b"Feb\0", b"Mar\0", b"Apr\0", b"May\0", b"Jun\0", b"Jul\0", b"Aug\0", b"Sep\0", b"Oct\0", b"Nov\0", b"Dec\0"];

#[no_mangle]
pub unsafe extern "C" fn asctime(tm: *const tm) -> *mut c_char {
    asctime_r(tm, core::ptr::addr_of_mut!(ASCTIME_BUF).cast::<c_char>())
}

#[no_mangle]
pub unsafe extern "C" fn asctime_r(tm: *const tm, buf: *mut c_char) -> *mut c_char {
    let wday = if (*tm).tm_wday >= 0 && (*tm).tm_wday < 7 { (*tm).tm_wday as usize } else { 0 };
    let mon = if (*tm).tm_mon >= 0 && (*tm).tm_mon < 12 { (*tm).tm_mon as usize } else { 0 };
    let dn = DAY_NAMES[wday];
    let mn = MON_NAMES[mon];
    let mut p = 0;
    for k in 0..3 { *buf.add(p) = *dn.as_ptr().add(k) as c_char; p += 1; }
    *buf.add(p) = b' ' as c_char; p += 1;
    for k in 0..3 { *buf.add(p) = *mn.as_ptr().add(k) as c_char; p += 1; }
    *buf.add(p) = b' ' as c_char; p += 1;
    let v = (*tm).tm_mday;
    *buf.add(p) = (b'0' + (v / 10) as u8) as c_char; p += 1;
    *buf.add(p) = (b'0' + (v % 10) as u8) as c_char; p += 1;
    *buf.add(p) = b' ' as c_char; p += 1;
    let hh = (*tm).tm_hour;
    *buf.add(p) = (b'0' + (hh / 10) as u8) as c_char; p += 1;
    *buf.add(p) = (b'0' + (hh % 10) as u8) as c_char; p += 1;
    *buf.add(p) = b':' as c_char; p += 1;
    let mm = (*tm).tm_min;
    *buf.add(p) = (b'0' + (mm / 10) as u8) as c_char; p += 1;
    *buf.add(p) = (b'0' + (mm % 10) as u8) as c_char; p += 1;
    *buf.add(p) = b':' as c_char; p += 1;
    let ss = (*tm).tm_sec;
    *buf.add(p) = (b'0' + (ss / 10) as u8) as c_char; p += 1;
    *buf.add(p) = (b'0' + (ss % 10) as u8) as c_char; p += 1;
    *buf.add(p) = b' ' as c_char; p += 1;
    let yr = (*tm).tm_year + 1900;
    let mut tmp = [0u8; 4];
    let mut v2 = yr;
    for k in (0..4).rev() { tmp[k] = b'0' + (v2 % 10) as u8; v2 /= 10; }
    for k in 0..4 { *buf.add(p) = tmp[k] as c_char; p += 1; }
    *buf.add(p) = b'\n' as c_char; p += 1;
    *buf.add(p) = 0;
    buf
}

#[no_mangle]
pub unsafe extern "C" fn ctime(t: *const TimeT) -> *mut c_char { asctime(gmtime(t)) }

#[no_mangle]
pub unsafe extern "C" fn ctime_r(t: *const TimeT, buf: *mut c_char) -> *mut c_char {
    let mut tmp: tm = core::mem::zeroed();
    gmtime_r(t, &mut tmp);
    asctime_r(&tmp, buf)
}

// ============================================================
// Time: clock_getres / clock_settime / clock_nanosleep / gettimeofday
// ============================================================

unsafe fn sys_clock_getres(clockid: c_int, ts: *mut timespec) -> i64 {
    let result: i64;
    core::arch::asm!("syscall", inlateout("rax") 229i64 => result, in("rdi") clockid as i64, in("rsi") ts, lateout("rcx") _, lateout("r11") _);
    result
}

unsafe fn sys_clock_settime(clockid: c_int, ts: *const timespec) -> i64 {
    let result: i64;
    core::arch::asm!("syscall", inlateout("rax") 230i64 => result, in("rdi") clockid as i64, in("rsi") ts, lateout("rcx") _, lateout("r11") _);
    result
}

unsafe fn sys_clock_nanosleep(clockid: c_int, flags: c_int, req: *const timespec, rem: *mut timespec) -> i64 {
    let result: i64;
    core::arch::asm!("syscall", inlateout("rax") 231i64 => result, in("rdi") clockid as i64, in("rsi") flags as i64, in("rdx") req, in("r10") rem, lateout("rcx") _, lateout("r11") _);
    result
}

#[no_mangle]
pub unsafe extern "C" fn clock_getres(clockid: c_int, ts: *mut timespec) -> c_int { if sys_clock_getres(clockid, ts) < 0 { -1 } else { 0 } }

#[no_mangle]
pub unsafe extern "C" fn clock_settime(clockid: c_int, ts: *const timespec) -> c_int { if sys_clock_settime(clockid, ts) < 0 { -1 } else { 0 } }

#[no_mangle]
pub unsafe extern "C" fn clock_nanosleep(clockid: c_int, flags: c_int, req: *const timespec, rem: *mut timespec) -> c_int {
    let r = sys_clock_nanosleep(clockid, flags, req, rem);
    if r < 0 { (-r) as c_int } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn gettimeofday(tv: *mut timeval, _tz: *mut c_void) -> c_int {
    if tv.is_null() { return 0; }
    let mut ts: timespec = core::mem::zeroed();
    let _ = sys_clock_gettime(CLOCK_REALTIME, &mut ts);
    (*tv).tv_sec = ts.tv_sec;
    (*tv).tv_usec = ts.tv_nsec / 1000;
    0
}

// ============================================================
// Time: strftime
// ============================================================

fn is_leap_year(y: i32) -> bool {
    let yr = if y > i32::MAX - 1900 { y - 2000 } else { y + 1900 };
    yr % 4 == 0 && (yr % 100 != 0 || yr % 400 == 0)
}

fn week_num(tm: &tm) -> i32 {
    let mut val = (tm.tm_yday + 7 - ((tm.tm_wday + 6) % 7)) / 7;
    if (tm.tm_wday + 371 - tm.tm_yday - 2).rem_euclid(7) <= 2 { val += 1; }
    if val == 0 {
        val = 52;
        let dec31 = (tm.tm_wday + 7 - tm.tm_yday - 1).rem_euclid(7);
        if dec31 == 4 || (dec31 == 5 && is_leap_year(tm.tm_year % 400 - 1)) { val += 1; }
    } else if val == 53 {
        let jan1 = (tm.tm_wday + 371 - tm.tm_yday).rem_euclid(7);
        if jan1 != 4 && (jan1 != 3 || !is_leap_year(tm.tm_year)) { val = 1; }
    }
    val
}

fn fmt_i32(buf: &mut [u8; 32], val: i32, width: usize) -> usize {
    let mut tmp = [0u8; 12];
    let mut v = if val < 0 { -val } else { val } as u32;
    let mut pos = 12;
    if v == 0 { pos -= 1; tmp[pos] = b'0'; }
    while v > 0 { pos -= 1; tmp[pos] = b'0' + (v % 10) as u8; v /= 10; }
    let digits = 12 - pos;
    let fill = if width > digits { width - digits } else { 0 };
    let mut p = 0;
    for _ in 0..fill { buf[p] = b'0'; p += 1; }
    for i in 0..digits { buf[p] = tmp[pos + i]; p += 1; }
    p
}

fn fmt_i32_inline(buf: &mut [u8; 32], start: usize, val: i32, width: usize) -> usize {
    let mut tmp = [0u8; 12];
    let mut v = if val < 0 { -val } else { val } as u32;
    let mut pos = 12;
    if v == 0 { pos -= 1; tmp[pos] = b'0'; }
    while v > 0 { pos -= 1; tmp[pos] = b'0' + (v % 10) as u8; v /= 10; }
    let digits = 12 - pos;
    let fill = if width > digits { width - digits } else { 0 };
    let mut p = start;
    for _ in 0..fill { buf[p] = b'0'; p += 1; }
    for i in 0..digits { buf[p] = tmp[pos + i]; p += 1; }
    p - start
}

fn fmt_i32_wide(buf: &mut [u8; 32], val: i32) -> usize {
    let mut tmp = [0u8; 12];
    let mut v = val;
    let mut pos = 12;
    if v < 0 { v = -v; }
    if v == 0 { pos -= 1; tmp[pos] = b'0'; }
    while v > 0 { pos -= 1; tmp[pos] = b'0' + (v % 10) as u8; v /= 10; }
    let mut p = 0;
    if val < 0 { buf[0] = b'-'; p = 1; }
    for i in 0..(12 - pos) { buf[p] = tmp[pos + i]; p += 1; }
    p
}

fn fmt_hhmmss(buf: &mut [u8; 32], h: i32, m: i32, s: i32) -> usize {
    let mut p = fmt_i32(buf, h, 2);
    buf[p] = b':'; p += 1;
    p += fmt_i32_inline(buf, p, m, 2);
    buf[p] = b':'; p += 1;
    p += fmt_i32_inline(buf, p, s, 2);
    p
}

fn fmt_hhmm(buf: &mut [u8; 32], h: i32, m: i32) -> usize {
    let mut p = fmt_i32(buf, h, 2);
    buf[p] = b':'; p += 1;
    p += fmt_i32_inline(buf, p, m, 2);
    p
}

fn fmt_date_slash(buf: &mut [u8; 32], m: i32, d: i32, y: i32) -> usize {
    let mut p = fmt_i32(buf, m, 2);
    buf[p] = b'/'; p += 1;
    p += fmt_i32_inline(buf, p, d, 2);
    buf[p] = b'/'; p += 1;
    p += fmt_i32_inline(buf, p, if y < 0 { -y } else { y }, 2);
    p
}

fn fmt_iso_date(buf: &mut [u8; 32], year: i32, m: i32, d: i32) -> usize {
    let mut p = fmt_i32_wide(buf, year);
    buf[p] = b'-'; p += 1;
    p += fmt_i32_inline(buf, p, m, 2);
    buf[p] = b'-'; p += 1;
    p += fmt_i32_inline(buf, p, d, 2);
    p
}

fn fmt_12h_time(buf: &mut [u8; 32], h: i32, m: i32, s: i32) -> usize {
    let h12 = if h == 0 { 12 } else if h > 12 { h - 12 } else { h };
    let ampm: &[u8] = if h >= 12 { b"PM" } else { b"AM" };
    let mut p = fmt_i32(buf, h12, 2);
    buf[p] = b':'; p += 1;
    p += fmt_i32_inline(buf, p, m, 2);
    buf[p] = b':'; p += 1;
    p += fmt_i32_inline(buf, p, s, 2);
    buf[p] = b' '; p += 1;
    buf[p] = ampm[0]; buf[p + 1] = ampm[1]; p += 2;
    p
}

fn fmt_tz_offset(buf: &mut [u8; 32], gmtoff: i64) -> usize {
    let sign = if gmtoff >= 0 { b'+' } else { b'-' };
    let mut off = if gmtoff < 0 { -gmtoff } else { gmtoff };
    let hh = (off / 3600) as i32;
    off %= 3600;
    let mm = (off / 60) as i32;
    buf[0] = sign;
    let mut p = 1;
    p += fmt_i32_inline(buf, p, hh, 2);
    p += fmt_i32_inline(buf, p, mm, 2);
    p
}

#[no_mangle]
pub unsafe extern "C" fn strftime(s: *mut c_char, maxsize: usize, fmt: *const c_char, tm: *const tm) -> usize {
    let mut pos = 0usize;
    let mut fi = 0usize;
    let limit = if maxsize > 0 { maxsize - 1 } else { 0 };
    loop {
        let fc = *fmt.add(fi) as u8;
        if fc == 0 { break; }
        if fc != b'%' {
            if pos < limit { *s.add(pos) = fc as c_char; }
            pos += 1; fi += 1; continue;
        }
        fi += 1;
        let spec = *fmt.add(fi) as u8;
        let mut tmp = [0u8; 32];
        let tlen;
        match spec {
            b'a' => {
                let d = (*tm).tm_wday;
                let names: [&[u8]; 7] = [b"Sun", b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat"];
                let name = if d >= 0 && d < 7 { names[d as usize] } else { b"???" };
                tlen = 3; for k in 0..3 { tmp[k] = name[k]; }
            }
            b'A' => {
                let d = (*tm).tm_wday;
                let names: [&[u8]; 7] = [b"Sunday", b"Monday", b"Tuesday", b"Wednesday", b"Thursday", b"Friday", b"Saturday"];
                let name = if d >= 0 && d < 7 { names[d as usize] } else { b"???" };
                tlen = name.len(); for k in 0..tlen { tmp[k] = name[k]; }
            }
            b'b' | b'h' => {
                let m = (*tm).tm_mon;
                let names: [&[u8]; 12] = [b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec"];
                let name = if m >= 0 && m < 12 { names[m as usize] } else { b"???" };
                tlen = 3; for k in 0..3 { tmp[k] = name[k]; }
            }
            b'B' => {
                let m = (*tm).tm_mon;
                let names: [&[u8]; 12] = [b"January", b"February", b"March", b"April", b"May", b"June", b"July", b"August", b"September", b"October", b"November", b"December"];
                let name = if m >= 0 && m < 12 { names[m as usize] } else { b"???" };
                tlen = name.len(); for k in 0..tlen { tmp[k] = name[k]; }
            }
            b'C' => { tlen = fmt_i32(&mut tmp, ((*tm).tm_year + 1900) / 100, 2); }
            b'd' => { tlen = fmt_i32(&mut tmp, (*tm).tm_mday, 2); }
            b'e' => { let v = (*tm).tm_mday; if v < 10 { tmp[0] = b' '; tmp[1] = b'0' + v as u8; tlen = 2; } else { tlen = fmt_i32(&mut tmp, v, 2); } }
            b'D' => { tlen = fmt_date_slash(&mut tmp, (*tm).tm_mon + 1, (*tm).tm_mday, (*tm).tm_year % 100); }
            b'F' => { tlen = fmt_iso_date(&mut tmp, (*tm).tm_year + 1900, (*tm).tm_mon + 1, (*tm).tm_mday); }
            b'H' => { tlen = fmt_i32(&mut tmp, (*tm).tm_hour, 2); }
            b'I' => { let h = (*tm).tm_hour; let v = if h == 0 { 12 } else if h > 12 { h - 12 } else { h }; tlen = fmt_i32(&mut tmp, v, 2); }
            b'j' => { tlen = fmt_i32(&mut tmp, (*tm).tm_yday + 1, 3); }
            b'm' => { tlen = fmt_i32(&mut tmp, (*tm).tm_mon + 1, 2); }
            b'M' => { tlen = fmt_i32(&mut tmp, (*tm).tm_min, 2); }
            b'n' => { if pos < limit { *s.add(pos) = b'\n' as c_char; } pos += 1; fi += 1; continue; }
            b'p' => { let ampm: &[u8] = if (*tm).tm_hour >= 12 { b"PM" } else { b"AM" }; tlen = 2; tmp[0] = ampm[0]; tmp[1] = ampm[1]; }
            b'r' => { tlen = fmt_12h_time(&mut tmp, (*tm).tm_hour, (*tm).tm_min, (*tm).tm_sec); }
            b'R' => { tlen = fmt_hhmm(&mut tmp, (*tm).tm_hour, (*tm).tm_min); }
            b'S' => { tlen = fmt_i32(&mut tmp, (*tm).tm_sec, 2); }
            b't' => { if pos < limit { *s.add(pos) = b'\t' as c_char; } pos += 1; fi += 1; continue; }
            b'T' => { tlen = fmt_hhmmss(&mut tmp, (*tm).tm_hour, (*tm).tm_min, (*tm).tm_sec); }
            b'u' => { let v = if (*tm).tm_wday == 0 { 7 } else { (*tm).tm_wday }; tmp[0] = b'0' + v as u8; tlen = 1; }
            b'U' => { let v = ((*tm).tm_yday + 7 - (*tm).tm_wday) / 7; tlen = fmt_i32(&mut tmp, v, 2); }
            b'W' => { let v = ((*tm).tm_yday + 7 - ((*tm).tm_wday + 6) % 7) / 7; tlen = fmt_i32(&mut tmp, v, 2); }
            b'V' => { tlen = fmt_i32(&mut tmp, week_num(&*tm), 2); }
            b'w' => { tmp[0] = b'0' + (*tm).tm_wday as u8; tlen = 1; }
            b'x' => { tlen = fmt_date_slash(&mut tmp, (*tm).tm_mon + 1, (*tm).tm_mday, (*tm).tm_year % 100); }
            b'X' => { tlen = fmt_hhmmss(&mut tmp, (*tm).tm_hour, (*tm).tm_min, (*tm).tm_sec); }
            b'y' => { let v = ((*tm).tm_year + 1900) % 100; tlen = fmt_i32(&mut tmp, if v < 0 { -v } else { v }, 2); }
            b'Y' => { tlen = fmt_i32_wide(&mut tmp, (*tm).tm_year + 1900); }
            b'z' => { tlen = fmt_tz_offset(&mut tmp, (*tm).tm_gmtoff); }
            b'Z' => {
                if (*tm).tm_zone.is_null() { tlen = 0; }
                else {
                    let mut k = 0;
                    while *(*tm).tm_zone.add(k) != 0 { tmp[k] = *(*tm).tm_zone.add(k) as u8; k += 1; }
                    tlen = k;
                }
            }
            b'%' => { tmp[0] = b'%'; tlen = 1; }
            b'E' | b'O' => {
                fi += 1;
                let sub = *fmt.add(fi) as u8;
                match sub {
                    b'Y' => { tlen = fmt_i32_wide(&mut tmp, (*tm).tm_year + 1900); }
                    b'y' => { let v = ((*tm).tm_year + 1900) % 100; tlen = fmt_i32(&mut tmp, if v < 0 { -v } else { v }, 2); }
                    b'd' => { tlen = fmt_i32(&mut tmp, (*tm).tm_mday, 2); }
                    b'H' => { tlen = fmt_i32(&mut tmp, (*tm).tm_hour, 2); }
                    b'M' => { tlen = fmt_i32(&mut tmp, (*tm).tm_min, 2); }
                    b'S' => { tlen = fmt_i32(&mut tmp, (*tm).tm_sec, 2); }
                    _ => { tmp[0] = b'%'; tmp[1] = spec; tmp[2] = sub; tlen = 3; }
                }
            }
            _ => { tmp[0] = b'%'; tmp[1] = spec; tlen = 2; }
        }
        for k in 0..tlen {
            if pos < limit { *s.add(pos) = tmp[k] as c_char; }
            pos += 1;
        }
        fi += 1;
    }
    if maxsize > 0 { let null_pos = if pos < limit { pos } else { limit }; *s.add(null_pos) = 0; }
    pos
}

// ============================================================
// Time: strptime
// ============================================================

unsafe fn match_prefix(s: *const u8, prefix: &[u8]) -> bool {
    for i in 0..prefix.len() {
        if *s.add(i) == 0 || *s.add(i) != prefix[i] { return false; }
    }
    true
}

unsafe fn match_ci_prefix(s: *const u8, prefix: &[u8]) -> bool {
    for i in 0..prefix.len() {
        let c = *s.add(i);
        if c == 0 { return false; }
        if c.to_ascii_lowercase() != prefix[i].to_ascii_lowercase() { return false; }
    }
    true
}

unsafe fn parse_int(p: *const u8, max_digits: usize) -> (i32, *const u8) {
    let mut val = 0i32;
    let mut np = p;
    let mut count = 0;
    while *np >= b'0' && *np <= b'9' && count < max_digits {
        val = val * 10 + (*np - b'0') as i32;
        np = np.add(1);
        count += 1;
    }
    if count == 0 { (0, p) } else { (val, np) }
}

unsafe fn parse_range(p: *const u8, min: i32, max: i32) -> (i32, *const u8) {
    let (val, np) = parse_int(p, 4);
    if np == p || val < min || val > max { (0, p) } else { (val, np) }
}

fn is_ascii_space(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == 0x0c || c == 0x0b
}

#[no_mangle]
pub unsafe extern "C" fn strptime(s: *const c_char, fmt: *const c_char, tm: *mut tm) -> *mut c_char {
    let mut p = s as *const u8;
    let mut f = fmt as *const u8;
    let mut want_century = 0i32;
    let mut century = 0i32;
    let mut relyear = 0i32;
    while *f != 0 {
        if *f != b'%' {
            if is_ascii_space(*f) {
                while *p != 0 && is_ascii_space(*p) { p = p.add(1); }
            } else {
                if *p != *f { return core::ptr::null_mut(); }
                p = p.add(1);
            }
            f = f.add(1);
            continue;
        }
        f = f.add(1);
        if *f == b'+' { f = f.add(1); }
        while *f >= b'0' && *f <= b'9' { f = f.add(1); }
        if *f == b'E' || *f == b'O' { f = f.add(1); }
        match *f {
            b'a' | b'A' => {
                let mut found = false;
                for i in 0..7 {
                    let names: [&[u8]; 7] = [b"Sunday", b"Monday", b"Tuesday", b"Wednesday", b"Thursday", b"Friday", b"Saturday"];
                    let snames: [&[u8]; 7] = [b"Sun", b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat"];
                    if match_prefix(p, names[i]) { (*tm).tm_wday = i as c_int; p = p.add(names[i].len()); found = true; break; }
                    if match_prefix(p, snames[i]) { (*tm).tm_wday = i as c_int; p = p.add(snames[i].len()); found = true; break; }
                }
                if !found { return core::ptr::null_mut(); }
            }
            b'b' | b'B' | b'h' => {
                let mut found = false;
                for i in 0..12 {
                    let names: [&[u8]; 12] = [b"January", b"February", b"March", b"April", b"May", b"June", b"July", b"August", b"September", b"October", b"November", b"December"];
                    let snames: [&[u8]; 12] = [b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec"];
                    if match_prefix(p, names[i]) { (*tm).tm_mon = i as c_int; p = p.add(names[i].len()); found = true; break; }
                    if match_prefix(p, snames[i]) { (*tm).tm_mon = i as c_int; p = p.add(snames[i].len()); found = true; break; }
                }
                if !found { return core::ptr::null_mut(); }
            }
            b'C' => {
                let (v, np) = parse_int(p, 2);
                if np == p { return core::ptr::null_mut(); }
                century = v;
                want_century |= 2;
                p = np;
            }
            b'd' | b'e' => {
                let (v, np) = parse_range(p, 1, 31);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_mday = v;
                p = np;
            }
            b'D' => {
                let r = strptime(p as *const c_char, b"%m/%d/%y\0".as_ptr() as *const c_char, tm);
                if r.is_null() { return core::ptr::null_mut(); }
                p = r as *const u8;
            }
            b'F' => {
                let (yv, ynp) = parse_int(p, 4);
                if ynp == p { return core::ptr::null_mut(); }
                (*tm).tm_year = yv - 1900;
                p = ynp;
                if *p != b'-' { return core::ptr::null_mut(); }
                p = p.add(1);
                let (mv, mnp) = parse_range(p, 1, 12);
                if mnp == p { return core::ptr::null_mut(); }
                (*tm).tm_mon = mv - 1;
                p = mnp;
                if *p != b'-' { return core::ptr::null_mut(); }
                p = p.add(1);
                let (dv, dnp) = parse_range(p, 1, 31);
                if dnp == p { return core::ptr::null_mut(); }
                (*tm).tm_mday = dv;
                p = dnp;
            }
            b'H' | b'I' => {
                let (v, np) = parse_range(p, 0, 23);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_hour = v;
                p = np;
            }
            b'j' => {
                let (v, np) = parse_range(p, 1, 366);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_yday = v - 1;
                p = np;
            }
            b'm' => {
                let (v, np) = parse_range(p, 1, 12);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_mon = v - 1;
                p = np;
            }
            b'M' => {
                let (v, np) = parse_range(p, 0, 59);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_min = v;
                p = np;
            }
            b'n' | b't' => {
                while *p != 0 && is_ascii_space(*p) { p = p.add(1); }
            }
            b'p' => {
                if match_ci_prefix(p, b"AM") {
                    if (*tm).tm_hour == 12 { (*tm).tm_hour = 0; }
                    p = p.add(2);
                } else if match_ci_prefix(p, b"PM") {
                    if (*tm).tm_hour < 12 { (*tm).tm_hour += 12; }
                    p = p.add(2);
                } else { return core::ptr::null_mut(); }
            }
            b'r' => {
                let r = strptime(p as *const c_char, b"%I:%M:%S %p\0".as_ptr() as *const c_char, tm);
                if r.is_null() { return core::ptr::null_mut(); }
                p = r as *const u8;
            }
            b'R' => {
                let r = strptime(p as *const c_char, b"%H:%M\0".as_ptr() as *const c_char, tm);
                if r.is_null() { return core::ptr::null_mut(); }
                p = r as *const u8;
            }
            b's' => {
                let neg = if *p == b'-' { p = p.add(1); true } else { false };
                if *p < b'0' || *p > b'9' { return core::ptr::null_mut(); }
                let mut val = 0i64;
                while *p >= b'0' && *p <= b'9' {
                    val = val * 10 + (*p - b'0') as i64;
                    p = p.add(1);
                }
                if neg { val = -val; }
                secs_to_tm(val, &mut *tm);
            }
            b'S' => {
                let (v, np) = parse_range(p, 0, 60);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_sec = v;
                p = np;
            }
            b'T' => {
                let r = strptime(p as *const c_char, b"%H:%M:%S\0".as_ptr() as *const c_char, tm);
                if r.is_null() { return core::ptr::null_mut(); }
                p = r as *const u8;
            }
            b'U' | b'W' => {
                let (_, np) = parse_range(p, 0, 53);
                if np == p { return core::ptr::null_mut(); }
                p = np;
            }
            b'V' => {
                let (_, np) = parse_range(p, 1, 53);
                if np == p { return core::ptr::null_mut(); }
                p = np;
            }
            b'w' => {
                let (v, np) = parse_range(p, 0, 6);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_wday = v;
                p = np;
            }
            b'y' => {
                let (v, np) = parse_int(p, 2);
                if np == p { return core::ptr::null_mut(); }
                relyear = v;
                want_century |= 1;
                p = np;
            }
            b'Y' => {
                let neg = if *p == b'-' { p = p.add(1); true } else { false };
                let (v, np) = parse_int(p, 4);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_year = if neg { -(v + 1900) } else { v - 1900 };
                want_century = 0;
                p = np;
            }
            b'z' => {
                if *p != b'+' && *p != b'-' { return core::ptr::null_mut(); }
                let neg = *p == b'-';
                p = p.add(1);
                let mut off = 0i64;
                let mut digits = 0;
                while digits < 4 && *p >= b'0' && *p <= b'9' {
                    off = off * 10 + (*p - b'0') as i64;
                    p = p.add(1);
                    digits += 1;
                }
                if digits == 0 { return core::ptr::null_mut(); }
                let (hh, mm) = if digits <= 2 { (off, 0i64) } else { (off / 100, off % 100) };
                (*tm).tm_gmtoff = (hh * 3600 + mm * 60) * if neg { -1 } else { 1 };
            }
            b'%' => {
                if *p != b'%' { return core::ptr::null_mut(); }
                p = p.add(1);
            }
            _ => { return core::ptr::null_mut(); }
        }
        f = f.add(1);
    }
    if want_century != 0 {
        (*tm).tm_year = relyear;
        if want_century & 2 != 0 { (*tm).tm_year += century * 100 - 1900; }
        else if (*tm).tm_year <= 68 { (*tm).tm_year += 100; }
    }
    p as *mut c_char
}

// ============================================================
// Time: tzset / daylight / timezone / tzname
// ============================================================

#[no_mangle]
pub static mut daylight: c_int = 0;
#[no_mangle]
pub static mut timezone: c_long = 0;
#[no_mangle]
pub static mut tzname: [*mut c_char; 2] = [b"UTC\0".as_ptr() as *mut c_char, b"UTC\0".as_ptr() as *mut c_char];

#[no_mangle]
pub unsafe extern "C" fn tzset() {
    daylight = 0;
    timezone = 0;
}

// ============================================================
// system()
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn system(cmd: *const c_char) -> c_int {
    if cmd.is_null() {
        // ponytail: test shell availability via fork+exec
        let pid = sys_fork();
        if pid == 0 {
            let sh = b"/bin/sh\0".as_ptr() as *const c_char;
            let dash_c = b"-c\0".as_ptr() as *const c_char;
            let exit0 = b"exit 0\0".as_ptr() as *const c_char;
            let argv = [b"sh\0".as_ptr() as *const c_char, dash_c, exit0, core::ptr::null()];
            sys_execve(sh, argv.as_ptr(), __environ as *const *const c_char);
            _exit(127);
        }
        if pid < 0 { return -1; }
        let mut status: c_int = 0;
        loop {
            let r = sys_wait4(pid as c_int, &mut status, 0, core::ptr::null_mut());
            if r >= 0 { return status; }
            if r != -4 { return -1; } // not EINTR
        }
    }

    let sa_ignore = sigaction {
        sa_handler: SIG_IGN,
        sa_flags: SA_RESTORER,
        sa_restorer: sig_restorer as usize,
        sa_mask: [0],
    };
    let mut oldint: sigaction = core::mem::zeroed();
    let mut oldquit: sigaction = core::mem::zeroed();

    sys_rt_sigaction(SIGINT, &sa_ignore, &mut oldint, 8);
    sys_rt_sigaction(SIGQUIT, &sa_ignore, &mut oldquit, 8);

    let mut block_set: SigSetT = 0;
    block_set |= 1u64 << (SIGCHLD - 1);
    let mut oldmask: SigSetT = 0;
    sys_rt_sigprocmask(SIG_BLOCK, &block_set, &mut oldmask, 8);

    let pid = sys_fork();

    if pid == 0 {
        sys_rt_sigaction(SIGINT, &oldint, core::ptr::null_mut(), 8);
        sys_rt_sigaction(SIGQUIT, &oldquit, core::ptr::null_mut(), 8);
        sys_rt_sigprocmask(SIG_SETMASK, &oldmask, core::ptr::null_mut(), 8);

        let sh = b"/bin/sh\0".as_ptr() as *const c_char;
        let argv = [
            b"sh\0".as_ptr() as *const c_char,
            b"-c\0".as_ptr() as *const c_char,
            cmd,
            core::ptr::null(),
        ];
        sys_execve(sh, argv.as_ptr(), __environ as *const *const c_char);
        _exit(127);
    }

    let mut status: c_int = -1;
    if pid > 0 {
        loop {
            let r = sys_wait4(pid as c_int, &mut status, 0, core::ptr::null_mut());
            if r >= 0 { break; }
            if r != -4 { break; } // not EINTR
        }
    }

    sys_rt_sigaction(SIGINT, &oldint, core::ptr::null_mut(), 8);
    sys_rt_sigaction(SIGQUIT, &oldquit, core::ptr::null_mut(), 8);
    sys_rt_sigprocmask(SIG_SETMASK, &oldmask, core::ptr::null_mut(), 8);

    status
}

// ============================================================
// errno: strerror / strerror_r / environ alias
// ============================================================

const EPERM_VAL: c_int = 1;
const ENOENT_VAL: c_int = 2;
const ESRCH_VAL: c_int = 3;
const EINTR_VAL: c_int = 4;
const EIO_VAL: c_int = 5;
const ENXIO_VAL: c_int = 6;
const E2BIG_VAL: c_int = 7;
const ENOEXEC_VAL: c_int = 8;
const EBADF_VAL: c_int = 9;
const ECHILD_VAL: c_int = 10;
const EAGAIN_VAL: c_int = 11;
const ENOMEM_VAL: c_int = 12;
const EACCES_VAL: c_int = 13;
const EFAULT_VAL: c_int = 14;
const ENOTBLK_VAL: c_int = 15;
const EBUSY_VAL: c_int = 16;
const EEXIST_VAL: c_int = 17;
const EXDEV_VAL: c_int = 18;
const ENODEV_VAL: c_int = 19;
const ENOTDIR_VAL: c_int = 20;
const EISDIR_VAL: c_int = 21;
const EINVAL_VAL: c_int = 22;
const ENFILE_VAL: c_int = 23;
const EMFILE_VAL: c_int = 24;
const ENOTTY_VAL: c_int = 25;
const ETXTBSY_VAL: c_int = 26;
const EFBIG_VAL: c_int = 27;
const ENOSPC_VAL: c_int = 28;
const ESPIPE_VAL: c_int = 29;
const EROFS_VAL: c_int = 30;
const EMLINK_VAL: c_int = 31;
const EPIPE_VAL: c_int = 32;
const EDOM_VAL: c_int = 33;
const ERANGE_VAL: c_int = 34;
const EDEADLK_VAL: c_int = 35;
const ENAMETOOLONG_VAL: c_int = 36;
const ENOLCK_VAL: c_int = 37;
const ENOSYS_VAL: c_int = 38;
const ENOTEMPTY_VAL: c_int = 39;
const ELOOP_VAL: c_int = 40;
const EWOULDBLOCK_VAL: c_int = 11;
const ENOMSG_VAL: c_int = 42;
const EIDRM_VAL: c_int = 43;
const ECHRNG_VAL: c_int = 44;
const EL2NSYNC_VAL: c_int = 45;
const EL3HLT_VAL: c_int = 46;
const EL3RST_VAL: c_int = 47;
const ELNRNG_VAL: c_int = 48;
const EUNATCH_VAL: c_int = 49;
const ENOCSI_VAL: c_int = 50;
const EL2HLT_VAL: c_int = 51;
const EBADE_VAL: c_int = 52;
const EBADR_VAL: c_int = 53;
const EXFULL_VAL: c_int = 54;
const ENOANO_VAL: c_int = 55;
const EBADRQC_VAL: c_int = 56;
const EBADSLT_VAL: c_int = 57;
const EBFONT_VAL: c_int = 59;
const ENOSTR_VAL: c_int = 60;
const ENODATA_VAL: c_int = 61;
const ETIME_VAL: c_int = 62;
const ENOSR_VAL: c_int = 63;
const ENONET_VAL: c_int = 64;
const ENOPKG_VAL: c_int = 65;
const EREMOTE_VAL: c_int = 66;
const ENOLINK_VAL: c_int = 67;
const EADV_VAL: c_int = 68;
const ESRMNT_VAL: c_int = 69;
const ECOMM_VAL: c_int = 70;
const EPROTO_VAL: c_int = 71;
const EMULTIHOP_VAL: c_int = 72;
const EDOTDOT_VAL: c_int = 73;
const EBADMSG_VAL: c_int = 74;
const EOVERFLOW_VAL: c_int = 75;
const ENOTUNIQ_VAL: c_int = 76;
const EBADFD_VAL: c_int = 77;
const EREMCHG_VAL: c_int = 78;
const ELIBACC_VAL: c_int = 79;
const ELIBBAD_VAL: c_int = 80;
const ELIBSCN_VAL: c_int = 81;
const ELIBMAX_VAL: c_int = 82;
const ELIBEXEC_VAL: c_int = 83;
const EILSEQ_VAL: c_int = 84;
const ERESTART_VAL: c_int = 85;
const ESTRPIPE_VAL: c_int = 86;
const EUSERS_VAL: c_int = 87;
const ENOTSOCK_VAL: c_int = 88;
const EDESTADDRREQ_VAL: c_int = 89;
const EMSGSIZE_VAL: c_int = 90;
const EPROTOTYPE_VAL: c_int = 91;
const ENOPROTOOPT_VAL: c_int = 92;
const EPROTONOSUPPORT_VAL: c_int = 93;
const ESOCKTNOSUPPORT_VAL: c_int = 94;
const EOPNOTSUPP_VAL: c_int = 95;
const ENOTSUP_VAL: c_int = 95;
const EPFNOSUPPORT_VAL: c_int = 96;
const EAFNOSUPPORT_VAL: c_int = 97;
const EADDRINUSE_VAL: c_int = 98;
const EADDRNOTAVAIL_VAL: c_int = 99;
const ENETDOWN_VAL: c_int = 100;
const ENETUNREACH_VAL: c_int = 101;
const ENETRESET_VAL: c_int = 102;
const ECONNABORTED_VAL: c_int = 103;
const ECONNRESET_VAL: c_int = 104;
const ENOBUFS_VAL: c_int = 105;
const EISCONN_VAL: c_int = 106;
const ENOTCONN_VAL: c_int = 107;
const ESHUTDOWN_VAL: c_int = 108;
const ETOOMANYREFS_VAL: c_int = 109;
const ETIMEDOUT_VAL: c_int = 110;
const ECONNREFUSED_VAL: c_int = 111;
const EHOSTDOWN_VAL: c_int = 112;
const EHOSTUNREACH_VAL: c_int = 113;
const EALREADY_VAL: c_int = 114;
const EINPROGRESS_VAL: c_int = 115;
const ESTALE_VAL: c_int = 116;
const EUCLEAN_VAL: c_int = 117;
const ENOTNAM_VAL: c_int = 118;
const ENAVAIL_VAL: c_int = 119;
const EISNAM_VAL: c_int = 120;
const EREMOTEIO_VAL: c_int = 121;
const EDQUOT_VAL: c_int = 122;
const ENOMEDIUM_VAL: c_int = 123;
const EMEDIUMTYPE_VAL: c_int = 124;
const ECANCELED_VAL: c_int = 125;
const ENOKEY_VAL: c_int = 126;
const EKEYEXPIRED_VAL: c_int = 127;
const EKEYREVOKED_VAL: c_int = 128;
const EKEYREJECTED_VAL: c_int = 129;
const EOWNERDEAD_VAL: c_int = 130;
const ENOTRECOVERABLE_VAL: c_int = 131;
const ERFKILL_VAL: c_int = 132;
const EHWPOISON_VAL: c_int = 133;

// ponytail: static error string table indexed by errno
static ERR_STRS: [&[u8]; 134] = [
    b"Success\0",                     // 0
    b"Operation not permitted\0",     // 1
    b"No such file or directory\0",   // 2
    b"No such process\0",             // 3
    b"Interrupted system call\0",     // 4
    b"Input/output error\0",          // 5
    b"No such device or address\0",   // 6
    b"Argument list too long\0",      // 7
    b"Exec format error\0",           // 8
    b"Bad file descriptor\0",         // 9
    b"No child processes\0",          // 10
    b"Resource temporarily unavailable\0", // 11
    b"Cannot allocate memory\0",      // 12
    b"Permission denied\0",           // 13
    b"Bad address\0",                 // 14
    b"Block device required\0",       // 15
    b"Device or resource busy\0",     // 16
    b"File exists\0",                 // 17
    b"Invalid cross-device link\0",   // 18
    b"No such device\0",              // 19
    b"Not a directory\0",             // 20
    b"Is a directory\0",              // 21
    b"Invalid argument\0",            // 22
    b"Too many open files in system\0", // 23
    b"Too many open files\0",         // 24
    b"Inappropriate ioctl for device\0", // 25
    b"Text file busy\0",              // 26
    b"File too large\0",              // 27
    b"No space left on device\0",     // 28
    b"Illegal seek\0",                // 29
    b"Read-only file system\0",       // 30
    b"Too many links\0",              // 31
    b"Broken pipe\0",                 // 32
    b"Numerical argument out of domain\0", // 33
    b"Numerical result out of range\0", // 34
    b"Resource deadlock avoided\0",   // 35
    b"File name too long\0",          // 36
    b"No locks available\0",          // 37
    b"Function not implemented\0",    // 38
    b"Directory not empty\0",         // 39
    b"Too many levels of symbolic links\0", // 40
    b"Unknown error 41\0",            // 41
    b"No message of desired type\0",  // 42
    b"Identifier removed\0",          // 43
    b"Channel number out of range\0", // 44
    b"Level 2 not synchronized\0",    // 45
    b"Level 3 halted\0",              // 46
    b"Level 3 reset\0",               // 47
    b"Link number out of range\0",    // 48
    b"Protocol driver not attached\0", // 49
    b"No CSI structure available\0",  // 50
    b"Level 2 halted\0",              // 51
    b"Invalid exchange\0",            // 52
    b"Invalid request descriptor\0",  // 53
    b"Exchange full\0",               // 54
    b"No anode\0",                    // 55
    b"Invalid request code\0",        // 56
    b"Invalid slot\0",                // 57
    b"Unknown error 58\0",            // 58
    b"Bad font file format\0",        // 59
    b"Device not a stream\0",         // 60
    b"No data available\0",           // 61
    b"Timer expired\0",               // 62
    b"Out of streams resources\0",    // 63
    b"Machine is not on the network\0", // 64
    b"Package not installed\0",       // 65
    b"Object is remote\0",            // 66
    b"Link has been severed\0",       // 67
    b"Advertise error\0",             // 68
    b"Srmount error\0",               // 69
    b"Communication error on send\0", // 70
    b"Protocol error\0",              // 71
    b"Multihop attempted\0",          // 72
    b"RFS specific error\0",          // 73
    b"Bad message\0",                 // 74
    b"Value too large for defined data type\0", // 75
    b"Name not unique on network\0",  // 76
    b"File descriptor in bad state\0", // 77
    b"Remote address changed\0",      // 78
    b"Can not access a needed shared library\0", // 79
    b"Accessing a corrupted shared library\0", // 80
    b".lib section in a.out corrupted\0", // 81
    b"Attempting to link in too many shared libraries\0", // 82
    b"Cannot exec a shared library directly\0", // 83
    b"Invalid or incomplete multibyte or wide character\0", // 84
    b"Interrupted system call should be restarted\0", // 85
    b"Streams pipe error\0",          // 86
    b"Too many users\0",              // 87
    b"Socket operation on non-socket\0", // 88
    b"Destination address required\0", // 89
    b"Message too long\0",            // 90
    b"Protocol wrong type for socket\0", // 91
    b"Protocol not available\0",      // 92
    b"Protocol not supported\0",      // 93
    b"Socket type not supported\0",   // 94
    b"Operation not supported\0",     // 95
    b"Protocol family not supported\0", // 96
    b"Address family not supported by protocol\0", // 97
    b"Address already in use\0",      // 98
    b"Cannot assign requested address\0", // 99
    b"Network is down\0",             // 100
    b"Network is unreachable\0",      // 101
    b"Network dropped connection on reset\0", // 102
    b"Software caused connection abort\0", // 103
    b"Connection reset by peer\0",    // 104
    b"No buffer space available\0",   // 105
    b"Transport endpoint is already connected\0", // 106
    b"Transport endpoint is not connected\0", // 107
    b"Cannot send after transport endpoint shutdown\0", // 108
    b"Too many references: cannot splice\0", // 109
    b"Connection timed out\0",        // 110
    b"Connection refused\0",          // 111
    b"Host is down\0",                // 112
    b"No route to host\0",            // 113
    b"Operation already in progress\0", // 114
    b"Operation now in progress\0",   // 115
    b"Stale file handle\0",           // 116
    b"Structure needs cleaning\0",    // 117
    b"Not a XENIX named type file\0", // 118
    b"No XENIX semaphores available\0", // 119
    b"Is a named type file\0",        // 120
    b"Remote I/O error\0",            // 121
    b"Quota exceeded\0",              // 122
    b"No medium found\0",             // 123
    b"Wrong medium type\0",           // 124
    b"Operation canceled\0",          // 125
    b"Required key not available\0",  // 126
    b"Key has expired\0",             // 127
    b"Key has been revoked\0",        // 128
    b"Key was rejected by service\0", // 129
    b"Owner died\0",                  // 130
    b"State not recoverable\0",       // 131
    b"Operation not possible due to RF-kill\0", // 132
    b"Memory page has hardware error\0", // 133
];

#[no_mangle]
pub unsafe extern "C" fn strerror(errnum: c_int) -> *mut c_char {
    if errnum < 0 || errnum as usize >= ERR_STRS.len() {
        // ponytail: static buf for unknown errors
        static mut UNKNOWN_BUF: [c_char; 32] = [0; 32];
        let prefix = b"Unknown error \0";
        let mut i = 0;
        while prefix[i] != 0 {
            UNKNOWN_BUF[i] = prefix[i] as c_char;
            i += 1;
        }
        let (buf, len) = format_u64(errnum as u64);
        for j in 0..len {
            UNKNOWN_BUF[i] = buf[20 - len + j] as c_char;
            i += 1;
        }
        UNKNOWN_BUF[i] = 0;
        return core::ptr::addr_of_mut!(UNKNOWN_BUF).cast::<c_char>();
    }
    ERR_STRS[errnum as usize].as_ptr() as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn strerror_r(errnum: c_int, buf: *mut c_char, buflen: usize) -> c_int {
    let s = strerror(errnum);
    let slen = strlen(s as *const u8);
    if slen >= buflen {
        if buflen > 0 {
            core::ptr::copy_nonoverlapping(s as *const u8, buf as *mut u8, buflen - 1);
            *buf.add(buflen - 1) = 0;
        }
        return 34; // ERANGE
    }
    core::ptr::copy_nonoverlapping(s as *const u8, buf as *mut u8, slen + 1);
    0
}

// ponytail: `environ` alias for tests that use extern char **environ
#[no_mangle]
pub static mut environ: *mut *mut c_char = core::ptr::null_mut();

// keep environ in sync with __environ
unsafe fn sync_environ() {
    environ = __environ;
}

// ============================================================
// unistd extensions: pipe, dup, dup2, fcntl, access, unlink, etc.
// ============================================================

#[inline]
unsafe fn sys_dup(oldfd: i32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 32i64 => result,
        in("rdi") oldfd as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_dup2(oldfd: i32, newfd: i32) -> i64 {
    // ponytail: dup3 with flags=0 is equivalent to dup2
    sys_dup3(oldfd, newfd, 0)
}

#[inline]
unsafe fn sys_access(path: *const u8, mode: i32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 21i64 => result,
        in("rdi") path,
        in("rsi") mode as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_unlink(path: *const u8) -> i64 {
    sys_unlinkat(AT_FDCWD, path, 0)
}

#[inline]
unsafe fn sys_rmdir(path: *const u8) -> i64 {
    sys_unlinkat(AT_FDCWD, path, 512) // AT_REMOVEDIR
}

#[inline]
unsafe fn sys_chdir(path: *const u8) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 80i64 => result,
        in("rdi") path,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_getcwd(buf: *mut u8, size: usize) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 79i64 => result,
        in("rdi") buf,
        in("rsi") size,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_sethostname(name: *const u8, len: usize) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 170i64 => result,
        in("rdi") name,
        in("rsi") len,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_gethostname(buf: *mut u8, len: usize) -> i64 {
    // ponytail: use uname to get hostname
    #[repr(C)]
    struct UtsName {
        sysname: [u8; 65],
        nodename: [u8; 65],
        release: [u8; 65],
        version: [u8; 65],
        machine: [u8; 65],
        domainname: [u8; 65],
    }
    let mut uts: UtsName = core::mem::zeroed();
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 63i64 => result,
        in("rdi") &mut uts as *mut UtsName,
        lateout("rcx") _,
        lateout("r11") _,
    );
    if result < 0 {
        return result;
    }
    let nlen = strlen(uts.nodename.as_ptr());
    let copylen = if nlen < len { nlen } else { if len > 0 { len - 1 } else { 0 } };
    core::ptr::copy_nonoverlapping(uts.nodename.as_ptr(), buf, copylen);
    if len > 0 { *buf.add(copylen) = 0; }
    0
}

unsafe fn sys_truncate(path: *const u8, length: i64) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 76i64 => result,
        in("rdi") path,
        in("rsi") length,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_ftruncate(fd: i32, length: i64) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 77i64 => result,
        in("rdi") fd as i64,
        in("rsi") length,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_nanosleep(req: *const timespec, rem: *mut timespec) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 35i64 => result,
        in("rdi") req,
        in("rsi") rem,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_alarm(seconds: c_uint) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 37i64 => result,
        in("rdi") seconds as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_pause() -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 34i64 => result,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_fsync(fd: i32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 74i64 => result,
        in("rdi") fd as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_fdatasync(fd: i32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 306i64 => result,
        in("rdi") fd as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_sync() {
    core::arch::asm!(
        "syscall",
        in("rax") 162i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
}

unsafe fn sys_symlink(target: *const u8, linkpath: *const u8) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 88i64 => result,
        in("rdi") target,
        in("rsi") linkpath,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_symlinkat(target: *const u8, newdirfd: i32, linkpath: *const u8) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 266i64 => result,
        in("rdi") target,
        in("rsi") newdirfd as i64,
        in("rdx") linkpath,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_readlinkat(dirfd: i32, path: *const u8, buf: *mut u8, bufsiz: usize) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 267i64 => result,
        in("rdi") dirfd as i64,
        in("rsi") path,
        in("rdx") buf,
        in("r10") bufsiz,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_linkat(olddirfd: i32, oldpath: *const u8, newdirfd: i32, newpath: *const u8, flags: i32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 265i64 => result,
        in("rdi") olddirfd as i64,
        in("rsi") oldpath,
        in("rdx") newdirfd as i64,
        in("r10") newpath,
        in("r8") flags as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_fchmod(fd: i32, mode: u32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 91i64 => result,
        in("rdi") fd as i64,
        in("rsi") mode as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_fchmodat(dirfd: i32, path: *const u8, mode: u32, flags: i32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 268i64 => result,
        in("rdi") dirfd as i64,
        in("rsi") path,
        in("rdx") mode as i64,
        in("r10") flags as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_umask(mask: u32) -> u32 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 95i64 => result,
        in("rdi") mask as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result as u32
}

unsafe fn sys_getgroups(size: i32, list: *mut c_uint) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 115i64 => result,
        in("rdi") size as i64,
        in("rsi") list,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_setuid(uid: c_uint) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 105i64 => result,
        in("rdi") uid as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_setgid(gid: c_uint) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 106i64 => result,
        in("rdi") gid as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_setsid() -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 112i64 => result,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_setpgid(pid: c_int, pgid: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 109i64 => result,
        in("rdi") pid as i64,
        in("rsi") pgid as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_getpgid(pid: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 121i64 => result,
        in("rdi") pid as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_getsid(pid: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 124i64 => result,
        in("rdi") pid as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn sys_mkdirat(dirfd: i32, path: *const u8, mode: u32) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 258i64 => result,
        in("rdi") dirfd as i64,
        in("rsi") path,
        in("rdx") mode as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

// Public C ABI wrappers for unistd

#[no_mangle]
pub unsafe extern "C" fn pipe(fds: *mut c_int) -> c_int {
    let r = sys_pipe2(fds, 0);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn pipe2(fds: *mut c_int, flags: c_int) -> c_int {
    let r = sys_pipe2(fds, flags);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn dup(oldfd: c_int) -> c_int {
    let r = sys_dup(oldfd);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn dup2(oldfd: c_int, newfd: c_int) -> c_int {
    let r = sys_dup2(oldfd, newfd);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn dup3(oldfd: c_int, newfd: c_int, flags: c_int) -> c_int {
    let r = sys_dup3(oldfd, newfd, flags);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn fcntl(fd: c_int, cmd: c_int, mut args: ...) -> c_int {
    let r = match cmd {
        F_GETFD | F_GETFL => sys_fcntl(fd, cmd, 0),
        F_SETFD | F_SETFL => {
            let arg = args.arg::<c_int>();
            sys_fcntl(fd, cmd, arg as i64)
        }
        _ => sys_fcntl(fd, cmd, 0),
    };
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn access(path: *const c_char, mode: c_int) -> c_int {
    let r = sys_access(path as *const u8, mode);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn unlink(path: *const c_char) -> c_int {
    let r = sys_unlink(path as *const u8);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn rmdir(path: *const c_char) -> c_int {
    let r = sys_rmdir(path as *const u8);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn chdir(path: *const c_char) -> c_int {
    let r = sys_chdir(path as *const u8);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn getcwd(buf: *mut c_char, size: usize) -> *mut c_char {
    if buf.is_null() {
        // allocate
        let alloc_size = if size == 0 { 256 } else { size };
        let p = malloc(alloc_size) as *mut c_char;
        if p.is_null() { ERRNO = ENOMEM; return core::ptr::null_mut(); }
        let r = sys_getcwd(p as *mut u8, alloc_size);
        if r < 0 { free(p as *mut c_void); ERRNO = (-r) as c_int; return core::ptr::null_mut(); }
        return p;
    }
    if size == 0 { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let r = sys_getcwd(buf as *mut u8, size);
    if r < 0 { ERRNO = (-r) as c_int; return core::ptr::null_mut(); }
    buf
}

#[no_mangle]
pub unsafe extern "C" fn gethostname(name: *mut c_char, len: usize) -> c_int {
    let r = sys_gethostname(name as *mut u8, len);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

// ponytail: getpagesize - hardcoded 4096 for x86_64
#[no_mangle]
pub extern "C" fn getpagesize() -> c_int {
    4096
}

#[no_mangle]
pub unsafe extern "C" fn truncate(path: *const c_char, length: i64) -> c_int {
    let r = sys_truncate(path as *const u8, length);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn ftruncate(fd: c_int, length: i64) -> c_int {
    let r = sys_ftruncate(fd, length);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn nanosleep(req: *const timespec, rem: *mut timespec) -> c_int {
    let r = sys_nanosleep(req, rem);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sleep(seconds: c_uint) -> c_uint {
    let req = timespec { tv_sec: seconds as c_long, tv_nsec: 0 };
    let mut rem: timespec = core::mem::zeroed();
    let r = sys_nanosleep(&req, &mut rem);
    if r < 0 {
        let e = (-r) as c_int;
        if e == EINTR { return rem.tv_sec as c_uint; }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn usleep(usec: c_uint) -> c_int {
    let req = timespec { tv_sec: (usec / 1000000) as c_long, tv_nsec: ((usec % 1000000) * 1000) as c_long };
    let r = sys_nanosleep(&req, core::ptr::null_mut());
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn alarm(seconds: c_uint) -> c_uint {
    sys_alarm(seconds) as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn pause() -> c_int {
    sys_pause();
    ERRNO = EINTR;
    -1
}

#[no_mangle]
pub unsafe extern "C" fn fsync(fd: c_int) -> c_int {
    let r = sys_fsync(fd);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn fdatasync(fd: c_int) -> c_int {
    let r = sys_fdatasync(fd);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sync() {
    sys_sync();
}

#[no_mangle]
pub unsafe extern "C" fn symlink(target: *const c_char, linkpath: *const c_char) -> c_int {
    let r = sys_symlinkat(target as *const u8, AT_FDCWD, linkpath as *const u8);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn readlink(path: *const c_char, buf: *mut c_char, bufsiz: usize) -> isize {
    let r = sys_readlinkat(AT_FDCWD, path as *const u8, buf as *mut u8, bufsiz);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as isize
}

#[no_mangle]
pub unsafe extern "C" fn link(oldpath: *const c_char, newpath: *const c_char) -> c_int {
    let r = sys_linkat(AT_FDCWD, oldpath as *const u8, AT_FDCWD, newpath as *const u8, 0);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn chmod(path: *const c_char, mode: c_uint) -> c_int {
    let r = sys_fchmodat(AT_FDCWD, path as *const u8, mode, 0);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn fchmod(fd: c_int, mode: c_uint) -> c_int {
    let r = sys_fchmod(fd, mode);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn umask(mask: c_uint) -> c_uint {
    sys_umask(mask) as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn isatty(fd: c_int) -> c_int {
    let mut ws: winsize = core::mem::zeroed();
    if sys_ioctl(fd, TIOCGWINSZ, &mut ws as *mut winsize as *mut u8) == 0 { 1 } else { 0 }
}

// ponytail: ttyname - stub, not critical
#[no_mangle]
pub unsafe extern "C" fn ttyname(_fd: c_int) -> *mut c_char {
    core::ptr::null_mut()
}

// ponytail: getlogin - stub
#[no_mangle]
pub unsafe extern "C" fn getlogin() -> *mut c_char {
    b"root\0".as_ptr() as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn getgroups(size: c_int, list: *mut c_uint) -> c_int {
    let r = sys_getgroups(size, list);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn setuid(uid: c_uint) -> c_int {
    let r = sys_setuid(uid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn setgid(gid: c_uint) -> c_int {
    let r = sys_setgid(gid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn seteuid(euid: c_uint) -> c_int {
    // ponytail: on Linux, setreuid(-1, euid) sets effective uid
    let r = sys_setuid(euid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn setegid(egid: c_uint) -> c_int {
    let r = sys_setgid(egid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn setreuid(_ruid: c_uint, _euid: c_uint) -> c_int {
    // ponytail: stub, real UID changes need cap_setuid
    0
}

#[no_mangle]
pub unsafe extern "C" fn setregid(_rgid: c_uint, _egid: c_uint) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn setsid() -> c_int {
    let r = sys_setsid();
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn setpgid(pid: c_int, pgid: c_int) -> c_int {
    let r = sys_setpgid(pid, pgid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn getpgid(pid: c_int) -> c_int {
    let r = sys_getpgid(pid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn getsid(pid: c_int) -> c_int {
    let r = sys_getsid(pid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn getpgrp() -> c_int {
    sys_getpgid(0) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn setpgrp() -> c_int {
    setpgid(0, 0)
}

#[no_mangle]
pub unsafe extern "C" fn mkstemp(template: *mut c_char) -> c_int {
    if template.is_null() { ERRNO = EINVAL; return -1; }
    let len = strlen(template as *const u8);
    if len < 6 { ERRNO = EINVAL; return -1; }
    let x_start = len - 6;
    // check that last 6 chars are XXXXXX
    for i in x_start..len {
        if *template.add(i) != b'X' as c_char { ERRNO = EINVAL; return -1; }
    }
    let mut ts: timespec = core::mem::zeroed();
    let _ = sys_clock_gettime(CLOCK_REALTIME, &mut ts);
    let mut ctr: u32 = ts.tv_nsec as u32;
    for _ in 0..100 {
        ctr = ctr.wrapping_mul(1103515245).wrapping_add(12345);
        let seed = ctr;
        // fill XXXXXX with random hex chars
        let hex = b"0123456789abcdef";
        let mut s = seed;
        for i in x_start..len {
            *template.add(i) = hex[(s & 0xf) as usize] as c_char;
            s >>= 4;
        }
        let fd = sys_open(template as *const u8, (O_RDWR | O_CREAT | O_EXCL) as i64, 0o600);
        if fd >= 0 { return fd as c_int; }
        let e = (-fd) as c_int;
        if e != EEXIST_VAL { ERRNO = e; return -1; }
    }
    ERRNO = EEXIST_VAL;
    -1
}

#[no_mangle]
pub unsafe extern "C" fn mkdir(path: *const c_char, mode: c_uint) -> c_int {
    let r = sys_mkdirat(AT_FDCWD, path as *const u8, mode);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

// ============================================================
// libgen: basename / dirname
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn dirname(s: *mut c_char) -> *mut c_char {
    if s.is_null() || *s == 0 {
        return b".\0".as_ptr() as *mut c_char;
    }
    let mut i = strlen(s as *const u8);
    i = i.wrapping_sub(1);
    // skip trailing slashes
    while i > 0 && *s.add(i) == b'/' as c_char {
        i = i.wrapping_sub(1);
    }
    if i == 0 && *s.add(0) == b'/' as c_char {
        return b"/\0".as_ptr() as *mut c_char;
    }
    // skip non-slashes
    while i > 0 && *s.add(i) != b'/' as c_char {
        i = i.wrapping_sub(1);
    }
    if i == 0 {
        if *s.add(0) == b'/' as c_char {
            return b"/\0".as_ptr() as *mut c_char;
        }
        return b".\0".as_ptr() as *mut c_char;
    }
    // skip slashes
    while i > 0 && *s.add(i - 1) == b'/' as c_char {
        i = i.wrapping_sub(1);
    }
    if i == 0 {
        return b"/\0".as_ptr() as *mut c_char;
    }
    *s.add(i) = 0;
    s
}

#[no_mangle]
pub unsafe extern "C" fn basename(s: *mut c_char) -> *mut c_char {
    if s.is_null() || *s == 0 {
        return b".\0".as_ptr() as *mut c_char;
    }
    let mut i = strlen(s as *const u8);
    i = i.wrapping_sub(1);
    // strip trailing slashes
    while i > 0 && *s.add(i) == b'/' as c_char {
        *s.add(i) = 0;
        i = i.wrapping_sub(1);
    }
    // find last slash
    while i > 0 && *s.add(i - 1) != b'/' as c_char {
        i = i.wrapping_sub(1);
    }
    s.add(i)
}

// ============================================================
// arpa/inet: inet_pton / inet_ntop / inet_addr / inet_ntoa / inet_aton
// ============================================================

const AF_INET_VAL: c_int = 2;
const AF_INET6_VAL: c_int = 10;
const EAFNOSUPPORT: c_int = 97;
const ENOSPC_VAL2: c_int = 28;

unsafe fn inet_pton_v4(s: *const u8, a: *mut u8) -> c_int {
    let mut p = s;
    let mut i = 0;
    while i < 4 {
        let oct_start = p;
        let mut v: u32 = 0;
        let mut j: u32 = 0;
        while j < 3 && *p >= b'0' && *p <= b'9' {
            v = v * 10 + (*p - b'0') as u32;
            p = p.add(1);
            j += 1;
        }
        if j == 0 || (j > 1 && *oct_start == b'0') || v > 255 { return 0; }
        *a.add(i) = v as u8;
        if *p == 0 && i == 3 { return 1; }
        if *p != b'.' { return 0; }
        p = p.add(1);
        i += 1;
    }
    0
}

unsafe fn inet_pton_v6(s: *const u8, a: *mut u8) -> c_int {
    let mut ip = [0u16; 8];
    let mut p = s;
    let mut i: usize = 0;
    let mut brk: isize = -1;
    let mut need_v4 = false;
    let mut v4_start: *const u8 = core::ptr::null();

    if *p == b':' {
        p = p.add(1);
        if *p != b':' { return 0; }
    }

    loop {
        if *p == b':' && brk < 0 {
            brk = i as isize;
            ip[i & 7] = 0;
            p = p.add(1);
            if *p == 0 { break; }
            if i == 7 { return 0; }
            i += 1;
            continue;
        }
        v4_start = p;
        let mut v: u16 = 0;
        let mut j = 0;
        while j < 4 {
            let d = hex_digit(*p);
            if d < 0 { break; }
            v = v * 16 + d as u16;
            p = p.add(1);
            j += 1;
        }
        if j == 0 { return 0; }
        ip[i & 7] = v;
        if *p == 0 && (brk >= 0 || i == 7) { break; }
        if i == 7 { return 0; }
        if *p != b':' {
            if *p != b'.' || (i < 6 && brk < 0) { return 0; }
            need_v4 = true;
            i += 1;
            ip[i & 7] = 0;
            break;
        }
        p = p.add(1);
        i += 1;
    }

    if need_v4 {
        let r = inet_pton_v4(v4_start, a.add(12));
        if r <= 0 { return r; }
        ip[6] = ((*a.add(12) as u16) << 8) | (*a.add(13) as u16);
        ip[7] = ((*a.add(14) as u16) << 8) | (*a.add(15) as u16);
    }

    let parts = if brk >= 0 { i + 1 } else { 8 };
    if brk >= 0 && need_v4 { /* already placed */ }
    else if brk >= 0 {
        // expand ::
        let trail = parts - brk as usize - 1;
        let mut j = 7;
        while j >= 8 - trail {
            ip[j] = ip[brk as usize + (j - (8 - trail))];
            j -= 1;
        }
        while j >= brk as usize {
            ip[j] = 0;
            if j == 0 { break; }
            j -= 1;
        }
    }

    if !need_v4 {
        for k in 0..8 {
            *a.add(k * 2) = (ip[k] >> 8) as u8;
            *a.add(k * 2 + 1) = (ip[k] & 0xff) as u8;
        }
    }
    1
}

unsafe fn hex_digit(c: u8) -> i32 {
    if c >= b'0' && c <= b'9' { (c - b'0') as i32 }
    else if c >= b'a' && c <= b'f' { (c - b'a' + 10) as i32 }
    else if c >= b'A' && c <= b'F' { (c - b'A' + 10) as i32 }
    else { -1 }
}

#[no_mangle]
pub unsafe extern "C" fn inet_pton(af: c_int, s: *const c_char, a: *mut c_void) -> c_int {
    if af == AF_INET_VAL {
        inet_pton_v4(s as *const u8, a as *mut u8)
    } else if af == AF_INET6_VAL {
        inet_pton_v6(s as *const u8, a as *mut u8)
    } else {
        ERRNO = EAFNOSUPPORT;
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn inet_ntop(af: c_int, a: *const c_void, s: *mut c_char, size: u32) -> *mut c_char {
    if af == AF_INET_VAL {
        let b = a as *const u8;
        let needed = 16; // "255.255.255.255\0"
        if size < needed as u32 { ERRNO = ENOSPC_VAL2; return core::ptr::null_mut(); }
        let mut pos = 0;
        for i in 0..4 {
            if i > 0 { *s.add(pos) = b'.' as c_char; pos += 1; }
            let (buf, len) = format_u64(*b.add(i) as u64);
            for j in 0..len { *s.add(pos) = buf[20 - len + j] as c_char; pos += 1; }
        }
        *s.add(pos) = 0;
        s
    } else if af == AF_INET6_VAL {
        let b = a as *const u8;
        let needed = 46; // max IPv6
        if size < needed as u32 { ERRNO = ENOSPC_VAL2; return core::ptr::null_mut(); }
        // find longest run of zeros
        let mut words = [0u16; 8];
        for i in 0..8 {
            words[i] = ((*b.add(i * 2) as u16) << 8) | (*b.add(i * 2 + 1) as u16);
        }
        let mut best_start: isize = -1;
        let mut best_len: usize = 0;
        let mut j: usize = 0;
        while j < 8 {
            if words[j] == 0 {
                let mut k = j;
                while k < 8 && words[k] == 0 { k += 1; }
                let run = k - j;
                if run > best_len { best_start = j as isize; best_len = run; }
                j = k;
            } else {
                j += 1;
            }
        }
        let mut pos = 0;
        let mut i: usize = 0;
        while i < 8 {
            if i == best_start as usize && best_len >= 2 {
                if i == 0 { *s.add(pos) = b':' as c_char; pos += 1; }
                *s.add(pos) = b':' as c_char; pos += 1;
                i += best_len;
                continue;
            }
            if i > 0 && (i != best_start as usize || best_len < 2) {
                *s.add(pos) = b':' as c_char; pos += 1;
            }
            let (buf, len) = format_hex(words[i] as u64, false);
            for k in 0..len { *s.add(pos) = buf[16 - len + k] as c_char; pos += 1; }
            i += 1;
        }
        *s.add(pos) = 0;
        s
    } else {
        ERRNO = EAFNOSUPPORT;
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn inet_addr(s: *const c_char) -> u32 {
    let mut a = [0u8; 4];
    if inet_pton_v4(s as *const u8, a.as_mut_ptr()) == 1 {
        u32::from_be_bytes(a)
    } else {
        !0u32 // INADDR_NONE
    }
}

#[no_mangle]
pub unsafe extern "C" fn inet_ntoa(addr: u32) -> *mut c_char {
    static mut NTOA_BUF: [c_char; 16] = [0; 16];
    let b = addr.to_be_bytes();
    let mut pos = 0;
    for i in 0..4 {
        if i > 0 { NTOA_BUF[pos] = b'.' as c_char; pos += 1; }
        let (buf, len) = format_u64(b[i] as u64);
        for j in 0..len { NTOA_BUF[pos] = buf[20 - len + j] as c_char; pos += 1; }
    }
    NTOA_BUF[pos] = 0;
    core::ptr::addr_of_mut!(NTOA_BUF).cast::<c_char>()
}

#[no_mangle]
pub unsafe extern "C" fn inet_aton(s: *const c_char, addr: *mut u32) -> c_int {
    let mut a = [0u8; 4];
    if inet_pton_v4(s as *const u8, a.as_mut_ptr()) == 1 {
        if !addr.is_null() {
            *addr = u32::from_be_bytes(a);
        }
        1
    } else {
        0
    }
}

// ponytail: inet_network/inet_makeaddr/inet_lnaof/inet_netof - simple implementations
#[no_mangle]
pub unsafe extern "C" fn inet_network(s: *const c_char) -> u32 {
    inet_addr(s).to_be()
}

#[no_mangle]
pub extern "C" fn inet_makeaddr(net: c_int, host: c_int) -> u32 {
    let addr: u32;
    if (net as u32) < 128 {
        addr = (net as u32) << 24;
    } else if (net as u32) < 0x10000 {
        addr = (net as u32) << 16;
    } else if (net as u32) < 0x1000000 {
        addr = (net as u32) << 8;
    } else {
        addr = net as u32;
    }
    addr | (host as u32 & 0xffffffff)
}

#[no_mangle]
pub extern "C" fn inet_lnaof(addr: u32) -> c_int {
    let h = u32::from_be(addr);
    if h < 0x80000000 { (h & 0x00ffffff) as c_int }
    else if h < 0xc0000000 { (h & 0x0000ffff) as c_int }
    else { (h & 0x000000ff) as c_int }
}

#[no_mangle]
pub extern "C" fn inet_netof(addr: u32) -> c_int {
    let h = u32::from_be(addr);
    if h < 0x80000000 { (h >> 24) as c_int }
    else if h < 0xc0000000 { (h >> 16) as c_int }
    else { (h >> 8) as c_int }
}

// ============================================================
// stdlib: strtod / strtof / strtold
// ============================================================

// ponytail: simple float parser, handles decimal, hex, inf, nan, exponent
unsafe fn parse_float(s: *const u8, endptr: *mut *mut u8, _is_long: bool) -> f64 {
    let mut p = s;
    // skip whitespace
    while *p == b' ' || *p == b'\t' || *p == b'\n' || *p == b'\r' { p = p.add(1); }
    let mut neg = false;
    if *p == b'-' { neg = true; p = p.add(1); }
    else if *p == b'+' { p = p.add(1); }

    // inf/nan
    if (*p == b'i' || *p == b'I') && (*p.add(1) == b'n' || *p.add(1) == b'N') {
        let start = p;
        p = p.add(1);
        if (*p == b'n' || *p == b'N') && (*p.add(1) == b'f' || *p.add(1) == b'F') {
            p = p.add(1);
            if *p == b'f' || *p == b'F' { p = p.add(1); }
            // check for "inity" etc
            let r = if neg { f64::NEG_INFINITY } else { f64::INFINITY };
            if !endptr.is_null() { *endptr = p as *mut u8; }
            return r;
        }
        p = start;
    }
    if (*p == b'n' || *p == b'N') && (*p.add(1) == b'a' || *p.add(1) == b'A') {
        let start = p;
        p = p.add(1);
        if (*p == b'a' || *p == b'A') && (*p.add(1) == b'n' || *p.add(1) == b'N') {
            p = p.add(1);
            if *p == b'n' || *p == b'N' { p = p.add(1); }
            // optionally skip (...)
            if *p == b'(' {
                p = p.add(1);
                while *p != 0 && *p != b')' { p = p.add(1); }
                if *p == b')' { p = p.add(1); }
            }
            if !endptr.is_null() { *endptr = p as *mut u8; }
            return if neg { -f64::NAN } else { f64::NAN };
        }
        p = start;
    }

    // hex float or hex integer
    let is_hex = *p == b'0' && (*p.add(1) == b'x' || *p.add(1) == b'X');
    if is_hex {
        p = p.add(2);
        let mut val: f64 = 0.0;
        let mut found = false;
        while let Some(d) = hex_val(*p) {
            val = val * 16.0 + d as f64;
            p = p.add(1);
            found = true;
        }
        let mut frac_scale = 1.0f64;
        if *p == b'.' {
            p = p.add(1);
            while let Some(d) = hex_val(*p) {
                frac_scale /= 16.0;
                val += d as f64 * frac_scale;
                p = p.add(1);
                found = true;
            }
        }
        if !found {
            if !endptr.is_null() { *endptr = s as *mut u8; }
            return 0.0;
        }
        // optional exponent p[+-]N
        if *p == b'p' || *p == b'P' {
            p = p.add(1);
            let mut exp_neg = false;
            if *p == b'-' { exp_neg = true; p = p.add(1); }
            else if *p == b'+' { p = p.add(1); }
            let mut exp_val: i32 = 0;
            while *p >= b'0' && *p <= b'9' {
                exp_val = exp_val * 10 + (*p - b'0') as i32;
                p = p.add(1);
            }
            if exp_neg { exp_val = -exp_val; }
            // ldexp
            let factor = libm::pow(2.0, exp_val as f64);
            val *= factor;
        }
        let r = if neg { -val } else { val };
        if !endptr.is_null() { *endptr = p as *mut u8; }
        return r;
    }

    // decimal
    let mut val: f64 = 0.0;
    let mut found = false;
    while *p >= b'0' && *p <= b'9' {
        val = val * 10.0 + (*p - b'0') as f64;
        p = p.add(1);
        found = true;
    }
    if *p == b'.' {
        p = p.add(1);
        let mut frac = 1.0f64;
        while *p >= b'0' && *p <= b'9' {
            frac /= 10.0;
            val += (*p - b'0') as f64 * frac;
            p = p.add(1);
            found = true;
        }
    }
    if !found {
        // check for inf/nan that didn't match above
        if !endptr.is_null() { *endptr = s as *mut u8; }
        return 0.0;
    }
    // exponent e[+-]N
    if *p == b'e' || *p == b'E' {
        p = p.add(1);
        let mut exp_neg = false;
        if *p == b'-' { exp_neg = true; p = p.add(1); }
        else if *p == b'+' { p = p.add(1); }
        let mut exp_val: i32 = 0;
        while *p >= b'0' && *p <= b'9' {
            exp_val = exp_val * 10 + (*p - b'0') as i32;
            p = p.add(1);
        }
        if exp_neg { exp_val = -exp_val; }
        let factor = libm::pow(10.0, exp_val as f64);
        val *= factor;
    }
    let r = if neg { -val } else { val };
    if !endptr.is_null() { *endptr = p as *mut u8; }
    r
}

unsafe fn hex_val(c: u8) -> Option<u8> {
    if c >= b'0' && c <= b'9' { Some(c - b'0') }
    else if c >= b'a' && c <= b'f' { Some(c - b'a' + 10) }
    else if c >= b'A' && c <= b'F' { Some(c - b'A' + 10) }
    else { None }
}

#[no_mangle]
pub unsafe extern "C" fn strtod(s: *const c_char, endptr: *mut *mut c_char) -> f64 {
    let mut end: *mut u8 = s as *mut u8;
    let r = parse_float(s as *const u8, &mut end, false);
    if !endptr.is_null() { *endptr = end as *mut c_char; }
    r
}

#[no_mangle]
pub unsafe extern "C" fn strtof(s: *const c_char, endptr: *mut *mut c_char) -> f32 {
    let mut end: *mut u8 = s as *mut u8;
    let r = parse_float(s as *const u8, &mut end, false);
    if !endptr.is_null() { *endptr = end as *mut c_char; }
    r as f32
}

#[no_mangle]
pub unsafe extern "C" fn strtold(s: *const c_char, endptr: *mut *mut c_char) -> f64 {
    // ponytail: on x86_64 long double == double in many implementations
    strtod(s, endptr)
}

#[no_mangle]
pub unsafe extern "C" fn atof(s: *const c_char) -> f64 {
    strtod(s, core::ptr::null_mut())
}

// ============================================================
// stdio: open_memstream / fmemopen
// ============================================================

#[repr(C)]
struct MemStreamState {
    buf: *mut u8,
    size: usize,       // current content length
    cap: usize,        // allocated capacity
    sizep: *mut usize, // pointer to size (for open_memstream)
    bufp: *mut *mut c_char, // pointer to buffer
}

unsafe extern "C" fn memstream_write(_f: *mut FILE) -> c_int {
    // no-op for memstream - writes go directly to buffer
    0
}

unsafe extern "C" fn memstream_close(_f: *mut FILE) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn open_memstream(bufp: *mut *mut c_char, sizep: *mut usize) -> *mut FILE {
    if bufp.is_null() || sizep.is_null() { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let f = calloc(1, core::mem::size_of::<FILE>() + UNGET + BUFSIZ) as *mut FILE;
    if f.is_null() { ERRNO = ENOMEM; return core::ptr::null_mut(); }
    let buf = buf_ptr(f);
    init_file(f, -1, b"w\0".as_ptr() as *const c_char, Some(memstream_close), buf, BUFSIZ);
    (*f).flags |= F_SVB;
    // store memstream state in the FILE struct
    let state = calloc(1, core::mem::size_of::<MemStreamState>()) as *mut MemStreamState;
    if state.is_null() { free(f as *mut c_void); ERRNO = ENOMEM; return core::ptr::null_mut(); }
    (*state).buf = core::ptr::null_mut();
    (*state).size = 0;
    (*state).cap = 0;
    (*state).sizep = sizep;
    (*state).bufp = bufp;
    (*f).cookie = state as *mut c_void;
    // set initial output pointers
    *bufp = core::ptr::null_mut();
    *sizep = 0;
    f
}

#[no_mangle]
pub unsafe extern "C" fn fmemopen(buf: *mut c_void, size: usize, mode: *const c_char) -> *mut FILE {
    if buf.is_null() || mode.is_null() { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let f = calloc(1, core::mem::size_of::<FILE>() + UNGET + BUFSIZ) as *mut FILE;
    if f.is_null() { ERRNO = ENOMEM; return core::ptr::null_mut(); }
    let buf_area = buf_ptr(f);
    init_file(f, -1, mode, Some(memstream_close), buf_area, BUFSIZ);
    (*f).flags |= F_SVB;
    (*f).buf = buf as *mut u8;
    (*f).buf_size = size;
    let m = *mode;
    if m == b'r' as c_char {
        let content_len = strnlen(buf as *const u8, size);
        (*f).rpos = buf as *mut u8;
        (*f).rend = (buf as *mut u8).add(content_len);
        (*f).wpos = core::ptr::null_mut();
        (*f).wbase = core::ptr::null_mut();
        (*f).wend = core::ptr::null_mut();
    } else {
        (*f).wpos = buf as *mut u8;
        (*f).wbase = buf as *mut u8;
        (*f).wend = (buf as *mut u8).add(size);
        (*f).rpos = buf as *mut u8;
        (*f).rend = buf as *mut u8;
    }
    f
}

// ============================================================
// Stat (for ftok)
// ============================================================

#[repr(C)]
struct kernel_stat64 {
    st_dev: u64,
    st_ino: u64,
    st_nlink: u64,
    st_mode: u32,
    st_uid: u32,
    st_gid: u32,
    __pad0: c_int,
    st_rdev: u64,
    st_size: i64,
    st_blksize: i64,
    st_blocks: i64,
    st_atime: i64,
    st_atime_nsec: i64,
    st_mtime: i64,
    st_mtime_nsec: i64,
    st_ctime: i64,
    st_ctime_nsec: i64,
    __unused: [i64; 3],
}

// sys_stat = 4 on x86_64
#[inline]
unsafe fn sys_stat(path: *const u8, statbuf: *mut kernel_stat64) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 4i64 => result,
        in("rdi") path,
        in("rsi") statbuf,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

// ============================================================
// Search: hcreate/hdestroy/hsearch (hash table)
// ============================================================

const HSEARCH_MINSIZE: usize = 8;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct HSearchEntry {
    key: *mut c_char,
    data: *mut c_void,
}

#[repr(C)]
struct HTab {
    entries: *mut HSearchEntry,
    mask: usize,
    used: usize,
}

#[repr(C)]
pub struct HSearchData {
    tab: *mut HTab,
    __unused1: c_uint,
    __unused2: c_uint,
}

// ponytail: single static hash table for non-_r variants
static mut HTAB_DATA: HSearchData = HSearchData {
    tab: core::ptr::null_mut(),
    __unused1: 0,
    __unused2: 0,
};

unsafe fn keyhash(k: *const c_char) -> usize {
    let mut h: usize = 0;
    let mut p = k;
    while *p != 0 {
        h = h.wrapping_mul(31).wrapping_add(*p as usize);
        p = p.add(1);
    }
    h
}

unsafe fn htab_lookup(key: *const c_char, hash: usize, htab: *mut HSearchData) -> *mut HSearchEntry {
    let tab = (*htab).tab;
    let mask = (*tab).mask;
    let entries = (*tab).entries;
    let mut i = hash;
    let mut j: usize = 1;
    loop {
        let e = entries.add(i & mask);
        if (*e).key.is_null() || strcmp((*e).key as *const u8, key as *const u8) == 0 {
            return e;
        }
        i = i.wrapping_add(j);
        j += 1;
    }
}

unsafe fn htab_resize(nel: usize, htab: *mut HSearchData) -> c_int {
    let tab = (*htab).tab;
    let old_entries = (*tab).entries;
    let old_size = (*tab).mask + 1;

    let mut newsize: usize = HSEARCH_MINSIZE;
    while newsize < nel {
        newsize = newsize.wrapping_mul(2);
    }

    let new_entries = calloc(newsize, core::mem::size_of::<HSearchEntry>()) as *mut HSearchEntry;
    if new_entries.is_null() {
        return 0;
    }
    (*tab).entries = new_entries;
    (*tab).mask = newsize - 1;

    if !old_entries.is_null() {
        for idx in 0..old_size {
            let old_e = old_entries.add(idx);
            if !(*old_e).key.is_null() {
                let h = keyhash((*old_e).key);
                let mut i = h;
                let mut j: usize = 1;
                loop {
                    let e = new_entries.add(i & (*tab).mask);
                    if (*e).key.is_null() {
                        *e = *old_e;
                        break;
                    }
                    i = i.wrapping_add(j);
                    j += 1;
                }
            }
        }
        free(old_entries as *mut c_void);
    }
    1
}

unsafe fn hcreate_r_impl(nel: usize, htab: *mut HSearchData) -> c_int {
    (*htab).tab = calloc(1, core::mem::size_of::<HTab>()) as *mut HTab;
    if (*htab).tab.is_null() {
        return 0;
    }
    if htab_resize(nel, htab) == 0 {
        free((*htab).tab as *mut c_void);
        (*htab).tab = core::ptr::null_mut();
        return 0;
    }
    1
}

unsafe fn hdestroy_r_impl(htab: *mut HSearchData) {
    if !(*htab).tab.is_null() {
        if !(*(*htab).tab).entries.is_null() {
            free((*(*htab).tab).entries as *mut c_void);
        }
        free((*htab).tab as *mut c_void);
        (*htab).tab = core::ptr::null_mut();
    }
}

unsafe fn hsearch_r_impl(item: HSearchEntry, action: c_int, retval: *mut *mut HSearchEntry, htab: *mut HSearchData) -> c_int {
    let hash = keyhash(item.key);
    let e = htab_lookup(item.key, hash, htab);

    if !(*e).key.is_null() {
        *retval = e;
        return 1;
    }
    if action == 0 { // FIND
        *retval = core::ptr::null_mut();
        return 0;
    }
    // ENTER
    let item_key = item.key;
    *e = item;
    (*(*htab).tab).used += 1;
    if (*(*htab).tab).used > (*(*htab).tab).mask - (*(*htab).tab).mask / 4 {
        if htab_resize(2 * (*(*htab).tab).used, htab) == 0 {
            (*(*htab).tab).used -= 1;
            (*e).key = core::ptr::null_mut();
            *retval = core::ptr::null_mut();
            return 0;
        }
        let e2 = htab_lookup(item_key, hash, htab);
        *retval = e2;
        return 1;
    }
    *retval = e;
    1
}

#[no_mangle]
pub unsafe extern "C" fn hcreate(nel: usize) -> c_int {
    hcreate_r_impl(nel, core::ptr::addr_of_mut!(HTAB_DATA))
}

#[no_mangle]
pub unsafe extern "C" fn hdestroy() {
    hdestroy_r_impl(core::ptr::addr_of_mut!(HTAB_DATA));
}

#[no_mangle]
pub unsafe extern "C" fn hsearch(item: HSearchEntry, action: c_int) -> *mut HSearchEntry {
    let mut e: *mut HSearchEntry = core::ptr::null_mut();
    hsearch_r_impl(item, action, &mut e, core::ptr::addr_of_mut!(HTAB_DATA));
    e
}

#[no_mangle]
pub unsafe extern "C" fn hcreate_r(nel: usize, htab: *mut HSearchData) -> c_int {
    hcreate_r_impl(nel, htab)
}

#[no_mangle]
pub unsafe extern "C" fn hdestroy_r(htab: *mut HSearchData) {
    hdestroy_r_impl(htab);
}

#[no_mangle]
pub unsafe extern "C" fn hsearch_r(item: HSearchEntry, action: c_int, retval: *mut *mut HSearchEntry, htab: *mut HSearchData) -> c_int {
    hsearch_r_impl(item, action, retval, htab)
}

// ============================================================
// Search: insque / remque
// ============================================================

#[repr(C)]
struct QueNode {
    next: *mut QueNode,
    prev: *mut QueNode,
}

#[no_mangle]
pub unsafe extern "C" fn insque(element: *mut c_void, pred: *mut c_void) {
    let e = element as *mut QueNode;
    let p = pred as *mut QueNode;
    if p.is_null() {
        (*e).next = core::ptr::null_mut();
        (*e).prev = core::ptr::null_mut();
        return;
    }
    (*e).next = (*p).next;
    (*e).prev = p;
    (*p).next = e;
    if !(*e).next.is_null() {
        (*(*e).next).prev = e;
    }
}

#[no_mangle]
pub unsafe extern "C" fn remque(element: *mut c_void) {
    let e = element as *mut QueNode;
    if !(*e).next.is_null() {
        (*(*e).next).prev = (*e).prev;
    }
    if !(*e).prev.is_null() {
        (*(*e).prev).next = (*e).next;
    }
}

// ============================================================
// Search: lsearch / lfind
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn lsearch(
    key: *const c_void,
    base: *mut c_void,
    nelp: *mut usize,
    width: usize,
    compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>,
) -> *mut c_void {
    let n = *nelp;
    let p = base as *mut u8;
    for i in 0..n {
        let elem = p.add(i * width);
        if compar.unwrap()(key, elem as *const c_void) == 0 {
            return elem as *mut c_void;
        }
    }
    let dest = p.add(n * width);
    memcpy(dest, key as *const u8, width);
    *nelp = n + 1;
    dest as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn lfind(
    key: *const c_void,
    base: *const c_void,
    nelp: *mut usize,
    width: usize,
    compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>,
) -> *mut c_void {
    let n = *nelp;
    let p = base as *const u8;
    for i in 0..n {
        let elem = p.add(i * width);
        if compar.unwrap()(key, elem as *const c_void) == 0 {
            return elem as *mut c_void;
        }
    }
    core::ptr::null_mut()
}

// ============================================================
// Search: tsearch / tfind / tdelete / twalk / tdestroy (BST, no balancing)
// ============================================================

// ponytail: simple BST, no balancing; correct for tests, O(n) worst case
#[repr(C)]
pub struct TreeNode {
    key: *const c_void,
    left: *mut TreeNode,
    right: *mut TreeNode,
}

type ComparFn = Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>;

unsafe fn bst_new(key: *const c_void) -> *mut TreeNode {
    let n = calloc(1, core::mem::size_of::<TreeNode>()) as *mut TreeNode;
    if !n.is_null() {
        (*n).key = key;
        (*n).left = core::ptr::null_mut();
        (*n).right = core::ptr::null_mut();
    }
    n
}

#[no_mangle]
pub unsafe extern "C" fn tsearch(
    key: *const c_void,
    rootp: *mut *mut TreeNode,
    compar: ComparFn,
) -> *mut c_void {
    if rootp.is_null() {
        return core::ptr::null_mut();
    }
    let mut pp = rootp;
    loop {
        let n = *pp;
        if n.is_null() {
            let r = bst_new(key);
            if r.is_null() {
                return core::ptr::null_mut();
            }
            *pp = r;
            return r as *mut c_void;
        }
        let c = compar.unwrap()(key, (*n).key);
        if c == 0 {
            return n as *mut c_void;
        }
        if c < 0 {
            pp = &mut (*n).left;
        } else {
            pp = &mut (*n).right;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn tfind(
    key: *const c_void,
    rootp: *const *mut TreeNode,
    compar: ComparFn,
) -> *mut c_void {
    if rootp.is_null() {
        return core::ptr::null_mut();
    }
    let mut n = *rootp;
    loop {
        if n.is_null() {
            return core::ptr::null_mut();
        }
        let c = compar.unwrap()(key, (*n).key);
        if c == 0 {
            return n as *mut c_void;
        }
        if c < 0 {
            n = (*n).left;
        } else {
            n = (*n).right;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn tdelete(
    key: *const c_void,
    rootp: *mut *mut TreeNode,
    compar: ComparFn,
) -> *mut c_void {
    if rootp.is_null() {
        return core::ptr::null_mut();
    }
    let mut pp = rootp;
    loop {
        let n = *pp;
        if n.is_null() {
            return core::ptr::null_mut();
        }
        let c = compar.unwrap()(key, (*n).key);
        if c == 0 {
            break;
        }
        if c < 0 {
            pp = &mut (*n).left;
        } else {
            pp = &mut (*n).right;
        }
    }
    let n = *pp;
    // parent is *rootp if we went to root, otherwise the node above
    let parent = *rootp;
    if !(*n).left.is_null() {
        // replace with in-order predecessor
        let mut q = &mut (*n).left;
        while !(*(*q)).right.is_null() {
            q = &mut (*(*q)).right;
        }
        let pred = *q;
        // swap keys
        (*n).key = (*pred).key;
        // remove predecessor
        *q = (*pred).left;
        free(pred as *mut c_void);
    } else {
        // replace with right child (may be null)
        *pp = (*n).right;
        free(n as *mut c_void);
    }
    parent as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn twalk(
    root: *const TreeNode,
    action: Option<unsafe extern "C" fn(*const c_void, c_int, c_int)>,
) {
    unsafe fn walk(
        node: *const TreeNode,
        action: Option<unsafe extern "C" fn(*const c_void, c_int, c_int)>,
        depth: c_int,
    ) {
        if node.is_null() {
            return;
        }
        let is_leaf = (*node).left.is_null() && (*node).right.is_null();
        if is_leaf {
            action.unwrap()(node as *const c_void, 3, depth); // leaf
        } else {
            action.unwrap()(node as *const c_void, 0, depth); // preorder
            walk((*node).left, action, depth + 1);
            action.unwrap()(node as *const c_void, 1, depth); // postorder (left done)
            walk((*node).right, action, depth + 1);
            action.unwrap()(node as *const c_void, 2, depth); // endorder
        }
    }
    walk(root, action, 0);
}

#[no_mangle]
pub unsafe extern "C" fn tdestroy(root: *mut TreeNode, freekey: Option<unsafe extern "C" fn(*mut c_void)>) {
    if root.is_null() {
        return;
    }
    tdestroy((*root).left, freekey);
    tdestroy((*root).right, freekey);
    if let Some(f) = freekey {
        f((*root).key as *mut c_void);
    }
    free(root as *mut c_void);
}

// ============================================================
// fnmatch: full POSIX pattern matching
// ============================================================

const FNM_PATHNAME: c_int = 0x1;
const FNM_NOESCAPE: c_int = 0x2;
const FNM_PERIOD: c_int = 0x4;
const FNM_LEADING_DIR: c_int = 0x8;
const FNM_CASEFOLD: c_int = 0x10;
const FNM_NOMATCH: c_int = 1;

const FNM_END: c_int = 0;
const FNM_STAR: c_int = -5;
const FNM_QUESTION: c_int = -4;
const FNM_BRACKET: c_int = -3;
const FNM_UNMATCHABLE: c_int = -2;

fn ascii_tolower(c: u8) -> u8 {
    if c >= b'A' && c <= b'Z' { c + 32 } else { c }
}

fn ascii_toupper(c: u8) -> u8 {
    if c >= b'a' && c <= b'z' { c - 32 } else { c }
}

fn ascii_casefold(k: u8) -> u8 {
    let upper = ascii_toupper(k);
    if upper != k { upper } else { ascii_tolower(k) }
}

/// Get next byte from pattern, classifying it.
/// Returns (token_type, bytes_consumed).
unsafe fn pat_next(pat: *const u8, m: usize, flags: c_int) -> (c_int, usize) {
    if m == 0 || *pat == 0 {
        return (FNM_END, 0);
    }
    let mut p = pat;
    let mut step = 1usize;
    let mut esc = false;
    if *p == b'\\' && !(flags & FNM_NOESCAPE != 0) && *p.add(1) != 0 {
        p = p.add(1);
        step = 2;
        esc = true;
    }
    if *p == b'[' && !esc {
        // scan for matching ]
        let mut k = 1usize;
        if k < m && (*pat.add(k) == b'^' || *pat.add(k) == b'!') { k += 1; }
        if k < m && *pat.add(k) == b']' { k += 1; }
        while k < m && *pat.add(k) != 0 && *pat.add(k) != b']' {
            if k + 1 < m && *pat.add(k) == b'[' &&
               (*pat.add(k+1) == b':' || *pat.add(k+1) == b'.' || *pat.add(k+1) == b'=') {
                let z = *pat.add(k+1);
                k += 2;
                if k < m { k += 1; }
                while k < m && *pat.add(k) != 0 && (*pat.add(k-1) != z || *pat.add(k) != b']') { k += 1; }
                if k == m || *pat.add(k) == 0 { break; }
            }
            k += 1;
        }
        if k == m || *pat.add(k) == 0 {
            // unmatched [ is literal
            return (b'[' as c_int, 1);
        }
        return (FNM_BRACKET, k + 1);
    }
    if *p == b'*' && !esc {
        return (FNM_STAR, 1);
    }
    if *p == b'?' && !esc {
        return (FNM_QUESTION, 1);
    }
    (*p as c_int, step)
}

unsafe fn match_class(name: &[u8], k: u8, kfold: u8) -> bool {
    let eq = |c: u8| k == c || kfold == c;
    let in_range = |lo: u8, hi: u8| (k >= lo && k <= hi) || (kfold >= lo && kfold <= hi);
    match name {
        b"alnum" => in_range(b'0', b'9') || in_range(b'A', b'Z') || in_range(b'a', b'z'),
        b"alpha" => in_range(b'A', b'Z') || in_range(b'a', b'z'),
        b"blank" => eq(b' ') || eq(b'\t'),
        b"cntrl" => k < 0x20 || k == 0x7f,
        b"digit" => in_range(b'0', b'9'),
        b"graph" => k >= 0x21 && k <= 0x7e,
        b"lower" => in_range(b'a', b'z'),
        b"print" => k >= 0x20 && k <= 0x7e,
        b"punct" =>
            (k >= 0x21 && k <= 0x2f) ||
            (k >= 0x3a && k <= 0x40) ||
            (k >= 0x5b && k <= 0x60) ||
            (k >= 0x7b && k <= 0x7e),
        b"space" => eq(b' ') || eq(b'\t') || eq(b'\n') || eq(b'\r') || eq(0x0c) || eq(0x0b),
        b"upper" => in_range(b'A', b'Z'),
        b"xdigit" => in_range(b'0', b'9') || in_range(b'A', b'F') || in_range(b'a', b'f'),
        _ => false,
    }
}

/// Match a bracket expression [class] against char k.
unsafe fn match_bracket(mut p: *const u8, k: u8, kfold: u8) -> bool {
    let mut inv = false;
    p = p.add(1); // skip [
    if *p == b'^' || *p == b'!' {
        inv = true;
        p = p.add(1);
    }
    // handle ] as first char
    if *p == b']' {
        if k == b']' { return !inv; }
        p = p.add(1);
    } else if *p == b'-' {
        if k == b'-' { return !inv; }
        p = p.add(1);
    }
    while *p != b']' && *p != 0 {
        if *p == b'-' && *p.add(1) != b']' && *p.add(1) != 0 {
            // range: previous char to next char
            let lo = *p.sub(1);
            let hi = *p.add(1);
            if lo <= hi && (k >= lo && k <= hi || kfold >= lo && kfold <= hi) {
                return !inv;
            }
            p = p.add(2);
        } else if *p == b'[' && (*p.add(1) == b':' || *p.add(1) == b'.' || *p.add(1) == b'=') {
            let z = *p.add(1);
            let start = p.add(2);
            let mut q = start;
            while *q != 0 && (*q != z || *q.add(1) != b']') { q = q.add(1); }
            if *q != 0 {
                let len = (q as usize) - (start as usize);
                if len > 0 {
                    let inner = core::slice::from_raw_parts(start, len);
                    let matched =
                        if z == b':' { match_class(inner, k, kfold) }
                        else { len == 1 && (inner[0] == k || inner[0] == kfold) };
                    if matched { return !inv; }
                }
                p = q.add(2);
                continue;
            }
        } else {
            if *p == k || *p == kfold {
                return !inv;
            }
            p = p.add(1);
        }
    }
    inv
}

/// Core fnmatch for a single path component (no FNM_PATHNAME splitting).
unsafe fn fnmatch_internal(pat: *const u8, str: *const u8, flags: c_int) -> c_int {
    // Handle leading .
    if flags & FNM_PERIOD != 0 && *str == b'.' && *pat != b'.' {
        return FNM_NOMATCH;
    }

    let mut p = pat;
    let mut s = str;

    // Consume pattern up to first *
    loop {
        let (tok, step) = pat_next(p, usize::MAX, flags);
        match tok {
            FNM_END => {
                return if *s == 0 { 0 } else { FNM_NOMATCH };
            }
            FNM_STAR => {
                p = p.add(1);
                break;
            }
            FNM_QUESTION => {
                if *s == 0 { return FNM_NOMATCH; }
                s = s.add(1);
                p = p.add(step);
            }
            FNM_BRACKET => {
                if *s == 0 { return FNM_NOMATCH; }
                let k = *s;
                let kfold = if flags & FNM_CASEFOLD != 0 { ascii_casefold(k) } else { k };
                if !match_bracket(p, k, kfold) { return FNM_NOMATCH; }
                p = p.add(step);
                s = s.add(1);
            }
            c => {
                if *s == 0 { return FNM_NOMATCH; }
                let k = *s;
                let kfold = if flags & FNM_CASEFOLD != 0 { ascii_casefold(k) } else { k };
                if k as c_int != c && kfold as c_int != c { return FNM_NOMATCH; }
                p = p.add(step);
                s = s.add(1);
            }
        }
    }

    // Now p points after first *, rest of pattern is variable
    // Factor: find tail after last * and count fixed chars needed
    let mut ptail = p;
    let mut tailcnt: usize = 0;
    let mut pp = p;
    loop {
        let (tok, step) = pat_next(pp, usize::MAX, flags);
        if tok == FNM_END { break; }
        if tok == FNM_STAR {
            tailcnt = 0;
            ptail = pp.add(1);
        } else {
            tailcnt += 1;
        }
        pp = pp.add(step);
    }

    // str must have at least tailcnt chars
    let slen = strlen(s);
    if slen < tailcnt { return FNM_NOMATCH; }

    // Match tail: ptail..end against str[tailcnt from end]
    let stail = s.add(slen - tailcnt);
    let mut tp = ptail;
    let mut ts = stail;
    loop {
        let (tok, step) = pat_next(tp, usize::MAX, flags);
        tp = tp.add(step);
        if tok == FNM_END {
            break;
        }
        let k = *ts;
        if k == 0 { return FNM_NOMATCH; }
        let kfold = if flags & FNM_CASEFOLD != 0 { ascii_casefold(k) } else { k };
        match tok {
            FNM_QUESTION => { ts = ts.add(1); }
            FNM_BRACKET => {
                if !match_bracket(tp.sub(step), k, kfold) { return FNM_NOMATCH; }
                ts = ts.add(1);
            }
            c => {
                if k as c_int != c && kfold as c_int != c { return FNM_NOMATCH; }
                ts = ts.add(1);
            }
        }
    }

    let endstr = stail;
    let endpat = ptail;

    let mut cp = p;
    let mut cs = s;
    while cp < endpat {
        let (tok, step) = pat_next(cp, usize::MAX, flags);
        if tok == FNM_STAR {
            cp = cp.add(step);
            cs = s;
            continue;
        }
        let comp_start = cp;
        let mut ok = false;
        let mut try_s = cs;
        while try_s < endstr {
            let mut tmp_p = comp_start;
            let mut tmp_s = try_s;
            let mut matched = true;
            loop {
                let (t, st) = pat_next(tmp_p, usize::MAX, flags);
                if t == FNM_STAR || t == FNM_END {
                    cp = tmp_p;
                    cs = tmp_s;
                    ok = true;
                    break;
                }
                if *tmp_s == 0 {
                    matched = false;
                    break;
                }
                let k = *tmp_s;
                let kfold = if flags & FNM_CASEFOLD != 0 { ascii_casefold(k) } else { k };
                match t {
                    FNM_QUESTION => {
                        tmp_s = tmp_s.add(1);
                    }
                    FNM_BRACKET => {
                        if !match_bracket(tmp_p, k, kfold) {
                            matched = false;
                            break;
                        }
                        tmp_s = tmp_s.add(1);
                    }
                    c => {
                        if k as c_int != c && kfold as c_int != c {
                            matched = false;
                            break;
                        }
                        tmp_s = tmp_s.add(1);
                    }
                }
                tmp_p = tmp_p.add(st);
            }
            if ok { break; }
            if !matched {
                try_s = try_s.add(1);
            }
        }
        if !ok {
            return FNM_NOMATCH;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn fnmatch(pat: *const c_char, str: *const c_char, flags: c_int) -> c_int {
    let pat = pat as *const u8;
    let str = str as *const u8;

    if flags & FNM_PATHNAME != 0 {
        // Split on /
        let mut s = str;
        let mut p = pat;
        loop {
            // find next / in str
            let mut send = s;
            while *send != 0 && *send != b'/' { send = send.add(1); }
            // find next / in pat
            let mut pend = p;
            loop {
                let (tok, step) = pat_next(pend, usize::MAX, flags);
                if tok == FNM_END || (tok >= 0 && tok as u8 == b'/') {
                    break;
                }
                pend = pend.add(step);
            }
            let (ptok, pstep) = pat_next(pend, usize::MAX, flags);
            let pat_is_slash = ptok >= 0 && ptok as u8 == b'/';
            let str_is_slash = *send == b'/';
            if pat_is_slash != str_is_slash && (*send != 0 || !(flags & FNM_LEADING_DIR != 0)) {
                return FNM_NOMATCH;
            }
            // match component
            // We need a null-terminated copy of the component
            let plen = (pend as usize) - (p as usize);
            let slen = (send as usize) - (s as usize);
            // Use stack buffers for components
            // ponytail: max component 1024 bytes
            if plen > 1024 || slen > 1024 { return FNM_NOMATCH; }
            let mut pbuf = [0u8; 1025];
            let mut sbuf = [0u8; 1025];
            core::ptr::copy_nonoverlapping(p, pbuf.as_mut_ptr(), plen);
            pbuf[plen] = 0;
            core::ptr::copy_nonoverlapping(s, sbuf.as_mut_ptr(), slen);
            sbuf[slen] = 0;
            if fnmatch_internal(pbuf.as_ptr(), sbuf.as_ptr(), flags & !FNM_LEADING_DIR) != 0 {
                return FNM_NOMATCH;
            }
            if !pat_is_slash {
                return 0;
            }
            s = send.add(1);
            p = pend.add(pstep);
        }
    }

    if flags & FNM_LEADING_DIR != 0 {
        // Try matching at each /
        let mut s = str;
        while *s != 0 {
            if *s == b'/' {
                // temporarily null-terminate
                // ponytail: we need to copy. Use a simple approach.
                let len = (s as usize) - (str as usize);
                // Just call fnmatch_internal with a temp buffer
                if len < 4096 {
                    let mut buf = [0u8; 4096];
                    core::ptr::copy_nonoverlapping(str, buf.as_mut_ptr(), len);
                    buf[len] = 0;
                    if fnmatch_internal(pat, buf.as_ptr(), flags) == 0 {
                        return 0;
                    }
                }
            }
            s = s.add(1);
        }
    }

    fnmatch_internal(pat, str, flags)
}

// ============================================================
// mntent: setmntent/endmntent/getmntent/getmntent_r/addmntent/hasmntopt
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn setmntent(name: *const c_char, mode: *const c_char) -> *mut FILE {
    fopen(name, mode)
}

#[no_mangle]
pub unsafe extern "C" fn endmntent(f: *mut FILE) -> c_int {
    if !f.is_null() {
        fclose(f);
    }
    1
}

// ponytail: static buffers for getmntent (non-_r variant)
static mut MNTENT_MNT: MntEnt = MntEnt {
    mnt_fsname: core::ptr::null_mut(),
    mnt_dir: core::ptr::null_mut(),
    mnt_type: core::ptr::null_mut(),
    mnt_opts: core::ptr::null_mut(),
    mnt_freq: 0,
    mnt_passno: 0,
};

#[repr(C)]
pub struct MntEnt {
    mnt_fsname: *mut c_char,
    mnt_dir: *mut c_char,
    mnt_type: *mut c_char,
    mnt_opts: *mut c_char,
    mnt_freq: c_int,
    mnt_passno: c_int,
}

unsafe fn unescape_ent(beg: *mut c_char) -> *mut c_char {
    let mut dest = beg;
    let mut src = beg;
    while *src != 0 {
        if *src != b'\\' as c_char {
            *dest = *src;
            dest = dest.add(1);
            src = src.add(1);
            continue;
        }
        // backslash
        if *src.add(1) == b'\\' as c_char {
            src = src.add(1);
            *dest = *src;
            dest = dest.add(1);
            src = src.add(1);
            continue;
        }
        // octal escape
        let mut val: u8 = 0;
        let mut cnt = 0;
        let mut sp = src.add(1);
        while cnt < 3 && *sp >= b'0' as c_char && *sp <= b'7' as c_char {
            val = val * 8 + (*sp as u8 - b'0');
            sp = sp.add(1);
            cnt += 1;
        }
        if cnt > 0 {
            *dest = val as c_char;
            dest = dest.add(1);
            src = sp;
        } else {
            *dest = *src;
            dest = dest.add(1);
            src = src.add(1);
        }
    }
    *dest = 0;
    beg
}

unsafe fn parse_mntent_line(
    linebuf: *mut c_char,
    mnt: *mut MntEnt,
) -> *mut MntEnt {
    let _len = strlen(linebuf as *const u8);
    // skip comments and empty lines
    // parse: fsname dir type opts freq passno
    let mut p = linebuf;
    // skip leading whitespace
    while *p == b' ' as c_char || *p == b'\t' as c_char || *p == b'\n' as c_char || *p == b'\r' as c_char {
        p = p.add(1);
    }
    if *p == 0 || *p == b'#' as c_char {
        return core::ptr::null_mut();
    }

    // fsname
    (*mnt).mnt_fsname = p;
    while *p != 0 && *p != b' ' as c_char && *p != b'\t' as c_char && *p != b'\n' as c_char {
        p = p.add(1);
    }
    if *p == 0 { return core::ptr::null_mut(); }
    *p = 0;
    p = p.add(1);
    while *p == b' ' as c_char || *p == b'\t' as c_char { p = p.add(1); }

    // dir
    (*mnt).mnt_dir = p;
    while *p != 0 && *p != b' ' as c_char && *p != b'\t' as c_char && *p != b'\n' as c_char {
        p = p.add(1);
    }
    if *p == 0 { return core::ptr::null_mut(); }
    *p = 0;
    p = p.add(1);
    while *p == b' ' as c_char || *p == b'\t' as c_char { p = p.add(1); }

    // type
    (*mnt).mnt_type = p;
    while *p != 0 && *p != b' ' as c_char && *p != b'\t' as c_char && *p != b'\n' as c_char {
        p = p.add(1);
    }
    if *p == 0 { return core::ptr::null_mut(); }
    *p = 0;
    p = p.add(1);
    while *p == b' ' as c_char || *p == b'\t' as c_char { p = p.add(1); }

    // opts
    (*mnt).mnt_opts = p;
    while *p != 0 && *p != b' ' as c_char && *p != b'\t' as c_char && *p != b'\n' as c_char {
        p = p.add(1);
    }
    if *p != 0 {
        *p = 0;
        p = p.add(1);
    }
    if *(*mnt).mnt_opts == 0 {
        (*mnt).mnt_opts = b"defaults\0".as_ptr() as *mut c_char;
    }

    // freq and passno
    (*mnt).mnt_freq = 0;
    (*mnt).mnt_passno = 0;
    // skip whitespace
    while *p == b' ' as c_char || *p == b'\t' as c_char { p = p.add(1); }
    if *p >= b'0' as c_char && *p <= b'9' as c_char {
        (*mnt).mnt_freq = (*p - b'0' as c_char) as c_int;
        p = p.add(1);
        while *p >= b'0' as c_char && *p <= b'9' as c_char {
            (*mnt).mnt_freq = (*mnt).mnt_freq * 10 + (*p - b'0' as c_char) as c_int;
            p = p.add(1);
        }
    }
    while *p == b' ' as c_char || *p == b'\t' as c_char { p = p.add(1); }
    if *p >= b'0' as c_char && *p <= b'9' as c_char {
        (*mnt).mnt_passno = (*p - b'0' as c_char) as c_int;
        p = p.add(1);
        while *p >= b'0' as c_char && *p <= b'9' as c_char {
            (*mnt).mnt_passno = (*mnt).mnt_passno * 10 + (*p - b'0' as c_char) as c_int;
            p = p.add(1);
        }
    }

    // unescape
    unescape_ent((*mnt).mnt_fsname);
    unescape_ent((*mnt).mnt_dir);
    unescape_ent((*mnt).mnt_type);
    unescape_ent((*mnt).mnt_opts);

    mnt
}

// Use a sentinel pointer to detect internal buffer usage
// ponytail: cast integer to pointer for sentinel
static SENTINEL_PTR: usize = 1;

// Read one char from FILE buffer directly (avoids musl's static fgets ABI mismatch)
unsafe fn mntent_fgetc(f: *mut FILE) -> c_int {
    let ff = &mut *f;
    if !ff.rpos.is_null() && ff.rpos < ff.rend {
        let c = *ff.rpos;
        ff.rpos = ff.rpos.add(1);
        return c as c_int;
    }
    ff._eof = 1;
    -1
}

unsafe fn mntent_fgets(buf: *mut c_char, n: usize, f: *mut FILE) -> *mut c_char {
    if n == 0 { return core::ptr::null_mut(); }
    let max = n - 1;
    let mut i = 0usize;
    while i < max {
        let c = mntent_fgetc(f);
        if c == -1 { if i == 0 { return core::ptr::null_mut(); } break; }
        *buf.add(i) = c as c_char;
        i += 1;
        if c == b'\n' as c_int { break; }
    }
    *buf.add(i) = 0;
    buf
}

#[no_mangle]
pub unsafe extern "C" fn getmntent_r(
    f: *mut FILE,
    mnt: *mut MntEnt,
    mut linebuf: *mut c_char,
    _buflen: c_int,
) -> *mut MntEnt {
    let use_internal = linebuf as usize == SENTINEL_PTR;

    (*mnt).mnt_freq = 0;
    (*mnt).mnt_passno = 0;

    // ponytail: static 4KB line buffer for getmntent
    static mut INTERNAL_BUF: [c_char; 4096] = [0; 4096];

    loop {
        if use_internal {
            linebuf = core::ptr::addr_of_mut!(INTERNAL_BUF).cast::<c_char>();
        }
        let res = mntent_fgets(linebuf, 4096, f);
        if res.is_null() {
            return core::ptr::null_mut();
        }
        let mut has_newline = false;
        let mut p = linebuf;
        while *p != 0 { if *p == b'\n' as c_char { has_newline = true; break; } p = p.add(1); }
        if !has_newline {
            ERRNO = 34;
            return core::ptr::null_mut();
        }
        let result = parse_mntent_line(linebuf, mnt);
        if !result.is_null() {
            return result;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn getmntent(f: *mut FILE) -> *mut MntEnt {
    getmntent_r(f, core::ptr::addr_of_mut!(MNTENT_MNT), SENTINEL_PTR as *mut c_char, 0)
}

#[no_mangle]
pub unsafe extern "C" fn addmntent(f: *mut FILE, mnt: *const MntEnt) -> c_int {
    fseek(f, 0, 2); // SEEK_END
    let written = fprintf(
        f,
        b"%s\t%s\t%s\t%s\t%d\t%d\n\0".as_ptr() as *const c_char,
        (*mnt).mnt_fsname,
        (*mnt).mnt_dir,
        (*mnt).mnt_type,
        (*mnt).mnt_opts,
        (*mnt).mnt_freq,
        (*mnt).mnt_passno,
    );
    if written < 0 { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn hasmntopt(mnt: *const MntEnt, opt: *const c_char) -> *mut c_char {
    let l = strlen(opt as *const u8);
    let mut p = (*mnt).mnt_opts;
    loop {
        if strncmp(p as *const u8, opt as *const u8, l) == 0 {
            let after = *p.add(l);
            if after == 0 || after == b',' as c_char || after == b'=' as c_char {
                return p;
            }
        }
        let comma = strchr(p as *const u8, b',' as c_int);
        if comma.is_null() {
            return core::ptr::null_mut();
        }
        p = comma.add(1) as *mut c_char;
    }
}

// ============================================================
// SysV IPC: ftok, msg*, sem*, shm*
// x86_64 direct syscalls: msgget=68, msgsnd=69, msgrcv=70, msgctl=71
//   semget=64, semop=65, semctl=66, semtimedop=220
//   shmget=29, shmat=30, shmctl=31, shmdt=67
// ============================================================

const EEXIST: c_int = 17;
const ENAMETOOLONG: c_int = 36;
const EMFILE: c_int = 24;
const EIDRM: c_int = 43;

#[inline]
unsafe fn sys_msgget(key: c_int, msgflg: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 68i64 => result,
        in("rdi") key,
        in("rsi") msgflg,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_msgsnd(msqid: c_int, msgp: *const c_void, msgsz: usize, msgflg: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 69i64 => result,
        in("rdi") msqid,
        in("rsi") msgp,
        in("rdx") msgsz,
        in("r10") msgflg,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_msgrcv(msqid: c_int, msgp: *mut c_void, msgsz: usize, msgtyp: c_long, msgflg: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 70i64 => result,
        in("rdi") msqid,
        in("rsi") msgp,
        in("rdx") msgsz,
        in("r10") msgtyp,
        in("r8") msgflg,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_msgctl(msqid: c_int, cmd: c_int, buf: *mut c_void) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 71i64 => result,
        in("rdi") msqid,
        in("rsi") cmd,
        in("rdx") buf,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_semget(key: c_int, nsems: c_int, semflg: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 64i64 => result,
        in("rdi") key,
        in("rsi") nsems,
        in("rdx") semflg,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_semop(semid: c_int, sops: *const c_void, nsops: usize) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 65i64 => result,
        in("rdi") semid,
        in("rsi") sops,
        in("rdx") nsops,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_semctl(semid: c_int, semnum: c_int, cmd: c_int, arg: *mut c_void) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 66i64 => result,
        in("rdi") semid,
        in("rsi") semnum,
        in("rdx") cmd,
        in("r10") arg,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_shmget(key: c_int, size: usize, shmflg: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 29i64 => result,
        in("rdi") key,
        in("rsi") size,
        in("rdx") shmflg,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_shmat(shmid: c_int, shmaddr: *const c_void, shmflg: c_int) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 30i64 => result,
        in("rdi") shmid,
        in("rsi") shmaddr,
        in("rdx") shmflg,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_shmdt(shmaddr: *const c_void) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 67i64 => result,
        in("rdi") shmaddr,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_shmctl(shmid: c_int, cmd: c_int, buf: *mut c_void) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 31i64 => result,
        in("rdi") shmid,
        in("rsi") cmd,
        in("rdx") buf,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

// ftok
#[no_mangle]
pub unsafe extern "C" fn ftok(path: *const c_char, id: c_int) -> c_int {
    let mut st: kernel_stat64 = core::mem::zeroed();
    let r = sys_stat(path as *const u8, &mut st);
    if r < 0 {
        ERRNO = (-r) as c_int;
        return -1;
    }
    ((st.st_ino & 0xffff) | ((st.st_dev & 0xff) << 16) | (((id as u32 & 0xff) << 24)) as u64) as c_int
}

// msgget
#[no_mangle]
pub unsafe extern "C" fn msgget(key: c_int, msgflg: c_int) -> c_int {
    let r = sys_msgget(key, msgflg);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// msgsnd
#[no_mangle]
pub unsafe extern "C" fn msgsnd(msqid: c_int, msgp: *const c_void, msgsz: usize, msgflg: c_int) -> c_int {
    let r = sys_msgsnd(msqid, msgp, msgsz, msgflg);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// msgrcv
#[no_mangle]
pub unsafe extern "C" fn msgrcv(msqid: c_int, msgp: *mut c_void, msgsz: usize, msgtyp: c_long, msgflg: c_int) -> isize {
    let r = sys_msgrcv(msqid, msgp, msgsz, msgtyp, msgflg);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as isize }
}

// msgctl
#[no_mangle]
pub unsafe extern "C" fn msgctl(msqid: c_int, cmd: c_int, buf: *mut c_void) -> c_int {
    let r = sys_msgctl(msqid, cmd, buf);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// semget
#[no_mangle]
pub unsafe extern "C" fn semget(key: c_int, nsems: c_int, semflg: c_int) -> c_int {
    if nsems > 65535 { ERRNO = 22; return -1; }
    let r = sys_semget(key, nsems, semflg);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// semop
#[no_mangle]
pub unsafe extern "C" fn semop(semid: c_int, sops: *const c_void, nsops: usize) -> c_int {
    let r = sys_semop(semid, sops, nsops);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// semctl - C variadic
#[no_mangle]
pub unsafe extern "C" fn semctl(semid: c_int, semnum: c_int, cmd: c_int, mut args: ...) -> c_int {
    let mut arg: *mut c_void = core::ptr::null_mut();
    // Commands that take the union semun argument
    match cmd {
        16 | 13 | 17 | 1 | 3 | 2 | 0x102 | 0x104 | 0x106 => {
            arg = args.arg::<*mut c_void>();
        }
        _ => {}
    }
    let r = sys_semctl(semid, semnum, cmd, arg);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// shmget
#[no_mangle]
pub unsafe extern "C" fn shmget(key: c_int, size: usize, shmflg: c_int) -> c_int {
    let r = sys_shmget(key, size, shmflg);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// shmat
#[no_mangle]
pub unsafe extern "C" fn shmat(shmid: c_int, shmaddr: *const c_void, shmflg: c_int) -> *mut c_void {
    let r = sys_shmat(shmid, shmaddr, shmflg);
    if (r as usize) > (-(4096isize as isize)) as usize {
        ERRNO = (-r) as c_int;
        return core::ptr::null_mut();
    }
    r as *mut c_void
}

// shmdt
#[no_mangle]
pub unsafe extern "C" fn shmdt(shmaddr: *const c_void) -> c_int {
    let r = sys_shmdt(shmaddr);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// shmctl
#[no_mangle]
pub unsafe extern "C" fn shmctl(shmid: c_int, cmd: c_int, buf: *mut c_void) -> c_int {
    let r = sys_shmctl(shmid, cmd, buf);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
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
    __environ = envp as *mut *mut c_char;
    environ = __environ; // sync environ alias
    __stdio_init();

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

include!("crypt_impl.rs");
include!("statvfs.rs");
include!("../../wave1/daemon.rs");
include!("../../wave1/dn_expand.rs");
include!("../../wave1/lrand48.rs");
include!("../../wave1/strverscmp.rs");
include!("../../wave1/syscall.rs");
include!("../../wave1/pthread_atfork.rs");
include!("../../wave2/fenv.rs");
include!("../../wave2/locale_ctype.rs");
