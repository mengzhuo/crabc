#![no_std]
#![feature(c_variadic)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void, VaListImpl};
use core::ptr::null_mut;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// ponytail: stub for linker; never called with panic=abort
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

// ============================================================
// C type aliases
// ============================================================

type SizeT = usize;
type SSizeT = isize;

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
    // ponytail: minimal FILE, only fd matters for write path
}

static mut STDIN_FILE: FILE = FILE { fd: 0 };
static mut STDOUT_FILE: FILE = FILE { fd: 1 };
static mut STDERR_FILE: FILE = FILE { fd: 2 };

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
