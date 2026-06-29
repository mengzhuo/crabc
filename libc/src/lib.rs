#![cfg_attr(not(test), no_std)]
#![feature(c_variadic)]
#![feature(linkage)]
#![allow(dead_code, non_camel_case_types)]

use core::ffi::{c_char, c_int, c_long, c_longlong, c_uint, c_ulong, c_ulonglong, c_void, VaListImpl};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

// ============================================================
// errno
// ============================================================

const EILSEQ: c_int = 84;
const EINVAL: c_int = 22;
const EFAULT: c_int = 14;
const ENOMEM: c_int = 12;
const EINTR: c_int = 4;
const EPERM: c_int = 1;
const EAGAIN: c_int = 11;
const EBUSY: c_int = 16;
const EDEADLK: c_int = 35;
const ETIMEDOUT: c_int = 110;
const ECANCELED: c_int = 125;
const EOVERFLOW: c_int = 75;

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
    sys_fork() as c_int
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

// --- pthread_mutex_* ---
unsafe fn mutex_trylock(m: *mut pthread_mutex_t) -> c_int {
    let type_ = (*m).__i[0] & 15;
    if type_ == PTHREAD_MUTEX_NORMAL {
        return a_cas(&raw mut (*m).__i[1], 0, EBUSY) & EBUSY;
    }
    let self_tid = sys_gettid() as c_int;
    let old = a_load(&raw const (*m).__i[1]);
    let own = old & 0x3fffffff;
    if own == self_tid {
        if type_ == PTHREAD_MUTEX_RECURSIVE {
            if (*m).__i[5] >= c_int::MAX { return EAGAIN; }
            (*m).__i[5] += 1;
            return 0;
        }
        return EDEADLK;
    }
    if own != 0 { return EBUSY; }
    if a_cas(&raw mut (*m).__i[1], old, self_tid) == old {
        (*m).__i[5] = 0;
        0
    } else { EBUSY }
}

unsafe fn mutex_lock_internal(m: *mut pthread_mutex_t, abs_timeout: *const timespec) -> c_int {
    let type_ = (*m).__i[0] & 15;
    if type_ == PTHREAD_MUTEX_NORMAL && a_cas(&raw mut (*m).__i[1], 0, EBUSY) == 0 {
        return 0;
    }
    let r = mutex_trylock(m);
    if r != EBUSY { return r; }
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
        if type_ == PTHREAD_MUTEX_ERRORCHECK {
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
    let type_ = (*mutex).__i[0] & 15;
    if type_ != PTHREAD_MUTEX_NORMAL {
        let self_tid = sys_gettid() as c_int;
        if (a_load(&raw const (*mutex).__i[1]) & 0x3fffffff) != self_tid { return EPERM; }
        if type_ == PTHREAD_MUTEX_RECURSIVE && (*mutex).__i[5] > 0 {
            (*mutex).__i[5] -= 1;
            return 0;
        }
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
#[no_mangle]
pub unsafe extern "C" fn pthread_cancel(thread: PthreadT) -> c_int {
    let slot = thread as *mut Thread;
    if slot.is_null() { return EINVAL; }
    a_store(&raw mut (*slot).cancel, 1);
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
            _exit(0);
        }
    }
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

pub type IconvT = *mut c_void;
const ICONV_INVALID: IconvT = !0usize as IconvT;
const ICONV_IDENTITY: IconvT = 1usize as IconvT;

#[no_mangle]
pub unsafe extern "C" fn iconv_open(tocode: *const c_char, fromcode: *const c_char) -> IconvT {
    if tocode.is_null() || fromcode.is_null() {
        return ICONV_INVALID;
    }
    if strcmp(tocode as *const u8, fromcode as *const u8) == 0 {
        ICONV_IDENTITY
    } else {
        ICONV_INVALID
    }
}

#[no_mangle]
pub unsafe extern "C" fn iconv(
    cd: IconvT,
    inbuf: *mut *mut c_char,
    inbytesleft: *mut SizeT,
    outbuf: *mut *mut c_char,
    outbytesleft: *mut SizeT,
) -> SizeT {
    if cd == ICONV_INVALID {
        return !0usize;
    }
    if cd != ICONV_IDENTITY {
        return !0usize;
    }
    if inbuf.is_null() || outbuf.is_null() {
        return 0;
    }
    let mut src = *inbuf;
    let mut dst = *outbuf;
    let mut src_left = *inbytesleft;
    let mut dst_left = *outbytesleft;
    while src_left > 0 && dst_left > 0 {
        *dst = *src;
        src = src.add(1);
        dst = dst.add(1);
        src_left -= 1;
        dst_left -= 1;
    }
    *inbuf = src;
    *outbuf = dst;
    *inbytesleft = src_left;
    *outbytesleft = dst_left;
    if src_left == 0 {
        0
    } else {
        !0usize
    }
}

#[no_mangle]
pub unsafe extern "C" fn iconv_close(cd: IconvT) -> c_int {
    if cd == ICONV_IDENTITY {
        0
    } else {
        -1
    }
}

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
    __funcs_on_exit();
    _exit(code);
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
    0
}

#[no_mangle]
pub unsafe extern "C" fn clearenv() -> c_int {
    let e = __environ;
    __environ = core::ptr::null_mut();
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
